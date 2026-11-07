use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

use review_eval::host::{Command, Output};
use review_eval::runner::TOOLS;
use testkit::TempDir;

use super::*;
use crate::fake::{self, FakeHost, agent_of, prompt_of};
use crate::plan::parse_dispatch;

mod controls;
mod resume;

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

/// The ids a report may cite: the invariants.
pub(crate) fn known() -> std::collections::BTreeSet<String> {
    review_gate::report::invariants()
}

fn options() -> Options {
    Options {
        parallel: 8,
        deadline: Duration::from_secs(77),
        split: crate::parts::LIMIT,
        // Each of these runs tests one run, so no failure trips the breaker.
        breaker: Fraction::WHOLE,
        ..Options::default()
    }
}

fn arg_after<'a>(command: &'a Command, flag: &str) -> &'a str {
    let at = command.args.iter().position(|a| a == flag).unwrap();
    &command.args[at + 1]
}

fn review_with(host: &FakeHost, dir: &Path, p: &Plan) -> Vec<AgentRun> {
    review(host, dir, p, &known(), options(), None, &|_| {}).unwrap()
}

#[test]
fn each_generalist_runs_after_its_own_specialists_and_the_runs_come_in_plan_order() {
    let dir = workdir();
    let events = std::sync::Arc::new(Mutex::new(Vec::new()));
    let seen = std::sync::Arc::clone(&events);
    let host = FakeHost::new(move |c| {
        let agent = agent_of(c);
        seen.lock().unwrap().push(format!("start {agent}"));
        let out = fake::either(c);
        seen.lock().unwrap().push(format!("end {agent}"));
        out
    });

    let runs = review_with(&host, dir.path(), &plan(DISPATCH));

    let events = events.lock().unwrap().clone();
    let at = |e: &str| events.iter().position(|x| x == e).unwrap();
    for (generalist, specialists) in [
        ("review-code", &["lang-rust", "test-engineer-unit"][..]),
        ("review-spec", &["lang-markdown"][..]),
    ] {
        for specialist in specialists {
            assert!(
                at(&format!("end {specialist}")) < at(&format!("start {generalist}")),
                "{events:?}"
            );
        }
    }
    assert_eq!(host.agents().len(), 5);
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
        "outcome: findings — lang-markdown saw one finding."
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
    assert_eq!(arg_after(rust, "--allowedTools"), TOOLS);
    assert_eq!(arg_after(rust, "--max-turns"), "20");
    assert_eq!(arg_after(rust, "--output-format"), "stream-json");
    assert!(!rust.args.iter().any(|a| a == "--model"));
    let prompt = prompt_of(rust);
    assert!(prompt.contains("`review/diffs/lang-rust.diff`"), "{prompt}");
    assert!(prompt.contains("You write no file"), "{prompt}");
    assert!(!prompt.contains("review/reports"), "{prompt}");
}

#[test]
fn a_specialist_s_report_is_assembled_from_its_log_and_written_for_its_generalist() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    let runs = review_with(&host, dir.path(), &plan(DISPATCH));

    assert!(runs.iter().all(AgentRun::answered));
    let report = fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap();
    assert_eq!(
        report,
        format!(
            "{}\noutcome: findings — lang-rust saw one finding.\n",
            fake::FINDING
        )
    );
    assert!(dir.path().join("review/reports/lang-markdown.md").is_file());
    assert!(!dir.path().join("review/reports/review-code.md").exists());
}

#[test]
fn a_run_cut_off_before_its_answer_keeps_what_it_stated_as_an_incomplete_report() {
    let dir = workdir();

    let run = one_run(dir.path(), |c| {
        fake::write_log(
            c,
            &fake::log_of(
                &[
                    "Reading.",
                    fake::FINDING,
                    "a.rs:9 — not a form",
                    "Checking more.",
                ],
                "error_max_turns",
                "",
                0.02,
            ),
        );
        Ok(Output::default())
    });

    assert_eq!(
        run.failure.as_deref(),
        Some("its run ended with error_max_turns")
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap(),
        format!(
            "incomplete: its run ended with error_max_turns\n{}\na.rs:9 — not a form\n",
            fake::FINDING
        )
    );
}

#[test]
fn a_run_that_cannot_start_leaves_an_incomplete_report_saying_so() {
    let dir = workdir();

    let run = one_run(dir.path(), |_| Err("cannot start".to_owned()));

    assert!(!run.answered());
    assert_eq!(
        fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap(),
        "incomplete: cannot start\n"
    );
}

#[test]
fn a_report_that_cannot_be_written_fails_the_run_closed() {
    let dir = workdir();
    fs::create_dir_all(dir.path().join("review/reports/lang-rust.md")).unwrap();

    let run = one_run(dir.path(), fake::specialist);

    let failure = run.failure.clone().unwrap();
    assert!(
        failure.starts_with("its report could not be written: "),
        "{failure}"
    );
    // The run was paid for, so it is not run again for a report that
    // could not be written.
    assert_eq!(run.attempts, 1);

    let run = one_run(dir.path(), |_| Err("cannot start".to_owned()));
    let failure = run.failure.unwrap();
    assert!(
        failure.starts_with("cannot start; its report could not be written: "),
        "{failure}"
    );
}

#[test]
fn a_generalist_s_run_writes_no_report() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    review_with(&host, dir.path(), &plan(DISPATCH));

    let reports: Vec<String> = fs::read_dir(dir.path().join("review/reports"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert!(
        !reports.iter().any(|r| r.starts_with("review-")),
        "{reports:?}"
    );
    assert_eq!(reports.len(), 3);
}

#[test]
fn a_generalist_is_told_its_specialists_report_files_and_reads_only() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    review_with(&host, dir.path(), &plan(DISPATCH));

    let calls = host.calls.lock().unwrap().clone();
    let code = calls.iter().find(|c| agent_of(c) == "review-code").unwrap();
    assert_eq!(arg_after(code, "--allowedTools"), TOOLS);
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

    review(
        &host,
        dir.path(),
        &plan(DISPATCH),
        &known(),
        options(),
        None,
        &|l| {
            lines.lock().unwrap().push(l.to_owned());
        },
    )
    .unwrap();

    let lines = lines.into_inner().unwrap();
    assert_eq!(lines.len(), 5);
    assert!(
        lines.contains(&"lang-rust (specialist): answered (0.0100 USD, 3 turns)".to_owned()),
        "{lines:?}"
    );
}

/// A meeting of two runs, again and again, that a run waits at for its
/// partner within a deadline: a barrier of two that cannot hang a test.
#[derive(Default)]
struct Meeting {
    /// The runs waiting, and how many meetings have closed.
    state: Mutex<(usize, u64)>,
    met: Condvar,
}

impl Meeting {
    /// Waits for a second run within `deadline`; whether one came.
    fn wait(&self, deadline: Duration) -> bool {
        let mut state = self.state.lock().unwrap();
        let meeting = state.1;
        state.0 += 1;
        if state.0 == 2 {
            *state = (0, meeting + 1);
            self.met.notify_all();
            return true;
        }
        let (state, waited) = self
            .met
            .wait_timeout_while(state, deadline, |s| s.1 == meeting)
            .unwrap();
        drop(state);
        !waited.timed_out()
    }
}

#[test]
fn a_meeting_no_second_run_comes_to_ends_at_its_deadline() {
    assert!(!Meeting::default().wait(Duration::from_millis(10)));
}

#[test]
fn runs_stay_within_the_bound_and_use_it() {
    let active = AtomicUsize::new(0);
    let most = AtomicUsize::new(0);
    let alone = AtomicUsize::new(0);
    let waits: Vec<Vec<usize>> = (0..6).map(|_| Vec::new()).collect();

    // Each run waits for a second one to be running, so two overlap
    // without a clock; a third would raise the count past the bound. A
    // run left alone past the deadline is counted, never waited on.
    let both = Meeting::default();

    let out = crate::pipeline::pipeline(&waits, 2, &|_| true, &|n, _: Vec<usize>| {
        let now = active.fetch_add(1, Ordering::SeqCst) + 1;
        most.fetch_max(now, Ordering::SeqCst);
        let met = both.wait(Duration::from_secs(30));
        alone.fetch_add(usize::from(!met), Ordering::SeqCst);
        active.fetch_sub(1, Ordering::SeqCst);
        n * 10
    });

    assert_eq!(out, [0, 10, 20, 30, 40, 50]);
    assert_eq!(most.load(Ordering::SeqCst), 2);
    assert_eq!(alone.load(Ordering::SeqCst), 0);
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
            Box::new(|c| fake::answer(c, &[], "  \n")),
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

    assert_eq!(run.attempts, 0);
    assert!(
        run.failure
            .unwrap()
            .starts_with("its definition does not read: read .claude/agents/lang-rust.md")
    );
}

#[test]
fn an_agent_without_a_definition_leaves_no_report_from_before() {
    let dir = TempDir::new().unwrap();
    let report = dir.path().join(report_file("lang-rust"));
    fs::create_dir_all(report.parent().unwrap()).unwrap();
    fs::write(&report, "outcome: none — stale.\n").unwrap();

    let run = one_run(dir.path(), fake::either);

    assert!(!report.exists());
    assert!(!run.failure.unwrap().contains("could not be removed"));

    fs::create_dir_all(report.join("held")).unwrap();
    let run = one_run(dir.path(), fake::either);
    let failure = run.failure.unwrap();
    assert!(
        failure.starts_with("its definition does not read: ")
            && failure.contains("; its report from before could not be removed: "),
        "{failure}"
    );
}

#[test]
fn a_report_from_before_the_run_is_written_over() {
    let dir = workdir();
    let report = dir.path().join("review/reports/lang-rust.md");
    fs::create_dir_all(report.parent().unwrap()).unwrap();
    fs::write(
        &report,
        "a.rs:1 — if not addressed: stale — I2\noutcome: findings — stale.\n",
    )
    .unwrap();

    let run = one_run(dir.path(), |c| fake::answer(c, &[], "outcome: none — x."));

    assert!(run.answered());
    assert_eq!(fs::read_to_string(&report).unwrap(), "outcome: none — x.\n");
}

#[test]
fn a_work_directory_without_room_for_logs_fails_the_review() {
    let dir = workdir();
    fs::write(dir.path().join("review"), "a file").unwrap();
    let host = FakeHost::new(fake::either);

    let err = review(
        &host,
        dir.path(),
        &plan(DISPATCH),
        &known(),
        options(),
        None,
        &|_| {},
    )
    .unwrap_err();

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
fn a_run_that_fails_after_it_cost_money_is_tried_once_and_keeps_its_cost() {
    let dir = workdir();

    let run = one_run(dir.path(), |c| {
        fake::write_log(c, &fake::log("  ", 0.25));
        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    });

    assert!(!run.answered());
    assert_eq!(run.attempts, 1);
    assert!((run.log.cost - 0.25).abs() < 1e-9, "{}", run.log.cost);
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

    assert!(runs.iter().all(|r| r.attempts == 0));
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
        &known(),
        options(),
        None,
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
fn a_first_log_and_its_errors_move_aside_and_missing_ones_are_no_error() {
    let dir = TempDir::new().unwrap();
    let logs = dir.path().join(LOGS);
    fs::create_dir_all(&logs).unwrap();
    fs::write(logs.join("lang-rust.jsonl"), "log").unwrap();

    assert_eq!(keep_log(dir.path(), "lang-rust", "first"), Ok(()));
    assert_eq!(
        fs::read_to_string(logs.join("lang-rust.first.jsonl")).unwrap(),
        "log"
    );
    assert!(!logs.join("lang-rust.jsonl").exists());
    assert!(!logs.join("lang-rust.first.stderr").exists());
    assert_eq!(keep_log(dir.path(), "lang-markdown", "first"), Ok(()));
}

/// A file's section of a diff, `size` bytes long, for `path`.
fn section(path: &str, size: usize) -> String {
    let head = format!("diff --git a/{path} b/{path}\n");
    let body = "+".repeat(size - head.len() - 1);
    format!("{head}{body}\n")
}

/// Writes lang-rust's diff, three files of which two fit a part of
/// [`SPLIT`] bytes, so it runs in two parts.
fn big_diff(dir: &Path) {
    let diffs = dir.join("review/diffs");
    fs::create_dir_all(&diffs).unwrap();
    let diff = [
        section("a.rs", 60),
        section("b.rs", 30),
        section("c.rs", 60),
    ]
    .concat();
    fs::write(diffs.join("lang-rust.diff"), diff).unwrap();
}

const SPLIT: usize = 100;

/// Runs the one-specialist plan, its split at [`SPLIT`], with `claude`
/// answering as `script`: its runs, the progress lines and the host.
fn split_run(
    dir: &Path,
    script: impl Fn(&Command) -> Result<Output, String> + Sync + 'static,
) -> (Vec<AgentRun>, Vec<String>, FakeHost) {
    let host = FakeHost::new(script);
    let lines = Mutex::new(Vec::new());
    let runs = review(
        &host,
        dir,
        &plan(r#"{"review-code": ["lang-rust"]}"#),
        &known(),
        Options {
            split: SPLIT,
            ..options()
        },
        None,
        &|l| lines.lock().unwrap().push(l.to_owned()),
    )
    .unwrap();
    (runs, lines.into_inner().unwrap(), host)
}

/// A specialist part that states a finding on the part it is told,
/// `src/part2.rs` for part 2, and one every part states.
fn part_specialist(c: &Command) -> Result<Output, String> {
    let prompt = prompt_of(c);
    if !prompt.contains("a specialist") {
        return fake::generalist(c);
    }
    let part = if prompt.contains("you are part 2 of 2") {
        2
    } else {
        1
    };
    let own = format!("src/part{part}.rs:1 — if not addressed: part {part} breaks — I9");
    let outcome = format!("outcome: findings — part {part} saw two.");
    fake::answer(c, &[fake::FINDING, &own, &outcome], &outcome)
}

#[test]
fn a_specialist_whose_diff_is_past_the_split_runs_in_parts_and_its_reports_merge() {
    let dir = workdir();
    big_diff(dir.path());

    let (runs, lines, host) = split_run(dir.path(), part_specialist);

    assert_eq!(host.agents(), ["lang-rust", "lang-rust", "review-code"]);
    let calls = host.calls.lock().unwrap().clone();
    let mut prompts: Vec<String> = calls[..2].iter().map(prompt_of).collect();
    prompts.sort();
    for (i, prompt) in prompts.iter().enumerate() {
        assert!(
            prompt.contains(&format!(
                "you are part {} of 2. Read only your part's diff, `review/diffs/lang-rust.{}.diff`",
                i + 1,
                i + 1
            )),
            "{prompt}"
        );
        assert!(
            prompt.contains("by part: part 1: a.rs, b.rs; part 2: c.rs."),
            "{prompt}"
        );
    }
    let mut logs: Vec<_> = calls[..2]
        .iter()
        .map(|c| c.stdout_to.clone().unwrap())
        .collect();
    logs.sort();
    assert_eq!(
        logs,
        [
            dir.path().join("review/logs/lang-rust.1.jsonl"),
            dir.path().join("review/logs/lang-rust.2.jsonl")
        ]
    );
    assert_eq!(runs.len(), 2);
    let rust = &runs[0];
    assert_eq!(rust.name, "lang-rust");
    assert!(rust.answered());
    assert_eq!((rust.parts, rust.attempts), (2, 2));
    assert_eq!(
        rust.log.answer,
        "outcome: findings — in 2 parts: part 1, findings — part 1 saw two.; part 2, \
         findings — part 2 saw two."
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap(),
        format!(
            "{}\nsrc/part1.rs:1 — if not addressed: part 1 breaks — I9\n\
             src/part2.rs:1 — if not addressed: part 2 breaks — I9\n{}\n",
            fake::FINDING,
            rust.log.answer
        )
    );
    assert!(
        lines.contains(
            &"lang-rust (specialist): answered (0.0200 USD, 6 turns) in 2 parts, 2 attempts"
                .to_owned()
        ),
        "{lines:?}"
    );
    assert!(
        lines.contains(
            &"lang-rust part 2 of 2 (specialist): answered (0.0100 USD, 3 turns)".to_owned()
        ),
        "{lines:?}"
    );
    assert!(!dir.path().join("review/reports/lang-rust.1.md").exists());
}

#[test]
fn a_part_that_does_not_answer_keeps_what_it_stated_and_fails_the_specialist_naming_it() {
    let dir = workdir();
    big_diff(dir.path());

    let (runs, lines, host) = split_run(dir.path(), |c| {
        if prompt_of(c).contains("you are part 2 of 2") {
            fake::write_log(
                c,
                &fake::log_of(
                    &["b.rs:2 — if not addressed: y — I2"],
                    "error_max_turns",
                    "",
                    0.1,
                ),
            );
            Ok(Output::default())
        } else {
            part_specialist(c)
        }
    });

    assert_eq!(host.agents().len(), 3);
    let rust = &runs[0];
    assert_eq!(
        rust.failure.as_deref(),
        Some("part 2 of 2: its run ended with error_max_turns")
    );
    assert_eq!((rust.parts, rust.attempts), (2, 2));
    assert_eq!(
        fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap(),
        format!(
            "incomplete: part 2 of 2: its run ended with error_max_turns\n{}\n\
             src/part1.rs:1 — if not addressed: part 1 breaks — I9\n\
             b.rs:2 — if not addressed: y — I2\n",
            fake::FINDING
        )
    );
    assert!(
        lines.iter().any(|l| l.starts_with(
            "lang-rust part 2 of 2 (specialist): did not answer: its run ended with \
             error_max_turns; not run again: it cost 0.1000 USD"
        )),
        "{lines:?}"
    );
}

#[test]
fn a_diff_that_cannot_be_split_fails_its_specialist_without_running_it() {
    let dir = workdir();
    big_diff(dir.path());
    fs::create_dir_all(dir.path().join("review/diffs/lang-rust.1.diff")).unwrap();

    let (runs, _, host) = split_run(dir.path(), fake::either);

    assert_eq!(host.agents(), ["review-code"]);
    let failure = runs[0].failure.clone().unwrap();
    assert!(
        failure.starts_with("its diff could not be split: write "),
        "{failure}"
    );
    assert_eq!((runs[0].attempts, runs[0].parts), (0, 0));
    assert!(
        fs::read_to_string(dir.path().join("review/reports/lang-rust.md"))
            .unwrap()
            .starts_with("incomplete: its diff could not be split: write ")
    );

    fs::remove_file(dir.path().join("review/reports/lang-rust.md")).unwrap();
    fs::create_dir_all(dir.path().join("review/reports/lang-rust.md")).unwrap();
    let (runs, _, _) = split_run(dir.path(), fake::either);
    let failure = runs[0].failure.clone().unwrap();
    assert!(
        failure.contains("; its report could not be written: "),
        "{failure}"
    );
}

#[test]
fn a_merged_report_that_cannot_be_written_fails_the_specialist() {
    let dir = workdir();
    big_diff(dir.path());
    fs::create_dir_all(dir.path().join("review/reports/lang-rust.md")).unwrap();

    let (runs, _, _) = split_run(dir.path(), part_specialist);
    let failure = runs[0].failure.clone().unwrap();
    assert!(
        failure.starts_with("its report could not be written: "),
        "{failure}"
    );

    let (runs, _, _) = split_run(dir.path(), |_| Err("cannot start".to_owned()));
    let failure = runs[0].failure.clone().unwrap();
    assert!(
        failure.starts_with("part 1 of 2: cannot start; part 2 of 2: cannot start; its report"),
        "{failure}"
    );
}

#[test]
fn a_part_without_a_definition_fails_its_specialist() {
    let dir = workdir();
    big_diff(dir.path());
    fs::remove_file(dir.path().join(".claude/agents/lang-rust.md")).unwrap();

    let (runs, _, host) = split_run(dir.path(), fake::either);

    assert_eq!(host.agents(), ["review-code"]);
    assert_eq!((runs[0].parts, runs[0].attempts), (2, 0));
    let failure = runs[0].failure.clone().unwrap();
    assert!(
        failure.starts_with("part 1 of 2: its definition does not read: "),
        "{failure}"
    );
    assert!(
        fs::read_to_string(dir.path().join("review/reports/lang-rust.md"))
            .unwrap()
            .starts_with("incomplete: part 1 of 2: its definition does not read: ")
    );
}

#[test]
fn a_note_is_added_to_the_prompt_and_args_without_one_are_left_alone() {
    let mut args = vec!["-p".to_owned(), "Read.".to_owned(), "--verbose".to_owned()];
    told(&mut args, "Part 1.");
    assert_eq!(args, ["-p", "Read. Part 1.", "--verbose"]);

    let mut args = vec!["--verbose".to_owned()];
    told(&mut args, "Part 1.");
    assert_eq!(args, ["--verbose"]);
    let mut args = vec!["-p".to_owned()];
    told(&mut args, "Part 1.");
    assert_eq!(args, ["-p"]);
}
