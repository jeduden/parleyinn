//! The drifts a specialist's own diff and context exist to rule out
//! (ENG-28, I9), apart from the registry's other cases so its file
//! grows no further: a part told to read the whole diff, a specialist
//! told to read neither its context nor away from the bundle, a context
//! that leaves out what it promises, and a context left out of the
//! answer key; and the stability measurement's finding overlap and
//! identity (ENG-32).

use super::case;
use crate::{Case, Injection, cargo_lib_test};

/// The mutations of the prompt and the context, each run through the
/// check that runs `review-run` and the bundle.
pub(super) fn review_context_cases() -> Vec<Case> {
    const RUNNER: &str = "tooling/review-eval/src/runner.rs";
    let check = || {
        cargo_lib_test(
            "engineering",
            "review_runs::tests::a_review_run_on_the_checkout_s_definitions_has_no_problem",
        )
    };
    let want = "a_review_run_on_the_checkout_s_definitions_has_no_problem";
    vec![
        case(
            "review-run telling a part of a split diff to read the whole diff",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/parts.rs",
                "            diff: own,\n",
                "            diff: diff_file(name),\n",
            ),
            cargo_lib_test(
                "engineering",
                "review_runs::tests::a_split_run_on_the_checkout_s_definitions_has_no_problem",
            ),
            "a_split_run_on_the_checkout_s_definitions_has_no_problem",
        ),
        case(
            "a specialist's prompt not naming the context the bundle wrote for it",
            "ENG-28",
            Injection::replace(
                RUNNER,
                "then `review/context/{name}.md`, which names",
                "then `review/contexts/{name}.md`, which names",
            ),
            check(),
            want,
        ),
        case(
            "a specialist's prompt not telling it to leave the bundle unread",
            "ENG-28",
            Injection::replace(RUNNER, "your files; never \\\n", "your files; see \\\n"),
            check(),
            want,
        ),
    ]
}

/// The mutations of the context the bundle writes, and of the answer
/// key that holds it.
pub(super) fn review_context_file_cases() -> Vec<Case> {
    const RENDER: &str = "tooling/review-bundle/src/render.rs";
    let check = || {
        cargo_lib_test(
            "engineering",
            "review_runs::tests::a_review_run_on_the_checkout_s_definitions_has_no_problem",
        )
    };
    let want = "a_review_run_on_the_checkout_s_definitions_has_no_problem";
    vec![
        case(
            "a specialist's context leaving out the review class",
            "ENG-28",
            Injection::replace(
                RENDER,
                "Class: **{}**. The change {ids}",
                "Kind: **{}**. The change {ids}",
            ),
            check(),
            want,
        ),
        case(
            "a specialist's context leaving out the touched ids",
            "ENG-28",
            Injection::replace(RENDER, "touches 1 id: {}", "touching 1 id: {}"),
            check(),
            want,
        ),
        case(
            "a specialist's context leaving out its dispatch row",
            "ENG-28",
            Injection::replace(
                RENDER,
                "\"- `{generalist}`, weighing {}\"",
                "\"- `{generalist}` weighs {}\"",
            ),
            check(),
            want,
        ),
        case(
            "a specialist's context leaving out the findings on its files",
            "ENG-28",
            Injection::replace(RENDER, "\"- {} `{}:{}`: {}\"", "\"- {} {}:{}: {}\""),
            check(),
            want,
        ),
        case(
            "a specialist's context naming the bundle it is told not to read",
            "ENG-28",
            Injection::replace(
                RENDER,
                "are not shown here; the",
                "are in `review/bundle.md`; the",
            ),
            cargo_lib_test(
                "review-bundle",
                "render::tests::context_names_the_class_the_specialist_s_row_its_files_and_their_findings",
            ),
            "context_names_the_class_the_specialist_s_row_its_files_and_their_findings",
        ),
        case(
            "an answer key that leaves a specialist's context out, so a changed context reuses an answer",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/key.rs",
                "{REVIEW}/context/{name}.md",
                "{REVIEW}/context/none.md",
            ),
            cargo_lib_test(
                "review-run",
                "key::tests::a_changed_context_changes_its_specialist_s_key_and_its_generalist_s_alone",
            ),
            "a_changed_context_changes_its_specialist_s_key_and_its_generalist_s_alone",
        ),
    ]
}

/// The drifts of the finding identity the stability measurement
/// compares (ENG-32): a bolded or reworded finding told from itself,
/// and a finding on another path taken for it.
pub(super) fn review_identity_cases() -> Vec<Case> {
    const IDENTITY: &str = "tooling/review-report/src/identity.rs";
    let check = || {
        cargo_lib_test(
            "engineering",
            "stability_measurement::tests::review_report_measures_a_flip_takes_the_log_s_cost_and_escapes_the_report",
        )
    };
    let want = "review_report_measures_a_flip_takes_the_log_s_cost_and_escapes_the_report";
    vec![
        case(
            "the stability measurement telling a bolded finding from itself",
            "ENG-32",
            Injection::replace(
                IDENTITY,
                ".map(|w| w.trim_matches(['*', '_', '`']).to_lowercase())",
                ".map(|w| w.to_lowercase())",
            ),
            check(),
            want,
        ),
        case(
            "the stability measurement taking a finding on another path for the same finding",
            "ENG-32",
            Injection::replace(
                IDENTITY,
                "            path: path.to_owned(),\n",
                "            path: String::new(),\n",
            ),
            check(),
            want,
        ),
    ]
}
