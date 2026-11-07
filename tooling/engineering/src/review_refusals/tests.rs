use super::*;
use crate::tests::checkout;

use testkit::TempDir;

fn repo() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn the_checkout_s_review_run_names_an_unknown_id_and_an_unread_line() {
    let work = TempDir::new().unwrap();

    let problems = review_run_names_what_it_refuses(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn an_outcome_that_approves_naming_neither_is_named() {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review")).unwrap();
    fs::write(
        work.path().join("review/outcome.json"),
        r#"{"verdict": "approve", "summary": "s", "findings": []}"#,
    )
    .unwrap();

    let problems = outcome_problems(work.path());

    assert_eq!(
        problems,
        [
            "the review does not request changes: approve",
            "no blocking finding names lang-rust's report citing ENG-999",
            "no blocking finding names review-code's finding line it cannot read",
            "lang-rust's report does not keep its finding apart from the one citing ENG-999: \"\"",
        ]
    );
}

#[test]
fn a_missing_outcome_is_named() {
    let work = TempDir::new().unwrap();

    assert_eq!(
        outcome_problems(work.path()),
        ["review-run wrote no review outcome that reads"]
    );
}

#[test]
fn a_checkout_without_the_definitions_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();

    let problems = review_run_names_what_it_refuses(&c, work.path());

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("read .claude/agents/lang-rust.md: "),
        "{problems:?}"
    );
}

#[test]
fn a_work_directory_that_cannot_be_made_or_written_is_named() {
    let work = TempDir::new().unwrap();
    fs::write(work.path().join("review"), "a file").unwrap();
    let err = prepare(&repo(), work.path()).unwrap_err();
    assert!(err.starts_with("make "), "{err}");

    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review/dispatch.json")).unwrap();
    let err = prepare(&repo(), work.path()).unwrap_err();
    assert!(err.starts_with("write "), "{err}");

    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("class.txt")).unwrap();
    let err = prepare(&repo(), work.path()).unwrap_err();
    assert!(err.starts_with("write "), "{err}");
}

#[test]
fn a_review_run_that_cannot_write_its_outcome_is_named() {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review/outcome.json")).unwrap();

    let problems = review_run_names_what_it_refuses(&repo(), work.path());

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("review-run exited 1: review-run: write "),
        "{problems:?}"
    );
}

#[test]
fn the_stand_in_fails_a_run_that_logs_nowhere_or_cannot_log() {
    let work = TempDir::new().unwrap();

    let generalist = Command::new("claude", &["-p", "x", "--agent", GENERALIST], work.path());
    assert_eq!(
        RefusalHost::default().run(&generalist).unwrap_err(),
        "the run logs to no file"
    );

    let specialist = Command::new("claude", &[], work.path()).logging_to(&work.path().join("x/y"));
    assert!(
        RefusalHost::default()
            .run(&specialist)
            .unwrap_err()
            .starts_with("write ")
    );
}

#[test]
fn the_stand_in_keeps_each_specialist_run_s_prompt() {
    let work = TempDir::new().unwrap();
    let host = RefusalHost::default();
    let log = work.path().join("log.jsonl");
    for (agent, prompt) in [
        (SPECIALIST, "first"),
        (GENERALIST, "weigh"),
        (SPECIALIST, "second"),
    ] {
        let c =
            Command::new("claude", &["-p", prompt, "--agent", agent], work.path()).logging_to(&log);
        host.run(&c).unwrap();
    }

    assert_eq!(host.prompts(), ["first", "second"]);
}

#[test]
fn a_specialist_run_again_after_its_report_was_refused_is_named() {
    assert!(retry_problems(&["a".to_owned()]).is_empty());
    assert_eq!(
        retry_problems(&["a".to_owned(), "b".to_owned()]),
        ["review-run ran lang-rust 2 times, not once after it cited an unknown id"]
    );
    assert_eq!(
        retry_problems(&[]),
        ["review-run ran lang-rust 0 times, not once after it cited an unknown id"]
    );
}

#[test]
fn a_gate_that_fails_on_what_review_run_wrote_is_named() {
    let work = TempDir::new().unwrap();

    let problems = gate_problems(work.path());

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("the gate failed rather than posting: review-gate: read "),
        "{problems:?}"
    );
}

#[test]
fn a_gate_that_posts_an_approval_is_named() {
    let work = TempDir::new().unwrap();
    prepare(&repo(), work.path()).unwrap();
    let write = |rel: &str, text: &str| {
        let path = work.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    };
    write(
        "review/outcome.json",
        r#"{"verdict": "approve", "summary": "s", "findings": []}"#,
    );
    write("review/consulted.json", r#"["lang-rust", "review-code"]"#);
    write(
        "review/answers.json",
        r#"{"lang-rust": "outcome: none — fine", "review-code": "r"}"#,
    );
    write("review/reports/lang-rust.md", "outcome: none — fine\n");

    assert_eq!(
        gate_problems(work.path()),
        ["the gate posts APPROVE, not a request for changes"]
    );
}

#[test]
fn a_gate_whose_files_cannot_be_written_is_named() {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join(CHECK_RUNS)).unwrap();

    let problems = gate_problems(work.path());

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("the gate failed rather than posting: write "),
        "{problems:?}"
    );
}
