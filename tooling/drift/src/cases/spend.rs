//! The drifts of how a pull request's reviews share one cost cap
//! (ENG-28): what earlier reviews spent ignored, the total not kept or
//! kept without them, and the workflow no longer carrying it from one
//! review to the next. Late, so adding them moves no earlier case to
//! another shard.

use super::case;
use crate::{Case, Check, Injection, cargo_lib_test};

/// The workflow the cases drift.
const WORKFLOW: &str = ".github/workflows/review.yml";
/// The script that fetches what earlier reviews spent.
const SCRIPT: &str = "scripts/review-spend.sh";

/// The drifts the pull request's cap exists to rule out (ENG-28, I6).
pub(super) fn review_spend_cases() -> Vec<Case> {
    [cap_cases(), carry_cases(), flight_cases()].concat()
}

/// The drifts of the runs in flight: a run started though the tier caps
/// of the runs in flight leave no room for its own, and a run paid for
/// that reported no cost counted free. Last, so adding them moves no
/// earlier case to another shard.
fn flight_cases() -> Vec<Case> {
    vec![
        case(
            "review-run starting runs whose tier caps the runs in flight already hold",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/controls.rs",
                "ledger.reserved += most;",
                "ledger.reserved += 0.0;",
            ),
            cap_check(),
            "3 runs ran at once, past the 2",
        ),
        case(
            "review-run counting a run paid for that reported no cost as free",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/retry.rs",
                "    } else if charged(log).is_some() {\n        most\n",
                "    } else if charged(log).is_some() {\n        0.0\n",
            ),
            cap_check(),
            "a review with a run cut off unbilled: review/spend.json says",
        ),
    ]
}

/// The engineering check of the cap on the checkout's `review-run`.
fn cap_check() -> Check {
    cargo_lib_test(
        "engineering",
        "review_spend::tests::the_checkout_s_review_run_holds_a_pull_request_s_reviews_to_its_cap",
    )
}

/// The engineering check of how the workflow carries the spend.
fn carry_check() -> Check {
    cargo_lib_test(
        "engineering",
        "review_spend::tests::the_checkout_s_review_carries_what_a_pull_request_s_reviews_spent",
    )
}

/// `review-run`'s drifts: the earlier reviews' spend left out of the
/// cap or of the file that keeps it, or the file never kept.
fn cap_cases() -> Vec<Case> {
    vec![
        case(
            "review-run leaving what earlier reviews spent out of the pull request's cap",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/controls.rs",
                "        self.before + own <= self.cap + SLACK\n",
                "        own <= self.cap + SLACK\n",
            ),
            cap_check(),
            "5 runs started, not 1",
        ),
        case(
            "review-run reading --spent-before and dropping it",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/cli.rs",
                "\"spent-before\" => options.spent_before = spent(name, &value)?,",
                "\"spent-before\" => drop(spent(name, &value)?),",
            ),
            cap_check(),
            "5 runs started, not 1",
        ),
        case(
            "review-run keeping its own spend without what earlier reviews spent",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/cli.rs",
                "&crate::spend::text(p.spent_before + usage::charged(runs)),",
                "&crate::spend::text(usage::charged(runs)),",
            ),
            cap_check(),
            "review/spend.json says",
        ),
        case(
            "review-run never keeping what the pull request's reviews spent",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/spend.rs",
                "    fs::rename(&part, path)\n",
                "    fs::rename(&part, part.with_extension(\"lost\"))\n",
            ),
            cap_check(),
            "review/spend.json: No such file",
        ),
        case(
            "a spend file past its size limit read as what was spent",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/spend.rs",
                "    if text.len() > MAX_BYTES {",
                "    if text.len() > MAX_BYTES * 1024 {",
            ),
            carry_check(),
            "took a spend file past the size limit",
        ),
    ]
}

/// The workflow and script drifts: the spend never handed on, kept
/// only after a review that succeeded, or fetched with no origin check
/// or no reading of the file.
fn carry_cases() -> Vec<Case> {
    vec![
        case(
            "a review never given what earlier reviews of its pull request spent",
            "ENG-28",
            Injection::replace(WORKFLOW, " --spent-before \"$SPENT\" \\\n", " \\\n"),
            carry_check(),
            "as --spent-before",
        ),
        case(
            "the spend kept only by a review that did not fail or get cancelled",
            "ENG-28",
            Injection::replace(
                WORKFLOW,
                "        if: always() && steps.scrub.outputs.scrubbed == 'true'\n        uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2\n        with:\n          name: review-spend-pr-",
                "        if: steps.scrub.outputs.scrubbed == 'true'\n        uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2\n        with:\n          name: review-spend-pr-",
            ),
            carry_check(),
            "reviews spent lacks",
        ),
        case(
            "what earlier reviews spent picked with no check of the run that uploaded it",
            "ENG-28",
            Injection::replace(
                SCRIPT,
                "review-run -- previous select --of spend",
                "review-run -- previous list --of spend",
            ),
            carry_check(),
            "previous select --of spend",
        ),
        case(
            "what earlier reviews spent fetched and never read",
            "ENG-28",
            Injection::replace(SCRIPT, "previous spend \"$dir", "previous check \"$dir"),
            carry_check(),
            "never reads the file it fetched",
        ),
    ]
}
