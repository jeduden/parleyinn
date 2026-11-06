//! The runs under the review's controls: the cost caps, the breaker,
//! the generalists started as soon as their own specialists ended, the
//! waits after an overloaded API, the failures never run again, and the
//! progress told while the agents run.

use std::sync::{Arc, Condvar};

use super::*;
use crate::fake::FakeClock;

/// What one review under the controls gave: its runs, the progress
/// lines, the host and the clock.
struct Ran {
    runs: Vec<AgentRun>,
    lines: Vec<String>,
    host: FakeHost,
    clock: FakeClock,
}

impl Ran {
    fn run(&self, name: &str) -> &AgentRun {
        self.runs.iter().find(|r| r.name == name).unwrap()
    }
}

/// Reviews `dispatch` under `options` on a fake clock drawing `jitter`,
/// `claude` answering as `script`.
fn controlled(
    dispatch: &str,
    options: Options,
    jitter: f64,
    script: impl Fn(&Command) -> Result<Output, String> + Sync + 'static,
) -> (TempDir, Ran) {
    told_to(Arc::default(), dispatch, options, jitter, script)
}

/// [`controlled`], the lines progress is told kept in `lines`, which
/// the script may read while the review runs.
fn told_to(
    lines: Arc<Mutex<Vec<String>>>,
    dispatch: &str,
    options: Options,
    jitter: f64,
    script: impl Fn(&Command) -> Result<Output, String> + Sync + 'static,
) -> (TempDir, Ran) {
    let dir = workdir();
    let host = FakeHost::new(script);
    let clock = FakeClock {
        jitter,
        ..FakeClock::default()
    };
    let runs = review_on(&Review {
        host: &host,
        clock: &clock,
        workdir: dir.path(),
        plan: &plan(dispatch),
        known: &known(),
        options,
        resume: None,
        keys: None,
        store: None,
        scrubber: None,
        progress: &|l| lines.lock().unwrap().push(l.to_owned()),
    })
    .unwrap();
    let lines = lines.lock().unwrap().clone();
    (
        dir,
        Ran {
            runs,
            lines,
            host,
            clock,
        },
    )
}

const ONE: &str = r#"{"review-code": ["lang-rust"]}"#;

/// A result the harness reports for an API that turned the run away:
/// a successful result that is an error, saying so.
const OVERLOADED: &str = r#"{"type":"result","subtype":"success","is_error":true,"result":"API Error: 529 {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}","total_cost_usd":0,"num_turns":1}"#;

/// Answers as [`fake::either`] after `fails` runs answered `failed`.
fn after(
    fails: usize,
    failed: &'static str,
) -> impl Fn(&Command) -> Result<Output, String> + Sync + 'static {
    let calls = AtomicUsize::new(0);
    move |c| {
        if prompt_of(c).contains("a specialist") && calls.fetch_add(1, Ordering::SeqCst) < fails {
            fake::write_log(c, &format!("{failed}\n"));
            return Ok(Output::default());
        }
        fake::either(c)
    }
}

#[test]
fn a_generalist_starts_while_an_unrelated_specialist_still_runs() {
    // lang-markdown is review-spec's; lang-rust, review-code's, ends
    // only once review-spec has started, or after half a minute.
    let started = Arc::new((Mutex::new(false), Condvar::new()));
    let seen = Arc::clone(&started);
    let (_dir, ran) = controlled(DISPATCH, options(), 0.0, move |c| {
        let agent = agent_of(c);
        if agent == "review-spec" {
            *seen.0.lock().unwrap() = true;
            seen.1.notify_all();
        }
        if agent == "lang-rust" {
            let flag = seen.0.lock().unwrap();
            let (flag, waited) = seen
                .1
                .wait_timeout_while(flag, Duration::from_secs(30), |s| !*s)
                .unwrap();
            assert!(
                *flag && !waited.timed_out(),
                "review-spec waited for lang-rust"
            );
        }
        fake::either(c)
    });

    assert!(ran.runs.iter().all(AgentRun::answered), "{:?}", ran.lines);
    assert!(*started.0.lock().unwrap());
}

#[test]
fn a_report_saying_a_run_was_not_started_that_cannot_be_written_is_named() {
    let dir = workdir();
    fs::create_dir_all(dir.path().join(report_file("lang-rust")).join("x")).unwrap();
    let host = FakeHost::new(fake::either);
    let options = Options {
        max_cost_usd: 0.0,
        ..options()
    };

    let runs = review(
        &host,
        dir.path(),
        &plan(ONE),
        &known(),
        options,
        None,
        &|_| {},
    )
    .unwrap();

    let failure = runs[0].failure.as_deref().unwrap();
    assert!(
        failure.starts_with("over budget: ")
            && failure.contains("; its report could not be written: "),
        "{failure}"
    );
    assert!(host.agents().is_empty());
}

#[test]
fn no_run_starts_once_the_finished_runs_cost_the_review_s_cap() {
    let options = Options {
        parallel: 1,
        max_cost_usd: 0.015,
        ..options()
    };

    let (dir, ran) = controlled(DISPATCH, options, 0.0, fake::either);

    // Each run costs 0.01: two finish, and nothing after them starts.
    assert_eq!(ran.host.agents(), ["lang-markdown", "lang-rust"]);
    let refused = "over budget: the review's runs cost 0.0200 USD, its cap is 0.01 USD, so it \
                   was not started";
    for name in ["test-engineer-unit", "review-code", "review-spec"] {
        assert_eq!(ran.run(name).failure.as_deref(), Some(refused), "{name}");
        assert_eq!(ran.run(name).attempts, 0);
    }
    let report = fs::read_to_string(dir.path().join(report_file("test-engineer-unit"))).unwrap();
    assert!(report.contains(refused), "{report}");
    assert!(
        ran.lines.contains(&format!(
            "review-code (generalist): did not answer: {refused}"
        )),
        "{:?}",
        ran.lines
    );
}

#[test]
fn a_run_that_cost_money_is_not_run_again_whether_or_not_a_cap_is_reached() {
    let options = Options {
        max_cost_usd: 0.03,
        ..options()
    };
    let costly = r#"{"type":"result","subtype":"error_max_turns","is_error":true,"result":"","total_cost_usd":0.05,"num_turns":20}"#;

    let (_dir, ran) = controlled(ONE, options, 0.0, after(1, costly));

    let run = ran.run("lang-rust");
    assert_eq!(
        run.failure.as_deref(),
        Some("its run ended with error_max_turns")
    );
    assert_eq!(run.attempts, 1);
    assert_eq!(ran.host.agents(), ["lang-rust"]);
}

#[test]
fn a_run_its_own_cap_stopped_is_never_run_again() {
    let capped = r#"{"type":"result","subtype":"error_max_budget_usd","is_error":true,"result":"","total_cost_usd":1.01,"num_turns":9}"#;

    let (_dir, ran) = controlled(ONE, options(), 0.0, after(1, capped));

    let run = ran.run("lang-rust");
    assert_eq!(
        run.failure.as_deref(),
        Some("its run ended with error_max_budget_usd")
    );
    assert_eq!(run.attempts, 1);
    assert!(
        ran.lines.contains(
            &"lang-rust (specialist): did not answer: its run ended with error_max_budget_usd; \
              not run again: a cost cap stopped it, and another run would cost more"
                .to_owned()
        ),
        "{:?}",
        ran.lines
    );
}

#[test]
fn a_run_whose_credential_was_refused_is_never_run_again() {
    let (_dir, ran) = controlled(ONE, options(), 0.0, |c| {
        if prompt_of(c).contains("a specialist") {
            fake::write_log(c, "");
            let errors = c.stdout_to.as_ref().unwrap().with_extension("stderr");
            fs::write(errors, "Invalid API key · Please run /login\n").unwrap();
            return Ok(Output::default());
        }
        fake::either(c)
    });

    assert_eq!(ran.run("lang-rust").attempts, 1);
    assert_eq!(ran.host.agents(), ["lang-rust", "review-code"]);
    assert!(
        ran.lines.iter().any(|l| l.ends_with(
            "not run again: its credential was refused, and no run with it can succeed"
        )),
        "{:?}",
        ran.lines
    );
}

#[test]
fn an_overloaded_run_waits_with_jitter_and_runs_once_more() {
    let (_dir, ran) = controlled(ONE, options(), 0.5, after(1, OVERLOADED));

    let run = ran.run("lang-rust");
    assert!(run.answered());
    assert_eq!(run.attempts, 2);
    assert_eq!(ran.clock.slept(), [Duration::from_millis(18_750)]);
    assert!(
        ran.lines[0].ends_with("; the API was overloaded, so it waits 18.8 s and runs once more"),
        "{:?}",
        ran.lines
    );
}

#[test]
fn a_run_overloaded_twice_waits_twice_as_long_the_second_time_and_keeps_both_logs() {
    let (dir, ran) = controlled(ONE, options(), 0.0, after(2, OVERLOADED));

    let run = ran.run("lang-rust");
    assert!(run.answered());
    assert_eq!(run.attempts, 3);
    assert_eq!(
        ran.clock.slept(),
        [Duration::from_secs(15), Duration::from_secs(30)]
    );
    let logs = dir.path().join(LOGS);
    for kept in ["lang-rust.first.jsonl", "lang-rust.second.jsonl"] {
        assert!(
            fs::read_to_string(logs.join(kept))
                .unwrap()
                .contains("overloaded_error"),
            "{kept}"
        );
    }
    assert!(
        ran.lines.contains(
            &"lang-rust (specialist): answered (0.0100 USD, 5 turns) on its third attempt"
                .to_owned()
        ),
        "{:?}",
        ran.lines
    );
}

#[test]
fn a_run_overloaded_three_times_ends_after_its_third_attempt() {
    let (_dir, ran) = controlled(ONE, options(), 0.0, after(9, OVERLOADED));

    let run = ran.run("lang-rust");
    assert!(!run.answered());
    assert_eq!(run.attempts, 3);
    assert_eq!(ran.clock.slept().len(), 2);
}

#[test]
fn a_run_overloaded_after_another_failure_waits_before_its_third_attempt() {
    let calls = AtomicUsize::new(0);
    let (_dir, ran) = controlled(ONE, options(), 0.0, move |c| {
        if !prompt_of(c).contains("a specialist") {
            return fake::either(c);
        }
        match calls.fetch_add(1, Ordering::SeqCst) {
            0 => fake::write_log(c, "cut\n"),
            1 => fake::write_log(c, &format!("{OVERLOADED}\n")),
            _ => return fake::either(c),
        }
        Ok(Output::default())
    });

    assert_eq!(ran.run("lang-rust").attempts, 3);
    assert_eq!(ran.clock.slept(), [Duration::from_secs(15)]);
    assert!(
        ran.lines[0].ends_with("; running it once more"),
        "{:?}",
        ran.lines
    );
}

#[test]
fn an_overloaded_run_past_its_deadline_is_not_run_again_and_stays_a_named_gap() {
    let options = Options {
        deadline: Duration::from_secs(5),
        ..options()
    };
    let clock = Arc::new(AtomicUsize::new(0));
    let (_dir, ran) = controlled(ONE, options, 0.0, {
        let calls = Arc::clone(&clock);
        move |c| {
            if prompt_of(c).contains("a specialist") && calls.fetch_add(1, Ordering::SeqCst) == 0 {
                fake::write_log(c, &format!("{OVERLOADED}\n"));
                return Ok(Output::default());
            }
            fake::either(c)
        }
    });
    // The fake clock stands still, so the deadline is left whole: this
    // run waits five seconds, the deadline's rest.
    assert_eq!(ran.clock.slept(), [Duration::from_secs(5)]);

    let options = Options {
        deadline: Duration::ZERO,
        ..options
    };
    let (_dir, ran) = controlled(ONE, options, 0.0, after(1, OVERLOADED));

    assert!(ran.clock.slept().is_empty());
    let agents = ran.host.agents();
    assert_eq!(
        agents.iter().filter(|a| *a == "lang-rust").count(),
        1,
        "{agents:?}"
    );
    assert_eq!(ran.run("lang-rust").attempts, 1);
    assert!(
        ran.lines[0].ends_with(
            "; the API was overloaded and its backoff deadline is spent, so it is not run again"
        ),
        "{:?}",
        ran.lines
    );
    let failure = ran.run("lang-rust").failure.clone().unwrap_or_default();
    assert!(
        failure.ends_with(
            "; the API was overloaded and its backoff deadline is spent, so it was not run again"
        ),
        "{failure}"
    );
}

#[test]
fn the_breaker_starts_no_generalist_once_more_than_its_fraction_of_specialists_failed() {
    let options = Options {
        breaker: BREAKER_FRACTION,
        ..options()
    };
    let (_dir, ran) = controlled(DISPATCH, options, 0.0, |c| {
        if ["lang-rust", "lang-markdown"].contains(&agent_of(c).as_str()) {
            return Err("claude: not found".to_owned());
        }
        fake::either(c)
    });

    let mut agents = ran.host.agents();
    agents.sort();
    agents.dedup();
    assert_eq!(agents, ["lang-markdown", "lang-rust", "test-engineer-unit"]);
    let aborted = "aborted: 2 of 3 specialists failed (mass failure; likely an outage or a \
                   broken prompt), so it was not started";
    for name in ["review-code", "review-spec"] {
        assert_eq!(ran.run(name).failure.as_deref(), Some(aborted), "{name}");
        assert_eq!(ran.run(name).attempts, 0);
    }
}

const CAPPED: &str = r#"{"type":"result","subtype":"error_max_budget_usd","is_error":true,"result":"","total_cost_usd":1.01,"num_turns":9}"#;

/// `DISPATCH` run with lang-rust's report refused by the gate and
/// lang-markdown, when `capped`, stopped by its cost cap.
fn refused_and_capped(capped: bool) -> Ran {
    let options = Options {
        breaker: BREAKER_FRACTION,
        ..options()
    };
    controlled(DISPATCH, options, 0.0, move |c| {
        match agent_of(c).as_str() {
            "lang-rust" => {
                let outcome = "outcome: none — clean.";
                fake::answer(c, &[fake::FINDING, outcome], outcome)
            }
            "lang-markdown" if capped => {
                fake::write_log(c, &format!("{CAPPED}\n"));
                Ok(Output::default())
            }
            _ => fake::either(c),
        }
    })
    .1
}

#[test]
fn one_refused_report_of_three_is_a_third_and_trips_nothing() {
    let ran = refused_and_capped(false);

    assert!(ran.run("review-code").answered(), "{:?}", ran.lines);
    assert!(ran.run("review-spec").answered(), "{:?}", ran.lines);
}

#[test]
fn a_refused_report_tips_the_breaker_past_a_third_beside_a_run_a_cap_stopped() {
    let ran = refused_and_capped(true);

    // The capped specialist counts as one that ended without an
    // accepted report, and the refused report is the second of three.
    let aborted = "aborted: 2 of 3 specialists failed (mass failure; likely an outage or a \
                   broken prompt), so it was not started";
    for generalist in ["review-code", "review-spec"] {
        let run = ran.run(generalist);
        assert!(!run.answered(), "{generalist}");
        assert_eq!(run.failure.as_deref(), Some(aborted), "{generalist}");
    }
}

#[test]
fn runs_a_cost_cap_stopped_tip_the_breaker_like_any_that_ended_without_a_report() {
    let options = Options {
        breaker: BREAKER_FRACTION,
        ..options()
    };
    let (_dir, ran) = controlled(DISPATCH, options, 0.0, |c| {
        if ["lang-rust", "lang-markdown"].contains(&agent_of(c).as_str()) {
            fake::write_log(c, &format!("{CAPPED}\n"));
            return Ok(Output::default());
        }
        fake::either(c)
    });

    assert!(!ran.run("review-code").answered());
    assert!(!ran.run("review-spec").answered());
}

#[test]
fn a_generalist_a_reached_cap_and_a_tripped_breaker_both_keep_back_is_named_over_budget() {
    let options = Options {
        breaker: BREAKER_FRACTION,
        max_cost_usd: 0.0,
        ..options()
    };
    let (_dir, ran) = controlled(DISPATCH, options, 0.0, fake::either);

    assert!(ran.host.agents().is_empty(), "nothing started");
    for generalist in ["review-code", "review-spec"] {
        let failure = ran.run(generalist).failure.clone().unwrap_or_default();
        assert!(failure.starts_with("over budget: "), "{failure}");
    }
}

#[test]
fn errors_that_cannot_be_read_are_told_and_taken_as_none() {
    let (_dir, ran) = controlled(ONE, options(), 0.0, |c| {
        if prompt_of(c).contains("a specialist") {
            let log = c.stdout_to.as_ref().unwrap();
            // A directory where the errors file belongs does not read.
            let _ = fs::create_dir(log.with_extension("stderr"));
            fake::write_log(c, "cut\n");
            return Ok(Output::default());
        }
        fake::either(c)
    });

    let told = ran
        .lines
        .iter()
        .find(|l| l.starts_with("lang-rust: read "))
        .unwrap_or_else(|| panic!("{:?}", ran.lines));
    assert!(
        told.contains("lang-rust.stderr: ") && told.ends_with("; its errors are taken as none"),
        "{told}"
    );
    // The run is run once more, as any failure no errors explain is.
    assert!(ran.run("lang-rust").attempts >= 2);
}

#[test]
fn what_is_running_is_told_every_so_often() {
    let options = Options {
        every: Duration::from_millis(1),
        ..options()
    };
    let lines = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&lines);
    let (_dir, ran) = told_to(lines, ONE, options, 0.0, move |c| {
        // The run ends once a block was told, or after half a minute.
        let give_up = std::time::Instant::now() + Duration::from_secs(30);
        while std::time::Instant::now() < give_up
            && !seen
                .lock()
                .unwrap()
                .iter()
                .any(|l| l.starts_with("progress: "))
        {
            std::thread::sleep(Duration::from_millis(1));
        }
        fake::either(c)
    });

    let block = ran
        .lines
        .iter()
        .find(|l| l.starts_with("progress: "))
        .unwrap();
    assert!(block.contains(" of 2 agents finished, "), "{block}");
}
