//! The mass failure, decided from the runs alone: when more than the
//! breaker's fraction of the dispatched specialists ended in a gap, a
//! specialist split in parts counting once and one a cost cap stopped
//! or kept from starting like any other, the review was aborted. A
//! specialist that only left lines of its diff unread is in no such
//! gap: its run did not fail. The aborted review's outcome opens with a
//! blocking finding of its own, distinct from each agent's, saying so
//! and how to finish it, and requests changes; what was not reviewed,
//! the generalists the breaker kept from starting among it, stays
//! named, so a resumed review runs just those.

use std::collections::BTreeSet;

use review_eval::component::Kind;
use review_gate::{BLOCKING, Finding, REQUEST_CHANGES_VERDICT};

use crate::controls::{Fraction, mass_failure, trips};
use crate::outcome::{Decision, REVIEW_PATH};
use crate::plan::Plan;

/// How the abort's finding starts.
pub const ABORTED: &str = "review aborted:";

/// How many of `plan`'s specialists `decision` left without an accepted
/// report, whatever the reason: a cost cap's counts as any other. One
/// that stated its report and only left lines of its diff unread is not
/// among them.
#[must_use]
pub fn failed(plan: &Plan, decision: &Decision) -> usize {
    let specialists = plan.specialists();
    decision
        .not_reviewed
        .iter()
        .filter(|g| g.kind == Kind::Specialist.word())
        .filter(|g| specialists.contains(&g.name))
        .filter(|g| !g.unread_lines())
        .map(|g| g.name.as_str())
        .collect::<BTreeSet<&str>>()
        .len()
}

/// The abort's finding, when more than `fraction` of `plan`'s
/// specialists failed in `decision`; none otherwise.
#[must_use]
pub fn abort(plan: &Plan, decision: &Decision, fraction: Fraction) -> Option<Finding> {
    let (failed, dispatched) = (failed(plan, decision), plan.specialists().len());
    trips(failed, dispatched, fraction).then(|| Finding {
        path: REVIEW_PATH.to_owned(),
        line: 0,
        severity: BLOCKING.to_owned(),
        body: format!(
            "{ABORTED} {}, so no generalist was started; resume with --resume once fixed \
             (scripts/review-local.sh --resume, or review-run --only <its gaps.json> --from <its \
             review directory>)",
            mass_failure(failed, dispatched)
        ),
    })
}

/// `decision`, opened with the abort's finding and requesting changes,
/// when more than `fraction` of `plan`'s specialists failed.
#[must_use]
pub fn amend(plan: &Plan, mut decision: Decision, fraction: Fraction) -> Decision {
    if let Some(finding) = abort(plan, &decision, fraction) {
        let outcome = &mut decision.outcome;
        outcome.summary = format!("{} {}", finding.body, outcome.summary);
        outcome.verdict = REQUEST_CHANGES_VERDICT.to_owned();
        outcome.findings.insert(0, finding);
    }
    decision
}

#[cfg(test)]
mod tests;
