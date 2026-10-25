//! The gate's public API as the command line chains it: parse the
//! agent's outcome, the check runs and the deterministic findings,
//! then decide.

#![allow(
    clippy::unwrap_used,
    reason = "a test's parse of its own fixture fails as the assertion"
)]

use review_gate::{
    APPROVE, Input, REQUEST_CHANGES, decide, parse_check_runs, parse_checked, parse_consulted,
    parse_dispatch, parse_outcome,
};

const HEAD: &str = "206b0a76a81abb510e53d87ef187e83b5aacbcbb";

fn input(outcome: &str, head: &str) -> Input {
    Input {
        outcome: parse_outcome(outcome).unwrap(),
        reviewed: HEAD.to_owned(),
        head: head.to_owned(),
        check_runs: parse_check_runs(
            "{\"name\":\"CI\",\"status\":\"completed\",\"conclusion\":\"success\"}\n",
        )
        .unwrap(),
        checked: parse_checked("[]").unwrap(),
        dispatched: parse_dispatch(r#"{"review-code":["lang-rust"]}"#).unwrap(),
        consulted: parse_consulted(r#"["lang-rust","review-code"]"#).unwrap(),
    }
}

#[test]
fn an_approval_on_a_green_head_is_posted_as_one() {
    let got = decide(&input(r#"{"verdict":"approve","summary":"s"}"#, HEAD)).unwrap();

    assert_eq!(
        got.map(|r| (r.event, r.commit_id)),
        Some((APPROVE.to_owned(), HEAD.to_owned()))
    );
}

#[test]
fn a_blocking_finding_turns_an_approval_into_a_change_request() {
    let outcome = r#"{"verdict":"approve","summary":"s","findings":[{"path":"a.rs","line":1,"severity":"blocking","body":"b"}]}"#;

    let got = decide(&input(outcome, HEAD)).unwrap();

    assert_eq!(got.map(|r| r.event), Some(REQUEST_CHANGES.to_owned()));
}

#[test]
fn a_head_that_moved_gets_no_review() {
    let got = decide(&input(r#"{"verdict":"approve","summary":"s"}"#, "490efa7")).unwrap();

    assert_eq!(got, None);
}

#[test]
fn a_blocking_deterministic_finding_overrules_the_agents_approval() {
    let mut i = input(r#"{"verdict":"approve","summary":"s"}"#, HEAD);
    i.checked = parse_checked(
        r#"[{"path":"features/record.feature","line":0,"severity":"blocking","body":"re-pended"}]"#,
    )
    .unwrap();

    let got = decide(&i).unwrap();

    assert_eq!(got.map(|r| r.event), Some(REQUEST_CHANGES.to_owned()));
}
