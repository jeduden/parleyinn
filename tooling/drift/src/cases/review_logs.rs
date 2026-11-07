//! The drifts of the review's reading of every run's log (ENG-28, I6),
//! late so adding them moves no earlier case to another shard: a run
//! cut off read as one that did not run, so what it found is lost; a
//! specialist's reads of its diff no longer weighed, so a partial review
//! reads as whole; the lines set aside as no finding left uncounted; a
//! finding-shaped line that does not read dropped unseen; the findings
//! beside a line that does not read discarded with it; and a run whose
//! log holds no model call taken as one that ran.

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
    vec![
        case(
            "a run cut off after a model call read as one that did not run",
            "ENG-28",
            Injection::replace(
                RUN,
                "    if crate::retry::charged(log).is_some() {\n",
                "    if crate::retry::charged(log).is_some() && why.is_none() {\n",
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
                "        for line in &run.unreadable {\n",
                "        for line in run.unreadable.iter().filter(|_| false) {\n",
            ),
            check(),
            "malformed: a line in no form is not its own blocking finding quoting it",
        ),
        case(
            "a line that does not read discarding the findings beside it",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/reading.rs",
                "    let report = with_outcome(&split.clean, answer);\n",
                "    let report = with_outcome(\n        if split.unreadable.is_empty() { &split.clean } else { \"\" },\n        answer,\n    );\n",
            ),
            check(),
            "malformed: the finding beside a line in no form is not kept",
        ),
        case(
            "a specialist whose log holds no model call consulted as one that ran",
            "ENG-28",
            Injection::replace(
                RUN,
                "    if crate::retry::charged(log).is_some() {\n",
                "    if crate::retry::charged(log).is_some() || log.subtype != \"none\" {\n",
            ),
            check(),
            "silent: a specialist whose log holds no model call is consulted",
        ),
    ]
}
