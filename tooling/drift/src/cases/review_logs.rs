//! The drifts of the review's reading of every run's log (ENG-28, I6),
//! late so adding them moves no earlier case to another shard: a run
//! cut off read as one that did not run, so what it found is lost; a
//! specialist's reads of its diff no longer weighed, so a partial review
//! reads as whole; the lines set aside as no finding left uncounted; a
//! finding-shaped line that does not read dropped unseen; the findings
//! beside a line that does not read discarded with it; a run whose log
//! holds no model call taken as one that ran; and a run cut off before
//! it stated any line or an outcome taken for a clean pass, by the
//! review or by the grader; and a part that did not run left out of the
//! review the parts check runs end to end.

use super::*;

/// What every case runs: the ENG-28 check of how the review reads each
/// run's log, through `review-run` and the gate.
fn check() -> Check {
    cargo_lib_test(
        "engineering",
        "review_logs::tests::the_checkout_s_review_run_reads_every_log_whatever_its_run_ended_with",
    )
}

pub(super) fn review_log_cases() -> Vec<Case> {
    const RUN: &str = "tooling/review-run/src/run.rs";
    const OUTCOME: &str = "tooling/review-run/src/outcome.rs";
    let mut cases = vec![
        case(
            "a run cut off after a model call read as one that did not run",
            "ENG-28",
            Injection::replace(
                RUN,
                "    if log.ran() { (None, why)",
                "    if log.ran() && why.is_none() { (None, why)",
            ),
            check(),
            "cut: a specialist cut off after reading its whole diff is not consulted",
        ),
        case(
            "review-run no longer deciding from a specialist's reads what it read",
            "ENG-28",
            Injection::replace(
                RUN,
                "reading::uncovered(workdir, &unit.stem(), &log.reads)",
                "reading::uncovered(workdir, &unit.stem(), &log.reads).filter(|_| false)",
            ),
            check(),
            "partial: the lines a specialist did not read are not its gap",
        ),
        case(
            "the lines set aside as no finding left out of review/ignored.json",
            "ENG-28",
            Injection::replace(
                OUTCOME,
                "        .flat_map(|run| run.ignored.iter().cloned())\n",
                "        .flat_map(|run| run.ignored.iter().cloned().filter(|_| false))\n",
            ),
            check(),
            "all-well: a line saying all is well is not listed in review/ignored.json",
        ),
        case(
            "a finding-shaped line that does not read dropped unseen",
            "ENG-28",
            Injection::replace(
                OUTCOME,
                "        for line in run.unreadable.iter().take(MOST_UNREADABLE) {\n",
                "        for line in run.unreadable.iter().take(MOST_UNREADABLE).filter(|_| false) {\n",
            ),
            check(),
            "malformed: a line in no form is not its own blocking finding quoting it",
        ),
        case(
            "a line that does not read discarding the findings beside it",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/reading.rs",
                "    let report = with_outcome(&split.clean, ending);\n",
                "    let report = with_outcome(\n        if split.unreadable.is_empty() { &split.clean } else { \"\" },\n        ending,\n    );\n",
            ),
            check(),
            "malformed: the finding beside a line in no form is not kept",
        ),
        case(
            "a specialist whose log holds no model call consulted as one that ran",
            "ENG-28",
            Injection::replace(
                RUN,
                "    if log.ran() { (None, why)",
                "    if log.ran() || log.subtype != \"none\" { (None, why)",
            ),
            check(),
            "silent: a specialist whose log holds no model call is consulted",
        ),
    ];
    cases.extend(cut_before_stating_cases());
    cases.push(part_case());
    cases
}

/// A run cut off before it stated any line or an outcome taken for a
/// clean pass, by the review and by the grader.
fn cut_before_stating_cases() -> Vec<Case> {
    vec![
        case(
            "a specialist cut off before it stated anything taken for a clean pass",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/reading.rs",
                "    let stated_nothing = cut && states_nothing(texts, answer);\n",
                "    let stated_nothing = cut && states_nothing(texts, answer) && texts.is_empty();\n",
            ),
            check(),
            "cut-silent: a specialist cut off before it stated anything is not named partial",
        ),
        case(
            "the grader passing a run cut off before it stated anything",
            "ENG-32",
            Injection::replace(
                "tooling/review-eval/src/grade.rs",
                "    if !log.succeeded() && states_nothing(&log.texts, &log.answer) {\n",
                "    if !log.succeeded() && states_nothing(&log.texts, &log.answer) && log.texts.is_empty() {\n",
            ),
            cargo_lib_test(
                "review-eval",
                "grade::tests::a_specialist_cut_off_before_it_stated_anything_fails_even_a_case_that_expects_none",
            ),
            "a_specialist_cut_off_before_it_stated_anything_fails_even_a_case_that_expects_none",
        ),
    ]
}

/// A part that did not run left out of the review the ENG-28 parts
/// check runs end to end.
fn part_case() -> Case {
    case(
        "a part that did not run left out of the review the ENG-28 parts check runs",
        "ENG-28",
        Injection::replace(
            "tooling/review-run/src/parts.rs",
            "if let Some(why) = &run.failure {",
            "if let Some(why) = run.failure.as_ref().filter(|_| false) {",
        ),
        cargo_lib_test(
            "review-run",
            "parts::tests::a_failed_part_makes_the_merged_run_incomplete_naming_it_and_keeps_every_part_s_findings",
        ),
        "a_failed_part_makes_the_merged_run_incomplete_naming_it_and_keeps_every_part_s_findings",
    )
}
