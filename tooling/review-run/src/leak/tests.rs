use review_eval::component::Kind;
use review_eval::log::RunLog;
use review_eval::secret::REDACTED;
use review_gate::APPROVE_VERDICT;

use super::*;

/// A stand-in for the access token; never a real one.
const TOKEN: &str = "dummy-access-token-0123456789";

fn leaking_run() -> AgentRun {
    AgentRun {
        name: "lang-rust".to_owned(),
        kind: Kind::Specialist,
        failure: Some(format!("its run ended near {TOKEN}")),
        log: RunLog {
            subtype: format!("x{TOKEN}"),
            answer: format!("outcome: none — {TOKEN}"),
            texts: vec![format!("env: {TOKEN}"), "clean".to_owned()],
            writes: vec![TOKEN.to_owned()],
            ..RunLog::default()
        },
        attempts: 1,
        parts: 2,
        failed_parts: vec![(1, format!("part {TOKEN}"))],
        reused: false,
    }
}

#[test]
fn every_agent_written_field_of_a_run_is_scrubbed_and_counted() {
    let scrubber = Scrubber::new(&[TOKEN]);
    let mut runs = vec![leaking_run()];

    assert_eq!(scrub_runs(&scrubber, &mut runs), 6);

    let run = &runs[0];
    let all = format!("{run:?}");
    assert!(!all.contains(TOKEN), "{all}");
    assert_eq!(
        run.log.texts,
        [format!("env: {REDACTED}"), "clean".to_owned()]
    );
    assert_eq!(run.failed_parts, [(1, format!("part {REDACTED}"))]);
    assert_eq!(scrub_runs(&scrubber, &mut runs), 0);
}

#[test]
fn a_leak_makes_the_review_request_changes_with_a_named_blocking_finding() {
    let mut outcome = ReviewOutcome {
        verdict: APPROVE_VERDICT.to_owned(),
        summary: "All agents answered.".to_owned(),
        findings: Vec::new(),
    };

    fail_closed(&mut outcome, 2);

    assert_eq!(outcome.verdict, REQUEST_CHANGES_VERDICT);
    assert_eq!(
        outcome.summary,
        format!("{}. All agents answered.", reason(2))
    );
    assert_eq!(outcome.findings.len(), 1);
    let finding = &outcome.findings[0];
    assert_eq!(finding.severity, BLOCKING);
    assert_eq!(finding.path, crate::outcome::REVIEW_PATH);
    assert_eq!(
        finding.body,
        "an agent's output contained the access token: 2 redacted from what the review \
         wrote; the token must be revoked and rotated, and nothing this review decided \
         is posted (ENG-28)"
    );
}

#[test]
fn the_reason_names_the_count_and_what_to_do() {
    assert_eq!(
        reason(1),
        "an agent's output contained the access token: 1 redacted; revoke and rotate it"
    );
}
