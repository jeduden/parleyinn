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

/// A resume keeping lang-rust's part 1 as a log that stated `text` and
/// answered `answer`.
fn keeping_part_one(text: &str, answer: &str) -> Resume {
    let log = RunLog {
        answered: true,
        answer: answer.to_owned(),
        texts: vec![text.to_owned(), answer.to_owned()],
        cost: 5.0,
        ..RunLog::default()
    };
    Resume {
        from: "old".to_owned(),
        reuse: [("lang-rust".to_owned(), Reuse::Parts([(1, log)].into()))].into(),
        notes: Vec::new(),
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
fn a_kept_part_the_report_check_refuses_runs_again() {
    let dir = workdir();
    big_diff(dir.path());
    let resume = keeping_part_one(
        "src/a.rs:3 — if not addressed: a bug — ENG-999",
        "outcome: findings — part 1 kept.",
    );

    let (_, lines, host) = resumed_split_run(dir.path(), &resume);

    assert_eq!(host.agents(), ["lang-rust", "lang-rust", "review-code"]);
    let told = lines
        .iter()
        .find(|l| l.starts_with("lang-rust part 1 of 2 (specialist): its kept report was refused, so it runs again: "))
        .unwrap_or_else(|| panic!("{lines:?}"));
    assert!(told.contains("ENG-999"), "{told}");
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
    let spec = runs.iter().find(|r| r.name == "review-spec").unwrap();
    assert_eq!(spec.kind, Kind::Generalist);
    assert!(spec.reused);
    assert_eq!(runs.len(), 5);
}
