use super::*;

const CORPUS: &str = "[[state]]\nname = \"pr-99\"\npull = 99\n\
    base = \"cbf144c6e74833535424c54cbfd04a23d90b47b0\"\n\
    head = \"86ec49030e8e8fe2af2bc2fd9441a64167e12ab6\"\n";

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn the_checkout_s_probe_corpus_review_workflow_and_report_hold() {
    let problems = problems(&checkout(), PROBE, corpus::CORPUS).unwrap();

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_file_that_does_not_read_fails_the_check() {
    let err = problems(&checkout(), "no/such.yml", corpus::CORPUS).unwrap_err();

    assert!(err.starts_with("read no/such.yml: "), "{err}");
}

#[test]
fn a_probe_that_is_scheduled_or_misses_a_part_is_named() {
    let probe = checkout().read(PROBE).unwrap();
    let scheduled = probe.replace("  # schedule:", "  schedule:");
    assert_eq!(
        probe_problems(&scheduled),
        vec![format!(
            "{PROBE} runs on a schedule, which only the stakeholder turns on"
        )]
    );

    let bare = "on:\n  push:\n  uses review/logs\n";
    let problems = probe_problems(bare);
    assert_eq!(problems.len(), 8, "{problems:?}");
    assert!(problems[0].ends_with("cannot be started on demand"));
    assert!(problems[7].ends_with("every tool result its agents read"));
}

#[test]
fn a_review_workflow_that_keeps_no_trace_or_page_is_named() {
    let review = checkout().read(REVIEW).unwrap();
    let without = review.replace("            review/trace.jsonl\n", "");

    assert_eq!(
        review_problems(&without),
        vec![format!(
            "{REVIEW} keeps no review/trace.jsonl with the reports"
        )]
    );
}

#[test]
fn a_corpus_that_does_not_read_is_named() {
    assert!(corpus_problems(CORPUS, 3).is_empty());
    assert_eq!(
        corpus_problems(CORPUS, 2),
        vec!["the probe reviews each state 2 times, not three"]
    );

    let problems = corpus_problems("state = []", 3);
    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].ends_with("the corpus holds no state"),
        "{problems:?}"
    );
}

#[test]
fn review_report_measures_a_flip_takes_the_log_s_cost_and_escapes_the_page() {
    assert!(report_problems(&made()).is_empty());
}

#[test]
fn review_report_missing_a_flip_recomputing_cost_or_leaving_markup_is_named() {
    let good = made();
    let bad = Made {
        flip: good.same.clone(),
        cost: 9.0,
        page: "<script>".to_owned(),
        ..good
    };

    assert_eq!(
        report_problems(&bad),
        vec![
            "review-report stability does not find a verdict that flips",
            "a span's cost is 9, not the 0.25 its run log reported",
            "the report page holds a run's text as markup",
        ]
    );
}
