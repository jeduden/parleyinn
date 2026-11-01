//! The `review-run` command line: a work directory in, the one the
//! review workflow prepares (main's checkout with the pull request's
//! tree in `pr-head/`, its diff in `pr.diff`, the bundle's printed
//! output in `class.txt` and what it wrote under `review/`), whose
//! SRS in both trees names the ids a specialist's finding may cite; the
//! review outcome, the agents consulted, their answers and their usage
//! written under `review/`, with each run's log under `review/logs/`
//! and each specialist's report, assembled from its log, under
//! `review/reports/`.
//! It also writes what the review did not review, `review/gaps.json`
//! and `review/not-reviewed.md`, and what it ran on,
//! `review/run-id.json`. Given `--only` a gap list and `--from` an
//! earlier run's review directory, it runs only what that run left
//! without an accepted answer, or whose inputs changed, and decides the
//! review as a full run would.
//! It prints the verdict and the counts for the workflow's step
//! outputs, and each run as it ends on standard error, then what was
//! not reviewed.
//! Given `--credential` the directory a staging step wrote the access
//! token into, it points every agent there and refuses it every read of
//! it; every text it writes or prints is first scrubbed of that token
//! and of the one it holds itself, and a review that found one requests
//! changes, names the leak in a blocking finding and fails (ENG-28).

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use review_eval::credential::{self, Confined};
use review_eval::host::Host;
use review_eval::secret::Scrubber;
use serde::Serialize;

use crate::gaps::{self, GAPS, Gap, NOT_REVIEWED};
use crate::leak;
use crate::outcome::{Decision, decide};
use crate::plan::{self, report_file};
use crate::requirements;
use crate::resume::{self, Resume};
use crate::run::{self, AgentRun, Options};
use crate::run_id::{self, RUN_ID};
use crate::usage::{self, AgentUsage};
use crate::{ANSWERS, CONSULTED, OUTCOME, USAGE as USAGE_FILE};

/// The review was decided and its files written, whatever it decided.
pub const EXIT_OK: u8 = 0;
/// The review could not run: its plan does not read, or a file could
/// not be written.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

/// How to call it.
pub const USAGE: &str = "usage: review-run --workdir DIR [--parallel N] [--deadline SECONDS] \
[--head-sha SHA] [--only GAPS --from DIR] [--credential DIR]";

/// The command line, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub workdir: String,
    pub options: Options,
    /// The head commit reviewed, when given; else `pr-head`'s.
    pub head: Option<String>,
    /// The gap list and the earlier run's review directory a resumed
    /// review runs from.
    pub resume: Option<(String, String)>,
    /// The directory the Claude token was staged in, when given.
    pub credential: Option<String>,
}

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `host` starts the agents. A review that could
/// not run prints why as `failed=<reason>` in place of its outputs.
pub fn run(
    host: &dyn Host,
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut (dyn Write + Send),
) -> u8 {
    run_with(host, args, None, stdout, stderr)
}

/// [`run`], handed `token`, the Claude token the process holds, when
/// it holds one: a form every text it writes or prints is scrubbed of.
pub fn run_with(
    host: &dyn Host,
    args: impl IntoIterator<Item = String>,
    token: Option<&str>,
    stdout: &mut dyn Write,
    stderr: &mut (dyn Write + Send),
) -> u8 {
    let args = match parse(args) {
        Ok(parsed) => parsed,
        Err(msg) => {
            let _ = writeln!(stderr, "review-run: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };
    let errors = Mutex::new(stderr);
    let progress = |line: &str| {
        if let Ok(mut out) = errors.lock() {
            let _ = writeln!(out, "review-run: {line}");
        }
    };
    let guarded = guard(&args, token).and_then(|(scrubber, located)| {
        let confined = located.as_deref().map(|dir| Confined::new(host, dir));
        let host: &dyn Host = match &confined {
            Some(confined) => confined,
            None => host,
        };
        let leaks = AtomicUsize::new(0);
        let scrubbed = |line: &str| {
            let (line, n) = scrubber.scrub(line);
            leaks.fetch_add(n, Ordering::SeqCst);
            progress(&line);
        };
        let reviewed =
            review(host, &args, &scrubber, &leaks, &scrubbed).map_err(|msg| scrubber.scrub(&msg).0);
        // The trace and the report page, after every run, best effort:
        // built from the files above, already scrubbed, and scrubbed
        // again, so a form they hold fails the review (ENG-28).
        let failure = reviewed.as_ref().err().map(String::as_str);
        let found = leak::report(&scrubber, Path::new(&args.workdir), failure, &scrubbed);
        match (reviewed, found) {
            (Ok(_), n) if n > 0 => Err(leak::reason(n)),
            (reviewed, _) => reviewed,
        }
    });
    match guarded {
        // The step outputs carry the verdict to the workflow: one that
        // does not print fails the step rather than leave them empty (I6).
        Ok(printed) => match write!(stdout, "{printed}") {
            Ok(()) => EXIT_OK,
            Err(e) => {
                progress(&format!("print the step outputs: {e}"));
                EXIT_FAILURE
            }
        },
        Err(msg) => {
            progress(&msg);
            // Printed too, for the job summary of a review that failed;
            // the line above already holds it should this not print.
            let _ = writeln!(stdout, "failed={}", msg.replace('\n', " "));
            EXIT_FAILURE
        }
    }
}

/// The scrubber for the token staged under `args`' credential and
/// `token`, and that credential's directory, located.
///
/// # Errors
///
/// Fails when the credential does not resolve or read, or a token is
/// too short for the scrub to match.
fn guard(args: &Args, token: Option<&str>) -> Result<(Scrubber, Option<PathBuf>), String> {
    let located = args
        .credential
        .as_deref()
        .map(|dir| credential::locate(Path::new(dir)))
        .transpose()?;
    let staged = located.as_deref().map(credential::read).transpose()?;
    let secrets: Vec<&str> = staged.as_deref().into_iter().chain(token).collect();

    Ok((Scrubber::guarded(&secrets)?, located))
}

/// Runs the review `args` name and writes its files, each scrubbed by
/// `scrubber`, counting into `leaks` every form of the token it
/// replaced; returns the step outputs to print.
///
/// # Errors
///
/// Fails when the review could not run, or found the token.
fn review(
    host: &dyn Host,
    args: &Args,
    scrubber: &Scrubber,
    leaks: &AtomicUsize,
    progress: &(dyn Fn(&str) + Sync),
) -> Result<String, String> {
    let workdir = Path::new(&args.workdir);
    let options = args.options;
    let plan = plan::read(workdir)?;
    // Read before any agent runs: an SRS that does not read fails the
    // review before it costs anything.
    let known = requirements::ids(workdir)?;
    let head = args
        .head
        .clone()
        .or_else(|| run_id::git_head(&workdir.join("pr-head")));
    let mut id = run_id::compute(workdir, &plan, head, options.split);
    let resume = match &args.resume {
        Some((only, from)) => {
            let resume = resumed(workdir, &plan, &id, only, from, progress)?;
            id.resumed_from = Some(from.clone());
            Some(resume)
        }
        None => None,
    };
    let mut runs = run::review(
        host,
        workdir,
        &plan,
        &known,
        options,
        resume.as_ref(),
        progress,
    )?;
    // Every text an agent produced is scrubbed before anything reads it
    // as the review's: its logs and reports on disk, its runs in memory.
    let mut leaked = leak::scrub_runs(scrubber, &mut runs);
    for dir in [crate::LOGS, crate::REPORTS] {
        leaked += scrubber.scrub_tree(&workdir.join(dir))?;
    }
    leaks.fetch_add(leaked, Ordering::SeqCst);
    let written = Written {
        workdir,
        scrubber,
        leaks,
    };
    let reports = reports(workdir, &plan, progress);
    let Decision {
        mut outcome,
        consulted,
        answers,
        not_reviewed,
    } = decide(&plan, &runs, &reports, &known);
    let usage = usage::usage(&runs, resume.as_ref().map(|r| r.from.as_str()));
    written.json(CONSULTED, &consulted)?;
    written.json(ANSWERS, &answers)?;
    written.json::<Vec<AgentUsage>>(USAGE_FILE, &usage)?;
    written.json::<Vec<Gap>>(GAPS, &not_reviewed)?;
    written.text(NOT_REVIEWED, &gaps::markdown(&not_reviewed))?;
    written.json(RUN_ID, &id)?;
    // The outcome goes last, so a leak any file above held is in it.
    let leaked = leaks.load(Ordering::SeqCst);
    if leaked > 0 {
        leak::fail_closed(&mut outcome, leaked);
    }
    written.json(OUTCOME, &outcome)?;
    if let Some(resume) = &resume {
        progress(&resumed_line(resume, &runs));
    }
    // What has no accepted report is said last, never left for a
    // reader to take as clean.
    if !not_reviewed.is_empty() {
        let names: Vec<String> = not_reviewed.iter().map(Gap::label).collect();
        progress(&format!("not reviewed: {}", names.join(", ")));
    }
    let leaked = leaks.load(Ordering::SeqCst);
    if leaked > 0 {
        return Err(leak::reason(leaked));
    }

    Ok(format!(
        "verdict={}\ndispatched={}\nconsulted={}\nfindings={}\ncost={:.4}\n",
        outcome.verdict,
        plan.agents().len(),
        consulted.len(),
        outcome.findings.len(),
        usage::total(&usage)
    ))
}

/// Each of `plan`'s specialists' reports under `workdir`, by name, when
/// it wrote one; a report that is there but does not read stands as
/// none, and `progress` is told why (I6).
fn reports(
    workdir: &Path,
    plan: &plan::Plan,
    progress: &(dyn Fn(&str) + Sync),
) -> BTreeMap<String, String> {
    plan.specialists()
        .into_iter()
        .filter_map(|name| {
            let path = workdir.join(report_file(&name));
            match fs::read_to_string(&path) {
                Ok(text) => Some((name, text)),
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::NotFound {
                        progress(&format!("read {}: {e}", path.display()));
                    }
                    None
                }
            }
        })
        .collect()
}

/// Reads the earlier run in `from` for the gap list `only`, plans what
/// the review of `plan` on `id` in `workdir` keeps of it, tells
/// `progress` why each other agent runs again, and copies what it keeps.
///
/// # Errors
///
/// Fails when the earlier run does not read, is this run's own review
/// directory, or what it keeps cannot be copied.
fn resumed(
    workdir: &Path,
    plan: &plan::Plan,
    id: &run_id::RunId,
    only: &str,
    from: &str,
    progress: &(dyn Fn(&str) + Sync),
) -> Result<Resume, String> {
    let own = workdir.join(crate::REVIEW);
    if fs::canonicalize(from).ok() == fs::canonicalize(&own).ok() {
        return Err(format!(
            "--from {from} is this run's own review directory, which it writes over"
        ));
    }
    let earlier = resume::read(Path::new(only), Path::new(from))?;
    let plan_of = resume::plan(plan, &earlier, id, from);
    for note in &plan_of.notes {
        progress(note);
    }
    let left = resume::copy(workdir, Path::new(from), plan, &plan_of)?;
    left.iter().for_each(|note| progress(note));

    Ok(plan_of)
}

/// The line on what a resumed review kept and ran again.
fn resumed_line(resume: &Resume, runs: &[AgentRun]) -> String {
    let reused = runs.iter().filter(|r| r.reused).count();
    let again: Vec<&str> = runs
        .iter()
        .filter(|r| !r.reused)
        .map(|r| r.name.as_str())
        .collect();
    format!(
        "resumed from {}: reused {reused} agents, re-ran {} ({})",
        resume.from,
        again.len(),
        again.join(", ")
    )
}

/// Where the review writes its files, each scrubbed first, every form
/// of the token it replaced counted into `leaks`.
struct Written<'a> {
    workdir: &'a Path,
    scrubber: &'a Scrubber,
    leaks: &'a AtomicUsize,
}

impl Written<'_> {
    /// Writes `text`, scrubbed, to `rel` under the work directory.
    fn text(&self, rel: &str, text: &str) -> Result<(), String> {
        let path = self.workdir.join(rel);
        let (text, n) = self.scrubber.scrub(text);
        self.leaks.fetch_add(n, Ordering::SeqCst);
        fs::write(&path, text).map_err(|e| format!("write {}: {e}", path.display()))
    }

    /// Writes `value` as JSON, scrubbed, to `rel` under the work
    /// directory.
    fn json<T: Serialize>(&self, rel: &str, value: &T) -> Result<(), String> {
        let text = serde_json::to_string_pretty(value)
            .map_err(|e| format!("encode {}: {e}", self.workdir.join(rel).display()))?;
        self.text(rel, &format!("{text}\n"))
    }
}

/// Reads `--name value` and `--name=value` pairs: the work directory,
/// required, the bound and the deadline, each a positive number, the
/// head commit, and the gap list and the earlier run, given together.
fn parse(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut workdir = None;
    let mut options = Options::default();
    let (mut head, mut only, mut from, mut credential) = (None, None, None, None);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let Some(flag) = arg.strip_prefix("--") else {
            return Err(format!("unexpected argument {arg:?}"));
        };
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_owned())),
            None => (flag, None),
        };
        if ![
            "workdir",
            "parallel",
            "deadline",
            "head-sha",
            "only",
            "from",
            "credential",
        ]
        .contains(&name)
        {
            return Err(format!("unknown flag {arg:?}"));
        }
        let value = inline
            .or_else(|| args.next())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("--{name} needs a value"))?;
        match name {
            "workdir" => workdir = Some(value),
            "parallel" => options.parallel = positive(name, &value)?,
            "deadline" => options.deadline = Duration::from_secs(positive(name, &value)?),
            "head-sha" if run_id::is_commit(&value) => head = Some(value),
            "head-sha" => return Err(format!("--head-sha {value:?} is no commit id")),
            "only" => only = Some(value),
            "credential" => credential = Some(value),
            _ => from = Some(value),
        }
    }
    let workdir = workdir.ok_or("--workdir is required")?;
    let resume = match (only, from) {
        (Some(only), Some(from)) => Some((only, from)),
        (None, None) => None,
        _ => return Err("--only and --from are given together".to_owned()),
    };

    Ok(Args {
        workdir,
        options,
        head,
        resume,
        credential,
    })
}

/// `value`, flag `name`'s, as a number above zero.
fn positive<T: std::str::FromStr + PartialOrd + Default>(
    name: &str,
    value: &str,
) -> Result<T, String> {
    value
        .parse()
        .ok()
        .filter(|n| *n > T::default())
        .ok_or_else(|| format!("--{name} {value:?} is not a number above zero"))
}

#[cfg(test)]
mod tests;
