use super::*;
use crate::tests::checkout;

use testkit::TempDir;

fn repo() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn the_checkout_s_review_run_resumes_from_what_it_did_not_review() {
    let work = TempDir::new().unwrap();

    let problems = review_resumes(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_checkout_without_the_definitions_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();

    let problems = review_resumes(&c, work.path());

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("read .claude/agents/"),
        "{problems:?}"
    );
}

#[test]
fn a_resumed_run_that_cannot_be_prepared_is_named() {
    let work = TempDir::new().unwrap();
    for dir in ["resumed", "moved", "changed"] {
        write(work.path(), dir, "a file");
    }

    let problems = review_resumes(&repo(), work.path());

    assert_eq!(problems.len(), 3, "{problems:?}");
    assert!(
        problems.iter().all(|p| p.starts_with("make ")),
        "{problems:?}"
    );
}

#[test]
fn a_review_run_that_fails_is_named_with_its_last_line() {
    let work = TempDir::new().unwrap();

    let err = run(work.path(), ResumeHost::default(), HEAD, &[]).unwrap_err();

    assert!(
        err.starts_with("review-run exited 1: review-run: read "),
        "{err}"
    );
}

#[test]
fn a_run_that_wrote_nothing_has_no_gap_list_and_no_outcome() {
    let work = TempDir::new().unwrap();

    assert_eq!(
        gap_problems(work.path()),
        [
            "review-run wrote no gap list that reads: malformed gaps: EOF while parsing a value \
             at line 1 column 0",
            "review-run wrote no review outcome that reads",
        ]
    );
}

#[test]
fn a_gap_list_naming_another_or_an_outcome_that_passes_it_by_is_named() {
    let work = TempDir::new().unwrap();
    write(
        work.path(),
        "review/gaps.json",
        r#"[{"name": "lang-markdown", "kind": "specialist", "reason": "silent: x"}]"#,
    );
    write(
        work.path(),
        "review/outcome.json",
        r#"{"verdict": "approve", "summary": "s", "findings": []}"#,
    );

    assert_eq!(
        gap_problems(work.path()),
        [
            "the gap list names [\"lang-markdown\"], not lang-rust alone",
            "the review approves without lang-rust",
        ]
    );

    write(
        work.path(),
        "review/outcome.json",
        r#"{"verdict": "request_changes", "summary": "s", "findings": []}"#,
    );
    assert_eq!(
        gap_problems(work.path())[1],
        "the review outcome does not name lang-rust not reviewed"
    );
}

#[test]
fn agents_started_other_than_wanted_are_named() {
    let started: BTreeSet<String> = ["lang-rust".to_owned()].into();

    assert!(started_problems(&started, &["lang-rust"], "a run").is_empty());
    assert_eq!(
        started_problems(&started, &["review-code"], "a run"),
        ["a run started [\"lang-rust\"], not [\"review-code\"]"]
    );
}

#[test]
fn a_resumed_review_that_does_not_approve_or_consult_every_agent_is_named() {
    let work = TempDir::new().unwrap();
    let problems = resumed_problems(work.path());
    assert_eq!(
        problems[0],
        "the resumed review wrote no outcome that reads"
    );
    assert_eq!(problems.len(), 6, "{problems:?}");

    write(
        work.path(),
        "review/outcome.json",
        r#"{"verdict": "request_changes", "summary": "s", "findings": []}"#,
    );
    write(work.path(), "review/consulted.json", r#"["lang-rust"]"#);
    let problems = resumed_problems(work.path());
    assert_eq!(problems[0], "the resumed review does not approve: s");
    assert_eq!(problems.len(), 5, "{problems:?}");
}

#[test]
fn a_definition_that_cannot_be_changed_is_named() {
    let work = TempDir::new().unwrap();

    let err = change_definition(work.path()).unwrap_err();

    assert!(err.starts_with("change "), "{err}");
}
