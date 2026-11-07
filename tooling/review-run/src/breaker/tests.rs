use std::collections::{BTreeMap, BTreeSet};

use review_gate::{APPROVE_VERDICT, ReviewOutcome};

use super::*;
use crate::controls::{BREAKER_FRACTION, Fraction};
use crate::gaps::Gap;
use crate::plan::parse_dispatch;

fn plan() -> Plan {
    Plan {
        class: "standard".to_owned(),
        touched: 1,
        dispatch: parse_dispatch(
            r#"{"review-code": ["lang-rust", "lang-shell", "test-engineer-unit"],
                "review-spec": ["lang-markdown", "srs-01-introduction", "srs-02-overall"]}"#,
        )
        .unwrap(),
    }
}

fn decision(gaps: Vec<Gap>) -> Decision {
    Decision {
        outcome: ReviewOutcome {
            verdict: APPROVE_VERDICT.to_owned(),
            summary: "6 of 8 dispatched agents answered.".to_owned(),
            findings: Vec::new(),
        },
        consulted: BTreeSet::new(),
        answers: BTreeMap::new(),
        weighings: BTreeMap::new(),
        not_reviewed: gaps,
    }
}

#[test]
fn a_specialist_counts_once_however_many_of_its_parts_failed() {
    let gaps = vec![
        Gap::new("lang-rust", Some(1), "silent: x".to_owned()),
        Gap::new("lang-rust", Some(2), "silent: x".to_owned()),
        Gap::new("lang-shell", None, "refused: y".to_owned()),
        Gap::new("lang-markdown", None, "over budget: z".to_owned()),
        Gap::new("review-code", None, "silent: w".to_owned()),
        Gap::new("lang-python", None, "silent: not dispatched".to_owned()),
    ];

    assert_eq!(
        failed(&plan(), &decision(gaps)),
        3,
        "a cost cap's gap counts, a part twice or a generalist does not"
    );
}

#[test]
fn more_than_a_third_failed_opens_the_outcome_with_the_abort_and_requests_changes() {
    let gaps = ["lang-rust", "lang-shell", "lang-markdown"]
        .into_iter()
        .map(|n| Gap::new(n, None, "silent: its run ended with deadline".to_owned()))
        .collect();

    let amended = amend(&plan(), decision(gaps), BREAKER_FRACTION);

    let want = "review aborted: 3 of 6 specialists failed (mass failure; likely an outage or a \
                broken prompt), so no generalist was started; resume with --resume once fixed \
                (scripts/review-local.sh --resume, or review-run --only <its gaps.json> --from <its \
                review directory>)";
    assert_eq!(amended.outcome.verdict, REQUEST_CHANGES_VERDICT);
    assert_eq!(amended.outcome.findings[0].body, want);
    assert_eq!(amended.outcome.findings[0].path, REVIEW_PATH);
    assert_eq!(amended.outcome.findings[0].severity, BLOCKING);
    assert!(amended.outcome.summary.starts_with(want));
    assert!(
        amended
            .outcome
            .summary
            .ends_with("6 of 8 dispatched agents answered.")
    );
    assert_eq!(amended.not_reviewed.len(), 3);
}

#[test]
fn a_third_or_fewer_failed_leaves_the_decision_as_it_was() {
    let gaps: Vec<Gap> = ["lang-rust", "lang-shell"]
        .into_iter()
        .map(|n| Gap::new(n, None, "silent: x".to_owned()))
        .collect();
    let before = decision(gaps);

    assert_eq!(amend(&plan(), before.clone(), BREAKER_FRACTION), before);
    assert_eq!(abort(&plan(), &before, BREAKER_FRACTION), None);
    assert!(abort(&plan(), &before, Fraction { num: 3, den: 10 }).is_some());
}
