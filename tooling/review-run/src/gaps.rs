//! What the review did not review: every dispatched agent, or part of
//! a specialist's, left without an accepted report or answer. A run
//! that did not answer after its second attempt is silent, or
//! incomplete when its turns ran out, over budget when a cost cap
//! stopped it or kept it from starting, and aborted when the circuit
//! breaker kept it from starting; a report the gate refuses is
//! refused, with the gate's problem; a generalist's answer the review
//! cannot read is unread. The review writes the list beside its outcome,
//! empty when there is none, so a later run can run just these again,
//! and names each in the outcome's summary and in its own file, never
//! as clean.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use review_eval::component::{Kind, kind_of};
use review_eval::runner::OVER_BUDGET as OVER_BUDGET_SUBTYPE;

use crate::controls::{ABORTED, OVER_BUDGET};

/// The list's file under the work directory.
pub const GAPS: &str = "review/gaps.json";
/// What the review did not review, for a person.
pub const NOT_REVIEWED: &str = "review/not-reviewed.md";

/// The result subtype of a run its turn budget cut off.
const MAX_TURNS: &str = "error_max_turns";

/// One agent, or one part of a specialist's, the review did not review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gap {
    pub name: String,
    /// `specialist` or `generalist`.
    pub kind: String,
    /// The part of the specialist's split diff, from one; absent when
    /// the agent as a whole is not reviewed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<usize>,
    /// Why: `silent:`, `incomplete:`, `over budget:`, `aborted:`,
    /// `refused:` or `unread:`, then the cause.
    pub reason: String,
}

impl Gap {
    /// Agent `name`'s gap, of the kind its name says.
    #[must_use]
    pub fn new(name: &str, part: Option<usize>, reason: String) -> Self {
        Self {
            name: name.to_owned(),
            kind: kind_of(name).word().to_owned(),
            part,
            reason,
        }
    }

    /// How the summary names it.
    #[must_use]
    pub fn label(&self) -> String {
        match self.part {
            Some(part) => format!("{} part {part}", self.name),
            None => self.name.clone(),
        }
    }
}

/// The reason of a run that did not answer for `why`: over budget when
/// a cost cap stopped it or kept it from starting, aborted when the
/// breaker kept it from starting, incomplete when its turn budget cut
/// it off, silent otherwise.
#[must_use]
pub fn silent(why: &str) -> String {
    if why.starts_with(OVER_BUDGET) || why.starts_with(ABORTED) {
        return why.to_owned();
    }
    let word = if why.contains(OVER_BUDGET_SUBTYPE) {
        OVER_BUDGET
    } else if why.contains(MAX_TURNS) {
        "incomplete:"
    } else {
        "silent:"
    };
    format!("{word} {why}")
}

/// Reads a gap list: every entry an agent the review can dispatch, of
/// its kind, a part from one.
///
/// # Errors
///
/// Fails on anything else, naming the entry.
pub fn parse(text: &str) -> Result<Vec<Gap>, String> {
    let gaps: Vec<Gap> = serde_json::from_str(text).map_err(|e| format!("malformed gaps: {e}"))?;
    for gap in &gaps {
        let kind = kind_of(&gap.name);
        if !review_bundle::dispatch::is_agent_name(&gap.name)
            || !matches!(kind, Kind::Specialist | Kind::Generalist)
        {
            return Err(format!(
                "malformed gaps: {:?} is no dispatched agent",
                gap.name
            ));
        }
        if gap.kind != kind.word() {
            return Err(format!(
                "malformed gaps: {} is a {}, not a {}",
                gap.name,
                kind.word(),
                gap.kind
            ));
        }
        if gap.part == Some(0) || (gap.part.is_some() && kind != Kind::Specialist) {
            return Err(format!(
                "malformed gaps: {} has no part {:?}",
                gap.name, gap.part
            ));
        }
    }

    Ok(gaps)
}

/// What the review did not review, for a person: each gap with why, or
/// that every dispatched agent has an accepted report.
#[must_use]
pub fn markdown(gaps: &[Gap]) -> String {
    let mut s = "# Not reviewed\n\n".to_owned();
    if gaps.is_empty() {
        s.push_str("Every dispatched agent has an accepted report or answer.\n");
        return s;
    }
    s.push_str(
        "These dispatched agents have no accepted report or answer, so the review read \
         nothing from them. None of them is clean.\n\n",
    );
    for gap in gaps {
        let _ = writeln!(s, "- `{}` ({}): {}", gap.label(), gap.kind, gap.reason);
    }

    s
}

#[cfg(test)]
mod tests;
