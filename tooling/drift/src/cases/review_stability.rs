//! The drifts of the review's run list, report and stability measurement.

use super::*;

/// The drifts the review's run list, report and stability measurement rule out
/// (ENG-28, ENG-32), late so adding them moves no earlier case to
/// another shard: a run's text reaching the report as markup, a
/// verdict that flips measured as stable, an agent run's cost recomputed from
/// its models instead of taken from its log, and the measurement, nine
/// reviews, put on a schedule or run from any ref, its agents handed the
/// token or run without the staged credential, and its reviews uploaded
/// before the scrub.
pub(super) fn review_stability_cases() -> Vec<Case> {
    let report_test = |name: &'static str| cargo_lib_test("review-report", name);
    vec![
        case(
            "the report writing a run's `<` unescaped",
            "ENG-32",
            Injection::replace(
                "tooling/review-report/src/html.rs",
                "'<' => out.push_str(\"&lt;\"),",
                "'<' => out.push('<'),",
            ),
            report_test(
                "html::tests::a_run_s_name_reason_and_the_failure_never_reach_the_report_as_markup",
            ),
            "a_run_s_name_reason_and_the_failure_never_reach_the_report_as_markup",
        ),
        case(
            "review-report stability counting no verdict that flips",
            "ENG-32",
            Injection::replace(
                "tooling/review-report/src/stability.rs",
                "let flipped = verdicts.iter().filter(|v| **v != mode).count();",
                "let flipped = verdicts.iter().filter(|v| **v != mode && v.is_empty()).count();",
            ),
            report_test(
                "stability::tests::one_verdict_flip_in_three_is_detected_and_past_the_limit",
            ),
            "one_verdict_flip_in_three_is_detected_and_past_the_limit",
        ),
        case(
            "an agent run's cost recomputed from its models instead of its log's total",
            "ENG-32",
            Injection::replace(
                "tooling/review-report/src/agent_runs.rs",
                "cost_usd: log.cost,",
                "cost_usd: log.usage.values().map(|u| u.cost).sum(),",
            ),
            report_test(
                "agent_runs::tests::the_agent_run_cost_is_the_log_s_own_total_never_recomputed",
            ),
            "the_agent_run_cost_is_the_log_s_own_total_never_recomputed",
        ),
        case(
            "a withdrawn finding still counted in a specialist's finding identities",
            "ENG-32",
            Injection::replace(
                "tooling/review-report/src/identity.rs",
                "Line::Withdrawn(w) => held.retain(|(at, _)| *at != (w.path.clone(), w.line)),",
                "Line::Withdrawn(_) => {}",
            ),
            report_test(
                "identity::tests::a_revision_replaces_the_finding_it_revises_and_a_withdrawal_removes_it",
            ),
            "a_revision_replaces_the_finding_it_revises_and_a_withdrawal_removes_it",
        ),
        case(
            "an agent a review missed raising the stability mean",
            "ENG-32",
            Injection::replace(
                "tooling/review-report/src/stability.rs",
                "jaccard: if sets.len() < reviews.len() {\n            0.0\n        } else {",
                "jaccard: if sets.len() < reviews.len() && false {\n            0.0\n        } else {",
            ),
            report_test(
                "stability::tests::an_agent_missing_from_one_review_lowers_the_mean_even_with_no_findings",
            ),
            "an_agent_missing_from_one_review_lowers_the_mean_even_with_no_findings",
        ),
        case(
            "the stability measurement put on a schedule",
            "ENG-32",
            Injection::replace(
                ".github/workflows/stability.yml",
                "  # schedule:\n  #   - cron: \"23 4 * * 1\"\n",
                "  schedule:\n    - cron: \"23 4 * * 1\"\n",
            ),
            scenario("ENG-32"),
            "runs on a schedule, which only the stakeholder turns on",
        ),
    ]
}

/// The drifts the stability measurement's workflow rules out (ENG-28,
/// ENG-32), late for the same reason: the Claude token handed to the step
/// that runs its agents, its agents run without the staged credential, its
/// reviews uploaded before the scrub, and a dispatch from any ref.
pub(super) fn stability_workflow_cases() -> Vec<Case> {
    vec![
        case(
            "the Claude token handed to the step that runs the measurement's agents",
            "ENG-28",
            Injection::replace(
                ".github/workflows/stability.yml",
                "      - name: Review\n        env:\n",
                "      - name: Review\n        env:\n          CLAUDE_CODE_OAUTH_TOKEN: ${{ secrets.CLAUDE_CODE_OAUTH_TOKEN }}\n",
            ),
            scenario("ENG-28"),
            "stability.yml: agent job sample hands CLAUDE_CODE_OAUTH_TOKEN to step \"Review\"",
        ),
        case(
            "the measurement's agents run without the staged credential",
            "ENG-28",
            Injection::replace(
                ".github/workflows/stability.yml",
                " \\\n            --credential \"$RUNNER_TEMP/claude-credential\" > \"$RUNNER_TEMP/work/review/run.txt\"",
                " > \"$RUNNER_TEMP/work/review/run.txt\"",
            ),
            scenario("ENG-28"),
            "stability.yml: agent job sample runs agents in step \"Review\" without --credential",
        ),
        case(
            "the measurement's reviews uploaded before the scrub of the token",
            "ENG-28",
            Injection::replace(
                ".github/workflows/stability.yml",
                "        if: always() && steps.scrub.outputs.scrubbed == 'true'\n",
                "        if: always()\n",
            ),
            scenario("ENG-28"),
            "stability.yml: agent job sample uploads in step \"Keep the review\" before a scrub",
        ),
        case(
            "a file the review uploads that its scrub does not list",
            "ENG-28",
            Injection::replace(
                ".github/workflows/review.yml",
                " --path review/agent-runs.jsonl --path review/report.html\n",
                "\n",
            ),
            scenario("ENG-28"),
            "review.yml: agent job review uploads in step \"Keep the specialists' reports\" before a scrub",
        ),
        case(
            "the measurement run from any ref a dispatch picks",
            "ENG-32",
            Injection::replace(
                ".github/workflows/stability.yml",
                "    if: github.ref == format('refs/heads/{0}', github.event.repository.default_branch)\n",
                "",
            ),
            scenario("ENG-32"),
            "not the default branch's definition alone",
        ),
    ]
}
