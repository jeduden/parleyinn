//! The loop breaker of resumed reviews: each run writes, beside its gap
//! list, `review/stalls.json`, every gap counted by its agent, its part
//! and the way it failed (silent, incomplete, over budget, refused or
//! unread), with how many runs in a row, the first and every resume from
//! it, ended in that same gap and why each time. Once a gap has recurred
//! in more than [`MAX_RESUMES`] (`--max-resumes`) resumes in a row, the
//! agent has stalled: no resume runs it again, the review stays failed
//! closed with it not reviewed, and its outcome holds a distinct
//! blocking finding, `stalled: <agent> failed <N> times the same way;
//! needs a person`, with the reasons, for a person to act on.

use std::collections::BTreeMap;

use review_gate::{BLOCKING, Finding, REQUEST_CHANGES_VERDICT, ReviewOutcome};
use serde::{Deserialize, Serialize};

use crate::findings::STALLED;
use crate::gaps::Gap;
use crate::outcome::REVIEW_PATH;

/// The file under the work directory.
pub const STALLS: &str = "review/stalls.json";
/// How many resumes in a row may end in the same gap before its agent
/// is not run again, unless the run says otherwise.
pub const MAX_RESUMES: u32 = 2;
/// The ways a gap fails that count toward a stall: a breaker's abort is
/// another agent's failure, and a stall is already one.
pub const COUNTED: [&str; 5] = [
    "silent:",
    "incomplete:",
    "over budget:",
    "refused:",
    "unread:",
];

/// One gap, counted across runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stall {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<usize>,
    /// The way it failed, one of [`COUNTED`].
    pub class: String,
    /// How many runs in a row ended in it.
    pub times: u32,
    /// Why, each time.
    pub reasons: Vec<String>,
}

impl Stall {
    /// Whether it recurred in more than `max_resumes` resumes in a row.
    #[must_use]
    pub fn stuck(&self, max_resumes: u32) -> bool {
        self.times > max_resumes
    }
}

/// The way `reason` fails, when it counts toward a stall.
#[must_use]
pub fn class_of(reason: &str) -> Option<&'static str> {
    COUNTED.into_iter().find(|c| reason.starts_with(c))
}

/// Reads a stalls file.
///
/// # Errors
///
/// Fails when it does not read as its shape.
pub fn parse(text: &str) -> Result<Vec<Stall>, String> {
    serde_json::from_str(text).map_err(|e| format!("malformed stalls: {e}"))
}

/// This run's stalls from its `gaps`, beside `earlier`, the stalls of
/// the run it resumed from, empty for a full run: a gap that counts is
/// one more time of the same gap there, or a first; an agent not run
/// again for its stall keeps its stalls as they were.
#[must_use]
pub fn tally(gaps: &[Gap], earlier: &[Stall]) -> Vec<Stall> {
    let mut out: Vec<Stall> = Vec::new();
    for gap in gaps {
        if gap.reason.starts_with(STALLED) {
            out.extend(earlier.iter().filter(|s| s.name == gap.name).cloned());
            continue;
        }
        let Some(class) = class_of(&gap.reason) else {
            continue;
        };
        let was = earlier
            .iter()
            .find(|s| s.name == gap.name && s.part == gap.part && s.class == class);
        let mut reasons = was.map(|s| s.reasons.clone()).unwrap_or_default();
        reasons.push(gap.reason.clone());
        out.push(Stall {
            name: gap.name.clone(),
            part: gap.part,
            class: class.to_owned(),
            times: was.map_or(1, |s| s.times + 1),
            reasons,
        });
    }
    out.sort_by(|a, b| (&a.name, a.part, &a.class).cmp(&(&b.name, b.part, &b.class)));
    out.dedup_by(|a, b| (&a.name, a.part, &a.class) == (&b.name, b.part, &b.class));

    out
}

/// Each agent `stalls` names as stuck past `max_resumes`, with why, the
/// text of its finding and its gap.
#[must_use]
pub fn stuck(stalls: &[Stall], max_resumes: u32) -> BTreeMap<String, String> {
    let mut by_agent: BTreeMap<String, Vec<&Stall>> = BTreeMap::new();
    for s in stalls.iter().filter(|s| s.stuck(max_resumes)) {
        by_agent.entry(s.name.clone()).or_default().push(s);
    }
    by_agent
        .into_iter()
        .map(|(name, list)| {
            let times = list.iter().map(|s| s.times).max().unwrap_or_default();
            let reasons: Vec<&str> = list
                .iter()
                .flat_map(|s| s.reasons.iter().map(String::as_str))
                .collect();
            let why = format!(
                "{STALLED} {name} failed {times} times the same way; needs a person: {}",
                reasons.join("; ")
            );
            (name, why)
        })
        .collect()
}

/// Adds to `outcome` a blocking finding for each agent `stalls` names as
/// stuck past `max_resumes`, requesting changes; returns how many.
pub fn amend(outcome: &mut ReviewOutcome, stalls: &[Stall], max_resumes: u32) -> usize {
    let stuck = stuck(stalls, max_resumes);
    for why in stuck.values() {
        outcome.findings.push(Finding {
            path: REVIEW_PATH.to_owned(),
            line: 0,
            severity: BLOCKING.to_owned(),
            body: why.clone(),
        });
        outcome.verdict = REQUEST_CHANGES_VERDICT.to_owned();
    }
    stuck.len()
}

#[cfg(test)]
mod tests;
