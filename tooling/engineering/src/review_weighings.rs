//! ENG-28: how the review's runner reads a generalist's weighing. It
//! reads it from the generalist's whole run log, each weighing line
//! stated as it went, the last at a place standing, then its final
//! answer, and it decides itself, not by what the generalist says of its
//! scope, that a weighing is partial when a place a specialist's
//! accepted report names is weighed nowhere. The check runs `review-run`
//! twice on a work directory with a stand-in for the `claude` command
//! line whose specialist states one finding: once with a generalist
//! that states its weighing of that finding in a message before its
//! final answer, a sentence on its scope alone, which must approve with
//! the nit it kept; once with one that weighs another place and says
//! its scope is whole, which must request changes naming the place it
//! weighed nowhere and the generalist as not reviewed; and once with one
//! whose line names the finding's file in a range wider than
//! [`review_run::outcome::MAX_RANGE`] lines, which covers its first line
//! alone and so must be decided partial the same way.

use std::fs;
use std::path::Path;

use review_eval::host::{Command, Host, Output};

use crate::Checkout;
use crate::review_refusals::{GENERALIST, SPECIALIST, prepare};
use crate::review_runs::agent_of;

/// The place the specialist's finding is at.
pub const PLACE: &str = "src/a.rs:1";

/// A weighing line, but for its source, that weighs the finding at
/// [`PLACE`].
pub const WHOLE: &str = "src/a.rs:1 — nit — a hook panics";
/// One at another place, naming none.
pub const ELSEWHERE: &str = "src/z.rs:9 — nit — a hook panics";
/// One at another place, naming [`PLACE`]'s file in a range past
/// [`review_run::outcome::MAX_RANGE`] lines, which covers its first
/// line alone.
pub const WIDE: &str = "src/z.rs:9 — nit — a hook panics, as all of src/a.rs:0-9999";

/// A stand-in for the `claude` command line: the specialist states a
/// finding at [`PLACE`] and answers its outcome line; the generalist
/// states its weighing in one message, the line `weighs` then its
/// source, and answers a sentence on its scope alone.
#[derive(Debug)]
pub struct WeighingHost {
    pub weighs: &'static str,
}

impl Host for WeighingHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        if let Some(out) = crate::review_runs::version(command) {
            return Ok(out);
        }
        let (texts, result) = if agent_of(command) == GENERALIST {
            let scope = format!("{GENERALIST} weighed its whole scope.");
            (
                vec![format!("{} — {SPECIALIST}", self.weighs), scope.clone()],
                scope,
            )
        } else {
            let outcome = "outcome: findings — one finding.".to_owned();
            (
                vec![
                    format!("{PLACE} — if not addressed: a hook panics — I9"),
                    outcome.clone(),
                ],
                outcome,
            )
        };
        let log = command
            .stdout_to
            .as_deref()
            .ok_or("the run logs to no file")?;
        let calls: Vec<String> = texts
            .iter()
            .enumerate()
            .map(|(at, text)| {
                serde_json::json!({"type": "assistant", "parent_tool_use_id": null,
                    "message": {"id": format!("m{at}"), "model": "claude-haiku-5-5",
                    "usage": {}, "content": [{"type": "text", "text": text}]}})
                .to_string()
            })
            .collect();
        let line = serde_json::json!({"type": "result", "subtype": "success",
            "is_error": false, "result": result, "total_cost_usd": 0.01, "num_turns": 2});
        let reads = crate::review_runs::reads_of(command, "m0");
        fs::write(log, format!("{reads}{}\n{line}\n", calls.join("\n")))
            .map_err(|e| format!("write {}: {e}", log.display()))?;

        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    }
}

/// What is wrong with the outcome `review-run` wrote in `workdir` for a
/// generalist that weighed the finding as it went: a review that does
/// not approve, or keeps no nit at [`PLACE`].
#[must_use]
pub fn whole_problems(workdir: &Path) -> Vec<String> {
    let Some(outcome) = outcome_in(workdir) else {
        return vec!["review-run wrote no review outcome that reads".to_owned()];
    };
    let mut problems = Vec::new();
    if outcome.verdict != review_gate::APPROVE_VERDICT {
        problems.push(format!(
            "a weighing stated before the final answer is not read: the review is {}",
            outcome.verdict
        ));
    }
    let kept = outcome
        .findings
        .iter()
        .any(|f| format!("{}:{}", f.path, f.line) == PLACE && f.severity == review_gate::NIT);
    if !kept {
        problems.push(format!(
            "the nit {GENERALIST} stated at {PLACE} before its final answer is not kept"
        ));
    }

    problems
}

/// What is wrong with what `review-run` wrote in `workdir` for a
/// generalist that weighed another place and said its scope was whole:
/// a review that does not request changes, no blocking finding naming
/// [`PLACE`] as weighed nowhere, or the generalist not named as not
/// reviewed.
#[must_use]
pub fn partial_problems(workdir: &Path) -> Vec<String> {
    let Some(outcome) = outcome_in(workdir) else {
        return vec!["review-run wrote no review outcome that reads".to_owned()];
    };
    let mut problems = Vec::new();
    if outcome.verdict != review_gate::REQUEST_CHANGES_VERDICT {
        problems.push(format!(
            "a partial weighing does not request changes: {}",
            outcome.verdict
        ));
    }
    let weighed_no = format!("weighed no finding at {PLACE}");
    let named = outcome.findings.iter().any(|f| {
        f.severity == review_gate::BLOCKING && f.path == "review/" && f.body.contains(&weighed_no)
    });
    if !named {
        problems.push(format!(
            "no blocking finding names {PLACE}, which {GENERALIST} weighed nowhere"
        ));
    }
    let gaps = fs::read_to_string(workdir.join(review_run::gaps::GAPS)).unwrap_or_default();
    let gapped = review_run::gaps::parse(&gaps)
        .unwrap_or_default()
        .iter()
        .any(|g| g.name == GENERALIST && g.reason.contains(&weighed_no));
    if !gapped {
        problems.push(format!(
            "{GENERALIST}, whose weighing is partial, is not named as not reviewed"
        ));
    }

    problems
}

/// The review outcome `review-run` wrote in `workdir`, when it reads.
fn outcome_in(workdir: &Path) -> Option<review_gate::ReviewOutcome> {
    let text = fs::read_to_string(workdir.join(review_run::OUTCOME)).ok()?;
    review_gate::parse_outcome(&text).ok()
}

/// Runs `review-run` on `workdir`, prepared from `checkout`, with a
/// [`WeighingHost`] weighing as `weighs`: why it could not run, or
/// nothing.
fn review(checkout: &Checkout, workdir: &Path, weighs: &'static str) -> Result<(), String> {
    prepare(checkout, workdir)?;
    let args = ["--workdir".to_owned(), workdir.display().to_string()];
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = review_run::cli::run(&WeighingHost { weighs }, args, &mut out, &mut err);
    if code != review_run::cli::EXIT_OK {
        let err = String::from_utf8_lossy(&err);
        return Err(format!(
            "review-run exited {code}: {}",
            err.lines().last().unwrap_or_default()
        ));
    }

    Ok(())
}

/// What is wrong with how `review-run`, on work directories under
/// `workdir` prepared from `checkout`, reads a weighing stated as it
/// went and decides a partial one, a range too wide to cover the place
/// included.
#[must_use]
pub fn review_run_reads_whole_weighings(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    let whole = workdir.join("whole");
    let partial = workdir.join("partial");
    let wide = workdir.join("wide");
    let ran = review(checkout, &whole, WHOLE)
        .and_then(|()| review(checkout, &partial, ELSEWHERE))
        .and_then(|()| review(checkout, &wide, WIDE));
    match ran {
        Ok(()) => {
            let mut problems = whole_problems(&whole);
            problems.extend(partial_problems(&partial));
            problems.extend(
                partial_problems(&wide)
                    .into_iter()
                    .map(|p| format!("a weighing naming its file in a range too wide: {p}")),
            );
            problems
        }
        Err(e) => vec![e],
    }
}

#[cfg(test)]
mod tests;
