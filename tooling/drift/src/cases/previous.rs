//! The drifts of how a review carries the findings of the previous
//! review of its pull request (ENG-28), apart from the registry's other
//! cases so its file grows no further.

use super::case;
use crate::{Case, Check, Injection, cargo_lib_test};

/// The drifts the per-pull-request findings exist to rule out (ENG-28,
/// I2, I6): an artifact of the name any workflow uploaded taken as the
/// previous review, the findings never passed to the next review, kept
/// under a name no pull request owns, or picked with no origin check.
pub(super) fn review_previous_cases() -> Vec<Case> {
    [selection_cases(), wiring_cases()].concat()
}

/// The engineering check of the checkout's `previous select`.
fn selection_check() -> Check {
    cargo_lib_test(
        "engineering",
        "review_previous::tests::the_checkout_s_selection_trusts_main_s_review_workflow_alone",
    )
}

/// The engineering check of how the workflow and script carry findings.
fn wiring_check() -> Check {
    cargo_lib_test(
        "engineering",
        "review_previous::tests::the_checkout_s_review_carries_findings_from_main_s_review_workflow_only",
    )
}

/// The drifts of the origin check in `previous select`.
fn selection_cases() -> Vec<Case> {
    const PREVIOUS: &str = "tooling/review-run/src/previous.rs";
    vec![
        case(
            "a previous review taken from a run of any workflow, so a pull request plants its own history",
            "ENG-28",
            Injection::replace(
                PREVIOUS,
                "file == WORKFLOW_PATH && EVENTS.contains(&self.event.as_str())",
                "EVENTS.contains(&self.event.as_str())",
            ),
            selection_check(),
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
            selection_check(),
            "previous select took",
        ),
        case(
            "a previous review taken from an artifact whose run is unknown",
            "ENG-28",
            Injection::replace(
                PREVIOUS,
                ".is_some_and(Run::is_main_review);",
                ".is_none_or(Run::is_main_review);",
            ),
            selection_check(),
            "an artifact whose run is unknown",
        ),
    ]
}

/// The workflow and script drifts: the findings never handed on, kept
/// under one name, fetched with no origin check, or lost to a warning.
fn wiring_cases() -> Vec<Case> {
    const WORKFLOW: &str = ".github/workflows/review.yml";
    vec![
        case(
            "a review never given the findings of the previous one",
            "ENG-28",
            Injection::replace(WORKFLOW, "${PREVIOUS:+--previous \"$PREVIOUS\"}", ""),
            wiring_check(),
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
            wiring_check(),
            "one artifact name per pull request",
        ),
        case(
            "a warning among the runs' lines that stops jq and the whole fetch",
            "ENG-28",
            Injection::replace(
                "scripts/review-previous.sh",
                "its artifact is refused\" >&2",
                "its artifact is refused\"",
            ),
            wiring_check(),
            "stdout, into jq",
        ),
        case(
            "the previous findings picked with no check of the run that uploaded them",
            "ENG-28",
            Injection::replace(
                "scripts/review-previous.sh",
                "previous select",
                "previous list",
            ),
            wiring_check(),
            "previous select",
        ),
    ]
}
