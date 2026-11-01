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

/// Every dispatched agent run once, each generalist once its own
/// specialists ended: review-code while lang-markdown, which it does not
/// weigh, still runs.
fn in_order() -> Vec<Event> {
    vec![
        started("lang-markdown"),
        started("lang-rust"),
        ended("lang-rust"),
        started("test-engineer-unit"),
        ended("test-engineer-unit"),
        started("review-code"),
        ended("lang-markdown"),
        started("review-spec"),
        ended("review-spec"),
        ended("review-code"),
    ]
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
fn a_generalist_started_before_one_of_its_own_specialists_ended_is_named() {
    let mut events = in_order();
    let end = events.remove(4);
    events.push(end);

    assert_eq!(
        order_problems(&events),
        ["review-code started before its specialist test-engineer-unit ended"]
    );
}

#[test]
fn a_generalist_that_waited_for_a_specialist_it_does_not_weigh_is_named() {
    let mut events = in_order();
    events.swap(5, 6);

    assert_eq!(
        order_problems(&events),
        [
            "review-code did not start while lang-markdown, a specialist it does not weigh, \
             still ran"
        ]
    );
}

#[test]
fn a_generalist_that_never_started_is_named_once_and_not_as_early() {
    let mut events = in_order();
    events.retain(|e| *e != started("review-spec") && *e != started("review-code"));

    assert_eq!(
        order_problems(&events),
        [
            "review-code ran 0 times, not once as its own agent",
            "review-spec ran 0 times, not once as its own agent"
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

/// Writes `outcome` and `consulted` where review-run writes them, and
/// each specialist's report as review-run assembles it.
fn written(outcome: &str, consulted: &str) -> TempDir {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review/reports")).unwrap();
    fs::write(work.path().join("review/outcome.json"), outcome).unwrap();
    fs::write(work.path().join("review/consulted.json"), consulted).unwrap();
    for name in SPECIALISTS {
        fs::write(
            work.path().join(format!("review/reports/{name}.md")),
            format!("{FINDING}\noutcome: findings — {name} saw one finding.\n"),
        )
        .unwrap();
    }
    work
}

#[test]
fn a_specialist_s_report_not_assembled_from_its_log_is_named() {
    let work = written(
        r#"{"verdict": "approve", "summary": "s", "findings": [{"path": "src/a.rs", "line": 1, "severity": "nit", "body": "b"}, {"path": "docs/a.md", "line": 1, "severity": "nit", "body": "b"}]}"#,
        r#"["lang-markdown", "lang-rust", "test-engineer-unit", "review-code", "review-spec"]"#,
    );
    assert!(outcome_problems(work.path()).is_empty());
    fs::write(
        work.path().join("review/reports/lang-rust.md"),
        "outcome: findings — lang-rust saw one finding.\n",
    )
    .unwrap();
    fs::remove_file(work.path().join("review/reports/lang-markdown.md")).unwrap();

    let problems = outcome_problems(work.path());

    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].starts_with("read "), "{problems:?}");
    assert_eq!(
        problems[1],
        "lang-rust's report is not assembled from its log: \"outcome: findings — lang-rust saw one finding.\\n\""
    );
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
    assert_eq!(problems.len(), 5, "{problems:?}");
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
fn the_stand_in_answers_as_the_agent_in_its_log_alone_and_records_its_run() {
    let work = TempDir::new().unwrap();
    let log = work.path().join("log.jsonl");
    let host = Recorder::default();
    let command = Command::new("claude", &["--agent", "lang-rust"], work.path()).logging_to(&log);

    let out = host.run(&command).unwrap();

    assert_eq!(out.code, Some(0));
    assert_eq!(
        host.events.lock().unwrap().clone(),
        [started("lang-rust"), ended("lang-rust")]
    );
    let logged = review_eval::log::parse(&fs::read_to_string(&log).unwrap());
    assert!(logged.answered);
    assert_eq!(
        logged.answer,
        "outcome: findings — lang-rust saw one finding."
    );
    assert_eq!(logged.texts, [format!("I read the diff.\n{FINDING}")]);
    assert!(!work.path().join("review/reports").exists());
}

#[test]
fn the_holding_stand_in_ends_the_held_run_once_its_generalist_started_or_after_its_hold() {
    let work = TempDir::new().unwrap();
    let run = |agent: &str| {
        Command::new("claude", &["--agent", agent], work.path())
            .logging_to(&work.path().join(format!("{agent}.jsonl")))
    };
    let host = Recorder::holding();

    std::thread::scope(|scope| {
        let held = scope.spawn(|| host.run(&run("lang-markdown")));
        while host.events.lock().unwrap().is_empty() {
            std::thread::yield_now();
        }
        host.run(&run("review-code")).unwrap();
        held.join().unwrap().unwrap();
    });

    assert!(
        order_problems(&host.events.lock().unwrap())
            .iter()
            .all(|p| !p.contains("did not start while"))
    );
    let unheld = Recorder::default();
    unheld.run(&run("lang-markdown")).unwrap();
    assert_eq!(unheld.events.lock().unwrap().len(), 2);
}

#[test]
fn the_stand_in_fails_a_run_that_logs_nowhere() {
    let work = TempDir::new().unwrap();
    let host = Recorder::default();

    let generalist = Command::new("claude", &["--agent", "review-code"], work.path());
    assert_eq!(
        host.run(&generalist).unwrap_err(),
        "the run logs to no file"
    );

    let specialist = Command::new("claude", &[], work.path()).logging_to(&work.path().join("x/y"));
    assert!(host.run(&specialist).unwrap_err().starts_with("write "));
    assert_eq!(agent_of(&specialist), "");
}

#[test]
fn a_dispatch_that_does_not_parse_is_a_problem_naming_why() {
    let problem = parse_dispatch("{").unwrap_err();
    assert!(problem.starts_with("the dispatch the host was given does not parse: "));
    assert!(parse_dispatch(DISPATCH).is_ok());
}
