//! ENG-08's first property, over randomly generated records: within
//! each writer's log, `seq` is strictly increasing and gap-free, so no
//! two events share an address (writer, `seq`) (REC-06). Each record is
//! a random sequence of appends, open appends, appends a failing event
//! rolls back, segment closes, payloads and appends from a second
//! connection, all to a few writers. The property runs again with the
//! records on several threads at once, each its own connection, as a
//! run's MCP server and `parinn ingest` append beside each other in two
//! processes (REC-06, REC-13). Every case is seeded, so a failure names
//! a seed that reproduces it; `PARINN_PROPERTY_CASES` runs more.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::BTreeMap;

use record::chain;
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::WriterId;
use serde_json::json;
use store::{NewEvent, Store, StoreError};
use testkit::TempDir;

/// How many records the property is checked over by default.
const CASES: u64 = 64;

/// A small seeded generator (xorshift64*): no clock, no OS randomness, so
/// a seed replays its record exactly.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number below `n`.
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// `n` new events, each large enough for the payload store with chance
/// one in eight, and one that has no canonical encoding at `bad`.
fn events(rng: &mut Rng, n: usize, bad: Option<usize>) -> Vec<NewEvent> {
    (0..n)
        .map(|i| {
            let size = if rng.below(8) == 0 { 9000 } else { 10 };
            let content = if bad == Some(i) {
                json!({"x": 0.5})
            } else {
                json!({"text": "x".repeat(size), "i": i})
            };
            let mut key = [0; 32];
            key[..8].copy_from_slice(&rng.next().to_le_bytes());
            NewEvent {
                kind: EventKind::UnparsedLine,
                provenance: Provenance::Unparsed,
                content,
                commitment_key: CommitmentKey::from_bytes(key),
                transcript_timestamp: None,
            }
        })
        .collect()
}

/// One random operation on one of `stores`.
fn step(rng: &mut Rng, stores: &mut [Store], writers: &[WriterId]) {
    let writer = writers[rng.below(writers.len() as u64) as usize];
    let store = &mut stores[rng.below(stores.len() as u64) as usize];
    let n = 1 + rng.below(220) as usize;
    match rng.below(5) {
        0 => drop(store.append(writer, &events(rng, n, None)).unwrap()),
        1 => drop(store.append_open(writer, &events(rng, n, None)).unwrap()),
        2 => {
            let bad = rng.below(n as u64) as usize;
            let err = store
                .append(writer, &events(rng, n, Some(bad)))
                .unwrap_err();
            assert!(matches!(err, StoreError::Encoding(_)), "{err}");
        }
        3 => store.close_segment(writer).unwrap(),
        _ => drop(store.append(writer, &[]).unwrap()),
    }
}

/// Checks the property over the store at `path`: every writer's seqs run
/// 1, 2, … with no gap or repeat, its chain checks, and its segments
/// tile its log.
fn check(path: &std::path::Path, seed: u64) {
    let store = Store::open(path).unwrap();
    let mut logs = BTreeMap::new();
    for writer in store.writers().unwrap() {
        let log = store.log(writer).unwrap();
        let seqs: Vec<u64> = log
            .iter()
            .map(|s| s.link.header.address.seq.get())
            .collect();
        let want: Vec<u64> = (1..=log.len() as u64).collect();
        assert_eq!(seqs, want, "seed {seed}: writer {writer}'s seqs");
        let links: Vec<_> = log.iter().map(|s| s.link.clone()).collect();
        assert_eq!(chain::check_writer(writer, &links), Ok(()), "seed {seed}");
        let mut next = 1;
        for segment in store.segments(writer).unwrap() {
            assert_eq!(
                segment.first.get(),
                next,
                "seed {seed}: writer {writer}'s segments"
            );
            next = segment.last.get() + 1;
        }
        assert_eq!(
            next,
            log.len() as u64 + 1,
            "seed {seed}: writer {writer}'s segments"
        );
        logs.insert(writer, links);
    }
    assert_eq!(chain::check_heads(&logs), Ok(()), "seed {seed}");
}

#[test]
fn within_each_writer_s_log_seq_is_strictly_increasing_and_gap_free() {
    let cases = std::env::var("PARINN_PROPERTY_CASES")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(CASES);
    let writers: Vec<WriterId> = (1..=3).map(|n| WriterId::derive(&[n; 32])).collect();
    for seed in 0..cases {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("store.db");
        // Two connections, as a run's MCP server and `parinn ingest`
        // append beside each other.
        let mut stores = [Store::open(&path).unwrap(), Store::open(&path).unwrap()];
        let mut rng = Rng::new(seed);
        for _ in 0..1 + rng.below(12) {
            step(&mut rng, &mut stores, &writers);
        }
        check(&path, seed);
    }
}

#[test]
fn seq_stays_gap_free_with_appends_on_several_threads_at_once() {
    let cases = std::env::var("PARINN_PROPERTY_CASES")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(CASES / 4);
    let writers: Vec<WriterId> = (1..=3).map(|n| WriterId::derive(&[n; 32])).collect();
    for seed in 0..cases {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("store.db");
        drop(Store::open(&path).unwrap());
        std::thread::scope(|scope| {
            for thread in 0..3 {
                let (path, writers) = (&path, &writers);
                scope.spawn(move || record(path, writers, seed * 7 + thread));
            }
        });
        check(&path, seed);
    }
}

/// One random record of up to eight steps, seeded by `seed`, over its
/// own connection to the store at `path`.
fn record(path: &std::path::Path, writers: &[WriterId], seed: u64) {
    let mut stores = [Store::open(path).unwrap()];
    let mut rng = Rng::new(seed);
    for _ in 0..1 + rng.below(8) {
        step(&mut rng, &mut stores, writers);
    }
}
