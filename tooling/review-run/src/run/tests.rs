use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Barrier, Mutex};
use std::time::Duration;

use review_eval::host::{Command, Output};
use review_eval::runner::{GENERALIST_TOOLS, SPECIALIST_TOOLS};
use testkit::TempDir;

use super::*;
use crate::fake::{self, FakeHost, agent_of, prompt_of};
use crate::plan::parse_dispatch;

fn plan(dispatch: &str) -> Plan {
    Plan {
        class: "standard".to_owned(),
        touched: 2,
        dispatch: parse_dispatch(dispatch).unwrap(),
    }
}

const DISPATCH: &str =
    r#"{"review-code": ["lang-rust", "test-engineer-unit"], "review-spec": ["lang-markdown"]}"#;

/// A work directory defining every agent of [`DISPATCH`].
fn workdir() -> TempDir {
    let dir = TempDir::new().unwrap();
    for name in ["lang-rust", "test-engineer-unit", "lang-markdown"] {
        fake::define(dir.path(), name, 20);
    }
    for name in ["review-code", "review-spec"] {
        fake::define(dir.path(), name, 40);
    }
    dir
}

fn options() -> Options {
    Options {
        parallel: 8,
        deadline: Duration::from_secs(77),
    }
}

fn arg_after<'a>(command: &'a Command, flag: &str) -> &'a str {
    let at = command.args.iter().position(|a| a == flag).unwrap();
    &command.args[at + 1]
}

fn review_with(host: &FakeHost, dir: &Path, p: &Plan) -> Vec<AgentRun> {
    review(host, dir, p, options(), &|_| {}).unwrap()
}

#[test]
fn every_specialist_runs_before_any_generalist() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    let runs = review_with(&host, dir.path(), &plan(DISPATCH));

    let agents = host.agents();
    assert_eq!(agents.len(), 5);
    let mut first: Vec<String> = agents[..3].to_vec();
    first.sort();
    assert_eq!(first, ["lang-markdown", "lang-rust", "test-engineer-unit"]);
    let mut last: Vec<String> = agents[3..].to_vec();
    last.sort();
    assert_eq!(last, ["review-code", "review-spec"]);
    let names: Vec<&str> = runs.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "lang-markdown",
            "lang-rust",
            "test-engineer-unit",
            "review-code",
            "review-spec"
        ]
    );
    assert!(runs.iter().all(AgentRun::answered));
    assert_eq!(runs[0].kind, Kind::Specialist);
    assert_eq!(runs[3].kind, Kind::Generalist);
    assert_eq!(
        runs[0].log.answer.lines().next().unwrap(),
        "outcome: nits — lang-markdown saw one nit."
    );
}

#[test]
fn a_specialist_runs_with_the_review_s_tools_turns_deadline_and_log() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    review_with(&host, dir.path(), &plan(DISPATCH));

    let calls = host.calls.lock().unwrap().clone();
    let rust = calls.iter().find(|c| agent_of(c) == "lang-rust").unwrap();
    assert_eq!(rust.program, "claude");
    assert_eq!(rust.cwd, dir.path());
    assert_eq!(rust.deadline, Some(Duration::from_secs(77)));
    assert_eq!(
        rust.stdout_to.as_deref(),
        Some(dir.path().join("review/logs/lang-rust.jsonl").as_path())
    );
    assert_eq!(arg_after(rust, "--allowedTools"), SPECIALIST_TOOLS);
    assert_eq!(arg_after(rust, "--max-turns"), "20");
    assert_eq!(arg_after(rust, "--output-format"), "stream-json");
    assert!(!rust.args.iter().any(|a| a == "--model"));
    let prompt = prompt_of(rust);
    assert!(prompt.contains("`review/diffs/lang-rust.diff`"), "{prompt}");
    assert!(prompt.contains("`review/reports/lang-rust.md`"), "{prompt}");
}

#[test]
fn a_generalist_is_told_its_specialists_report_files_and_reads_only() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    review_with(&host, dir.path(), &plan(DISPATCH));

    let calls = host.calls.lock().unwrap().clone();
    let code = calls.iter().find(|c| agent_of(c) == "review-code").unwrap();
    assert_eq!(arg_after(code, "--allowedTools"), GENERALIST_TOOLS);
    assert_eq!(arg_after(code, "--max-turns"), "40");
    let prompt = prompt_of(code);
    assert!(
        prompt.contains(
            "reports are `review/reports/lang-rust.md`, `review/reports/test-engineer-unit.md`."
        ),
        "{prompt}"
    );
    assert!(prompt.contains("The review class is standard"), "{prompt}");
    assert!(prompt.contains("lists the 2 touched ids"), "{prompt}");
}

#[test]
fn progress_names_each_run() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);
    let lines = Mutex::new(Vec::new());

    review(&host, dir.path(), &plan(DISPATCH), options(), &|l| {
        lines.lock().unwrap().push(l.to_owned());
    })
    .unwrap();

    let lines = lines.into_inner().unwrap();
    assert_eq!(lines.len(), 5);
    assert!(
        lines.contains(&"lang-rust (specialist): answered (0.0100 USD, 3 turns)".to_owned()),
        "{lines:?}"
    );
}

#[test]
fn runs_stay_within_the_bound_and_use_it() {
    let active = AtomicUsize::new(0);
    let most = AtomicUsize::new(0);
    let items: Vec<usize> = (0..6).collect();

    // Each run waits for a second one to be running, so two overlap
    // without a clock; a third would raise the count past the bound.
    let both = Barrier::new(2);

    let out = parallel(&items, 2, &|n| {
        let now = active.fetch_add(1, Ordering::SeqCst) + 1;
        most.fetch_max(now, Ordering::SeqCst);
        both.wait();
        active.fetch_sub(1, Ordering::SeqCst);
        n * 10
    });

    assert_eq!(out, [0, 10, 20, 30, 40, 50]);
    assert_eq!(most.load(Ordering::SeqCst), 2);
}

#[test]
fn a_bound_of_zero_still_runs_one_at_a_time_and_nothing_runs_nothing() {
    assert_eq!(parallel(&[1, 2], 0, &|n| n + 1), [2, 3]);
    assert_eq!(parallel::<u8, u8>(&[], 4, &|n| *n), Vec::<u8>::new());
}

/// Runs the one-specialist plan with `claude` answering as `script`.
fn one_run(
    dir: &Path,
    script: impl Fn(&Command) -> Result<Output, String> + Sync + 'static,
) -> AgentRun {
    let host = FakeHost::new(script);
    let runs = review_with(&host, dir, &plan(r#"{"review-code": ["lang-rust"]}"#));
    runs.into_iter().next().unwrap()
}

#[test]
fn a_run_that_does_not_answer_says_why() {
    let cases: Vec<(&str, fake::Claude)> = vec![
        ("cannot start", Box::new(|_| Err("cannot start".to_owned()))),
        (
            "its run ended with none",
            Box::new(|c| {
                fake::write_log(c, "not json\n");
                Ok(Output::default())
            }),
        ),
        (
            "its run ended with deadline",
            Box::new(|c| {
                fake::write_log(c, &fake::log("late", 0.0));
                Ok(Output {
                    timed_out: true,
                    ..Output::default()
                })
            }),
        ),
        (
            "its run answered nothing",
            Box::new(|c| fake::answer(c, "  \n", None)),
        ),
        ("read ", Box::new(|_| Ok(Output::default()))),
    ];
    for (want, script) in cases {
        let dir = workdir();
        let run = one_run(dir.path(), script);
        assert!(!run.answered());
        let failure = run.failure.clone().unwrap();
        assert!(failure.starts_with(want), "{want}: {failure}");
        assert!(describe(&run).starts_with("lang-rust (specialist): did not answer: "));
    }
}

#[test]
fn an_agent_without_a_definition_does_not_run() {
    let dir = TempDir::new().unwrap();

    let run = one_run(dir.path(), fake::either);

    assert!(
        run.failure
            .unwrap()
            .starts_with("its definition does not read: read .claude/agents/lang-rust.md")
    );
}

#[test]
fn a_report_from_before_the_run_is_removed_first() {
    let dir = workdir();
    let report = dir.path().join("review/reports/lang-rust.md");
    fs::create_dir_all(report.parent().unwrap()).unwrap();
    fs::write(&report, "outcome: sound — stale.\n").unwrap();

    let run = one_run(dir.path(), |c| fake::answer(c, "outcome: sound — x.", None));

    assert!(run.answered());
    assert!(!report.exists());

    fs::create_dir_all(&report).unwrap();
    let run = one_run(dir.path(), fake::either);
    assert!(run.failure.unwrap().starts_with("remove "));
}

#[test]
fn a_work_directory_without_room_for_logs_fails_the_review() {
    let dir = workdir();
    fs::write(dir.path().join("review"), "a file").unwrap();
    let host = FakeHost::new(fake::either);

    let err = review(&host, dir.path(), &plan(DISPATCH), options(), &|_| {}).unwrap_err();

    assert!(err.starts_with("make "), "{err}");
    assert!(host.agents().is_empty());
}

#[test]
fn the_default_options_are_the_evaluation_s_deadline_and_eight_at_once() {
    let o = Options::default();
    assert_eq!(o.parallel, 8);
    assert_eq!(o.deadline, Duration::from_secs(900));
}

#[test]
fn a_run_that_did_not_answer_is_tried_once_more() {
    let dir = workdir();
    let calls = std::sync::atomic::AtomicUsize::new(0);

    let run = one_run(dir.path(), move |c| {
        if calls.fetch_add(1, Ordering::SeqCst) == 0 {
            Err("cannot start".to_owned())
        } else {
            fake::specialist(c)
        }
    });

    assert!(run.answered(), "{:?}", run.failure);
    assert_eq!(run.attempts, 2);
}

#[test]
fn a_run_that_fails_twice_fails_after_two_attempts_and_keeps_both_costs() {
    let dir = workdir();

    let run = one_run(dir.path(), |c| {
        fake::write_log(c, &fake::log("  ", 0.25));
        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    });

    assert!(!run.answered());
    assert_eq!(run.attempts, 2);
    assert!((run.log.cost - 0.5).abs() < 1e-9, "{}", run.log.cost);
}

#[test]
fn a_run_that_answers_is_tried_once() {
    let dir = workdir();

    let run = one_run(dir.path(), fake::specialist);

    assert!(run.answered());
    assert_eq!(run.attempts, 1);
}

#[test]
fn a_definition_that_does_not_read_is_not_tried_again() {
    let dir = TempDir::new().unwrap();
    let host = FakeHost::new(fake::either);

    let runs = review_with(
        &host,
        dir.path(),
        &plan(r#"{"review-code": ["lang-rust"]}"#),
    );

    assert!(runs.iter().all(|r| r.attempts == 1));
    assert!(host.agents().is_empty());
}

/// Runs the one-specialist plan with `claude` answering as `script`,
/// and returns its run and the progress lines it printed.
fn one_run_told(
    dir: &Path,
    script: impl Fn(&Command) -> Result<Output, String> + Sync + 'static,
) -> (AgentRun, Vec<String>) {
    let host = FakeHost::new(script);
    let lines = Mutex::new(Vec::new());
    let runs = review(
        &host,
        dir,
        &plan(r#"{"review-code": ["lang-rust"]}"#),
        options(),
        &|l| lines.lock().unwrap().push(l.to_owned()),
    )
    .unwrap();
    (
        runs.into_iter().next().unwrap(),
        lines.into_inner().unwrap(),
    )
}

/// Fails the first run with a log reading `first`, then answers.
fn fails_once(c: &Command, calls: &AtomicUsize) -> Result<Output, String> {
    if calls.fetch_add(1, Ordering::SeqCst) == 0 {
        fake::write_log(c, "first\n");
        let errors = c.stdout_to.as_ref().unwrap().with_extension("stderr");
        fs::write(errors, "first error\n").unwrap();
        Ok(Output::default())
    } else {
        fake::specialist(c)
    }
}

#[test]
fn a_first_run_that_did_not_answer_is_told_before_the_second() {
    let dir = workdir();
    let calls = AtomicUsize::new(0);

    let (_, lines) = one_run_told(dir.path(), move |c| fails_once(c, &calls));

    assert_eq!(
        lines[..2],
        [
            "lang-rust (specialist): did not answer: its run ended with none; running it once more",
            "lang-rust (specialist): answered (0.0100 USD, 3 turns) on its second attempt",
        ]
    );
}

#[test]
fn a_first_run_that_did_not_answer_keeps_its_log_beside_the_second_s() {
    let dir = workdir();
    let calls = AtomicUsize::new(0);

    one_run_told(dir.path(), move |c| fails_once(c, &calls));

    let logs = dir.path().join("review/logs");
    assert_eq!(
        fs::read_to_string(logs.join("lang-rust.first.jsonl")).unwrap(),
        "first\n"
    );
    assert_eq!(
        fs::read_to_string(logs.join("lang-rust.first.stderr")).unwrap(),
        "first error\n"
    );
    assert!(
        fs::read_to_string(logs.join("lang-rust.jsonl"))
            .unwrap()
            .contains("\"result\"")
    );
}

#[test]
fn a_first_log_that_cannot_be_kept_is_told() {
    let dir = workdir();
    let calls = AtomicUsize::new(0);
    fs::create_dir_all(dir.path().join("review/logs/lang-rust.first.jsonl/x")).unwrap();

    let (run, lines) = one_run_told(dir.path(), move |c| fails_once(c, &calls));

    assert!(run.answered());
    assert!(
        lines[1].starts_with("lang-rust (specialist): its first log was not kept: rename "),
        "{lines:?}"
    );
}

#[test]
fn a_stale_file_is_removed_and_a_missing_one_is_no_error() {
    let dir = TempDir::new().unwrap();
    let stale = dir.path().join("a.md");
    fs::write(&stale, "x").unwrap();

    assert_eq!(remove_stale(&stale), Ok(()));
    assert!(!stale.exists());
    assert_eq!(remove_stale(&stale), Ok(()));
}

#[test]
fn a_stale_path_that_cannot_be_removed_is_told() {
    let dir = TempDir::new().unwrap();
    let at = dir.path().join("a.md");
    fs::create_dir(&at).unwrap();

    let err = remove_stale(&at).unwrap_err();
    assert!(
        err.starts_with(&format!("remove {}: ", at.display())),
        "{err}"
    );
}

#[test]
fn a_first_log_and_its_errors_move_aside_and_missing_ones_are_no_error() {
    let dir = TempDir::new().unwrap();
    let logs = dir.path().join(LOGS);
    fs::create_dir_all(&logs).unwrap();
    fs::write(logs.join("lang-rust.jsonl"), "log").unwrap();

    assert_eq!(keep_first_log(dir.path(), "lang-rust"), Ok(()));
    assert_eq!(
        fs::read_to_string(logs.join("lang-rust.first.jsonl")).unwrap(),
        "log"
    );
    assert!(!logs.join("lang-rust.jsonl").exists());
    assert!(!logs.join("lang-rust.first.stderr").exists());
    assert_eq!(keep_first_log(dir.path(), "lang-markdown"), Ok(()));
}
