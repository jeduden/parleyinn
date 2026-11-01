//! The kept findings across reviews of one pull request: each run keeps
//! its outcome's findings in `review/findings.json`, each by its
//! identity ([`identity`]) with its severity, the agent that kept it,
//! the run it was first seen in, how many reviews have seen it and how
//! many in a row kept it blocking with its cited file unchanged. Given
//! the previous review's file, a run marks each finding `new` or `still
//! open (seen N times)`, lists those no longer found as `resolved`, in
//! the outcome's summary and in `review/findings.md` (I6), and names a
//! blocking finding kept in [`RULING_REVIEWS`] reviews in a row, its
//! file unchanged, in a blocking finding of its own: it has stalled and
//! needs a stakeholder ruling, which no further run gives.

pub mod identity;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path};

use review_eval::world::PR_HEAD;
use review_gate::{BLOCKING, Finding};
use serde::{Deserialize, Serialize};

use crate::outcome;
use crate::plan::Plan;
use crate::run_id::sha256;

use identity::{Fid, of_finding};

/// The kept findings' file under the work directory.
pub const FINDINGS: &str = "review/findings.json";
/// The same, for a person.
pub const FINDINGS_NOTES: &str = "review/findings.md";
/// How many reviews in a row may keep one blocking finding, its cited
/// file unchanged, before it needs a stakeholder ruling.
pub const RULING_REVIEWS: u32 = 3;
/// How a finding that needs a stakeholder ruling starts.
pub const RULING: &str = "stalled: needs a stakeholder ruling";
/// How every finding a stall raises starts; none is itself tracked.
pub const STALLED: &str = "stalled:";

/// One kept finding, as reviews track it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tracked {
    pub fid: Fid,
    /// Its line in this run.
    pub line: i64,
    pub severity: String,
    /// The generalist that kept it, or `review-run` for one the review
    /// raised itself.
    pub agent: String,
    pub body: String,
    /// The run it was first seen in: the head reviewed.
    pub first_seen_run: String,
    /// How many reviews have kept it, this one among them.
    pub seen_count: u32,
    /// How many reviews in a row kept it blocking with its cited file
    /// unchanged; none when it is a nit.
    pub blocking_streak: u32,
    /// The SHA-256 of its cited file in the pull request's tree; absent
    /// when that does not read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

impl Tracked {
    /// What it is beside the previous review.
    #[must_use]
    pub fn status(&self) -> String {
        if self.seen_count <= 1 {
            "new".to_owned()
        } else {
            format!("still open (seen {} times)", self.seen_count)
        }
    }
}

/// One review's findings beside the previous review's.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tracking {
    /// This review's, each once, in the outcome's order.
    pub kept: Vec<Tracked>,
    /// The previous review's no longer kept.
    pub resolved: Vec<Tracked>,
    /// Whether there was a previous review.
    pub previous: bool,
}

/// Each of `findings`, the outcome's, with the generalist of `plan`
/// whose answer in `answers` kept it at its place, or `review-run` for
/// one the review raised itself.
#[must_use]
pub fn attributed(
    findings: &[Finding],
    answers: &BTreeMap<String, String>,
    plan: &Plan,
) -> Vec<(Finding, String)> {
    let generalists = plan.generalists();
    findings
        .iter()
        .map(|f| {
            let agent = generalists
                .iter()
                .find(|g| {
                    answers.get(*g).is_some_and(|a| {
                        outcome::kept(a).iter().any(|k| {
                            k.path == f.path && k.line == f.line && f.body.starts_with(&k.body)
                        })
                    })
                })
                .map_or_else(|| REVIEW_RUN.to_owned(), Clone::clone);
            (f.clone(), agent)
        })
        .collect()
}

/// Who raised a finding no generalist kept: the review itself.
pub const REVIEW_RUN: &str = "review-run";

/// The SHA-256 of the file `path` names in the pull request's tree
/// under `workdir`; none when it does not read, or `path` is absolute or
/// leaves the tree.
#[must_use]
pub fn cited(workdir: &Path, path: &str) -> Option<String> {
    let rel = Path::new(path);
    if !rel.components().all(|c| matches!(c, Component::Normal(_))) {
        return None;
    }
    fs::read(workdir.join(PR_HEAD).join(rel))
        .ok()
        .map(|bytes| sha256(&bytes))
}

/// Reads a findings file.
///
/// # Errors
///
/// Fails when it does not read as its shape.
pub fn parse(text: &str) -> Result<Vec<Tracked>, String> {
    serde_json::from_str(text).map_err(|e| format!("malformed findings: {e}"))
}

/// Tracks `found`, each finding the outcome kept with the agent that
/// kept it, in the run `run`, beside `previous`, the previous review's,
/// when given, each cited file hashed by `file`.
#[must_use]
pub fn track(
    found: &[(Finding, String)],
    previous: Option<&[Tracked]>,
    run: &str,
    file: &dyn Fn(&str) -> Option<String>,
) -> Tracking {
    let before = previous.unwrap_or_default();
    let mut kept: Vec<Tracked> = Vec::new();
    for (f, agent) in found.iter().filter(|(f, _)| !f.body.starts_with(STALLED)) {
        let fid = of_finding(f);
        if kept.iter().any(|t| t.fid == fid) {
            continue;
        }
        let hash = file(&f.path);
        let was = before.iter().find(|t| t.fid == fid);
        let blocking = f.severity == BLOCKING;
        let streak = match was {
            Some(t) if blocking && t.severity == BLOCKING && hash.is_some() && t.file == hash => {
                t.blocking_streak + 1
            }
            _ => u32::from(blocking),
        };
        kept.push(Tracked {
            line: f.line,
            severity: f.severity.clone(),
            agent: agent.clone(),
            body: f.body.clone(),
            first_seen_run: was.map_or_else(|| run.to_owned(), |t| t.first_seen_run.clone()),
            seen_count: was.map_or(1, |t| t.seen_count + 1),
            blocking_streak: streak,
            file: hash,
            fid,
        });
    }
    let resolved = before
        .iter()
        .filter(|t| !kept.iter().any(|k| k.fid == t.fid))
        .cloned()
        .collect();

    Tracking {
        kept,
        resolved,
        previous: previous.is_some(),
    }
}

/// A blocking finding for each of `tracking`'s kept findings that stayed
/// blocking in [`RULING_REVIEWS`] reviews in a row, its file unchanged.
#[must_use]
pub fn rulings(tracking: &Tracking) -> Vec<Finding> {
    tracking
        .kept
        .iter()
        .filter(|t| t.blocking_streak >= RULING_REVIEWS)
        .map(|t| Finding {
            path: t.fid.path.clone(),
            line: 0,
            severity: BLOCKING.to_owned(),
            body: format!(
                "{RULING}: {} kept this finding blocking in {} reviews in a row with {} \
                 unchanged, so another run will not settle it: {}",
                t.agent, t.blocking_streak, t.fid.path, t.body
            ),
        })
        .collect()
}

/// The summary's sentence on `tracking` beside the previous review; empty
/// when there was none.
#[must_use]
pub fn summary(tracking: &Tracking) -> String {
    if !tracking.previous {
        return String::new();
    }
    let new = tracking.kept.iter().filter(|t| t.seen_count <= 1).count();
    let mut s = format!(
        " Since the previous review: {new} new, {} still open and {} resolved",
        tracking.kept.len() - new,
        tracking.resolved.len()
    );
    if !tracking.resolved.is_empty() {
        let names: Vec<String> = tracking.resolved.iter().map(place).collect();
        let _ = write!(s, " ({})", names.join(", "));
    }
    s.push('.');
    s
}

/// Where `t` is and what it cites, said.
fn place(t: &Tracked) -> String {
    if t.fid.ids.is_empty() {
        format!("{}:{}", t.fid.path, t.line)
    } else {
        format!("{}:{} ({})", t.fid.path, t.line, t.fid.ids.join(", "))
    }
}

/// The kept and resolved findings, for a person.
#[must_use]
pub fn markdown(tracking: &Tracking) -> String {
    let mut s = "# Findings across reviews\n\n".to_owned();
    if !tracking.previous {
        s.push_str("No previous review was given, so every finding is new.\n\n");
    }
    let new: Vec<&Tracked> = tracking.kept.iter().filter(|t| t.seen_count <= 1).collect();
    let open: Vec<&Tracked> = tracking.kept.iter().filter(|t| t.seen_count > 1).collect();
    let resolved: Vec<&Tracked> = tracking.resolved.iter().collect();
    for (heading, list) in [("New", new), ("Still open", open), ("Resolved", resolved)] {
        let _ = writeln!(s, "## {heading}\n");
        if list.is_empty() {
            s.push_str("None.\n\n");
            continue;
        }
        for t in list {
            let status = if heading == "Resolved" {
                "resolved".to_owned()
            } else {
                t.status()
            };
            let _ = writeln!(
                s,
                "- `{}` ({}, {}): {} — {status}",
                place(t),
                t.severity,
                t.agent,
                t.body,
            );
        }
        s.push('\n');
    }

    s
}

#[cfg(test)]
mod tests;
