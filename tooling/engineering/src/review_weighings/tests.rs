use super::*;
use crate::tests::checkout;

use testkit::TempDir;

fn repo() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

fn write_outcome(dir: &Path, text: &str) {
    fs::create_dir_all(dir.join("review")).unwrap();
    fs::write(dir.join(review_run::OUTCOME), text).unwrap();
}

#[test]
fn the_checkout_s_review_run_reads_a_whole_weighing_and_decides_a_partial_one() {
    let work = TempDir::new().unwrap();

    let problems = review_run_reads_whole_weighings(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_whole_weighing_not_read_is_named() {
    let work = TempDir::new().unwrap();
    write_outcome(
        work.path(),
        r#"{"verdict": "request_changes", "summary": "s", "findings": []}"#,
    );

    assert_eq!(
        whole_problems(work.path()),
        [
            "a weighing stated before the final answer is not read: the review is request_changes",
            "the nit review-code stated at src/a.rs:1 before its final answer is not kept"
        ]
    );
}

#[test]
fn a_partial_weighing_passed_as_whole_is_named() {
    let work = TempDir::new().unwrap();
    write_outcome(
        work.path(),
        r#"{"verdict": "approve", "summary": "s", "findings": []}"#,
    );

    assert_eq!(
        partial_problems(work.path()),
        [
            "a partial weighing does not request changes: approve",
            "no blocking finding names src/a.rs:1, which review-code weighed nowhere",
            "review-code, whose weighing is partial, is not named as not reviewed"
        ]
    );
}

#[test]
fn a_missing_outcome_is_named_by_each_check() {
    let work = TempDir::new().unwrap();
    let missing = ["review-run wrote no review outcome that reads"];

    assert_eq!(whole_problems(work.path()), missing);
    assert_eq!(partial_problems(work.path()), missing);
}

#[test]
fn a_checkout_without_the_definitions_or_a_review_that_cannot_run_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();
    let problems = review_run_reads_whole_weighings(&c, work.path());
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("read .claude/agents/lang-rust.md: "),
        "{problems:?}"
    );

    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("whole/review/outcome.json")).unwrap();
    let problems = review_run_reads_whole_weighings(&repo(), work.path());
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("review-run exited 1: "),
        "{problems:?}"
    );
}

#[test]
fn the_stand_in_fails_a_run_that_logs_nowhere_or_cannot_write_its_log() {
    let work = TempDir::new().unwrap();
    let host = WeighingHost::default();
    let args = ["--agent", "lang-rust"];
    let nowhere = Command::new("claude", &args, Path::new("/w"));
    assert_eq!(host.run(&nowhere).unwrap_err(), "the run logs to no file");

    let unwritable =
        Command::new("claude", &args, Path::new("/w")).logging_to(&work.path().join("no/dir"));
    assert!(host.run(&unwritable).unwrap_err().starts_with("write "));
}
