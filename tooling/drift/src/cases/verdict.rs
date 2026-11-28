//! The drifts of how a comment's review reuses the review outcome an
//! earlier review of the same head kept (ENG-37), apart from the
//! registry's other cases so its file grows no further.

use super::case;
use crate::{Case, Check, Injection, cargo_lib_test};

/// The drifts the reuse exists to rule out (ENG-37, I6): an outcome
/// reused for a head that moved, under a dispatch that changed or from a
/// review that left a gap, agents that run although one is reused, a
/// review that keeps nothing to reuse, and a comment that cancels the
/// review it waits for.
pub(super) fn review_verdict_cases() -> Vec<Case> {
    [weighing_cases(), wiring_cases()].concat()
}

/// The engineering check of the checkout's `previous verdict`.
fn weighing_check() -> Check {
    cargo_lib_test(
        "engineering",
        "review_verdict::tests::the_checkout_s_weighing_reuses_a_verdict_for_the_same_head_and_dispatch_alone",
    )
}

/// The engineering check of how the workflow and script reuse one.
fn wiring_check() -> Check {
    cargo_lib_test(
        "engineering",
        "review_verdict::tests::the_checkout_s_review_reuses_the_agents_verdict_after_a_comment",
    )
}

/// The drifts of the weighing in `previous verdict`.
fn weighing_cases() -> Vec<Case> {
    const VERDICT: &str = "tooling/review-run/src/verdict.rs";
    vec![
        case(
            "an earlier review's outcome reused for a head that moved",
            "ENG-37",
            Injection::replace(VERDICT, "Some(was) if was == head => {}", "Some(_) => {}"),
            weighing_check(),
            "for a head that moved",
        ),
        case(
            "an earlier review's outcome reused from a review that left a gap",
            "ENG-37",
            Injection::replace(VERDICT, "if !gaps.is_empty() {", "if gaps.len() > 9 {"),
            weighing_check(),
            "of a review that left a gap",
        ),
    ]
}

/// The workflow and script drifts.
fn wiring_cases() -> Vec<Case> {
    const WORKFLOW: &str = ".github/workflows/review.yml";
    vec![
        case(
            "the agents run although an earlier review's outcome is reused",
            "ENG-37",
            Injection::replace(
                WORKFLOW,
                "number != '' && steps.verdict.outputs.reused != 'true'",
                "number != ''",
            ),
            wiring_check(),
            "the agents run although an earlier verdict is reused",
        ),
        case(
            "a review that keeps no run id for a comment's review to weigh",
            "ENG-37",
            Injection::replace(WORKFLOW, "            review/run-id.json\n", ""),
            wiring_check(),
            "does not keep review/run-id.json",
        ),
        case(
            "a comment's review that cancels the review it waits for",
            "ENG-37",
            Injection::replace(
                WORKFLOW,
                "cancel-in-progress: ${{ github.event_name == 'workflow_run' }}",
                "cancel-in-progress: true",
            ),
            wiring_check(),
            "cancels a running review",
        ),
        case(
            "an earlier review's outcome looked for after a CI run too",
            "ENG-37",
            Injection::replace(
                "scripts/review-verdict.sh",
                "[ \"$EVENT\" = issue_comment ] || decided false",
                "[ \"$EVENT\" != none ] || decided false",
            ),
            wiring_check(),
            "looks only after a comment",
        ),
    ]
}
