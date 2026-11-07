//! The answer store's and the loop breakers' drifts (ENG-28), apart from
//! the registry's other cases so its file grows no further.

use super::case;
use crate::{Case, Injection, cargo_lib_test};

/// The drifts the answer store and the loop breakers exist to rule out
/// (ENG-28, I6), late so adding them moves no earlier case to another
/// shard: reuse of an unchanged key silently disabled, a run whose
/// report holds a line that does not read stored, a gap loop breaker that never trips, and a blocking finding
/// stuck on an unchanged file never named for a stakeholder.
pub(super) fn review_answer_cases() -> Vec<Case> {
    const MEMO: &str = "tooling/review-run/src/memo.rs";
    let test = |name: &'static str| cargo_lib_test("review-run", name);
    vec![
        case(
            "an answer stored under an unchanged key run again, so reuse is silently off",
            "ENG-28",
            Injection::replace(
                MEMO,
                "Ok(Some(stored)) => stored,",
                "Ok(Some(stored)) => {\n            if true {\n                return None;\n            }\n            stored\n        }",
            ),
            test("memo::tests::an_unchanged_key_is_taken_from_the_store_and_runs_nothing"),
            "an_unchanged_key_is_taken_from_the_store_and_runs_nothing",
        ),
        case(
            "a run whose report holds a line that does not read kept in the answer store",
            "ENG-28",
            Injection::replace(
                MEMO,
                "    let taken = store::accepted(unit.kind, &log, report.as_deref(), r.known)\n        .and_then(|()| partial(r, unit, &log, held));\n",
                "    let taken = Ok::<(), String>(());\n",
            ),
            test(
                "memo::tests::a_run_with_a_line_that_does_not_read_or_that_did_not_run_is_never_stored",
            ),
            "a_run_with_a_line_that_does_not_read_or_that_did_not_run_is_never_stored",
        ),
        case(
            "a loop breaker that never trips, so a resume reruns a stalled agent forever",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/stall.rs",
                "        self.times > max_resumes\n",
                "        self.times > max_resumes && false\n",
            ),
            test("stall::tests::a_gap_stalls_once_it_recurred_in_more_resumes_than_allowed"),
            "a_gap_stalls_once_it_recurred_in_more_resumes_than_allowed",
        ),
        case(
            "a blocking finding stuck on an unchanged file never named for a stakeholder ruling",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/findings.rs",
                ".filter(|t| t.blocking_streak >= RULING_REVIEWS)",
                ".filter(|t| t.blocking_streak >= u32::MAX)",
            ),
            test(
                "findings::tests::a_blocking_finding_kept_three_reviews_in_a_row_on_an_unchanged_file_needs_a_ruling",
            ),
            "a_blocking_finding_kept_three_reviews_in_a_row_on_an_unchanged_file_needs_a_ruling",
        ),
        case(
            "a drop command that leaves the stored answer in the store",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/store.rs",
                "match fs::remove_dir_all(&at) {",
                "match Ok::<(), io::Error>(()) {",
            ),
            cargo_lib_test(
                "engineering",
                "review_answers::tests::the_checkout_s_review_run_drops_a_stored_answer_by_its_key_and_no_other",
            ),
            "answers drop left lang-rust's answer stored as",
        ),
        case(
            "a blocking finding kept three reviews in a row never named for a ruling",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/findings.rs",
                ".filter(|t| t.blocking_streak >= RULING_REVIEWS)",
                ".filter(|t| t.blocking_streak > RULING_REVIEWS)",
            ),
            cargo_lib_test(
                "engineering",
                "review_answers::tests::the_checkout_s_review_run_names_a_blocking_finding_kept_three_reviews_as_needing_a_ruling",
            ),
            "names no stakeholder ruling",
        ),
    ]
}
