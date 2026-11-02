//! The drifts of how a review carries the findings of the previous
//! review of its pull request (ENG-28), apart from the registry's other
//! cases so its file grows no further.

use super::case;
use crate::{Case, Injection, cargo_lib_test};

/// The drifts the per-pull-request findings exist to rule out (ENG-28,
/// I2, I6): an artifact of the name any workflow uploaded taken as the
/// previous review, the findings never passed to the next review, kept
/// under a name no pull request owns, or picked with no origin check.
pub(super) fn review_previous_cases() -> Vec<Case> {
    const PREVIOUS: &str = "tooling/review-run/src/previous.rs";
    const WORKFLOW: &str = ".github/workflows/review.yml";
    let carried = |test: &'static str| {
        cargo_lib_test(
            "engineering",
            match test {
                "workflow" => {
                    "review_previous::tests::the_checkout_s_review_carries_findings_from_main_s_review_workflow_only"
                }
                _ => {
                    "review_previous::tests::the_checkout_s_selection_trusts_main_s_review_workflow_alone"
                }
            },
        )
    };
    vec![
        case(
            "a previous review taken from a run of any workflow, so a pull request plants its own history",
            "ENG-28",
            Injection::replace(
                PREVIOUS,
                "file == WORKFLOW_PATH && EVENTS.contains(&self.event.as_str())",
                "EVENTS.contains(&self.event.as_str())",
            ),
            carried("selection"),
            "previous select took",
        ),
        case(
            "a previous review taken from a run on the pull request's own event",
            "ENG-28",
            Injection::replace(
                PREVIOUS,
                "file == WORKFLOW_PATH && EVENTS.contains(&self.event.as_str())",
                "file == WORKFLOW_PATH",
            ),
            carried("selection"),
            "previous select took",
        ),
        case(
            "a review never given the findings of the previous one",
            "ENG-28",
            Injection::replace(WORKFLOW, "${PREVIOUS:+--previous \"$PREVIOUS\"}", ""),
            carried("workflow"),
            "as --previous",
        ),
        case(
            "the tracked findings kept under one artifact name for every pull request",
            "ENG-28",
            Injection::replace(
                WORKFLOW,
                "name: review-findings-pr-${{ steps.pull_head.outputs.number }}",
                "name: review-findings",
            ),
            carried("workflow"),
            "one artifact name per pull request",
        ),
        case(
            "the previous findings picked with no check of the run that uploaded them",
            "ENG-28",
            Injection::replace(
                "scripts/review-previous.sh",
                "previous select",
                "previous list",
            ),
            carried("workflow"),
            "previous select",
        ),
    ]
}
