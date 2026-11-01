//! The mass failure, decided from the runs alone: when more than the
//! breaker's fraction of the dispatched specialists ended in a gap, a
//! specialist split in parts counting once and one a cost cap stopped
//! not at all, the review was aborted. Its outcome then opens with a
//! blocking finding of its own, distinct from each agent's, saying so
//! and how to finish it, and requests changes; what was not reviewed,
//! the generalists the breaker kept from starting among it, stays
//! named, so a resumed review runs just those.

use std::collections::BTreeSet;

use review_eval::component::Kind;
use review_gate::{BLOCKING, Finding, REQUEST_CHANGES_VERDICT};

use crate::controls::{OVER_BUDGET, mass_failure, trips};
use crate::outcome::{Decision, REVIEW_PATH};
use crate::plan::Plan;

/// How the abort's finding starts.
pub const ABORTED: &str = "review aborted:";

/// How many of `plan`'s specialists `decision` left without an accepted
/// report for a reason no cost cap gave.
#[must_use]
pub fn failed(plan: &Plan, decision: &Decision) -> usize {
    let specialists = plan.specialists();
    decision
        .not_reviewed
        .iter()
        .filter(|g| g.kind == Kind::Specialist.word() && !g.reason.starts_with(OVER_BUDGET))
        .filter(|g| specialists.contains(&g.name))
        .map(|g| g.name.as_str())
        .collect::<BTreeSet<&str>>()
        .len()
}

/// The abort's finding, when more than `fraction` of `plan`'s
/// specialists failed in `decision`; none otherwise.
#[must_use]
pub fn abort(plan: &Plan, decision: &Decision, fraction: f64) -> Option<Finding> {
    let (failed, dispatched) = (failed(plan, decision), plan.specialists().len());
    trips(failed, dispatched, fraction).then(|| Finding {
        path: REVIEW_PATH.to_owned(),
        line: 0,
        severity: BLOCKING.to_owned(),
        body: format!(
            "{ABORTED} {}, so no generalist that had not started was started; once fixed, \
             resume with --only <its gaps.json> --from <its review directory>",
            mass_failure(failed, dispatched)
        ),
    })
}

/// `decision`, opened with the abort's finding and requesting changes,
/// when more than `fraction` of `plan`'s specialists failed.
#[must_use]
pub fn amend(plan: &Plan, mut decision: Decision, fraction: f64) -> Decision {
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
