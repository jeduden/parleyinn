use std::collections::BTreeSet;
use std::fs;

use review_eval::log::ReadCall;
use testkit::TempDir;

use super::*;

fn known() -> BTreeSet<String> {
    ["ENG-28".to_owned(), "I9".to_owned()].into_iter().collect()
}

fn texts(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|l| (*l).to_owned()).collect()
}

const FINDING: &str = "src/a.rs:3 — if not addressed: a hook panics — I9";

#[test]
fn a_report_keeps_what_reads_and_ends_with_the_outcome_the_specialist_answered() {
    let answer = "outcome: findings — one finding.";
    let read = read(
        ("lang-rust", None),
        &texts(&["I read the diff.", FINDING]),
        answer,
        &known(),
    );

    assert_eq!(read.report, format!("{FINDING}\n{answer}\n"));
    assert_eq!(read.answer, answer);
    assert!(read.ignored.is_empty() && read.unreadable.is_empty());
}

#[test]
fn a_line_that_is_no_finding_is_set_aside_with_why_and_the_finding_kept() {
    let read = read(
        ("lang-rust", Some(2)),
        &texts(&[FINDING, "src/b.rs:1 — no finding"]),
        "outcome: findings — one.",
        &known(),
    );

    assert_eq!(
        read.report,
        format!("{FINDING}\noutcome: findings — one.\n")
    );
    assert_eq!(
        read.ignored,
        vec![ReportLine {
            agent: "lang-rust".to_owned(),
            part: Some(2),
            line: 2,
            text: "note: all is well, not a finding: src/b.rs:1 — no finding".to_owned(),
            why: "says all is well, which is not a finding".to_owned(),
        }]
    );
}

#[test]
fn a_finding_line_that_does_not_read_is_named_and_the_others_kept() {
    let bad = "src/c.rs:2 — if not addressed: it breaks — ENG-999";
    let read = read(
        ("lang-rust", None),
        &texts(&[FINDING, bad]),
        "outcome: findings — two.",
        &known(),
    );

    assert_eq!(
        read.report,
        format!("{FINDING}\noutcome: findings — two.\n")
    );
    assert_eq!(read.unreadable.len(), 1);
    assert_eq!(read.unreadable[0].text, bad);
    assert!(read.unreadable[0].why.contains("cites ENG-999"));
}

#[test]
fn a_missing_or_contradicted_outcome_is_derived_from_the_lines() {
    let cut = read(("lang-rust", None), &texts(&[FINDING]), "", &known());
    let none = read(
        ("lang-rust", None),
        &texts(&[FINDING]),
        "outcome: none — all well.",
        &known(),
    );

    for read in [cut, none] {
        assert_eq!(
            read.answer,
            "outcome: findings — 1 finding stated in the run log"
        );
        assert!(read.report.ends_with(&format!("{}\n", read.answer)));
    }
}

#[test]
fn a_report_with_no_outcome_line_answers_nothing() {
    assert_eq!(answer_of("src/a.rs:1 — x\n"), "");
    assert_eq!(answer_of("outcome: none — x\n"), "outcome: none — x");
}

fn reading(path: &str, offset: Option<usize>, limit: Option<usize>) -> ReadCall {
    ReadCall {
        path: path.to_owned(),
        offset,
        limit,
        failed: false,
    }
}

fn with_diff(stem: &str, lines: usize) -> TempDir {
    let dir = TempDir::new().unwrap();
    let diffs = dir.path().join("review/diffs");
    fs::create_dir_all(&diffs).unwrap();
    fs::write(diffs.join(format!("{stem}.diff")), "+x\n".repeat(lines)).unwrap();
    dir
}

#[test]
fn a_diff_read_whole_in_chunks_is_covered() {
    let dir = with_diff("lang-rust", 30);
    let path = "/work/review/diffs/lang-rust.diff";
    let reads = [
        reading(path, None, Some(10)),
        reading(path, Some(11), Some(20)),
    ];

    assert_eq!(uncovered(dir.path(), "lang-rust", &reads), None);
}

#[test]
fn each_range_no_read_covered_is_named() {
    let dir = with_diff("lang-rust.2", 30);
    let reads = [
        reading("review/diffs/lang-rust.2.diff", Some(5), Some(5)),
        reading("review/diffs/lang-rust.2.diff", Some(20), Some(1)),
    ];

    assert_eq!(
        uncovered(dir.path(), "lang-rust.2", &reads).unwrap(),
        "its log shows no Read of lines 1-4, 10-19, 21-30 of review/diffs/lang-rust.2.diff, of 30 lines"
    );
}

#[test]
fn a_diff_never_read_or_read_by_another_name_is_unread_whole() {
    let dir = with_diff("lang-rust", 1);
    let reads = [reading("review/diffs/lang-markdown.diff", None, None)];

    assert_eq!(
        uncovered(dir.path(), "lang-rust", &reads).unwrap(),
        "its log shows no Read of line 1 of review/diffs/lang-rust.diff, of 1 line"
    );
}

#[test]
fn no_diff_has_nothing_to_cover_and_one_that_does_not_read_is_named() {
    let dir = TempDir::new().unwrap();
    assert_eq!(uncovered(dir.path(), "lang-rust", &[]), None);
    fs::create_dir_all(dir.path().join("review/diffs/lang-rust.diff")).unwrap();

    let why = uncovered(dir.path(), "lang-rust", &[]).unwrap();

    assert!(why.starts_with("its diff review/diffs/lang-rust.diff does not read: "));
}

#[test]
fn a_report_line_reads_back_as_written_without_a_part_when_whole() {
    let line = ReportLine {
        agent: "lang-rust".to_owned(),
        part: None,
        line: 1,
        text: "t".to_owned(),
        why: "w".to_owned(),
    };
    let json = serde_json::to_string(&line).unwrap();

    assert!(!json.contains("part"));
    assert_eq!(serde_json::from_str::<ReportLine>(&json).unwrap(), line);
}
