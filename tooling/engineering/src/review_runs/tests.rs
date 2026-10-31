use super::*;
use crate::tests::checkout;

use review_eval::host::Host;
use testkit::TempDir;

use Event::{Ended, Started};

fn started(name: &str) -> Event {
    Started(name.to_owned())
}

fn ended(name: &str) -> Event {
    Ended(name.to_owned())
}

/// Every dispatched agent run once: the specialists one after another,
/// then the generalists.
fn in_order() -> Vec<Event> {
    let mut events = Vec::new();
    for name in SPECIALISTS
        .iter()
        .chain(GENERALISTS.iter().map(|(g, _, _)| g))
    {
        events.extend([started(name), ended(name)]);
    }
    events
}

#[test]
fn a_review_run_on_the_checkout_s_definitions_has_no_problem() {
    let work = TempDir::new().unwrap();

    let problems = review_runs_by_code(&Checkout::new(&testkit::repo_root()), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn every_agent_run_once_and_the_generalists_last_is_in_order() {
    assert!(order_problems(&in_order()).is_empty());
}

#[test]
fn a_generalist_started_before_a_specialist_ended_is_named() {
    let mut events = in_order();
    let last = events.remove(5);
    events.push(last);

    assert_eq!(
        order_problems(&events),
        [
            "review-code started before test-engineer-unit ended",
            "review-spec started before test-engineer-unit ended"
        ]
    );
}

#[test]
fn a_generalist_that_never_started_is_named_once_and_not_as_early() {
    let mut events = in_order();
    events.retain(|e| *e != started("review-spec"));
    let last = events.remove(5);
    events.push(last);

    assert_eq!(
        order_problems(&events),
        [
            "review-spec ran 0 times, not once as its own agent",
            "review-code started before test-engineer-unit ended"
        ]
    );
}

#[test]
fn an_agent_run_twice_never_or_undispatched_is_named() {
    let mut events = in_order();
    events.retain(|e| *e != started("lang-rust"));
    events.extend([started("review-code"), started("lang-toml")]);

    assert_eq!(
        order_problems(&events),
        [
            "lang-rust ran 0 times, not once as its own agent",
            "review-code ran 2 times, not once as its own agent",
            "lang-toml ran but was not dispatched"
        ]
    );
}

/// Writes `outcome` and `consulted` where review-run writes them.
fn written(outcome: &str, consulted: &str) -> TempDir {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review")).unwrap();
    fs::write(work.path().join("review/outcome.json"), outcome).unwrap();
    fs::write(work.path().join("review/consulted.json"), consulted).unwrap();
    work
}

#[test]
fn an_outcome_without_an_answered_finding_or_a_consulted_agent_is_named() {
    let work = written(
        r#"{"verdict": "approve", "summary": "s", "findings": [{"path": "src/a.rs", "line": 1, "severity": "nit", "body": "b"}]}"#,
        r#"["lang-markdown", "lang-rust", "test-engineer-unit", "review-code"]"#,
    );

    assert_eq!(
        outcome_problems(work.path()),
        [
            "the review outcome leaves out the finding on docs/a.md review-spec answered",
            "review-spec answered but is not consulted"
        ]
    );
}

#[test]
fn an_outcome_or_consulted_list_that_does_not_read_is_named() {
    let work = written("{", "{");

    let problems = outcome_problems(work.path());

    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(
        problems[0].starts_with("review/outcome.json: "),
        "{problems:?}"
    );
    assert!(
        problems[1].starts_with("review/consulted.json: "),
        "{problems:?}"
    );

    let empty = TempDir::new().unwrap();
    let problems = outcome_problems(empty.path());
    assert!(
        problems.iter().all(|p| p.starts_with("read ")),
        "{problems:?}"
    );
}

#[test]
fn a_checkout_without_a_dispatched_agent_s_definition_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();

    let problems = review_runs_by_code(&c, work.path());

    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].starts_with("read .claude/agents/lang-markdown.md: "),
        "{problems:?}"
    );
}

#[test]
fn a_work_directory_that_cannot_be_written_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();
    fs::write(work.path().join("review"), "a file").unwrap();

    let problems = review_runs_by_code(&c, work.path());

    assert!(problems[0].starts_with("make "), "{problems:?}");
}

#[test]
fn a_review_run_that_cannot_write_its_outcome_is_named() {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review/outcome.json")).unwrap();

    let problems = review_runs_by_code(&Checkout::new(&testkit::repo_root()), work.path());

    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].starts_with("review-run exited 1: review-run: write "),
        "{problems:?}"
    );
}

#[test]
fn the_stand_in_answers_as_the_agent_and_records_its_run() {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review/reports")).unwrap();
    let log = work.path().join("log.jsonl");
    let host = Recorder::default();
    let command = Command::new("claude", &["--agent", "lang-rust"], work.path()).logging_to(&log);

    let out = host.run(&command).unwrap();

    assert_eq!(out.code, Some(0));
    assert_eq!(
        host.events.lock().unwrap().clone(),
        [started("lang-rust"), ended("lang-rust")]
    );
    let report = fs::read_to_string(work.path().join("review/reports/lang-rust.md")).unwrap();
    assert!(
        report.ends_with("outcome: nits — lang-rust saw one nit.\n"),
        "{report}"
    );
    assert!(fs::read_to_string(&log).unwrap().contains("\"success\""));
}

#[test]
fn the_stand_in_fails_a_run_that_logs_nowhere_or_cannot_report() {
    let work = TempDir::new().unwrap();
    let host = Recorder::default();

    let generalist = Command::new("claude", &["--agent", "review-code"], work.path());
    assert_eq!(
        host.run(&generalist).unwrap_err(),
        "the run logs to no file"
    );

    let specialist = Command::new("claude", &[], work.path());
    assert!(host.run(&specialist).unwrap_err().starts_with("write "));
    assert_eq!(agent_of(&specialist), "");
}
