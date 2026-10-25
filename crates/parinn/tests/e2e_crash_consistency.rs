//! ENG-06 (plan 2609292012, phases 6 and 16): the built core is killed
//! with `SIGKILL` at a random point while `parinn ingest` records a
//! transcript with payloads, turn ends and a subagent, while `parinn
//! purge --range` erases one of its events at a terminal, and while
//! `parinn rebuild` derives the derived artifacts again, in an isolated
//! home, every start confined (ENG-12). After every kill `parinn verify`
//! exits 0 on the state the kill left, which it repairs nothing of:
//! every writer's chain gap-free and whole, every payload its event
//! names present and opening its commitment, every purged event's
//! erasure confirmed by its tombstone, and every writer's segments
//! tiling its log. A purge a kill left unfinished is reported, and the
//! next rebuild finishes it, after which verify reports none. A kill before the store exists leaves nothing to
//! verify.
//!
//! The kill points come from a seeded generator, so a failing iteration
//! names a seed that replays it. CI runs `PARINN_CRASH_ITERATIONS`
//! iterations of each operation, a few by default; the nightly jobs run
//! 10,000 of each. Migration joins when M3 ships the first one
//! (ADM-05).

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]
#![allow(
    clippy::panic,
    reason = "a start the confinement refuses fails the test with its message"
)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use testkit::TempDir;
use testkit::boundary::{Component, Confinement};

/// The transcript each iteration ingests: 115 lines, four above the
/// payload threshold, with turn ends and a subagent's result.
const TRANSCRIPT: &str = "testdata/claude-code/2.1.296/transcripts/projects/-w-my-app/205beaae-e4b8-4798-a1aa-cdf50c807136.jsonl";

/// Iterations when `PARINN_CRASH_ITERATIONS` names none.
const ITERATIONS: u64 = 6;

/// Iterations a home takes before a fresh one, so a long run also kills
/// many first starts, and no home grows past what verify reads quickly.
const PER_HOME: u64 = 25;

/// The latest kill, past which most ingests have finished.
const LATEST: u64 = 400;

/// The latest kill of a purge or a rebuild, past which most have
/// finished.
const LATEST_ACT: u64 = 300;

fn core() -> Confinement {
    Confinement::of(Component::Core).unwrap_or_else(|e| panic!("{e}"))
}

fn program() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_parinn"))
}

/// A number from the environment variable `name`, or `default`.
fn setting(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(default)
}

/// The kill point of iteration `n` under `seed`, in milliseconds after
/// the start, below `latest`: a splitmix64 step, so every point replays.
fn kill_point_below(seed: u64, n: u64, latest: u64) -> u64 {
    let mut z = seed.wrapping_add(n).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) % latest
}

/// The kill point of an ingest's iteration `n` under `seed`.
fn kill_point(seed: u64, n: u64) -> u64 {
    kill_point_below(seed, n, LATEST)
}

/// Kills `parinn ingest --path transcript` in `home` at `after`, then
/// checks the record: `parinn verify` exits 0 once a store exists.
fn kill_and_verify(home: &Path, transcript: &Path, after: Duration, label: &str) -> bool {
    let path = transcript.display().to_string();
    let killed = core()
        .run_killed(program(), &["ingest", "--path", &path], home, &[], after)
        .unwrap_or_else(|e| panic!("{label}: {e}"));
    if !home.join(".parinn/store.db").exists() {
        return killed;
    }
    let out = core()
        .run(program(), &["verify"], home)
        .unwrap_or_else(|e| panic!("{label}: {e}"));
    assert!(
        out.status.success(),
        "{label}: after a kill at {after:?}, parinn verify exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );

    killed
}

/// A fresh home with the transcript under its transcript root.
fn fresh() -> (TempDir, PathBuf) {
    let home = TempDir::new().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(TRANSCRIPT);
    let placed = testkit::place_transcript(home.path(), &source).unwrap();
    (home, placed)
}

/// Runs every `threads`th iteration from `first` up to `iterations`,
/// each `PER_HOME` in a fresh home; how many were killed mid-ingest.
fn iterate(first: u64, threads: u64, iterations: u64, seed: u64) -> u64 {
    let mut home = fresh();
    let mut killed = 0;
    for (i, n) in (first..iterations).step_by(threads as usize).enumerate() {
        if i > 0 && (i as u64).is_multiple_of(PER_HOME) {
            home = fresh();
        }
        let after = Duration::from_millis(kill_point(seed, n));
        let label = format!("seed {seed}, iteration {n}");
        killed += u64::from(kill_and_verify(home.0.path(), &home.1, after, &label));
    }

    killed
}

#[test]
fn every_kill_during_ingest_leaves_a_record_that_verifies() {
    let iterations = setting("PARINN_CRASH_ITERATIONS", ITERATIONS);
    let seed = setting("PARINN_CRASH_SEED", 0);
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    let threads = setting("PARINN_CRASH_THREADS", cores).clamp(1, iterations.max(1));
    let killed = std::sync::atomic::AtomicU64::new(0);
    // Each thread kills in homes of its own; the scope ends once every
    // thread has, and a failing iteration fails the test.
    std::thread::scope(|scope| {
        for first in 0..threads {
            let killed = &killed;
            scope.spawn(move || {
                let n = iterate(first, threads, iterations, seed);
                killed.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
            });
        }
    });
    println!(
        "{iterations} iterations on {threads} threads, {} killed mid-ingest",
        killed.into_inner()
    );
}

/// Asserts that `parinn verify --json` exits 0 in `home`, on the state
/// as it is: verify repairs nothing. Returns the ranges it reports a
/// purge left unfinished.
fn verified(home: &Path, after: Duration, label: &str) -> Vec<String> {
    let out = core()
        .run(program(), &["verify", "--json"], home)
        .unwrap_or_else(|e| panic!("{label}: {e}"));
    assert!(
        out.status.success(),
        "{label}: after a kill at {after:?}, parinn verify exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("{label}: parinn verify --json printed no JSON: {e}"));
    // The unit tests hold the report's fields; here a report without
    // the field reads as one naming no unfinished purge.
    report["unfinished_purges"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|r| r.as_str().unwrap_or_default().to_owned())
        .collect()
}

/// After a kill left a purge unfinished, runs the command that finishes
/// it, `parinn rebuild`, and asserts verify then reports none and exits
/// 0.
fn finished(home: &Path, after: Duration, label: &str) {
    let out = core()
        .run(program(), &["rebuild"], home)
        .unwrap_or_else(|e| panic!("{label}: {e}"));
    assert!(
        out.status.success(),
        "{label}: parinn rebuild exited {}",
        out.status
    );
    let left = verified(home, after, label);
    assert!(left.is_empty(), "{label}: rebuild left {left:?} unfinished");
}

/// A fresh home that ingested the transcript whole and recorded the
/// standing risk acceptance a purge needs (OWN-22), and the writer of
/// the transcript's run.
fn ingested() -> (TempDir, String) {
    let (home, transcript) = fresh();
    let path = transcript.display().to_string();
    let out = core()
        .run(program(), &["ingest", "--path", &path], home.path())
        .unwrap_or_else(|e| panic!("{e}"));
    let said = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "parinn ingest exited {}: {said}",
        out.status
    );
    let writer = said
        .lines()
        .find_map(|l| {
            l.split_once(" into writer ")
                .map(|(_, w)| w.trim().to_owned())
        })
        .unwrap_or_else(|| panic!("parinn ingest named no writer: {said}"));
    let accepted = core()
        .run_at_terminal(program(), &["risk-acceptance", "record"], home.path(), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        accepted.status.success(),
        "parinn risk-acceptance record exited {}",
        accepted.status
    );

    (home, writer)
}

/// The operations killed after ingest, each by the arguments of
/// iteration `i` in its home of `writer`'s run, and whether it runs at a
/// terminal.
#[derive(Clone, Copy)]
enum Act {
    /// `parinn purge --range` of one event, a widening act at a terminal.
    Purge,
    /// `parinn rebuild`.
    Rebuild,
}

impl Act {
    fn args(self, writer: &str, i: u64) -> Vec<String> {
        match self {
            // Seq 1 opens the run; each iteration of a home purges an
            // event of its own after it.
            Self::Purge => vec![
                "purge".into(),
                "--range".into(),
                format!("{writer}:{0}-{0}", i + 2),
            ],
            Self::Rebuild => vec!["rebuild".into()],
        }
    }
}

/// Runs every `threads`th iteration of `act` from `first` up to
/// `iterations`, each `PER_HOME` in a freshly ingested home; how many
/// were killed mid-way, and how many kills left a purge unfinished,
/// which verify reported and the next rebuild finished.
fn iterate_act(act: Act, first: u64, threads: u64, iterations: u64, seed: u64) -> (u64, u64) {
    let mut home = ingested();
    let mut killed = 0;
    let mut unfinished = 0;
    for (i, n) in (first..iterations).step_by(threads as usize).enumerate() {
        let i = i as u64 % PER_HOME;
        if n > first && i == 0 {
            home = ingested();
        }
        let after = Duration::from_millis(kill_point_below(seed, n, LATEST_ACT));
        let label = format!("seed {seed}, iteration {n}");
        let args = act.args(&home.1, i);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let run = match act {
            Act::Purge => {
                core().run_killed_at_terminal(program(), &args, home.0.path(), &[], after)
            }
            Act::Rebuild => core().run_killed(program(), &args, home.0.path(), &[], after),
        };
        killed += u64::from(run.unwrap_or_else(|e| panic!("{label}: {e}")));
        if !verified(home.0.path(), after, &label).is_empty() {
            unfinished += 1;
            finished(home.0.path(), after, &label);
        }
    }

    (killed, unfinished)
}

/// Kills `act` at `PARINN_CRASH_ITERATIONS` points on as many threads
/// as the machine has cores; how many iterations ran, how many were
/// killed mid-way and how many kills left a purge unfinished.
fn kill_act(act: Act) -> (u64, u64, u64) {
    let iterations = setting("PARINN_CRASH_ITERATIONS", ITERATIONS);
    let seed = setting("PARINN_CRASH_SEED", 0);
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    let threads = setting("PARINN_CRASH_THREADS", cores).clamp(1, iterations.max(1));
    let (killed, unfinished) = std::thread::scope(|scope| {
        let lanes: Vec<_> = (0..threads)
            .map(|first| scope.spawn(move || iterate_act(act, first, threads, iterations, seed)))
            .collect();
        lanes.into_iter().fold((0, 0), |(k, u), lane| {
            let (n, m) = lane.join().unwrap_or_else(|e| std::panic::resume_unwind(e));
            (k + n, u + m)
        })
    });

    (iterations, killed, unfinished)
}

/// The state a kill between a purge's transaction and its finishing
/// steps leaves, made on purpose: a purge of `writer`'s event 2 that
/// finished, its posting put back in the search index and its steps back
/// in the journal. The seeded kills reach this window only now and then.
fn cut_short_purge(home: &Path, writer: &str) {
    let range = format!("{writer}:2-2");
    let out = core()
        .run_at_terminal(program(), &["purge", "--range", &range], home, &[])
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(out.status.success(), "parinn purge exited {}", out.status);
    let address = store::Address::parse(&format!("{writer}:2")).unwrap().0;
    let row = store::search_index::row_id_of(address).unwrap();
    let parinn = home.join(".parinn");
    rusqlite::Connection::open(parinn.join("search_index"))
        .unwrap()
        .execute(
            "INSERT INTO events_fts (rowid, text) VALUES (?1, 'residue')",
            [row],
        )
        .unwrap();
    rusqlite::Connection::open(parinn.join("store.db"))
        .unwrap()
        .execute(
            "INSERT INTO purge_journal (kind, target) VALUES ('index', ?1), ('vacuum', '')",
            [&range],
        )
        .unwrap();
}

#[test]
fn a_purge_cut_short_after_its_transaction_verifies_and_the_next_rebuild_finishes_it() {
    let (home, writer) = ingested();
    cut_short_purge(home.path(), &writer);
    let label = "a purge cut short";

    let left = verified(home.path(), Duration::ZERO, label);

    assert_eq!(left, [format!("{writer}:2-2")], "verify names what is left");
    finished(home.path(), Duration::ZERO, label);
}

#[test]
fn every_kill_during_purge_leaves_a_record_that_verifies() {
    let (iterations, killed, unfinished) = kill_act(Act::Purge);
    println!(
        "{iterations} purge iterations, {killed} killed mid-purge, {unfinished} left it unfinished"
    );
}

#[test]
fn every_kill_during_rebuild_leaves_a_record_that_verifies() {
    let (iterations, killed, unfinished) = kill_act(Act::Rebuild);
    println!(
        "{iterations} rebuild iterations, {killed} killed mid-rebuild, {unfinished} left a purge unfinished"
    );
}

#[test]
fn the_kill_points_replay_and_spread_over_an_ingest() {
    let points: Vec<u64> = (0..1000).map(|n| kill_point(7, n)).collect();

    assert_eq!(
        points,
        (0..1000).map(|n| kill_point(7, n)).collect::<Vec<_>>()
    );
    assert!(points.iter().all(|p| *p < LATEST));
    assert!(points.iter().any(|p| *p < LATEST / 10));
    assert!(points.iter().any(|p| *p > LATEST * 9 / 10));
    assert_ne!(kill_point(7, 1), kill_point(8, 1));
}
