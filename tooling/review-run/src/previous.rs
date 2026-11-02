//! The `review-run previous` command: which earlier review of a pull
//! request its findings are carried from (ENG-28). `previous select`
//! reads the artifacts GitHub lists under a pull request's name and the
//! runs that uploaded them, and names the newest one a run of main's
//! review workflow uploaded; an artifact of that name from any other
//! run is skipped and counted, since any workflow a pull request
//! defines may upload one. `previous check` refuses a findings file
//! that is too large or does not read, so a broken history is named
//! and never taken for an empty one (I6). The tool decides and the
//! caller's script only fetches.

use std::fs;
use std::io::Write;

use serde::Deserialize;

use crate::cli::{EXIT_FAILURE, EXIT_OK, EXIT_USAGE};
use crate::findings;

/// How to call it.
pub const USAGE: &str = "usage: review-run previous select --pull N --artifacts FILE --runs FILE\n\
       review-run previous check FILE";

/// The workflow whose runs may upload the findings a review carries.
pub const WORKFLOW_PATH: &str = ".github/workflows/review.yml";
/// The events on which main defines that workflow's run: a run on any
/// other event, `pull_request` among them, may use the pull request's
/// own definition.
pub const EVENTS: [&str; 2] = ["workflow_run", "issue_comment"];
/// The most bytes a findings file may hold.
pub const MAX_BYTES: usize = 1 << 20;

/// What the lookup found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// The artifact to carry findings from, when one qualified.
    pub artifact: Option<u64>,
    /// The artifacts of the name, unexpired, refused for their origin.
    pub skipped: u32,
}

impl Selection {
    /// The lines the job's outputs take.
    #[must_use]
    pub fn lines(&self) -> String {
        let id = self.artifact.map(|a| a.to_string()).unwrap_or_default();
        format!(
            "previous_artifact={id}\nprevious_skipped={}\n",
            self.skipped
        )
    }
}

/// GitHub's listing of a repository's artifacts.
#[derive(Deserialize)]
struct Listing {
    artifacts: Vec<Artifact>,
}

/// One artifact of it.
#[derive(Deserialize)]
struct Artifact {
    id: u64,
    name: String,
    #[serde(default)]
    expired: bool,
    created_at: String,
    workflow_run: Option<RunRef>,
}

/// The run an artifact says it came from.
#[derive(Deserialize)]
struct RunRef {
    id: u64,
}

/// One run: where its workflow lives and what started it.
#[derive(Deserialize)]
struct Run {
    id: u64,
    path: String,
    event: String,
}

impl Run {
    /// Whether it is a run of main's review workflow: its file, less
    /// the ref GitHub may append, is [`WORKFLOW_PATH`], and an event
    /// ran on main's definition of it.
    fn is_main_review(&self) -> bool {
        let file = self.path.split('@').next().unwrap_or_default();
        file == WORKFLOW_PATH && EVENTS.contains(&self.event.as_str())
    }
}

/// The artifact name a pull request's findings are kept under.
///
/// # Errors
///
/// Fails when `pull` is not a pull request number.
pub fn name(pull: &str) -> Result<String, String> {
    if pull.is_empty() || !pull.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("{pull:?} is not a pull request number"));
    }

    Ok(format!("review-findings-pr-{pull}"))
}

/// Picks the artifact of the pull request `pull` to carry findings
/// from: `artifacts` is GitHub's listing, `runs` the runs it names as
/// an array of `{id, path, event}`. Newest first, the first artifact of
/// the name, not expired, whose run is one of main's review workflow
/// wins; each one before it is counted. An artifact whose run is not in
/// `runs` has an origin no one checked and is refused as well.
///
/// # Errors
///
/// Fails when either input does not read, or `pull` is no number.
pub fn select(pull: &str, artifacts: &str, runs: &str) -> Result<Selection, String> {
    let wanted = name(pull)?;
    let listing: Listing =
        serde_json::from_str(artifacts).map_err(|e| format!("artifacts do not read: {e}"))?;
    let runs: Vec<Run> =
        serde_json::from_str(runs).map_err(|e| format!("runs do not read: {e}"))?;
    let mut named: Vec<&Artifact> = listing
        .artifacts
        .iter()
        .filter(|a| a.name == wanted && !a.expired)
        .collect();
    named.sort_by(|a, b| (&b.created_at, b.id).cmp(&(&a.created_at, a.id)));
    let mut skipped = 0;
    for a in named {
        let trusted = a
            .workflow_run
            .as_ref()
            .and_then(|r| runs.iter().find(|run| run.id == r.id))
            .is_some_and(Run::is_main_review);
        if trusted {
            return Ok(Selection {
                artifact: Some(a.id),
                skipped,
            });
        }
        skipped += 1;
    }

    Ok(Selection {
        artifact: None,
        skipped,
    })
}

/// Checks a findings file's text.
///
/// # Errors
///
/// Fails when it is past [`MAX_BYTES`] or does not read as findings.
pub fn check(text: &str) -> Result<(), String> {
    if text.len() > MAX_BYTES {
        return Err(format!(
            "the findings file is {} bytes, past the {MAX_BYTES} a review reads",
            text.len()
        ));
    }
    findings::parse(text).map(drop)
}

/// The arguments after `previous` when `args`, past the job summary's
/// flag the executable adds first, name the `previous` command.
#[must_use]
pub fn subcommand(args: &[String]) -> Option<Vec<String>> {
    let mut rest = args.iter().skip_while(|a| a.starts_with("--summary="));
    (rest.next().map(String::as_str) == Some("previous")).then(|| rest.cloned().collect())
}

/// Runs `previous` with `args`, the arguments after it, and returns its
/// exit code.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let result = match args.first().map(String::as_str) {
        Some("select") => parse_select(&args[1..]).and_then(|(pull, a, r)| {
            let read =
                |path: &str| fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"));
            let selection = select(&pull, &read(&a)?, &read(&r)?)?;
            Ok(selection.lines())
        }),
        Some("check") => parse_check(&args[1..]).and_then(|path| {
            let text = fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?;
            check(&text).map_err(|e| format!("{path}: {e}"))?;
            Ok(String::new())
        }),
        _ => Err(Usage::Wrong("the commands are select and check".to_owned())),
    };
    match result {
        Ok(text) => match write!(stdout, "{text}") {
            Ok(()) => EXIT_OK,
            Err(e) => {
                let _ = writeln!(stderr, "review-run previous: say what it found: {e}");
                EXIT_FAILURE
            }
        },
        Err(Usage::Wrong(msg)) => {
            let _ = writeln!(stderr, "review-run previous: {msg}\n{USAGE}");
            EXIT_USAGE
        }
        Err(Usage::Failed(msg)) => {
            let _ = writeln!(stderr, "review-run previous: {msg}");
            EXIT_FAILURE
        }
    }
}

/// Why a command did not give its output.
enum Usage {
    /// The command line is wrong.
    Wrong(String),
    /// The command could not do what it was asked.
    Failed(String),
}

impl From<String> for Usage {
    fn from(msg: String) -> Self {
        Self::Failed(msg)
    }
}

/// Reads `--pull N --artifacts FILE --runs FILE`, in any order.
fn parse_select(args: &[String]) -> Result<(String, String, String), Usage> {
    let (mut pull, mut artifacts, mut runs) = (None, None, None);
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let slot = match arg.as_str() {
            "--pull" => &mut pull,
            "--artifacts" => &mut artifacts,
            "--runs" => &mut runs,
            other => return Err(Usage::Wrong(format!("unknown argument {other:?}"))),
        };
        *slot = Some(
            args.next()
                .cloned()
                .ok_or_else(|| Usage::Wrong(format!("{arg} needs a value")))?,
        );
    }
    match (pull, artifacts, runs) {
        (Some(p), Some(a), Some(r)) => Ok((p, a, r)),
        _ => Err(Usage::Wrong(
            "select needs --pull, --artifacts and --runs".to_owned(),
        )),
    }
}

/// Reads the one file `check` names.
fn parse_check(args: &[String]) -> Result<String, Usage> {
    match args {
        [path] => Ok(path.clone()),
        _ => Err(Usage::Wrong("check needs one FILE".to_owned())),
    }
}

#[cfg(test)]
mod tests;
