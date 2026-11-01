//! How stable N reviews of one pull-request state are (ENG-32): the
//! `claude` command line takes no seed, so an agent's answer cannot be
//! pinned, only measured by repetition. From each review's work
//! directory it reads the verdict, each agent's finding identities
//! ([`crate::identity`]) and each agent's cost, and writes
//! `stability.json`:
//!
//! - `flip_rate`, the share of runs whose verdict differs from the most
//!   common one (the mode; a tie goes to the verdict first in order);
//! - per agent, the Jaccard overlap of its finding sets across the runs
//!   it reported in, the size of the sets they share over the size of
//!   their union, one when every set is empty; how many runs it reported
//!   in; its findings per run; and the spread of its cost;
//! - `jaccard`, the mean of the agents' overlaps;
//! - `stable`, true only when the comparison is measured and every
//!   agent, dispatched or reported, reported in every run with the same
//!   finding set and no verdict flipped; an agent left in a gap counts
//!   as one that did not report;
//! - `same_state`, whether every run reviewed the same head and inputs,
//!   without which the comparison means nothing;
//! - `unmeasured`, why the comparison measures nothing: no runs, runs
//!   of different states, or a run with no verdict (a review that
//!   failed); empty when it is measured;
//! - `within_limits`, whether it is measured, the flip rate is at most
//!   [`FLIP_RATE_LIMIT`] and the overlap at least [`JACCARD_FLOOR`],
//!   which the stability measurement warns on.
//!
//! Every function here but [`read`] is pure, and the output is ordered,
//! so the same runs give the same file.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::agent_runs::NO_VERDICT;
use crate::identity::{self, Fid};
use crate::inputs;

/// The highest flip rate the measurement passes without a warning: more than
/// one run in five deciding otherwise than the rest.
pub const FLIP_RATE_LIMIT: f64 = 0.2;
/// The lowest mean finding overlap the measurement passes without a warning.
pub const JACCARD_FLOOR: f64 = 0.7;

/// One review, as the comparison reads it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Run {
    /// Its work directory.
    pub dir: String,
    pub verdict: String,
    /// The head it reviewed and its inputs' hash.
    pub state: (Option<String>, String),
    /// Each agent's finding identities, by name.
    pub findings: BTreeMap<String, BTreeSet<Fid>>,
    /// Each agent's cost, by name.
    pub costs: BTreeMap<String, f64>,
    /// The agents left without an accepted report or answer.
    pub gaps: BTreeSet<String>,
}

/// One agent's stability across the runs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Agent {
    pub agent: String,
    pub jaccard: f64,
    /// How many runs it reported findings, or none, in.
    pub reported: usize,
    /// Its findings in each run, in the runs' order; none where it did
    /// not report.
    pub findings: Vec<Option<usize>>,
    pub cost_min: f64,
    pub cost_max: f64,
    pub cost_spread: f64,
}

/// The stability of the runs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stability {
    pub runs: usize,
    pub dirs: Vec<String>,
    pub same_state: bool,
    pub verdicts: Vec<String>,
    pub mode: String,
    pub flip_rate: f64,
    pub jaccard: f64,
    pub agents: Vec<Agent>,
    pub stable: bool,
    /// Why the comparison measures nothing; empty when it measures.
    pub unmeasured: Vec<String>,
    pub within_limits: bool,
    pub flip_rate_limit: f64,
    pub jaccard_floor: f64,
}

/// `x` rounded to six places, so float noise never changes the file.
fn round(x: f64) -> f64 {
    (x * 1e6).round() / 1e6
}

/// The Jaccard overlap of `sets`: the identities all of them hold over
/// those any holds; one when none holds any, or there are no sets.
#[must_use]
pub fn jaccard(sets: &[&BTreeSet<Fid>]) -> f64 {
    let union: BTreeSet<&Fid> = sets.iter().flat_map(|s| s.iter()).collect();
    if union.is_empty() {
        return 1.0;
    }
    let shared = union
        .iter()
        .filter(|fid| sets.iter().all(|s| s.contains(*fid)))
        .count();

    round(shared as f64 / union.len() as f64)
}

/// The most common of `verdicts`, the first in order on a tie; empty
/// for none.
#[must_use]
pub fn mode(verdicts: &[String]) -> String {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for v in verdicts {
        *counts.entry(v).or_default() += 1;
    }
    let most = counts.values().copied().max().unwrap_or_default();
    counts
        .into_iter()
        .find(|(_, n)| *n == most)
        .map(|(v, _)| v.to_owned())
        .unwrap_or_default()
}

/// The share of `verdicts` that differ from their mode; none for none.
#[must_use]
pub fn flip_rate(verdicts: &[String]) -> f64 {
    if verdicts.is_empty() {
        return 0.0;
    }
    let mode = mode(verdicts);
    let flipped = verdicts.iter().filter(|v| **v != mode).count();

    round(flipped as f64 / verdicts.len() as f64)
}

/// Agent `name`'s stability across `runs`.
fn agent(name: &str, runs: &[Run]) -> Agent {
    let sets: Vec<&BTreeSet<Fid>> = runs.iter().filter_map(|r| r.findings.get(name)).collect();
    let costs: Vec<f64> = runs
        .iter()
        .filter_map(|r| r.costs.get(name).copied())
        .collect();
    let min = costs.iter().copied().reduce(f64::min).unwrap_or_default();
    let max = costs.iter().copied().reduce(f64::max).unwrap_or_default();

    Agent {
        agent: name.to_owned(),
        jaccard: jaccard(&sets),
        reported: sets.len(),
        findings: runs
            .iter()
            .map(|r| r.findings.get(name).map(BTreeSet::len))
            .collect(),
        cost_min: round(min),
        cost_max: round(max),
        cost_spread: round(max - min),
    }
}

/// Why `runs` measure nothing: none, different states, or a run with
/// no verdict, which is a review that failed and is no sample.
fn unmeasured(runs: &[Run], same_state: bool) -> Vec<String> {
    let mut why = Vec::new();
    if runs.is_empty() {
        why.push("there are no runs".to_owned());
    }
    if !same_state {
        why.push("the runs reviewed different heads or inputs".to_owned());
    }
    let failed: Vec<&str> = runs
        .iter()
        .filter(|r| r.verdict == NO_VERDICT)
        .map(|r| r.dir.as_str())
        .collect();
    if !failed.is_empty() {
        why.push(format!(
            "{} of {} runs have no verdict: {}",
            failed.len(),
            runs.len(),
            failed.join(", ")
        ));
    }

    why
}

/// The stability of `runs`. No runs measure nothing, so they are
/// neither stable nor within the limits.
#[must_use]
pub fn compare(runs: &[Run]) -> Stability {
    let verdicts: Vec<String> = runs.iter().map(|r| r.verdict.clone()).collect();
    let names: BTreeSet<&String> = runs
        .iter()
        .flat_map(|r| r.findings.keys().chain(r.costs.keys()).chain(&r.gaps))
        .collect();
    let agents: Vec<Agent> = names.into_iter().map(|n| agent(n, runs)).collect();
    let jaccard = if agents.is_empty() {
        1.0
    } else {
        round(agents.iter().map(|a| a.jaccard).sum::<f64>() / agents.len() as f64)
    };
    let flip_rate = flip_rate(&verdicts);
    let identical = agents
        .iter()
        .all(|a| a.reported == runs.len() && a.jaccard >= 1.0);
    let same_state = runs.windows(2).all(|w| w[0].state == w[1].state);
    let unmeasured = unmeasured(runs, same_state);
    let measured = unmeasured.is_empty();

    Stability {
        runs: runs.len(),
        dirs: runs.iter().map(|r| r.dir.clone()).collect(),
        same_state,
        mode: mode(&verdicts),
        verdicts,
        flip_rate,
        jaccard,
        agents,
        unmeasured,
        stable: measured && identical && flip_rate == 0.0,
        within_limits: measured && flip_rate <= FLIP_RATE_LIMIT && jaccard >= JACCARD_FLOOR,
        flip_rate_limit: FLIP_RATE_LIMIT,
        jaccard_floor: JACCARD_FLOOR,
    }
}

/// Each specialist's report text under `dir`, by name. A missing
/// directory holds none; any other file or directory that does not read
/// is named in `notes` (I6).
fn reports(dir: &Path, notes: &mut Vec<String>) -> BTreeMap<String, String> {
    let mut reports = BTreeMap::new();
    let reports_dir = dir.join(inputs::REPORTS);
    let entries = match fs::read_dir(&reports_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return reports,
        Err(e) => {
            notes.push(format!("{} does not read: {e}", reports_dir.display()));
            return reports;
        }
    };
    let paths = entries.map(|entry| entry.map(|entry| entry.path()));
    read_reports(paths, &reports_dir, notes, &mut reports);
    reports
}

/// Reads each `.md` file among `paths`, the entries of `dir`, into
/// `reports` by name; an entry or file that does not read is named in
/// `notes`.
fn read_reports(
    paths: impl Iterator<Item = io::Result<PathBuf>>,
    dir: &Path,
    notes: &mut Vec<String>,
    reports: &mut BTreeMap<String, String>,
) {
    for path in paths {
        let path = match path {
            Ok(path) => path,
            Err(e) => {
                notes.push(format!("{} does not read: {e}", dir.display()));
                continue;
            }
        };
        let Some(name) = path
            .file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_suffix(".md"))
        else {
            continue;
        };
        match fs::read_to_string(&path) {
            Ok(text) => {
                reports.insert(name.to_owned(), text);
            }
            Err(e) => notes.push(format!("{} does not read: {e}", path.display())),
        }
    }
}

/// The review in work directory `dir`: its verdict, each specialist's
/// report and each generalist's answer read for their findings, and
/// each agent's cost from its usage; each file that does not read named
/// in `notes`.
pub fn read(dir: &Path, notes: &mut Vec<String>) -> Run {
    let outcome = inputs::outcome(dir, notes);
    let run_id = inputs::run_id(dir, notes);
    let answers = inputs::answers(dir, notes);
    let reports = reports(dir, notes);
    let gaps = inputs::gaps(dir, notes)
        .into_iter()
        .map(|g| g.name)
        .collect();
    let costs = inputs::usage(dir, notes)
        .into_iter()
        .map(|u| (u.agent, u.cost_usd))
        .collect();

    Run {
        dir: dir.display().to_string(),
        verdict: if outcome.verdict.is_empty() {
            NO_VERDICT.to_owned()
        } else {
            outcome.verdict
        },
        state: (run_id.head, run_id.inputs),
        findings: identity::of_review(&reports, &answers),
        costs,
        gaps,
    }
}

#[cfg(test)]
mod tests;
