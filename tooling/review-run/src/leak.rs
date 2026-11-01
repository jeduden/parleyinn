//! The review failing closed on a leaked Claude token (ENG-28, I6):
//! every text an agent's run produced is scrubbed of the token before
//! the review writes or prints it, each form found counted, and a
//! review that found one requests changes, names the leak in a
//! blocking finding of its own and exits as one that could not run, so
//! nothing it decided is posted.

use review_eval::secret::Scrubber;
use review_gate::{BLOCKING, Finding, REQUEST_CHANGES_VERDICT, ReviewOutcome};

use crate::outcome::REVIEW_PATH;
use crate::run::AgentRun;

/// Scrubs every field of `runs` an agent's output reaches, in place,
/// and answers how many forms of the token it replaced.
pub fn scrub_runs(scrubber: &Scrubber, runs: &mut [AgentRun]) -> usize {
    let mut count = 0;
    let mut scrub = |text: &mut String| {
        let (clean, n) = scrubber.scrub(text);
        if n > 0 {
            *text = clean;
            count += n;
        }
    };
    for run in runs.iter_mut() {
        if let Some(failure) = run.failure.as_mut() {
            scrub(failure);
        }
        scrub(&mut run.log.subtype);
        scrub(&mut run.log.answer);
        run.log.texts.iter_mut().for_each(&mut scrub);
        run.log.writes.iter_mut().for_each(&mut scrub);
        run.failed_parts.iter_mut().for_each(|(_, why)| scrub(why));
    }

    count
}

/// Why a review that found the token in `n` places fails.
#[must_use]
pub fn reason(n: usize) -> String {
    format!("an agent's output contained the Claude token: {n} redacted; revoke and rotate it")
}

/// Makes `outcome` request changes for a leak of `n` forms of the
/// token, named in a blocking finding of its own and first in its
/// summary.
pub fn fail_closed(outcome: &mut ReviewOutcome, n: usize) {
    REQUEST_CHANGES_VERDICT.clone_into(&mut outcome.verdict);
    outcome.summary = format!("{}. {}", reason(n), outcome.summary);
    outcome.findings.push(Finding {
        path: REVIEW_PATH.to_owned(),
        line: 0,
        severity: BLOCKING.to_owned(),
        body: format!(
            "an agent's output contained the Claude token: {n} redacted from what the \
             review wrote; the token must be revoked and rotated, and nothing this review \
             decided is posted (ENG-28)"
        ),
    });
}

#[cfg(test)]
mod tests;
