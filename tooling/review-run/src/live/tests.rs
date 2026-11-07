use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;
use std::time::Duration;

use review_eval::log::{RunLog, Usage};
use testkit::TempDir;

use super::*;
use crate::fake::{FakeClock, log_of};

fn running(log: PathBuf, since: u64) -> Running {
    Running {
        kind: Kind::Specialist,
        log,
        since: Duration::from_secs(since),
    }
}

#[test]
fn a_running_agent_s_row_reads_its_log_so_far() {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("lang-rust.jsonl");
    // A log cut mid-line, its result not yet written.
    let full = log_of(&["I read the diff."], "success", "outcome: none — x.", 0.01);
    fs::write(&log, &full[..full.find('\n').unwrap() + 5]).unwrap();
    let live = Live::default();
    live.start("lang-rust", running(log, 10));
    live.start("lang-shell", running(dir.path().join("none.jsonl"), 70));

    let rows = live.rows(Duration::from_secs(75));

    assert_eq!(
        rows,
        [
            Row {
                label: "lang-rust".to_owned(),
                kind: Kind::Specialist,
                ran: Duration::from_secs(65),
                calls: 1,
                input: 60,
                output: 5,
            },
            Row {
                label: "lang-shell".to_owned(),
                kind: Kind::Specialist,
                ran: Duration::from_secs(5),
                calls: 0,
                input: 0,
                output: 0,
            },
        ]
    );
    live.end("lang-rust");
    assert_eq!(live.rows(Duration::ZERO).len(), 1);
}

#[test]
fn the_block_counts_the_finished_the_cost_and_each_running_agent() {
    let live = Live::default();
    live.finish();
    live.finish();
    let rows = [Row {
        label: "review-code".to_owned(),
        kind: Kind::Generalist,
        ran: Duration::from_secs(125),
        calls: 7,
        input: 4200,
        output: 310,
    }];

    assert_eq!(
        block(&rows, live.finished(), 5, 0.123_45),
        "progress: 2 of 5 agents finished, 0.1235 USD so far, 1 running\n  \
         review-code (generalist), 2m05s: 7 calls, 4200 tokens in, 310 out"
    );
    assert_eq!(
        block(&[], 0, 3, 0.0),
        "progress: 0 of 3 agents finished, 0.0000 USD so far, 0 running"
    );
}

#[test]
fn a_duration_reads_as_minutes_and_seconds() {
    assert_eq!(clock(Duration::from_secs(0)), "0m00s");
    assert_eq!(clock(Duration::from_millis(61_900)), "1m01s");
    assert_eq!(clock(Duration::from_secs(3600)), "60m00s");
}

fn agent_run(name: &str, failure: Option<&str>, reused: bool) -> AgentRun {
    let mut usage = std::collections::BTreeMap::new();
    usage.insert(
        "claude-haiku-5-5".to_owned(),
        Usage {
            calls: 2,
            fresh: 1,
            cache_write: 2,
            cache_read: 3,
            output: 4,
            cost: 0.5,
        },
    );
    AgentRun {
        name: name.to_owned(),
        kind: review_eval::component::kind_of(name),
        failure: failure.map(str::to_owned),
        log: RunLog {
            cost: 0.5,
            turns: 3,
            usage,
            ..RunLog::default()
        },
        attempts: 2,
        parts: 1,
        failed_parts: Vec::new(),
        reused,
        from_store: false,
        answer_key: None,
        store_failed: false,
        charged: 0.0,
    }
}

#[test]
fn the_table_names_each_run_how_it_ended_and_what_it_took() {
    let long = format!("its run ended with x | y\n{}", "z".repeat(200));
    let runs = [
        agent_run("lang-rust", None, false),
        agent_run("lang-shell", Some(&long), false),
        agent_run("review-code", None, true),
        agent_run("review-spec", Some("its run ended with deadline"), false),
    ];

    let table = table(&runs);

    let lines: Vec<&str> = table.lines().collect();
    assert_eq!(lines[0], "### How each agent ran");
    assert_eq!(
        lines[2],
        "| agent | kind | result | attempts | turns | tokens in | out | cost |"
    );
    assert_eq!(
        lines[4],
        "| lang-rust | specialist | answered | 2 | 3 | 6 | 4 | 0.5000 |"
    );
    let cell = format!("its run ended with x   y {}…", "z".repeat(94));
    assert_eq!(
        lines[5],
        format!("| lang-shell | specialist | {cell} | 2 | 3 | 6 | 4 | 0.5000 |")
    );
    assert_eq!(
        lines[6],
        "| review-code | generalist | reused | 2 | 3 | 6 | 4 | 0.5000 |"
    );
    assert_eq!(
        lines[7],
        "| review-spec | generalist | its run ended with deadline | 2 | 3 | 6 | 4 | 0.5000 |"
    );
    assert_eq!(lines.len(), 8);
}

#[test]
fn the_watch_prints_while_the_work_runs_and_stops_with_it() {
    let live = Live::default();
    let clock = FakeClock::default();
    let lines = Mutex::new(Vec::new());
    let watch = Watch {
        clock: &clock,
        every: Duration::from_millis(1),
        total: 4,
        cost: &|| 0.25,
        progress: &|l| lines.lock().unwrap().push(l.to_owned()),
    };

    let out = watching(&live, &watch, || {
        while lines.lock().unwrap().len() < 2 {
            std::thread::yield_now();
        }
        7
    });

    assert_eq!(out, 7);
    let lines = lines.into_inner().unwrap();
    assert_eq!(
        lines[0],
        "progress: 0 of 4 agents finished, 0.2500 USD so far, 0 running"
    );
}

#[test]
fn the_watch_stops_when_its_work_panics() {
    let live = Live::default();
    let clock = FakeClock::default();
    let watch = Watch {
        clock: &clock,
        every: Duration::from_secs(3600),
        total: 1,
        cost: &|| 0.0,
        progress: &|_| {},
    };

    let out = catch_unwind(AssertUnwindSafe(|| {
        watching(&live, &watch, || panic!("the work fails"))
    }));

    assert!(out.is_err());
    assert!(live.wait(Duration::ZERO));
}
