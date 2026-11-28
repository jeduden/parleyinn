//! The error a run ended on in place of an answer, named where the
//! review reports that run's silence. A result that is an error holds
//! the harness's text, a usage limit or a call the model's API refused
//! among them, which is never the agent's answer: a generalist so ended
//! states no weighing, and the blocking finding on its silence says
//! why, in the harness's own words, so the author is not sent looking
//! for a turn budget, a deadline or a cost cap.

use review_gate::Finding;

use super::REVIEW_PATH;
use crate::run::AgentRun;

/// Adds the error each `unread` generalist's run in `runs` ended on, on
/// one line, to the finding in `findings` that names its silence. One
/// whose run ended on no error keeps its finding as it stands.
pub(super) fn name(runs: &[AgentRun], unread: &[(String, String)], findings: &mut [Finding]) {
    for (name, _) in unread {
        let Some(run) = runs.iter().find(|r| &r.name == name) else {
            continue;
        };
        let error: Vec<&str> = run.log.error.split_whitespace().collect();
        let silence = format!("{name} answered no finding");
        let finding = findings
            .iter_mut()
            .find(|f| f.path == REVIEW_PATH && f.body.starts_with(&silence));
        if let (Some(finding), false) = (finding, error.is_empty()) {
            finding.body = format!(
                "{silence}, no dropped one and no `none —` line (its run ended on an error in \
                 place of an answer: {}), so its weighing cannot be read",
                error.join(" ")
            );
        }
    }
}

#[cfg(test)]
mod tests;
