use super::*;
use crate::tests::checkout;

use testkit::TempDir;

fn repo() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn a_specialist_split_on_the_checkout_s_definitions_has_no_problem() {
    let work = TempDir::new().unwrap();

    let problems = review_runs_in_parts(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn the_diff_is_past_the_limit_and_no_two_of_its_files_fit_one_part() {
    let diff = big_diff();

    assert!(diff.len() > LIMIT);
    let parts = review_bundle::diff::split(&diff, LIMIT);
    assert_eq!(parts.len(), FILES.len());
}

#[test]
fn a_part_is_read_from_its_prompt() {
    assert_eq!(part_of("so: you are part 2 of 3. Read"), Some(2));
    assert_eq!(part_of("you are part two of 3"), None);
    assert_eq!(part_of("Read your diff."), None);
}

#[test]
fn a_whole_run_that_did_not_split_or_merge_is_named() {
    let work = TempDir::new().unwrap();

    let problems = whole_problems(&["you are part 1 of 1".to_owned()], work.path());

    assert_eq!(
        problems,
        [
            "test-engineer-unit ran 1 times, not once per part of its diff",
            "no run of test-engineer-unit was told part 1, `review/diffs/test-engineer-unit.1.diff`",
            "no run of test-engineer-unit was told part 2, `review/diffs/test-engineer-unit.2.diff`",
            "no run of test-engineer-unit was told part 3, `review/diffs/test-engineer-unit.3.diff`",
            "test-engineer-unit's report is not the merge of its parts' reports: \"\"",
            "test-engineer-unit answered in every part but is not consulted",
        ]
    );
}

#[test]
fn a_cut_run_that_neither_fails_nor_keeps_the_findings_is_named() {
    let work = TempDir::new().unwrap();

    let problems = cut_problems(work.path());

    assert_eq!(
        problems,
        [
            "a part of test-engineer-unit that did not answer does not fail the review naming it",
            "test-engineer-unit's report does not keep every part's findings under its \
             incomplete line: \"\"",
        ]
    );
}

#[test]
fn a_checkout_without_the_definitions_is_named_for_both_runs() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();

    let problems = review_runs_in_parts(&c, work.path());

    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(
        problems
            .iter()
            .all(|p| p.starts_with("read .claude/agents/test-engineer-unit.md: ")),
        "{problems:?}"
    );
}

#[test]
fn a_work_directory_that_cannot_be_written_is_named() {
    let work = TempDir::new().unwrap();
    fs::write(work.path().join("whole"), "a file").unwrap();

    let err = run(&repo(), &work.path().join("whole"), None).unwrap_err();

    assert!(err.starts_with("make "), "{err}");
}

#[test]
fn a_review_run_that_cannot_write_its_outcome_is_named() {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("whole/review/outcome.json")).unwrap();
    fs::create_dir_all(work.path().join("cut/review/outcome.json")).unwrap();

    let problems = review_runs_in_parts(&repo(), work.path());

    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(
        problems
            .iter()
            .all(|p| p.starts_with("review-run exited 1: review-run: write ")),
        "{problems:?}"
    );
}

#[test]
fn the_stand_in_fails_a_run_that_logs_nowhere_or_cannot_log() {
    let work = TempDir::new().unwrap();
    let host = PartHost::default();

    let generalist = Command::new("claude", &["-p", "x", "--agent", GENERALIST], work.path());
    assert_eq!(
        host.run(&generalist).unwrap_err(),
        "the run logs to no file"
    );

    let specialist = Command::new("claude", &[], work.path()).logging_to(&work.path().join("x/y"));
    assert!(host.run(&specialist).unwrap_err().starts_with("write "));
}
