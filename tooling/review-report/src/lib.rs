//! `review-report`: what one review did, and how stable repeated
//! reviews of one pull-request snapshot are, from the files the review
//! wrote and nothing else: no server, no socket, no clock read beyond
//! the logs' own file times (ENG-28, ENG-32).
//!
//! After every run, review-run writes two files beside its outcome:
//!
//! - `review/agent-runs.jsonl`, one JSON object per line. The first line is
//!   the run list, `{"review": {id, started, finished, verdict,
//!   findings_blocking, findings_nit, gaps, failure?, notes?}}`: the
//!   head commit reviewed (or `run-<inputs hash>`), the first agent run's
//!   start and the last one's end in milliseconds since the Unix epoch,
//!   the verdict (`failed` when the review could not run, `failure`
//!   then saying why), its finding counts by severity, the agents not
//!   reviewed, and each review file that did not read. Every later line
//!   is one entry, one agent run: `{name, kind, part?, attempt, model,
//!   effort, started_ms, ended_ms, turns, tokens: {fresh, cache_write,
//!   cache_read, output}, cost_usd, status, reason?, reused?}`. Its cost
//!   is the `total_cost_usd` the `claude` command line reported in the
//!   run's log, never recomputed from tokens; `status` is `accepted`
//!   or `gap`, and a gap says why.
//! - `review/report.html`, a static run report built from the run list: the
//!   verdict and counts, a timeline, cost and token tables, the gap
//!   reasons and what was not reviewed. Every string an agent's run
//!   could shape is escaped, and the run report is size-bounded.
//!
//! `review-report stability` compares the work directories of N
//! reviews of one pull-request snapshot and writes `stability.json`
//! ([`stability`]); `review-report snapshots` reads the stability measurement's
//! such snapshots ([`snapshots`]).
//!
//! It is repository tooling and never links into a shipped executable.

use std::fs;
use std::path::Path;

pub mod agent_runs;
pub mod cli;
pub mod html;
pub mod identity;
pub mod inputs;
pub mod snapshots;
pub mod stability;

/// The review's directory, under the work directory, where both go.
const REVIEW: &str = "review";
/// The run list, under the work directory.
pub const AGENT_RUNS: &str = "review/agent-runs.jsonl";
/// The run report, under the work directory.
pub const REPORT: &str = "review/report.html";

/// Writes the run list and the run report of the review in `workdir`,
/// `failure` saying why it could not run, when it could not. Returns
/// each file that could not be written, with why; best effort, so a
/// review's verdict never depends on it.
#[must_use]
pub fn write(workdir: &Path, failure: Option<&str>) -> Vec<String> {
    let runs = agent_runs::build(workdir, failure);
    let gaps = inputs::gaps(workdir, &mut Vec::new());
    let report = html::render(&runs, &gaps);
    let mut problems = Vec::new();
    for (rel, text) in [(AGENT_RUNS, runs.jsonl()), (REPORT, Ok(report))] {
        let path = workdir.join(rel);
        let written = text.and_then(|text| {
            fs::create_dir_all(workdir.join(REVIEW)).map_err(|e| e.to_string())?;
            fs::write(&path, text).map_err(|e| e.to_string())
        });
        if let Err(e) = written {
            problems.push(format!("{rel} not written: {e}"));
        }
    }

    problems
}

/// [`write`], telling `progress` each file not written and how many
/// were not (I6).
pub fn write_reported(workdir: &Path, failure: Option<&str>, progress: &dyn Fn(&str)) {
    let problems = write(workdir, failure);
    for problem in &problems {
        progress(&format!("run report: {problem}"));
    }
    if !problems.is_empty() {
        progress(&format!(
            "run report: {} of 2 files not written",
            problems.len()
        ));
    }
}

#[cfg(test)]
mod tests;
