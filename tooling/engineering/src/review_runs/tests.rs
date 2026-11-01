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

/// The prompt a specialist is told to read only its own diff and its
/// own context in.
fn own_prompt(name: &str) -> String {
    format!(
        "You are `{name}`. Read only your own diff, `review/diffs/{name}.diff`, and \
         `review/context/{name}.md`; never `review/bundle.md`."
    )
}

#[test]
fn a_specialist_told_to_read_its_own_diff_and_context_alone_has_no_problem() {
    let prompts: Vec<(String, String)> = SPECIALISTS
        .iter()
        .map(|n| ((*n).to_owned(), own_prompt(n)))
        .collect();

    assert!(prompt_problems(&prompts).is_empty());
}

#[test]
fn a_specialist_told_to_read_the_bundle_or_not_its_own_files_is_named() {
    let prompts = vec![
        (
            "lang-rust".to_owned(),
            "Read `review/bundle.md` first, then `review/diffs/lang-rust.diff`.".to_owned(),
        ),
        ("lang-markdown".to_owned(), own_prompt("lang-rust")),
    ];

    assert_eq!(
        prompt_problems(&prompts),
        [
            "lang-rust is not told to read review/context/lang-rust.md",
            "lang-rust is not told to leave review/bundle.md unread",
            "lang-markdown is not told to read review/diffs/lang-markdown.diff",
            "lang-markdown is not told to read review/context/lang-markdown.md",
        ]
    );
}

#[test]
fn a_generalist_s_prompt_is_not_held_to_the_specialist_s_reads() {
    let prompts = vec![("review-code".to_owned(), "Read the bundle.".to_owned())];

    assert!(prompt_problems(&prompts).is_empty());
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
    assert_eq!(
        order_problems_by(&in_order(), "{"),
        [problem],
        "an unparsable dispatch is the one problem"
    );
}

/// The prompt of part `index` of `name`'s split diff, told to read
/// `diff`.
fn part_prompt(name: &str, diff: &str) -> (String, String) {
    (
        name.to_owned(),
        format!("Read only your own diff, `{diff}`, alone first; never `review/bundle.md`."),
    )
}

#[test]
fn each_part_told_to_read_its_own_part_alone_has_no_problem() {
    let prompts = vec![
        part_prompt("lang-rust", "review/diffs/lang-rust.1.diff"),
        part_prompt("lang-rust", "review/diffs/lang-rust.2.diff"),
        part_prompt("lang-markdown", "review/diffs/lang-markdown.diff"),
    ];

    assert!(part_problems("lang-rust", 2, &prompts).is_empty());
}

#[test]
fn a_part_told_to_read_the_whole_diff_or_the_bundle_or_missing_is_named() {
    let prompts = vec![
        part_prompt("lang-rust", "review/diffs/lang-rust.diff"),
        (
            "lang-rust".to_owned(),
            "Read only your own diff, `review/diffs/lang-rust.2.diff`.".to_owned(),
        ),
    ];

    assert_eq!(
        part_problems("lang-rust", 2, &prompts),
        [
            "lang-rust part 1 is not told to read its own part's diff review/diffs/lang-rust.1.diff",
            "lang-rust is told to read the whole diff in a part",
            "lang-rust is not told to leave review/bundle.md unread in a part",
        ]
    );
}

#[test]
fn a_split_run_on_the_checkout_s_definitions_has_no_problem() {
    let work = TempDir::new().unwrap();

    let problems = split_runs_by_code(&Checkout::new(&testkit::repo_root()), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_split_run_that_cannot_be_prepared_written_or_run_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();
    assert!(split_runs_by_code(&c, work.path())[0].starts_with("read "));

    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review")).unwrap();
    fs::write(work.path().join("review/diffs"), "a file").unwrap();
    let problems = split_runs_by_code(&Checkout::new(&testkit::repo_root()), work.path());
    assert!(problems[0].starts_with("write "), "{problems:?}");

    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review/outcome.json")).unwrap();
    let problems = split_runs_by_code(&Checkout::new(&testkit::repo_root()), work.path());
    assert_eq!(problems, ["review-run exited 1 on a split diff"]);
}

/// The outputs of a review bundle run on a change to the file
/// [`CONTEXT`] names, and the context it wrote.
fn bundle_outputs() -> (String, String, String, String) {
    (
        "class=standard\ntouched=1\nfindings=1\nconsults=2\n".to_owned(),
        r#"{"review-spec": ["lang-markdown"]}"#.to_owned(),
        r#"[{"path": "docs/srs/10-engineering-quality.md", "line": 0, "severity": "blocking", "body": "x"}]"#.to_owned(),
        "Class: **standard**. The change touches 1 id: ENG-28.\n\n- `review-spec`, weighing `lang-markdown`\n\n- blocking `docs/srs/10-engineering-quality.md:0`: x\n".to_owned(),
    )
}

fn lacks(outputs: &(String, String, String, String), context: &str) -> Vec<String> {
    let (name, path, id) = CONTEXT;
    context_lacks(name, path, id, &outputs.0, &outputs.1, &outputs.2, context)
}

#[test]
fn a_context_with_the_class_ids_row_and_findings_has_no_problem() {
    let o = bundle_outputs();

    assert!(lacks(&o, &o.3).is_empty());
}

#[test]
fn a_context_that_lacks_a_part_of_what_the_bundle_says_is_named() {
    let o = bundle_outputs();

    assert_eq!(
        lacks(&o, "nothing"),
        [
            "the context for lang-markdown lacks the review class \"standard\"",
            "the context for lang-markdown lacks the touched id ENG-28",
            "the context for lang-markdown lacks its dispatch row under review-spec",
            "the context for lang-markdown lacks the deterministic finding on docs/srs/10-engineering-quality.md:0",
        ]
    );
}

#[test]
fn a_bundle_that_names_no_class_row_or_finding_is_named() {
    let (name, path, id) = CONTEXT;

    let got = context_lacks(name, path, id, "", "not json", "not json", "");

    assert_eq!(
        got,
        [
            "the context for lang-markdown lacks the review class \"\"",
            "the context for lang-markdown lacks the touched id ENG-28",
            "the dispatch has no row naming lang-markdown",
            "the bundle found nothing on docs/srs/10-engineering-quality.md to put in the context",
        ]
    );
}

#[test]
fn a_bundle_context_on_the_checkout_has_no_problem() {
    let work = TempDir::new().unwrap();

    let problems = context_problems(&Checkout::new(&testkit::repo_root()), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_bundle_that_cannot_be_written_or_run_is_named() {
    let work = TempDir::new().unwrap();
    let file = work.path().join("a file");
    fs::write(&file, "x").unwrap();
    let problems = context_problems(&Checkout::new(&testkit::repo_root()), &file);
    assert!(problems[0].starts_with("write "), "{problems:?}");

    let problems = context_problems(
        &Checkout::new(&work.path().join("none")),
        &work.path().join("out"),
    );
    assert!(
        problems[0].starts_with("review-bundle exited "),
        "{problems:?}"
    );
}

#[test]
fn a_file_the_bundle_did_not_write_is_named() {
    let work = TempDir::new().unwrap();

    let problems = context_in(work.path(), "");

    assert_eq!(problems.len(), 3, "{problems:?}");
    assert!(
        problems.iter().all(|p| p.starts_with("read ")),
        "{problems:?}"
    );
}

#[test]
fn a_recorder_with_a_poisoned_lock_is_an_error_not_an_empty_list() {
    let host = Recorder::default();
    host.record(started("lang-rust"));
    assert_eq!(host.recorded().unwrap().0, [started("lang-rust")]);

    for poisoned in ["events", "prompts"] {
        let host = std::sync::Arc::new(Recorder::default());
        let h = host.clone();
        let _ = std::thread::spawn(move || {
            let _events = (poisoned == "events").then(|| h.events.lock().unwrap());
            let _prompts = (poisoned == "prompts").then(|| h.prompts.lock().unwrap());
            panic!("poison");
        })
        .join();
        let host = std::sync::Arc::try_unwrap(host).unwrap();

        assert_eq!(
            host.recorded().unwrap_err(),
            format!("the stand-in's {poisoned} lock is poisoned")
        );
    }
}

#[test]
fn what_a_recorder_with_a_poisoned_lock_recorded_is_checked_by_naming_the_lock() {
    let host = std::sync::Arc::new(Recorder::default());
    let h = host.clone();
    let _ = std::thread::spawn(move || {
        let _prompts = h.prompts.lock().unwrap();
        panic!("poison");
    })
    .join();
    let host = std::sync::Arc::try_unwrap(host).unwrap();

    let problems = with_recorded(host, |_, _| vec!["checked".to_owned()]);

    assert_eq!(problems, ["the stand-in's prompts lock is poisoned"]);
    let ok = with_recorded(Recorder::default(), |_, _| vec!["checked".to_owned()]);
    assert_eq!(ok, ["checked"]);
}
