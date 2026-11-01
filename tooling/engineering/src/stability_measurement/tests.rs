use super::*;

const SNAPSHOTS: &str = "[[snapshot]]\nname = \"pr-99\"\npull = 99\n\
    base = \"cbf144c6e74833535424c54cbfd04a23d90b47b0\"\n\
    head = \"86ec49030e8e8fe2af2bc2fd9441a64167e12ab6\"\n";

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn the_checkout_s_measurement_states_review_workflow_and_report_hold() {
    let problems = problems(&checkout(), MEASUREMENT, snapshots::SNAPSHOTS).unwrap();

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_file_that_does_not_read_fails_the_check() {
    let err = problems(&checkout(), "no/such.yml", snapshots::SNAPSHOTS).unwrap_err();

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

    let bare = "on:\n  push:\njobs:\n  j:\n    steps:\n      - uses: actions/upload-artifact@x\n        with:\n          path: work/review/logs\n";
    let problems = measurement_problems(bare);
    assert_eq!(problems.len(), 9, "{problems:?}");
    assert!(problems[0].ends_with("cannot be started on demand"));
    assert!(problems[1].ends_with("not the default branch's definition alone"));
    assert!(problems[8].ends_with("every tool result its agents read"));
}

#[test]
fn a_measurement_that_hands_its_agents_the_token_or_keeps_unscrubbed_is_named() {
    let measurement = checkout().read(MEASUREMENT).unwrap();
    let handed = measurement.replace(
        "      - name: Review\n        env:\n",
        "      - name: Review\n        env:\n          CLAUDE_CODE_OAUTH_TOKEN: ${{ secrets.CLAUDE_CODE_OAUTH_TOKEN }}\n",
    );
    let problems = measurement_problems(&handed);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("which does more than stage it"));

    let ungated = measurement.replace(
        "if: always() && steps.scrub.outputs.scrubbed == 'true'",
        "if: always()",
    );
    let problems = measurement_problems(&ungated);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("uploads in step \"Keep the review\" before a scrub"));

    let unguarded = measurement.replace("github.event.repository.default_branch", "x");
    assert_eq!(measurement_problems(&unguarded).len(), 1);
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
fn a_snapshots_file_that_does_not_read_is_named() {
    assert!(snapshots_problems(SNAPSHOTS, 3).is_empty());
    assert_eq!(
        snapshots_problems(SNAPSHOTS, 2),
        vec!["the measurement reviews each snapshot 2 times, not three"]
    );

    let problems = snapshots_problems("snapshot = []", 3);
    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].ends_with("the snapshots file holds no snapshot"),
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
