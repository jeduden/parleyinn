//! The `review-run previous` command: which earlier review of a pull
//! request its findings are carried from (ENG-28). `previous select`
//! reads the artifacts GitHub lists under a pull request's name and the
//! runs that uploaded them, and names the newest one a run of main's
//! review workflow uploaded; an artifact of that name from any other
//! run is skipped and counted, since any workflow a pull request
//! defines may upload one. `previous check` refuses a findings file
//! that is too large or does not read, so a broken history is named
//! and never taken for an empty one (I6). Each review keeps what it
//! spent alone, so `previous select --of spend` names every artifact of
//! the pull request's spend that a run of main's review workflow
//! uploaded, by the same origin rule, but for the review's own run, and
//! `previous spend` reads each and sums them, refusing a spend file that
//! is too large or does not read, so it is never taken for nothing
//! spent. Two reviews of one pull request that run side by side, or one
//! cancelled, each keep their own, so neither erases the other's. The
//! tool decides and the caller's script only fetches.

use std::fs;
use std::io::Write;

use serde::Deserialize;

use crate::cli::{EXIT_FAILURE, EXIT_OK, EXIT_USAGE};
use crate::findings;

/// How to call it.
pub const USAGE: &str = "usage: review-run previous select --pull N --artifacts FILE --runs FILE \
[--of findings|spend [--except-run ID]]\n\
       review-run previous check FILE\n\
       review-run previous spend FILE...";

/// The workflow whose runs may upload the findings a review carries.
pub const WORKFLOW_PATH: &str = ".github/workflows/review.yml";
/// The events on which main defines that workflow's run: a run on any
/// other event, `pull_request` among them, may use the pull request's
/// own definition.
pub const EVENTS: [&str; 2] = ["workflow_run", "issue_comment"];
/// The most bytes a findings file may hold.
pub const MAX_BYTES: usize = 1 << 20;
/// How the name of the artifact a pull request's findings are kept
/// under starts, its number after it.
pub const FINDINGS: &str = "review-findings-pr-";
/// How the name of the artifact what a pull request's reviews spent is
/// kept under starts, its number after it.
pub const SPEND: &str = "review-spend-pr-";

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

/// The most earlier reviews' spend a review sums: past it the step
/// fails, naming how many, rather than leave any out.
pub const MAX_SPENDS: usize = 100;

/// What the lookup of the earlier reviews' spend found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spends {
    /// The artifacts to sum, each kept by one earlier review, by id.
    pub artifacts: Vec<u64>,
    /// The artifacts of the name, unexpired, refused for their origin.
    pub skipped: u32,
}

impl Spends {
    /// The lines the job's outputs take: the artifacts, space apart.
    #[must_use]
    pub fn lines(&self) -> String {
        let ids: Vec<String> = self.artifacts.iter().map(u64::to_string).collect();
        format!(
            "spend_artifacts={}\nprevious_skipped={}\n",
            ids.join(" "),
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
    /// Absent or null for an artifact GitHub gives no time, which then
    /// ranks after every one that has one.
    #[serde(default)]
    created_at: Option<String>,
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
    name_of(FINDINGS, pull)
}

/// The artifact name starting `prefix` that pull request `pull`'s
/// reviews keep.
///
/// # Errors
///
/// Fails when `pull` is not a pull request number.
pub fn name_of(prefix: &str, pull: &str) -> Result<String, String> {
    if pull.is_empty() || !pull.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("{pull:?} is not a pull request number"));
    }

    Ok(format!("{prefix}{pull}"))
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
    select_of(FINDINGS, pull, artifacts, runs)
}

/// [`select`] of the artifact whose name starts `prefix`.
///
/// # Errors
///
/// Fails when either input does not read, or `pull` is no number.
pub fn select_of(
    prefix: &str,
    pull: &str,
    artifacts: &str,
    runs: &str,
) -> Result<Selection, String> {
    let wanted = name_of(prefix, pull)?;
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

/// Picks every artifact of the pull request `pull`'s spend to sum:
/// `artifacts` and `runs` as [`select`] reads them. Each artifact of the
/// name, not expired and not uploaded by the run `except`, the review's
/// own, is taken when its run is one of main's review workflow, and
/// otherwise refused and counted, its run unknown included; sorted by
/// id.
///
/// # Errors
///
/// Fails when either input does not read, `pull` is no number, or more
/// than [`MAX_SPENDS`] artifacts qualify.
pub fn spends(
    pull: &str,
    artifacts: &str,
    runs: &str,
    except: Option<u64>,
) -> Result<Spends, String> {
    let wanted = name_of(SPEND, pull)?;
    let listing: Listing =
        serde_json::from_str(artifacts).map_err(|e| format!("artifacts do not read: {e}"))?;
    let runs: Vec<Run> =
        serde_json::from_str(runs).map_err(|e| format!("runs do not read: {e}"))?;
    let mut out = Spends {
        artifacts: Vec::new(),
        skipped: 0,
    };
    for a in listing
        .artifacts
        .iter()
        .filter(|a| a.name == wanted && !a.expired)
    {
        let run = a.workflow_run.as_ref().map(|r| r.id);
        if run.is_some() && run == except {
            continue;
        }
        let trusted = run
            .and_then(|id| runs.iter().find(|r| r.id == id))
            .is_some_and(Run::is_main_review);
        if trusted {
            out.artifacts.push(a.id);
        } else {
            out.skipped += 1;
        }
    }
    if out.artifacts.len() > MAX_SPENDS {
        return Err(format!(
            "{} earlier reviews of pull request {pull} kept what they spent, past the \
             {MAX_SPENDS} a review sums",
            out.artifacts.len()
        ));
    }
    out.artifacts.sort_unstable();

    Ok(out)
}

/// What the spend files' `texts` say was spent, summed.
///
/// # Errors
///
/// Fails naming the first that [`crate::spend::check`] refuses.
pub fn summed(texts: &[(String, String)]) -> Result<f64, String> {
    texts.iter().try_fold(0.0, |sum, (path, text)| {
        crate::spend::check(text)
            .map(|spent| sum + spent)
            .map_err(|e| format!("{path}: {e}"))
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
        Some("select") => parse_select(&args[1..]).and_then(|s| {
            let (a, r) = (read(&s.artifacts)?, read(&s.runs)?);
            Ok(match s.except {
                Some(except) => spends(&s.pull, &a, &r, except)?.lines(),
                None => select_of(s.prefix, &s.pull, &a, &r)?.lines(),
            })
        }),
        Some("spend") => spend_of(&args[1..]),
        Some("check") => parse_check(&args[1..]).and_then(|path| {
            check(&read(&path)?).map_err(|e| format!("{path}: {e}"))?;
            Ok(String::new())
        }),
        _ => Err(Usage::Wrong(
            "the commands are select, check and spend".to_owned(),
        )),
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

/// What the spend files at `paths` say was spent, summed, as the
/// job's output takes it.
fn spend_of(paths: &[String]) -> Result<String, Usage> {
    if paths.is_empty() {
        return Err(Usage::Wrong("spend needs a FILE or more".to_owned()));
    }
    let texts = paths
        .iter()
        .map(|p| read(p).map(|text| (p.clone(), text)))
        .collect::<Result<Vec<_>, String>>()?;

    Ok(format!("spent_before={}\n", summed(&texts)?))
}

/// The text of the file at `path`.
fn read(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))
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

/// What `select` was asked: the name's start, the pull request, the two
/// files and, for the spend, the review's own run to leave out, if any.
struct Select {
    prefix: &'static str,
    pull: String,
    artifacts: String,
    runs: String,
    /// Set for the spend alone: the run to leave out.
    except: Option<Option<u64>>,
}

/// Reads `--pull N --artifacts FILE --runs FILE [--of findings|spend
/// [--except-run ID]]`, in any order.
fn parse_select(args: &[String]) -> Result<Select, Usage> {
    let (mut pull, mut artifacts, mut runs, mut of, mut except) = (None, None, None, None, None);
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let slot = match arg.as_str() {
            "--pull" => &mut pull,
            "--artifacts" => &mut artifacts,
            "--runs" => &mut runs,
            "--of" => &mut of,
            "--except-run" => &mut except,
            other => return Err(Usage::Wrong(format!("unknown argument {other:?}"))),
        };
        *slot = Some(
            args.next()
                .cloned()
                .ok_or_else(|| Usage::Wrong(format!("{arg} needs a value")))?,
        );
    }
    let prefix = match of.as_deref() {
        None | Some("findings") => FINDINGS,
        Some("spend") => SPEND,
        Some(other) => {
            return Err(Usage::Wrong(format!(
                "--of {other:?} is neither findings nor spend"
            )));
        }
    };
    let except = match (prefix, except) {
        (SPEND, None) => Some(None),
        (SPEND, Some(id)) => {
            Some(Some(id.parse().map_err(|_| {
                Usage::Wrong(format!("--except-run {id:?} is no run id"))
            })?))
        }
        (_, None) => None,
        (_, Some(_)) => {
            return Err(Usage::Wrong(
                "--except-run is for --of spend alone".to_owned(),
            ));
        }
    };
    match (pull, artifacts, runs) {
        (Some(pull), Some(artifacts), Some(runs)) => Ok(Select {
            prefix,
            pull,
            artifacts,
            runs,
            except,
        }),
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
