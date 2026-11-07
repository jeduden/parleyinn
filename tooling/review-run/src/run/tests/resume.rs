//! The runs of a resumed review: what its resume plan keeps is not
//! started again.

use super::*;
use crate::resume::{Resume, Reuse};

/// Runs the one-specialist plan split at [`SPLIT`] as `resume` says,
/// `claude` answering as [`part_specialist`]: its runs, the progress
/// lines and the host.
fn resumed_split_run(dir: &Path, resume: &Resume) -> (Vec<AgentRun>, Vec<String>, FakeHost) {
    let host = FakeHost::new(part_specialist);
    let lines = Mutex::new(Vec::new());
    let runs = review(
        &host,
        dir,
        &plan(r#"{"review-code": ["lang-rust"]}"#),
        &known(),
        Options {
            split: SPLIT,
            ..options()
        },
        Some(resume),
        &|l| lines.lock().unwrap().push(l.to_owned()),
    )
    .unwrap();
    (runs, lines.into_inner().unwrap(), host)
}

/// A resume keeping lang-rust's part 1 as a log that read its whole
/// diff, stated `text` and answered `answer`.
fn keeping_part_one(text: &str, answer: &str) -> Resume {
    keeping_part_one_reading(text, answer, None)
}

/// [`keeping_part_one`], its one read of its diff `limit` lines long.
fn keeping_part_one_reading(text: &str, answer: &str, limit: Option<usize>) -> Resume {
    let log = RunLog {
        answered: true,
        answer: answer.to_owned(),
        texts: vec![text.to_owned(), answer.to_owned()],
        cost: 5.0,
        reads: vec![review_eval::log::ReadCall {
            path: "review/diffs/lang-rust.1.diff".to_owned(),
            limit,
            answered: true,
            ..review_eval::log::ReadCall::default()
        }],
        ..RunLog::default()
    };
    Resume {
        from: "old".to_owned(),
        reuse: [("lang-rust".to_owned(), Reuse::Parts([(1, log)].into()))].into(),
        notes: Vec::new(),
        ..Resume::default()
    }
}

#[test]
fn a_resumed_specialist_runs_only_its_parts_not_kept_and_merges_the_kept_one() {
    let dir = workdir();
    big_diff(dir.path());
    let resume = keeping_part_one(fake::FINDING, "outcome: findings — part 1 kept.");

    let (runs, lines, host) = resumed_split_run(dir.path(), &resume);

    let prompts: Vec<String> = host.calls.lock().unwrap().iter().map(prompt_of).collect();
    assert_eq!(host.agents(), ["lang-rust", "review-code"]);
    assert!(prompts[0].contains("you are part 2 of 2"), "{}", prompts[0]);
    let rust = &runs[0];
    assert!(rust.answered(), "{:?}", rust.failure);
    assert!(!rust.reused);
    assert_eq!((rust.parts, rust.attempts), (2, 1));
    // The kept part cost nothing this time.
    assert!((rust.log.cost - 0.01).abs() < 1e-9);
    assert!(
        lines.contains(
            &"lang-rust part 1 of 2 (specialist): reused from the earlier run".to_owned()
        ),
        "{lines:?}"
    );
    let report = fs::read_to_string(dir.path().join(report_file("lang-rust"))).unwrap();
    assert!(
        report.contains("part 1, findings — part 1 kept."),
        "{report}"
    );
    assert!(report.contains("src/part2.rs:1"), "{report}");
}

#[test]
fn a_kept_part_is_read_again_and_its_line_that_does_not_read_named_by_its_part() {
    let dir = workdir();
    big_diff(dir.path());
    let resume = keeping_part_one(
        "src/a.rs:3 — if not addressed: a bug — ENG-999",
        "outcome: findings — part 1 kept.",
    );

    let (runs, _, host) = resumed_split_run(dir.path(), &resume);

    assert_eq!(host.agents(), ["lang-rust", "review-code"]);
    let rust = &runs[0];
    assert_eq!(rust.unreadable.len(), 1, "{rust:?}");
    assert_eq!(rust.unreadable[0].part, Some(1));
    assert!(rust.unreadable[0].why.contains("ENG-999"));
    let report = fs::read_to_string(dir.path().join(report_file("lang-rust"))).unwrap();
    assert!(!report.contains("ENG-999"), "{report}");
    assert!(
        report.contains("part 1, none — nothing stated in the run log"),
        "{report}"
    );
}

#[test]
fn a_kept_part_is_held_to_its_reads_of_its_diff_again() {
    let dir = workdir();
    big_diff(dir.path());
    let resume = keeping_part_one_reading(fake::FINDING, "outcome: findings — kept.", Some(3));

    let (runs, _, _) = resumed_split_run(dir.path(), &resume);

    let rust = &runs[0];
    assert!(!rust.whole(), "{rust:?}");
    assert_eq!(rust.partial.len(), 1, "{:?}", rust.partial);
    assert_eq!(
        rust.partial[0],
        (
            Some(1),
            "its log shows no Read of line 4 of review/diffs/lang-rust.1.diff, of 4 lines"
                .to_owned()
        )
    );
}

#[test]
fn a_resume_keeping_an_agent_whole_starts_no_run_of_it() {
    let dir = workdir();
    let resume = Resume {
        from: "old".to_owned(),
        reuse: [
            (
                "lang-rust".to_owned(),
                Reuse::Whole("outcome: none — clean.".to_owned()),
            ),
            (
                "review-spec".to_owned(),
                Reuse::Whole("none — x".to_owned()),
            ),
        ]
        .into(),
        notes: Vec::new(),
        ignored: [("lang-rust".to_owned(), vec![aside()])].into(),
        ..Resume::default()
    };
    let host = FakeHost::new(fake::either);

    let runs = review(
        &host,
        dir.path(),
        &plan(DISPATCH),
        &known(),
        options(),
        Some(&resume),
        &|_| {},
    )
    .unwrap();

    let mut agents = host.agents();
    agents.sort();
    assert_eq!(
        agents,
        ["lang-markdown", "review-code", "test-engineer-unit"]
    );
    let rust = runs.iter().find(|r| r.name == "lang-rust").unwrap();
    assert!(rust.reused && rust.answered());
    assert_eq!(rust.log.answer, "outcome: none — clean.");
    assert_eq!(rust.attempts, 0);
    assert!(rust.log.cost.abs() < f64::EPSILON);
    // What it set aside in the earlier run is set aside still (I6).
    assert_eq!(rust.ignored, [aside()]);
    let spec = runs.iter().find(|r| r.name == "review-spec").unwrap();
    assert_eq!(spec.kind, Kind::Generalist);
    assert!(spec.reused);
    assert_eq!(runs.len(), 5);
}

/// A line lang-rust set aside in an earlier run.
fn aside() -> crate::reading::ReportLine {
    crate::reading::ReportLine {
        agent: "lang-rust".to_owned(),
        part: None,
        line: 1,
        text: "src/a.rs:1 — fine".to_owned(),
        why: "says all is well, which is not a finding".to_owned(),
    }
}
