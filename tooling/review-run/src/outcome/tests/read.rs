//! The outcome of specialists whose log was read whatever their run
//! ended with: one cut off with its whole diff read is consulted and
//! said, one that read part of its diff or stated a line that does not
//! read blocks naming why, and the lines set aside are counted.

use super::*;
use crate::gaps::MOST_QUOTED;
use crate::reading::ReportLine;

/// A line of lang-rust's report, `why` it is set aside or unread.
fn line(part: Option<usize>, text: &str, why: &str) -> ReportLine {
    ReportLine {
        agent: "lang-rust".to_owned(),
        part,
        line: 2,
        text: text.to_owned(),
        why: why.to_owned(),
    }
}

const WEIGHED: &str = "src/a.rs:3 — nit — a long line — lang-rust\n\
                       src/a.rs:3 — nit — again — test-engineer-unit\nSound.";

#[test]
fn a_specialist_cut_off_with_its_whole_diff_read_is_consulted_and_said() {
    let (mut runs, reports) = all_answered(WEIGHED);
    runs[0].cut = Some("its run ended with error_max_turns".to_owned());

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "approve", "{:?}", d.outcome);
    assert!(d.consulted.contains("lang-rust"));
    assert!(d.not_reviewed.is_empty(), "{:?}", d.not_reviewed);
    assert!(
        d.outcome.summary.contains(
            " 1 specialist ended without a successful result, cut off by a turn budget, \
             deadline or cost cap, with every line of their diff read: what each one's log \
             states is its report."
        ),
        "{}",
        d.outcome.summary
    );
}

#[test]
fn a_specialist_that_read_part_of_its_diff_blocks_naming_the_lines_by_part() {
    let (mut runs, reports) = all_answered(WEIGHED);
    let why = "its log shows no Read of lines 5-9 of review/diffs/lang-rust.2.diff, of 9 lines";
    runs[0].cut = Some("part 2 of 2: its run ended with error_max_turns".to_owned());
    runs[0].partial = vec![(Some(2), why.to_owned())];

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    assert_eq!(
        d.outcome.findings[0],
        blocking(
            "review/",
            format!(
                "lang-rust part 2 did not read its whole diff (part 2 of 2: its run ended with \
                 error_max_turns): {why}, so the review of it is partial"
            )
        )
    );
    assert_eq!(
        d.not_reviewed,
        [gap(
            "lang-rust",
            "specialist",
            Some(2),
            &format!("partial: {why}")
        )]
    );
    assert!(d.consulted.contains("lang-rust"));
    assert!(
        d.outcome
            .summary
            .contains(" Read part of their diff: lang-rust part 2."),
        "{}",
        d.outcome.summary
    );
    assert!(
        !d.outcome.summary.contains("with every line"),
        "{}",
        d.outcome.summary
    );
}

#[test]
fn a_line_that_does_not_read_blocks_quoted_and_the_rest_of_its_report_stands() {
    let (mut runs, reports) = all_answered(WEIGHED);
    let problem = "has a report whose line 2 is neither a linked finding nor a requirement request";
    runs[0].unreadable = vec![line(None, "a.rs:9 — not a form", problem)];

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    assert!(
        d.outcome.findings.contains(&blocking(
            "review/reports/lang-rust.md",
            format!("lang-rust {problem}: `a.rs:9 — not a form`")
        )),
        "{:?}",
        d.outcome.findings
    );
    // Its finding is weighed as any other, so the nit is kept.
    assert!(d.outcome.findings.iter().any(|f| f.severity == "nit"));
    assert_eq!(
        d.not_reviewed,
        [gap(
            "lang-rust",
            "specialist",
            None,
            &format!("unreadable: {problem}: `a.rs:9 — not a form`")
        )]
    );
    assert!(
        d.outcome
            .summary
            .contains(" Reports holding a line that does not read: lang-rust."),
        "{}",
        d.outcome.summary
    );
}

#[test]
fn the_lines_set_aside_are_listed_and_counted_and_block_nothing() {
    let (mut runs, reports) = all_answered(WEIGHED);
    let aside = line(Some(1), "src/b.rs:1 — no finding", "says all is well");
    runs[0].ignored = vec![aside.clone()];
    // A specialist run in parts that did not run, one part having not
    // run, still lists what the parts that ran set aside (I6).
    let mut silent = silent("test-engineer-unit", Kind::Specialist, "cannot start");
    silent.ignored = vec![aside.clone()];
    let mut two = runs.clone();
    two[1] = silent;

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "approve", "{:?}", d.outcome);
    assert_eq!(d.ignored, [aside]);
    assert!(
        d.outcome.summary.contains(
            " Set aside as no finding: 1 line of the specialists' reports (review/ignored.json)."
        ),
        "{}",
        d.outcome.summary
    );
    assert_eq!(
        decide(&plan(DISPATCH), &two, &reports, &known())
            .ignored
            .len(),
        2
    );
}

#[test]
fn many_are_counted_in_the_plural() {
    let (mut runs, reports) = all_answered(WEIGHED);
    for run in &mut runs[..2] {
        run.cut = Some("its run ended with deadline".to_owned());
    }
    runs[0].ignored = vec![line(None, "a", "w"), line(None, "b", "w")];

    let summary = decide(&plan(DISPATCH), &runs, &reports, &known())
        .outcome
        .summary;

    assert!(
        summary.contains(" 2 specialists ended without"),
        "{summary}"
    );
    assert!(summary.contains("no finding: 2 lines of"), "{summary}");
}

#[test]
fn a_part_that_read_part_of_its_diff_is_a_gap_beside_a_part_that_did_not_run() {
    let (mut runs, reports) = all_answered(WEIGHED);
    let why = "its log shows no Read of lines 5-9 of review/diffs/lang-rust.1.diff, of 9 lines";
    runs[0].failure = Some("part 2 of 2: its run ended with none".to_owned());
    runs[0].failed_parts = vec![(2, "its run ended with none".to_owned())];
    runs[0].partial = vec![(Some(1), why.to_owned())];
    runs[0].ignored = vec![line(Some(1), "a.rs:1 — fine", "says all is well")];

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    // Part 1's gap is kept, so a resume runs it again rather than keep
    // a part that skimmed its diff.
    assert_eq!(
        d.not_reviewed
            .iter()
            .filter(|g| g.name == "lang-rust")
            .map(|g| (g.part, g.reason.clone()))
            .collect::<Vec<_>>(),
        [
            (Some(1), format!("partial: {why}")),
            (Some(2), "silent: its run ended with none".to_owned()),
        ]
    );
    assert!(
        d.outcome.findings.iter().any(|f| f.body.contains(why)),
        "{:?}",
        d.outcome.findings
    );
    // What it set aside is listed all the same.
    assert_eq!(d.ignored.len(), 1);
}

#[test]
fn a_generalist_cut_off_is_consulted_and_counted_in_the_summary() {
    let (mut runs, reports) = all_answered(WEIGHED);
    let at = runs.iter().position(|r| r.name == "review-code").unwrap();
    runs[at].cut = Some("its run ended with error_max_turns".to_owned());

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "approve", "{:?}", d.outcome);
    assert!(d.consulted.contains("review-code"));
    assert!(
        d.outcome.summary.contains(
            " 1 generalist ended without a successful result, cut off by a turn budget, \
             deadline or cost cap: what each one's log states is its weighing."
        ),
        "{}",
        d.outcome.summary
    );
}

#[test]
fn the_lines_that_do_not_read_are_named_ten_at_most_each_cut_then_counted() {
    let (mut runs, reports) = all_answered(WEIGHED);
    let problem = "has a report whose line 2 is in no form";
    let long = format!("a.rs:1 — {}", "x".repeat(300));
    runs[0].unreadable = (0..12)
        .map(|at| {
            let text = if at == 0 {
                long.clone()
            } else {
                format!("a.rs:{at} — not a form")
            };
            line(None, &text, problem)
        })
        .collect();

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    let on_report: Vec<&str> = d
        .outcome
        .findings
        .iter()
        .filter(|f| f.path == "review/reports/lang-rust.md")
        .map(|f| f.body.as_str())
        .collect();
    assert_eq!(on_report.len(), MOST_UNREADABLE + 1, "{on_report:?}");
    let cut: String = long.chars().take(MOST_QUOTED).collect();
    assert_eq!(on_report[0], format!("lang-rust {problem}: `{cut}…`"));
    assert_eq!(on_report[10], "lang-rust and 2 more lines that do not read");
    let gaps: Vec<&str> = d
        .not_reviewed
        .iter()
        .filter(|g| g.name == "lang-rust")
        .map(|g| g.reason.as_str())
        .collect();
    assert_eq!(gaps.len(), MOST_UNREADABLE + 1, "{gaps:?}");
    assert_eq!(gaps[0], format!("unreadable: {problem}: `{cut}…`"));
    assert_eq!(gaps[10], "unreadable: and 2 more lines that do not read");
    // The summary names the agent once, however many lines.
    let summary = &d.outcome.summary;
    assert!(
        summary.contains(" Reports holding a line that does not read: lang-rust."),
        "{summary}"
    );
    assert!(summary.contains(" Not reviewed: lang-rust."), "{summary}");
}
