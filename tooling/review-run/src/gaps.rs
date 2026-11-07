//! What the review did not review whole: every dispatched agent, or
//! part of a specialist's, that did not run, read only part of its diff
//! or stated a line that does not read, and every answer it could not
//! take. A run that did not run after its second attempt is silent,
//! over budget when a cost cap kept it from starting, aborted when the
//! circuit breaker kept it from starting, and stalled when the loop
//! breaker kept it from running again; a specialist whose log shows no
//! read of some lines of its diff is partial, naming them; one whose
//! report holds a line that does not read is unreadable, quoting it; a
//! report the gate refuses is refused, with the gate's problem; a
//! generalist's answer the review cannot read is unread. The review
//! writes the list beside its outcome, empty when there is none, so a
//! later run can run just these again, and names each in the outcome's
//! summary and in its own file, never as clean.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use review_eval::component::{Kind, kind_of};
use review_eval::runner::OVER_BUDGET as OVER_BUDGET_SUBTYPE;

use crate::controls::{ABORTED, OVER_BUDGET};
use crate::findings::STALLED;
use crate::reading::ReportLine;

/// The list's file under the work directory.
pub const GAPS: &str = "review/gaps.json";
/// What the review did not review, for a person.
pub const NOT_REVIEWED: &str = "review/not-reviewed.md";

/// The result subtype of a run its turn budget cut off.
const MAX_TURNS: &str = "error_max_turns";
/// How the reason of a specialist that did not read its whole diff
/// starts.
pub const PARTIAL: &str = "partial:";
/// How the reason of one whose report holds a line that does not read
/// starts.
pub const UNREADABLE: &str = "unreadable:";

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
    /// `stalled:`, `partial:`, `unreadable:`, `refused:` or `unread:`,
    /// then the cause.
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
    if why.starts_with(OVER_BUDGET) || why.starts_with(ABORTED) || why.starts_with(STALLED) {
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

/// The reason of a specialist that did not read its whole diff, `why`
/// saying which lines.
#[must_use]
pub fn partial(why: &str) -> String {
    format!("{PARTIAL} {why}")
}

/// The reason of a specialist whose report holds `line`, which does not
/// read: its problem, the line quoted.
#[must_use]
pub fn unreadable(line: &ReportLine) -> String {
    format!("{UNREADABLE} {}: `{}`", line.why, line.text)
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

/// What the review did not review whole, for a person: each gap with
/// why, or that every dispatched agent has an accepted report.
#[must_use]
pub fn markdown(gaps: &[Gap]) -> String {
    let mut s = "# Not reviewed\n\n".to_owned();
    if gaps.is_empty() {
        s.push_str("Every dispatched agent has an accepted report or answer.\n");
        return s;
    }
    s.push_str(
        "The review did not review these dispatched agents, or parts, whole: each did \
         not run, read only part of its diff, stated a line that does not read, or gave an \
         answer the review could not take. None of them is clean.\n\n",
    );
    for gap in gaps {
        let _ = writeln!(s, "- `{}` ({}): {}", gap.label(), gap.kind, gap.reason);
    }

    s
}

#[cfg(test)]
mod tests;
