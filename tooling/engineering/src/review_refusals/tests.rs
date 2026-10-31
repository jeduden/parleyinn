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
fn a_specialist_not_run_once_more_told_its_refusal_is_named() {
    let told = format!(
        "x {} cites {UNKNOWN_ID} y {}",
        "z",
        review_run::retry::FORMS
    );

    assert!(retry_problems(&["a".to_owned(), told.clone()]).is_empty());
    assert_eq!(
        retry_problems(&["a".to_owned()]),
        ["review-run ran lang-rust 1 time, not once more after its report was refused"]
    );
    assert_eq!(
        retry_problems(&["a".to_owned(), "b".to_owned()]),
        [
            "lang-rust's second run was not told its report cites ENG-999",
            "lang-rust's second run was not told the forms again",
        ]
    );
}
