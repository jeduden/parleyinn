//! ENG-15: a benchmark suite covers NFR-01..05, and a CI gate fails on
//! a statistically significant regression greater than 10%. The suite
//! and the gate live in the `benchmark` crate; these checks hold the
//! repository to them: the suite registers every NFR id ENG-15 names,
//! either with a benchmark of real code or declared awaiting one, and
//! the CI workflow runs the suite and the gate in a job the `CI` check
//! needs.

use std::io;
use std::path::Path;

pub use benchmark::suite;

use benchmark::cli;
use benchmark::samples;
use benchmark::suite::{Awaiting, Benchmark, Coverage};

use crate::review_workflow::Workflow;

/// The CI job that runs the suite and the gate.
pub const JOB: &str = "benchmark";

/// The job every other job joins, whose result the rulesets require.
pub const AGGREGATOR: &str = "ci";

/// What the job must run: each side's suite, then the head's gate.
pub const RUNS: [&str; 2] = ["benchmark\" run --core", "benchmark\" gate --base"];

/// The ids with `prefix` that `text` names, in order and once each; a
/// range such as `NFR-01..05` names every id from its first to its last.
#[must_use]
pub fn named_ids(text: &str, prefix: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    let lead = format!("{prefix}-");
    for (at, _) in text.match_indices(&lead) {
        let rest = &text[at + lead.len()..];
        let Some(first) = two_digits(rest) else {
            continue;
        };
        let last = rest[2..]
            .strip_prefix("..")
            .and_then(two_digits)
            .unwrap_or(first);
        for n in first..=last {
            let id = format!("{lead}{n:02}");
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }

    ids
}

/// The number the two digits at the start of `text` spell.
fn two_digits(text: &str) -> Option<u32> {
    let digits = text.get(..2)?;
    if digits.bytes().all(|b| b.is_ascii_digit()) {
        digits.parse().ok()
    } else {
        None
    }
}

/// Requires each of `ids` to have a benchmark of real code or to be
/// declared awaiting one, never both, and the suite to register nothing
/// against an id outside `ids`.
#[must_use]
pub fn registered(ids: &[String], suite: &[Benchmark], awaiting: &[Awaiting]) -> Vec<String> {
    let mut problems = Vec::new();
    for id in ids {
        match (covered(id, suite), awaited(id, awaiting).is_some()) {
            (false, false) => problems.push(format!(
                "{id} has no benchmark of real code and is not declared awaiting one"
            )),
            (true, true) => problems.push(format!(
                "{id} has a benchmark of real code yet is still declared awaiting one"
            )),
            _ => {}
        }
    }
    for b in suite
        .iter()
        .filter(|b| !ids.contains(&b.requirement.to_owned()))
    {
        problems.push(format!(
            "benchmark {} registers against {}, which ENG-15 does not name",
            b.name, b.requirement
        ));
    }
    for a in awaiting
        .iter()
        .filter(|a| !ids.contains(&a.requirement.to_owned()))
    {
        problems.push(format!(
            "{} is declared awaiting a benchmark, but ENG-15 does not name it",
            a.requirement
        ));
    }

    problems
}

/// One line for each of `ids` that no benchmark of real code covers,
/// with what it waits on.
#[must_use]
pub fn uncovered(ids: &[String], suite: &[Benchmark], awaiting: &[Awaiting]) -> Vec<String> {
    ids.iter()
        .filter(|id| !covered(id, suite))
        .map(|id| match awaited(id, awaiting) {
            Some(a) => format!("{id} has no benchmark of real code yet: it awaits {}", a.on),
            None => format!("{id} has no benchmark of real code"),
        })
        .collect()
}

/// Whether a benchmark of `id`'s target covers it.
fn covered(id: &str, suite: &[Benchmark]) -> bool {
    suite
        .iter()
        .any(|b| b.requirement == id && b.coverage == Coverage::Target)
}

/// What `id` is declared awaiting, if anything.
fn awaited<'a>(id: &str, awaiting: &'a [Awaiting]) -> Option<&'a Awaiting> {
    awaiting.iter().find(|a| a.requirement == id)
}

/// Requires `workflow` to have the [`JOB`] that runs each of [`RUNS`],
/// and the [`AGGREGATOR`] job to need it, so its failure fails `CI`.
#[must_use]
pub fn gated(workflow: &Workflow) -> Vec<String> {
    let job = |name: &str| workflow.jobs.iter().find(|j| j.name == name);
    let Some(gate) = job(JOB) else {
        return vec![format!("the CI workflow has no job {JOB}")];
    };
    let mut problems: Vec<String> = RUNS
        .iter()
        .filter(|run| !gate.text.contains(*run))
        .map(|run| format!("job {JOB} does not run {run:?}"))
        .collect();
    let needed = job(AGGREGATOR).is_some_and(|ci| {
        ci.text
            .lines()
            .filter_map(|l| l.trim_start().strip_prefix("needs:"))
            .any(|needs| {
                needs
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                    .any(|n| n == JOB)
            })
    });
    if !needed {
        problems.push(format!("job {AGGREGATOR} does not need {JOB}"));
    }

    problems
}

/// Samples files for `benchmark`, as the base's and the head's: thirty
/// quiet samples a side, the head's `percent` slower.
#[must_use]
pub fn slowed(benchmark: &str, percent: u64) -> (String, String) {
    let base: Vec<u64> = (0..30).map(|i| 800_000 + i % 7).collect();
    let head: Vec<u64> = base.iter().map(|ns| ns * (100 + percent) / 100).collect();
    let file =
        |taken: &[u64]| format!("{}\n{}", samples::HEADER, samples::render(benchmark, taken));

    (file(&base), file(&head))
}

/// Runs the gate as the CI job does on the `base` and `head` samples
/// files, and requires it to fail naming `benchmark` as regressed.
///
/// # Errors
///
/// Fails when the gate passes, or fails without naming the benchmark.
pub fn gate_fails_naming(base: &str, head: &str, benchmark: &str) -> Result<(), String> {
    let read = |path: &str| -> io::Result<String> {
        Ok(if path == "base" { base } else { head }.to_owned())
    };
    let time = |_: &Path, _: &Benchmark| Err("the gate starts no program".to_owned());
    let args = ["gate", "--base", "base", "--head", "head"].map(str::to_owned);
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = cli::run(args, &mut out, &mut err, &read, &time);

    let err = String::from_utf8_lossy(&err);
    let named = err
        .lines()
        .any(|l| l.starts_with(&format!("benchmark: {benchmark} regressed by ")));
    if code == cli::EXIT_FAILURE && named {
        Ok(())
    } else {
        Err(format!(
            "the gate exited {code} without naming {benchmark} as regressed: {err}"
        ))
    }
}

#[cfg(test)]
mod tests;
