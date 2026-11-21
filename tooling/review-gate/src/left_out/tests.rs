use super::*;
use crate::{BLOCKING, REVIEW_PATH};

/// Why a run the pull request's cost cap kept from starting did not
/// run, as `review-run` says it.
const DRY: &str = "over budget: the pull request's reviews cost 4.9100 USD, and a run capped at \
                   1.50 USD does not fit in its cap of 5.00 USD, so it was not started";

fn finding(body: String) -> Finding {
    Finding {
        path: REVIEW_PATH.into(),
        line: 0,
        severity: BLOCKING.into(),
        body,
    }
}

fn out(agent: &str, why: Option<&str>) -> LeftOut {
    LeftOut {
        agent: agent.into(),
        why: why.map(str::to_owned),
    }
}

#[test]
fn did_not_run_names_the_agent_and_why() {
    assert_eq!(
        did_not_run("review-code", "silent: no log"),
        "review-code did not run (silent: no log), so the review is incomplete"
    );
}

#[test]
fn why_not_run_reads_what_did_not_run_wrote() {
    let body = did_not_run("review-code", DRY);

    assert_eq!(why_not_run("review-code", &body), Some(DRY));
}

#[test]
fn why_not_run_reads_no_other_agent_s_finding_nor_another_sentence() {
    let body = did_not_run("review-code-deep", DRY);

    assert_eq!(why_not_run("review-code", &body), None);
    assert_eq!(why_not_run("review-spec", &body), None);
    assert_eq!(
        why_not_run("review-code", "review-code did not run (silent: no log)"),
        None
    );
    assert_eq!(why_not_run("review-code", "add a test"), None);
}

#[test]
fn left_out_takes_each_agent_s_reason_from_the_outcome() {
    let findings = [
        finding("add a test".into()),
        finding(did_not_run("review-code", DRY)),
    ];

    let got = left_out(
        ["lang-rust".to_owned(), "review-code".to_owned()],
        &findings,
    );

    assert_eq!(got, [out("lang-rust", None), out("review-code", Some(DRY))]);
}

#[test]
fn left_out_takes_no_reason_from_a_finding_on_a_file_of_the_change() {
    // An agent's finding may state the sentence; only the review's own
    // finding says why an agent did not run.
    let stated = Finding {
        path: "src/lib.rs".into(),
        ..finding(did_not_run("review-code", DRY))
    };

    let got = left_out(["review-code".to_owned()], &[stated]);

    assert_eq!(got, [out("review-code", None)]);
}

#[test]
fn over_budget_holds_only_for_a_run_a_cost_cap_kept_from_starting() {
    assert!(out("review-code", Some(DRY)).over_budget());
    assert!(!out("review-code", Some("silent: no log")).over_budget());
    assert!(!out("review-code", None).over_budget());
}

#[test]
fn an_agent_left_out_reads_with_its_reason_when_it_has_one() {
    assert_eq!(out("lang-rust", None).to_string(), "lang-rust");
    assert_eq!(
        out("lang-rust", Some("silent: no log")).to_string(),
        "lang-rust (silent: no log)"
    );
}

#[test]
fn listed_names_the_reset_once_a_cost_cap_kept_an_agent_from_starting() {
    let silent = [
        out("lang-rust", None),
        out("srs-security", Some("silent: x")),
    ];
    let dry = [out("lang-rust", None), out("review-code", Some(DRY))];

    assert_eq!(listed(&silent), "lang-rust, srs-security (silent: x)");
    assert_eq!(
        listed(&dry),
        format!("lang-rust, review-code ({DRY}); {RESET}")
    );
    assert!(RESET.contains("review-spend-pr-<N>-a*"));
}
