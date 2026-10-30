//! The `review-run` command line: a work directory in, the one the
//! review workflow prepares (main's checkout with the pull request's
//! tree in `pr-head/`, its diff in `pr.diff`, the bundle's printed
//! output in `class.txt` and what it wrote under `review/`); the
//! review outcome, the agents consulted, their answers and their usage
//! written under `review/`, with each run's log under `review/logs/`.
//! It prints the verdict and the counts for the workflow's step
//! outputs, and each run as it ends on standard error.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use review_eval::host::Host;
use serde::Serialize;

use crate::outcome::{Decision, decide};
use crate::plan::{self, report_file};
use crate::run::{self, Options};
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
pub const USAGE: &str = "usage: review-run --workdir DIR [--parallel N] [--deadline SECONDS]";

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `host` starts the agents.
pub fn run(
    host: &dyn Host,
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut (dyn Write + Send),
) -> u8 {
    let (workdir, options) = match parse(args) {
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
    match review(host, Path::new(&workdir), options, &progress) {
        Ok(printed) => {
            let _ = write!(stdout, "{printed}");
            EXIT_OK
        }
        Err(msg) => {
            progress(&msg);
            EXIT_FAILURE
        }
    }
}

/// Runs the review in `workdir` and writes its files; returns the step
/// outputs to print.
fn review(
    host: &dyn Host,
    workdir: &Path,
    options: Options,
    progress: &(dyn Fn(&str) + Sync),
) -> Result<String, String> {
    let plan = plan::read(workdir)?;
    let runs = run::review(host, workdir, &plan, options, progress)?;
    let reports: BTreeMap<String, String> = plan
        .specialists()
        .into_iter()
        .filter_map(|name| {
            fs::read_to_string(workdir.join(report_file(&name)))
                .ok()
                .map(|text| (name, text))
        })
        .collect();
    let Decision {
        outcome,
        consulted,
        answers,
    } = decide(&plan, &runs, &reports);
    let usage = usage::usage(&runs);
    write_json(workdir, OUTCOME, &outcome)?;
    write_json(workdir, CONSULTED, &consulted)?;
    write_json(workdir, ANSWERS, &answers)?;
    write_json::<Vec<AgentUsage>>(workdir, USAGE_FILE, &usage)?;

    Ok(format!(
        "verdict={}\ndispatched={}\nconsulted={}\nfindings={}\ncost={:.4}\n",
        outcome.verdict,
        plan.agents().len(),
        consulted.len(),
        outcome.findings.len(),
        usage::total(&usage)
    ))
}

/// Writes `value` as JSON to `rel` under `workdir`.
fn write_json<T: Serialize>(workdir: &Path, rel: &str, value: &T) -> Result<(), String> {
    let path = workdir.join(rel);
    // Maps of strings, lists and plain structs always encode; were one
    // not to, the empty file fails the gate's read of it.
    let text = serde_json::to_string_pretty(value).unwrap_or_default();
    fs::write(&path, format!("{text}\n")).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Reads `--name value` and `--name=value` pairs: the work directory,
/// required, and the bound and the deadline, each a positive number.
fn parse(args: impl IntoIterator<Item = String>) -> Result<(String, Options), String> {
    let mut workdir = None;
    let mut options = Options::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let Some(flag) = arg.strip_prefix("--") else {
            return Err(format!("unexpected argument {arg:?}"));
        };
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_owned())),
            None => (flag, None),
        };
        if !["workdir", "parallel", "deadline"].contains(&name) {
            return Err(format!("unknown flag {arg:?}"));
        }
        let value = inline
            .or_else(|| args.next())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("--{name} needs a value"))?;
        match name {
            "workdir" => workdir = Some(value),
            "parallel" => options.parallel = positive(name, &value)?,
            _ => options.deadline = Duration::from_secs(positive(name, &value)?),
        }
    }
    let workdir = workdir.ok_or("--workdir is required")?;

    Ok((workdir, options))
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
