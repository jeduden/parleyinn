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

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use review_eval::host::Host;
use serde::Serialize;

use crate::gaps::{self, GAPS, Gap, NOT_REVIEWED};
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
[--head-sha SHA] [--only GAPS --from DIR]";

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
}

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `host` starts the agents.
pub fn run(
    host: &dyn Host,
    args: impl IntoIterator<Item = String>,
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
    match review(host, &args, &progress) {
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
            EXIT_FAILURE
        }
    }
}

/// Runs the review `args` name and writes its files; returns the step
/// outputs to print.
fn review(
    host: &dyn Host,
    args: &Args,
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
    let runs = run::review(
        host,
        workdir,
        &plan,
        &known,
        options,
        resume.as_ref(),
        progress,
    )?;
    let reports: BTreeMap<String, String> = plan
        .specialists()
        .into_iter()
        .filter_map(|name| {
            let path = workdir.join(report_file(&name));
            match fs::read_to_string(&path) {
                Ok(text) => Some((name, text)),
                // A report that is there but does not read stands as
                // none, and the cause is named (I6).
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::NotFound {
                        progress(&format!("read {}: {e}", path.display()));
                    }
                    None
                }
            }
        })
        .collect();
    let Decision {
        outcome,
        consulted,
        answers,
        not_reviewed,
    } = decide(&plan, &runs, &reports, &known);
    let usage = usage::usage(&runs, resume.as_ref().map(|r| r.from.as_str()));
    write_json(workdir, OUTCOME, &outcome)?;
    write_json(workdir, CONSULTED, &consulted)?;
    write_json(workdir, ANSWERS, &answers)?;
    write_json::<Vec<AgentUsage>>(workdir, USAGE_FILE, &usage)?;
    write_json::<Vec<Gap>>(workdir, GAPS, &not_reviewed)?;
    write_text(workdir, NOT_REVIEWED, &gaps::markdown(&not_reviewed))?;
    write_json(workdir, RUN_ID, &id)?;
    if let Some(resume) = &resume {
        progress(&resumed_line(resume, &runs));
    }
    // What has no accepted report is said last, never left for a
    // reader to take as clean.
    if !not_reviewed.is_empty() {
        let names: Vec<String> = not_reviewed.iter().map(Gap::label).collect();
        progress(&format!("not reviewed: {}", names.join(", ")));
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

/// Writes `text` to `rel` under `workdir`.
fn write_text(workdir: &Path, rel: &str, text: &str) -> Result<(), String> {
    let path = workdir.join(rel);
    fs::write(&path, text).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Writes `value` as JSON to `rel` under `workdir`.
fn write_json<T: Serialize>(workdir: &Path, rel: &str, value: &T) -> Result<(), String> {
    let path = workdir.join(rel);
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| format!("encode {}: {e}", path.display()))?;
    fs::write(&path, format!("{text}\n")).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Reads `--name value` and `--name=value` pairs: the work directory,
/// required, the bound and the deadline, each a positive number, the
/// head commit, and the gap list and the earlier run, given together.
fn parse(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut workdir = None;
    let mut options = Options::default();
    let (mut head, mut only, mut from) = (None, None, None);
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
            "workdir", "parallel", "deadline", "head-sha", "only", "from",
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
