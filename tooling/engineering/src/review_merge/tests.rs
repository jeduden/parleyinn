use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::*;

/// A checkout under `root` whose merge script reads `script`.
fn with_script(root: &Path, script: &str) -> Checkout {
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join(SCRIPT), script).unwrap();
    Checkout::new(root)
}

/// What the review did before it merged the head: it diffed from the
/// first merge base and read the head alone.
const MERGE_BASE: &str = "#!/usr/bin/env bash\nset -e\n\
    git -C pr-head diff \"$(git rev-parse HEAD)...$(git -C pr-head rev-parse HEAD)\" > pr.diff\n";

fn set(files: &[&str]) -> BTreeSet<String> {
    files.iter().map(|f| (*f).to_owned()).collect()
}

fn tried(changes: Option<Vec<&'static str>>) -> Tried {
    Tried {
        name: "a branch",
        main: "m".to_owned(),
        head: "h".to_owned(),
        changes,
        conflict: "base.txt",
        holds_main: false,
    }
}

fn merged(diffed: &[&str], differs: &[&str], at: &str) -> Merged {
    Merged {
        success: true,
        output: String::new(),
        diffed: set(diffed),
        differs: set(differs),
        at: at.to_owned(),
    }
}

#[test]
fn the_checkout_s_review_reads_the_head_merged_into_main() {
    let checkout = Checkout::new(&testkit::repo_root());
    assert_eq!(wiring_problems(&checkout), Ok(Vec::new()));
}

#[test]
fn the_checkout_s_script_merges_the_head_into_main() {
    let work = testkit::TempDir::new().unwrap();
    let checkout = Checkout::new(&testkit::repo_root());
    assert_eq!(merge_problems(&checkout, work.path()), Vec::<String>::new());
}

#[test]
fn a_script_that_reads_the_head_alone_is_caught_on_three_histories() {
    let root = testkit::TempDir::new().unwrap();
    let work = testkit::TempDir::new().unwrap();
    let checkout = with_script(root.path(), MERGE_BASE);
    let problems = merge_problems(&checkout, work.path());
    assert_eq!(problems.len(), 3, "{problems:?}");
    // Whichever of the two merge bases git diffs from, the diff reads
    // the other one's change as the branch's own.
    let stacked = "scripts/review-merge-head.sh's pr.diff for a branch that merged main and another pull request apart, once main merged that one names";
    assert!(
        [
            format!("{stacked} main.txt, own.txt, not own.txt"),
            format!("{stacked} own.txt, parent.txt, not own.txt"),
        ]
        .contains(&problems[0]),
        "{problems:?}"
    );
    assert_eq!(
        problems[1],
        "scripts/review-merge-head.sh's pr-head for a branch behind main differs from main in base.txt, behind.txt, later.txt, not behind.txt"
    );
    assert_eq!(
        problems[2],
        "scripts/review-merge-head.sh reviews a head that does not merge with main"
    );
}

#[test]
fn a_missing_script_is_named() {
    let root = testkit::TempDir::new().unwrap();
    let work = testkit::TempDir::new().unwrap();
    assert_eq!(
        merge_problems(&Checkout::new(root.path()), work.path()),
        ["scripts/review-merge-head.sh is missing"]
    );
}

#[test]
fn a_history_that_cannot_be_built_is_named() {
    let root = testkit::TempDir::new().unwrap();
    let work = testkit::TempDir::new().unwrap();
    let checkout = with_script(root.path(), MERGE_BASE);
    // The work directory's home cannot be made under a file.
    let file = work.path().join("file");
    fs::write(&file, "").unwrap();
    let problems = merge_problems(&checkout, &file);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("scripts/review-merge-head.sh could not be tried: make "),
        "{problems:?}"
    );
}

#[test]
fn a_script_that_fails_on_a_head_that_merges_is_named() {
    let root = testkit::TempDir::new().unwrap();
    let work = testkit::TempDir::new().unwrap();
    let checkout = with_script(root.path(), "echo cannot >&2\nexit 1\n");
    let problems = merge_problems(&checkout, work.path());
    assert_eq!(problems.len(), 4, "{problems:?}");
    assert_eq!(
        problems[1],
        "scripts/review-merge-head.sh fails on a branch behind main: cannot"
    );
    assert_eq!(
        problems[3],
        "scripts/review-merge-head.sh refuses a head that does not merge with main without naming base.txt"
    );
}

#[test]
fn tried_problems_hold_a_head_that_merges_to_its_own_change() {
    let want = Some(vec!["own.txt"]);
    assert!(
        tried_problems(
            &tried(want.clone()),
            &merged(&["own.txt"], &["own.txt"], "x")
        )
        .is_empty()
    );
    assert_eq!(
        tried_problems(&tried(want.clone()), &merged(&[], &["own.txt"], "x")),
        ["scripts/review-merge-head.sh's pr.diff for a branch names nothing, not own.txt"]
    );
    assert_eq!(
        tried_problems(
            &tried(want.clone()),
            &merged(&["own.txt"], &["a", "own.txt"], "x")
        ),
        [
            "scripts/review-merge-head.sh's pr-head for a branch differs from main in a, own.txt, not own.txt"
        ]
    );
    let holds = Tried {
        holds_main: true,
        ..tried(want)
    };
    assert!(tried_problems(&holds, &merged(&["own.txt"], &["own.txt"], "h")).is_empty());
    assert_eq!(
        tried_problems(&holds, &merged(&["own.txt"], &["own.txt"], "x")),
        ["scripts/review-merge-head.sh leaves pr-head for a branch at x, not the head"]
    );
}

#[test]
fn tried_problems_hold_a_head_that_does_not_merge_to_a_refusal_naming_the_file() {
    let refused = |output: &str| Merged {
        output: output.to_owned(),
        ..Merged::default()
    };
    assert!(tried_problems(&tried(None), &refused("conflict in base.txt")).is_empty());
    assert_eq!(
        tried_problems(&tried(None), &refused("conflict")),
        ["scripts/review-merge-head.sh refuses a branch without naming base.txt"]
    );
    assert_eq!(
        tried_problems(&tried(None), &merged(&[], &[], "x")),
        ["scripts/review-merge-head.sh reviews a branch"]
    );
}

#[test]
fn diffed_files_are_those_a_diff_names() {
    let diff = "diff --git a/x.txt b/x.txt\n+x\ndiff --git a/d/y.md b/d/y.md\n-y\n";
    assert_eq!(diffed_files(diff), set(&["d/y.md", "x.txt"]));
    assert!(diffed_files("+x\n").is_empty());
}

#[test]
fn listed_names_each_file_or_nothing() {
    assert_eq!(listed(&set(&[])), "nothing");
    assert_eq!(listed(&set(&["b", "a"])), "a, b");
}

#[test]
fn order_problems_hold_the_first_before_each_later() {
    let text = "merge\nbundle\nplan\n";
    assert!(order_problems("w", text, "merge", &["bundle", "plan"]).is_empty());
    assert_eq!(
        order_problems("w", text, "absent", &["bundle"]),
        ["w does not hold absent"]
    );
    assert_eq!(
        order_problems("w", text, "bundle", &["merge", "gone"]),
        ["w holds merge before bundle", "w does not hold gone"]
    );
}

#[test]
fn merge_base_problems_name_a_three_dot_diff() {
    let text = "        run: git -C pr-head diff \"origin/main...$HEAD_SHA\" > pr.diff\n";
    assert_eq!(
        merge_base_problems("w", text),
        [
            "w diffs from a merge base: run: git -C pr-head diff \"origin/main...$HEAD_SHA\" > pr.diff"
        ]
    );
    assert!(merge_base_problems("w", "git diff \"$main\" \"$merge\"\n").is_empty());
}

#[test]
fn history_problems_want_the_head_checked_out_with_its_history() {
    assert!(history_problems("j", "          fetch-depth: 0\n").is_empty());
    assert_eq!(
        history_problems("j", "          path: pr-head\n"),
        ["j checks the head out without its history, so it cannot merge it into main"]
    );
}

#[test]
fn head_problems_want_the_run_told_the_head() {
    assert!(head_problems("w", "review-run -- --head-sha \"$HEAD_SHA\"\n").is_empty());
    assert_eq!(
        head_problems("w", "review-run -- --workdir .\n"),
        ["w runs review-run without --head-sha, so its run names the merge in pr-head as the head"]
    );
}

#[test]
fn wiring_problems_name_a_file_that_cannot_be_read() {
    let root = testkit::TempDir::new().unwrap();
    assert!(
        wiring_problems(&Checkout::new(root.path()))
            .unwrap_err()
            .contains(WORKFLOW)
    );
}
