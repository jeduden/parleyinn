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
        RefusalHost.run(&generalist).unwrap_err(),
        "the run logs to no file"
    );

    let specialist = Command::new("claude", &[], work.path()).logging_to(&work.path().join("x/y"));
    assert!(
        RefusalHost
            .run(&specialist)
            .unwrap_err()
            .starts_with("write ")
    );
}
