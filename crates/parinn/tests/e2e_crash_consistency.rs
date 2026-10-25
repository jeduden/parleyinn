//! ENG-06 over ingest (plan 2609292012, phase 6): the built core is
//! killed with `SIGKILL` at a random point while `parinn ingest` records
//! a transcript with payloads, turn ends and a subagent, in an isolated
//! home, every start confined (ENG-12). After every kill `parinn verify`
//! exits 0: every writer's chain gap-free and whole, every payload its
//! event names present and opening its commitment, and every writer's
//! segments tiling its log. A kill before the store exists leaves
//! nothing to verify.
//!
//! The kill points come from a seeded generator, so a failing iteration
//! names a seed that replays it. CI runs `PARINN_CRASH_ITERATIONS`
//! iterations, a few by default; the nightly job runs 10,000.
//! Purge, migration and rebuild join as their phases land.

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
/// the start: a splitmix64 step, so every point replays.
fn kill_point(seed: u64, n: u64) -> u64 {
    let mut z = seed.wrapping_add(n).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) % LATEST
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
