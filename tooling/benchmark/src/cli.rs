//! The `benchmark` command line, as the CI job runs it.
//!
//! - `benchmark run --core PATH [--samples N]` samples every benchmark
//!   of the suite `N` times against the core executable at `PATH` and
//!   writes the samples to stdout. The CI job runs the base commit's
//!   build and the head's with the same flags, so these stay stable: a
//!   change to them breaks the comparison until it reaches the base.
//! - `benchmark gate --base FILE --head FILE [--accepted FILE]
//!   [--accepted-on-base FILE]` decides from the two samples files,
//!   writes the report to stdout as a Markdown table, and fails naming
//!   each benchmark that regressed, went missing or has too few samples.
//!   A regression the head's acceptances cover, and the base's do not,
//!   passes as accepted (see [`accepted`](crate::accepted)); a missing
//!   base acceptances file counts as empty.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use crate::{accepted, gate, measure, samples, suite};

/// The command did what it was asked, and the gate, if it ran, found every benchmark within the threshold.
pub const EXIT_OK: u8 = 0;
/// A sample could not be taken, or the gate failed.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

const USAGE: &str = "usage: benchmark run --core PATH [--samples N]\n       \
                     benchmark gate --base FILE --head FILE [--accepted FILE] [--accepted-on-base FILE]";

/// How many samples `run` takes of each benchmark unless told.
pub const DEFAULT_SAMPLES: usize = 10;

/// Reads a file's text by its path.
pub type Read<'a> = &'a dyn Fn(&str) -> io::Result<String>;

/// Times one sample of a benchmark: the core executable at a path,
/// started with the benchmark's arguments over its fixture, until it
/// exits.
pub type Time<'a> = &'a dyn Fn(&Path, &suite::Benchmark) -> Result<Duration, String>;

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `read` maps a path to a file's text, `time`
/// times one start of a program, and every test drives the command
/// through here.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    read: Read<'_>,
    time: Time<'_>,
) -> u8 {
    let mut args = args.into_iter();
    let command = args.next().unwrap_or_default();
    let rest: Vec<String> = args.collect();
    let done = match command.as_str() {
        "run" => flags(&rest, &["core", "samples"])
            .and_then(|f| run_options(&f))
            .map(|(core, count)| sample_suite(&core, count, stdout, time)),
        "gate" => flags(&rest, &["base", "head", "accepted", "accepted-on-base"])
            .and_then(|f| Ok((required(&f, "base")?, required(&f, "head")?, f)))
            .map(|(base, head, f)| {
                let acceptances = (f.get("accepted"), f.get("accepted-on-base"));
                decide(&base, &head, acceptances, stdout, read)
            }),
        _ => Err(format!("unknown command {command:?}")),
    };

    match done {
        Err(usage) => {
            let _ = writeln!(stderr, "benchmark: {usage}\n{USAGE}");
            EXIT_USAGE
        }
        Ok(Ok(())) => EXIT_OK,
        Ok(Err(problems)) => {
            for p in problems {
                let _ = writeln!(stderr, "benchmark: {p}");
            }
            EXIT_FAILURE
        }
    }
}

/// `run`'s core executable and sample count.
fn run_options(f: &BTreeMap<String, String>) -> Result<(String, usize), String> {
    let core = required(f, "core")?;
    let count = match f.get("samples") {
        None => DEFAULT_SAMPLES,
        Some(n) => n
            .parse()
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| format!("--samples {n:?} is not a positive whole number"))?,
    };

    Ok((core, count))
}

/// Samples every benchmark of the suite and writes the samples file.
fn sample_suite(
    core: &str,
    count: usize,
    stdout: &mut dyn Write,
    time: Time<'_>,
) -> Result<(), Vec<String>> {
    let core = Path::new(core);
    let mut out = format!("{}\n", samples::HEADER);
    for benchmark in &suite::SUITE {
        let taken =
            measure::sample(benchmark, count, &mut |b| time(core, b)).map_err(|e| vec![e])?;
        out.push_str(&samples::render(benchmark.name, &taken));
    }

    stdout
        .write_all(out.as_bytes())
        .map_err(|e| vec![format!("write the samples: {e}")])
}

/// Decides from the two samples files and writes the report.
fn decide(
    base: &str,
    head: &str,
    (accepted, accepted_on_base): (Option<&String>, Option<&String>),
    stdout: &mut dyn Write,
    read: Read<'_>,
) -> Result<(), Vec<String>> {
    let load = |path: &str| {
        read(path)
            .map_err(|e| format!("read {path}: {e}"))
            .and_then(|text| samples::parse(&text).map_err(|e| format!("{path}: {e}")))
            .map_err(|e| vec![e])
    };
    let (base, head) = (load(base)?, load(head)?);
    let acceptances = |path: Option<&String>, missing_is_empty: bool| {
        let Some(path) = path else {
            return Ok(Vec::new());
        };
        let text = match read(path) {
            Err(e) if missing_is_empty && e.kind() == io::ErrorKind::NotFound => String::new(),
            other => other.map_err(|e| vec![format!("read {path}: {e}")])?,
        };
        accepted::parse(&text).map_err(|e| vec![format!("{path}: {e}")])
    };
    let fresh = accepted::fresh(
        &acceptances(accepted, false)?,
        &acceptances(accepted_on_base, true)?,
    );

    let comparisons = gate::accept(gate::decide(&base, &head, &suite::RETIRED), &fresh);
    let report = gate::report(&comparisons);
    stdout
        .write_all(report.as_bytes())
        .map_err(|e| vec![format!("write the report: {e}")])?;

    let failures = gate::failures(&comparisons);
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

/// Reads `--name value` and `--name=value` pairs, each name one of
/// `known`.
fn flags(args: &[String], known: &[&str]) -> Result<BTreeMap<String, String>, String> {
    let mut values = BTreeMap::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let flag = arg
            .strip_prefix("--")
            .ok_or_else(|| format!("unexpected argument {arg:?}"))?;
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_owned())),
            None => (flag, None),
        };
        if !known.contains(&name) {
            return Err(format!("unknown flag {arg:?}"));
        }
        let value = inline
            .or_else(|| args.next().cloned())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("--{name} needs a value"))?;
        values.insert(name.to_owned(), value);
    }

    Ok(values)
}

/// The value of a required flag.
fn required(flags: &BTreeMap<String, String>, name: &str) -> Result<String, String> {
    flags
        .get(name)
        .cloned()
        .ok_or_else(|| format!("--{name} is required"))
}

#[cfg(test)]
mod tests;
