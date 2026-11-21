//! ENG-28: what the review gate says of a review the pull request's
//! cost cap stopped. Such a review leaves agents out, so the gate fails
//! without deciding, and the check it fails must tell its reader what
//! to do. The gate's error names each agent a cost cap kept from
//! starting with the reason the review outcome gives, which holds what
//! the pull request's reviews cost and the cap, and says how what they
//! spent is reset; the job that holds the reviewer app's key puts that
//! error in the summary of the check it fails. The checks run
//! `review-run` under a cap its first run reaches, then the gate on
//! what it wrote, and read the job's text.

use std::collections::BTreeMap;
use std::path::Path;

use review_eval::runner::SPECIALIST_BUDGET_USD;
use review_gate::left_out::{OVER_BUDGET, RESET};

use crate::Checkout;
use crate::review_controls::{ControlHost, gaps, review};
use crate::review_refusals::gate;
use crate::review_workflow::Workflow;

/// What the job that decides the review does to say why the gate did
/// not decide, and the text that does it.
const SAYS: [(&str, &str); 3] = [
    ("keep the gate's errors:", "2> gate-error.txt"),
    (
        "take the gate's own line from them:",
        "grep -m 1 '^review-gate: ' gate-error.txt || tail -n 1 gate-error.txt",
    ),
    (
        "say the gate's error in the check it fails:",
        "failed before deciding: $why",
    ),
];

/// What the job of `flow` that holds `key` lacks to say, in the check
/// it fails, why the gate did not decide.
///
/// # Errors
///
/// Fails when the key's job cannot be found.
pub fn check_problems(flow: &Workflow, key: &str) -> Result<Vec<String>, String> {
    let job = flow.key_job(key)?;

    Ok(SAYS
        .iter()
        .filter(|(_, text)| !job.text.contains(text))
        .map(|(what, text)| format!("job {} does not {what} {text}", job.name))
        .collect())
}

/// What is wrong with what the gate made of a review whose `gaps` name
/// the agents a cost cap kept from starting: a review posted, an agent
/// whose reason the gate's error does not repeat, or an error that does
/// not say how the spend is reset.
#[must_use]
pub fn said_problems(
    gate: &Result<review_gate::Review, String>,
    gaps: &BTreeMap<String, String>,
) -> Vec<String> {
    let said = match gate {
        Ok(review) => {
            return vec![format!(
                "the gate posts {} for a review a cost cap kept agents out of",
                review.event
            )];
        }
        Err(said) => said,
    };
    let mut problems: Vec<String> = gaps
        .iter()
        .filter(|(_, why)| why.starts_with(OVER_BUDGET))
        .filter(|(name, why)| !said.contains(&format!("{name} ({why})")))
        .map(|(name, _)| format!("the gate's error does not say why {name} did not run: {said}"))
        .collect();
    if !said.contains(RESET) {
        problems.push(format!(
            "the gate's error does not say how what the pull request's reviews spent is reset: \
             {said}"
        ));
    }

    problems
}

/// What is wrong with what the gate says of a review of `checkout`'s
/// whose cost cap its first run reaches, run in `workdir`.
#[must_use]
pub fn capped_problems(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    // One specialist's cap fits, and none beside what it cost.
    let cap = (SPECIALIST_BUDGET_USD + 0.005).to_string();
    let flags = ["--parallel", "1", "--max-cost-usd", cap.as_str()];
    let host = ControlHost::default();
    match review(checkout, workdir, &host, &flags).and_then(|_| gaps(workdir)) {
        Ok(gaps) => said_problems(&gate(workdir), &gaps),
        Err(e) => vec![e],
    }
}

#[cfg(test)]
mod tests;
