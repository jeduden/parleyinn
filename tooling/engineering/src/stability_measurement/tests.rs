use super::*;

const STATES: &str = "[[state]]\nname = \"pr-99\"\npull = 99\n\
    base = \"cbf144c6e74833535424c54cbfd04a23d90b47b0\"\n\
    head = \"86ec49030e8e8fe2af2bc2fd9441a64167e12ab6\"\n";

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn the_checkout_s_measurement_states_review_workflow_and_report_hold() {
    let problems = problems(&checkout(), MEASUREMENT, states::STATES).unwrap();

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_file_that_does_not_read_fails_the_check() {
    let err = problems(&checkout(), "no/such.yml", states::STATES).unwrap_err();

    assert!(err.starts_with("read no/such.yml: "), "{err}");
}

#[test]
fn a_measurement_that_is_scheduled_or_misses_a_part_is_named() {
    let measurement = checkout().read(MEASUREMENT).unwrap();
    let scheduled = measurement.replace("  # schedule:", "  schedule:");
    assert_eq!(
        measurement_problems(&scheduled),
        vec![format!(
            "{MEASUREMENT} runs on a schedule, which only the stakeholder turns on"
        )]
    );

    let bare = "on:\n  push:\n  uses review/logs\n";
    let problems = measurement_problems(bare);
    assert_eq!(problems.len(), 8, "{problems:?}");
    assert!(problems[0].ends_with("cannot be started on demand"));
    assert!(problems[7].ends_with("every tool result its agents read"));
}

#[test]
fn a_review_workflow_that_keeps_no_run_list_or_report_is_named() {
    let review = checkout().read(REVIEW).unwrap();
    let without = review.replace("            review/agent-runs.jsonl\n", "");

    assert_eq!(
        review_problems(&without),
        vec![format!(
            "{REVIEW} keeps no review/agent-runs.jsonl with the reports"
        )]
    );
}

#[test]
fn a_states_that_does_not_read_is_named() {
    assert!(states_problems(STATES, 3).is_empty());
    assert_eq!(
        states_problems(STATES, 2),
        vec!["the measurement reviews each state 2 times, not three"]
    );

    let problems = states_problems("state = []", 3);
    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].ends_with("the states holds no state"),
        "{problems:?}"
    );
}

#[test]
fn review_report_measures_a_flip_takes_the_log_s_cost_and_escapes_the_report() {
    assert!(report_problems(&made()).is_empty());
}

#[test]
fn review_report_missing_a_flip_recomputing_cost_or_leaving_markup_is_named() {
    let good = made();
    let bad = Made {
        flip: good.same.clone(),
        cost: 9.0,
        report: "<script>".to_owned(),
        ..good
    };

    assert_eq!(
        report_problems(&bad),
        vec![
            "review-report stability does not find a verdict that flips",
            "an agent run's cost is 9, not the 0.25 its run log reported",
            "the report holds a run's text as markup",
        ]
    );
}
