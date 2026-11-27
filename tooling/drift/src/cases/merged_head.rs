//! The drifts of how the review reads a pull request as landing it
//! would leave main (ENG-21): the head merged into main, never the head
//! alone, apart from the registry's other cases so its file grows no
//! further.

use super::case;
use crate::{Case, Check, Injection, cargo_lib_test};

/// The workflow whose review and evaluation jobs read the head.
const WORKFLOW: &str = ".github/workflows/review.yml";
/// The script that merges the head into main.
const SCRIPT: &str = "scripts/review-merge-head.sh";

/// The drifts the merge exists to rule out (ENG-21): a job or the local
/// review that reads the head alone, an evaluation that cannot merge
/// it, a review run that names the merge as the head, and a merge that
/// diffs from a merge base or reviews a head that does not merge.
pub(super) fn review_merged_head_cases() -> Vec<Case> {
    [wiring_cases(), merge_cases()].concat()
}

/// The engineering check of how the workflow and the local review run
/// the merge.
fn wiring_check() -> Check {
    cargo_lib_test(
        "engineering",
        "review_merge::tests::the_checkout_s_review_reads_the_head_merged_into_main",
    )
}

/// The engineering check of the checkout's merge on its four histories.
fn merge_check() -> Check {
    cargo_lib_test(
        "engineering",
        "review_merge::tests::the_checkout_s_script_merges_the_head_into_main",
    )
}

/// The workflow and local review drifts.
fn wiring_cases() -> Vec<Case> {
    vec![
        case(
            "the review job that diffs the head alone from a merge base",
            "ENG-21",
            Injection::replace(
                WORKFLOW,
                "        if: steps.pull_head.outputs.number != ''\n        run: scripts/review-merge-head.sh\n",
                "        if: steps.pull_head.outputs.number != ''\n        run: git -C pr-head diff \"origin/main...$HEAD_SHA\" > pr.diff\n",
            ),
            wiring_check(),
            "job review diffs from a merge base",
        ),
        case(
            "the evaluation's job that checks the head out without its history",
            "ENG-21",
            Injection::replace(
                WORKFLOW,
                "          ref: ${{ needs.review.outputs.head_sha }}\n          path: pr-head\n          fetch-depth: 0\n",
                "          ref: ${{ needs.review.outputs.head_sha }}\n          path: pr-head\n",
            ),
            wiring_check(),
            "job evaluate checks the head out without its history",
        ),
        case(
            "a review run that names the merge as the head",
            "ENG-21",
            Injection::replace(
                WORKFLOW,
                "--head-sha \"$HEAD_SHA\" ${PREVIOUS",
                "${PREVIOUS",
            ),
            wiring_check(),
            "job review runs review-run without --head-sha",
        ),
        case(
            "the local review that diffs the head alone from a merge base",
            "ENG-21",
            Injection::replace(
                "scripts/review-local.sh",
                "\"$repo/scripts/review-merge-head.sh\"\n",
                "git -C pr-head diff \"$base...HEAD\" > pr.diff\n",
            ),
            wiring_check(),
            "scripts/review-local.sh diffs from a merge base",
        ),
    ]
}

/// The merge's own drifts.
fn merge_cases() -> Vec<Case> {
    vec![
        case(
            "a merge whose diff reads another pull request's change as the head's",
            "ENG-21",
            Injection::replace(
                SCRIPT,
                "git -C pr-head diff \"$main\" \"$merge\" > pr.diff",
                "git -C pr-head diff \"$main...$head\" > pr.diff",
            ),
            merge_check(),
            "pr.diff for a branch that merged main and another pull request apart",
        ),
        case(
            "a merge that reviews a head that does not merge with main",
            "ENG-21",
            Injection::replace(SCRIPT, "  code=$?\n", "  code=0\n"),
            merge_check(),
            "reviews a head that does not merge with main",
        ),
    ]
}
