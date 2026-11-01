//! The review's own files, read leniently: a file that is not there
//! reads as empty, as a review that could not run leaves it, and one
//! that does not read is named in `notes`, never silently dropped (I6).
//! The shapes are review-run's, read by field name so a field it adds
//! later changes nothing here.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;
use serde::de::DeserializeOwned;

/// The review outcome.
pub const OUTCOME: &str = "review/outcome.json";
/// Each agent's usage and cost.
pub const USAGE: &str = "review/usage.json";
/// Every agent, or part, left without an accepted report or answer.
pub const GAPS: &str = "review/gaps.json";
/// What the review ran on.
pub const RUN_ID: &str = "review/run-id.json";
/// Each answering agent's final answer, by name.
pub const ANSWERS: &str = "review/answers.json";
/// The run logs, `<name>[.<part>][.first].jsonl`.
pub const LOGS: &str = "review/logs";
/// The specialists' reports, `<name>.md`.
pub const REPORTS: &str = "review/reports";

/// The review outcome: its verdict and the severity of each finding.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Outcome {
    pub verdict: String,
    pub findings: Vec<OutcomeFinding>,
}

/// One finding of the outcome, by its severity.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct OutcomeFinding {
    pub severity: String,
}

/// One agent's usage entry: whether it was reused, and its cost.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Usage {
    pub agent: String,
    pub cost_usd: f64,
    pub reused: bool,
}

/// One agent, or part, the review did not review.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Gap {
    pub name: String,
    pub kind: String,
    pub part: Option<usize>,
    pub reason: String,
}

impl Gap {
    /// How a person reads its name: the agent, and its part if any.
    #[must_use]
    pub fn label(&self) -> String {
        match self.part {
            Some(part) => format!("{} part {part}", self.name),
            None => self.name.clone(),
        }
    }
}

/// What the review ran on: its head and its inputs' hash.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct RunId {
    pub head: Option<String>,
    pub inputs: String,
}

/// The JSON file `rel` under `workdir` as `T`: the default when it is
/// not there, and when it does not read, with why in `notes`.
pub fn read<T: DeserializeOwned + Default>(
    workdir: &Path,
    rel: &str,
    notes: &mut Vec<String>,
) -> T {
    match fs::read_to_string(workdir.join(rel)) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            notes.push(format!("{rel} does not read: {e}"));
            T::default()
        }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => T::default(),
        Err(e) => {
            notes.push(format!("{rel} does not read: {e}"));
            T::default()
        }
    }
}

/// The review outcome in `workdir`.
pub fn outcome(workdir: &Path, notes: &mut Vec<String>) -> Outcome {
    read(workdir, OUTCOME, notes)
}

/// The usage entries in `workdir`.
pub fn usage(workdir: &Path, notes: &mut Vec<String>) -> Vec<Usage> {
    read(workdir, USAGE, notes)
}

/// The gaps in `workdir`.
pub fn gaps(workdir: &Path, notes: &mut Vec<String>) -> Vec<Gap> {
    read(workdir, GAPS, notes)
}

/// What the review in `workdir` ran on.
pub fn run_id(workdir: &Path, notes: &mut Vec<String>) -> RunId {
    read(workdir, RUN_ID, notes)
}

/// Each answering agent's final answer in `workdir`, by name.
pub fn answers(workdir: &Path, notes: &mut Vec<String>) -> BTreeMap<String, String> {
    read(workdir, ANSWERS, notes)
}

#[cfg(test)]
mod tests;
