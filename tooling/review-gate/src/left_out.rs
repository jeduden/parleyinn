//! The dispatched agents a review left out, and why (ENG-28). The
//! review outcome names each agent that did not run in a blocking
//! finding, in one form this module writes and reads, so the gate's
//! error repeats the outcome's reason and the check it fails says what
//! to do. For a review the pull request's cost cap stopped, the reason
//! holds what the reviews cost and the cap, and the gate adds how what
//! they spent is reset.

use std::fmt;

use crate::Finding;

/// How the reason starts of a run the cost caps stopped or kept from
/// starting.
pub const OVER_BUDGET: &str = "over budget:";

/// What the gate adds for a review the pull request's cost cap
/// stopped: how what its reviews spent is reset.
pub const RESET: &str = "the pull request's cost cap is reached, and deleting its \
                         review-spend-pr-<N>-a* artifacts resets what its reviews spent \
                         (docs/ci.md)";

/// How a finding that names an agent that did not run goes on after
/// the agent's name, and how it ends after the reason.
const DID_NOT_RUN: (&str, &str) = (" did not run (", "), so the review is incomplete");

/// A dispatched agent the review left out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeftOut {
    pub agent: String,
    /// Why it did not run, as the review outcome says it; none when no
    /// finding of the outcome names it.
    pub why: Option<String>,
}

impl LeftOut {
    /// Whether a cost cap kept it from starting.
    #[must_use]
    pub fn over_budget(&self) -> bool {
        (self.why.as_deref()).is_some_and(|why| why.starts_with(OVER_BUDGET))
    }
}

impl fmt::Display for LeftOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.why {
            Some(why) => write!(f, "{} ({why})", self.agent),
            None => f.write_str(&self.agent),
        }
    }
}

/// The body of the review outcome's finding that names `agent`, which
/// did not run, and `why`.
#[must_use]
pub fn did_not_run(agent: &str, why: &str) -> String {
    format!("{agent}{}{why}{}", DID_NOT_RUN.0, DID_NOT_RUN.1)
}

/// Why `agent` did not run, as `body`, a finding of the review outcome,
/// says it; none when the body names another agent or says something
/// else.
#[must_use]
pub fn why_not_run<'a>(agent: &str, body: &'a str) -> Option<&'a str> {
    body.strip_prefix(agent)?
        .strip_prefix(DID_NOT_RUN.0)?
        .strip_suffix(DID_NOT_RUN.1)
}

/// Each of `agents` with why it did not run, from the review outcome's
/// `findings`.
#[must_use]
pub fn left_out(agents: impl IntoIterator<Item = String>, findings: &[Finding]) -> Vec<LeftOut> {
    agents
        .into_iter()
        .map(|agent| {
            let why = findings
                .iter()
                .find_map(|f| why_not_run(&agent, &f.body))
                .map(str::to_owned);
            LeftOut { agent, why }
        })
        .collect()
}

/// How the gate's error lists `agents`: each with its reason, and how
/// the spend is reset once a cost cap kept one from starting.
#[must_use]
pub fn listed(agents: &[LeftOut]) -> String {
    let names: Vec<String> = agents.iter().map(LeftOut::to_string).collect();
    let reset = if agents.iter().any(LeftOut::over_budget) {
        format!("; {RESET}")
    } else {
        String::new()
    };

    format!("{}{reset}", names.join(", "))
}

#[cfg(test)]
mod tests;
