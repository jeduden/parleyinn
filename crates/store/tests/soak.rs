//! ENG-09's concurrency soak: 50 writers append at once, each through a
//! connection of its own, as 50 runs' MCP servers do (NFR-05). Every
//! writer's log is then gap-free from seq 1, no two events share an
//! address, every chain checks and every head an append cited is one
//! its writer holds. CI runs `PARINN_SOAK_EVENTS` events, 10,000 by
//! default; the nightly job runs 1,000,000 in a release build.
//!
//! An append that waits past the store's busy deadline fails whole and
//! appends nothing. The MCP server counts it as `live_ingest_failed`
//! and reads the transcript again on its next pass; each writer here
//! does the same, and the soak prints how many appends it retried.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use record::chain;
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::{Address, WriterId};
use rusqlite::ErrorCode;
use serde_json::json;
use store::{NewEvent, Store, StoreError};
use testkit::TempDir;

/// The writers appending at once.
const WRITERS: u8 = 50;

/// Events across every writer when `PARINN_SOAK_EVENTS` names none.
const EVENTS: usize = 10_000;

/// Events per append.
const BATCH: usize = 100;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn events(from: usize, n: usize) -> Vec<NewEvent> {
    (from..from + n)
        .map(|i| NewEvent {
            kind: EventKind::UnparsedLine,
            provenance: Provenance::Unparsed,
            content: json!({"text": format!("soak line {i}")}),
            commitment_key: CommitmentKey::from_bytes([7; 32]),
            transcript_timestamp: None,
        })
        .collect()
}

/// Whether `e` is an append that waited past the busy deadline.
fn busy(e: &StoreError) -> bool {
    matches!(e, StoreError::Sqlite(rusqlite::Error::SqliteFailure(f, _)) if f.code == ErrorCode::DatabaseBusy)
}

/// Appends `per` events to writer `w`'s log through a connection of its
/// own, retrying an append the busy deadline failed: how many it
/// retried.
fn append_all(path: &Path, w: u8, per: usize) -> usize {
    let mut store = Store::open(path).unwrap();
    let (mut done, mut retried) = (0, 0);
    while done < per {
        let n = BATCH.min(per - done);
        match store.append(writer(w), &events(done, n)) {
            Ok(_) => done += n,
            Err(e) => {
                assert!(busy(&e), "writer {w}: {e}");
                retried += 1;
            }
        }
    }

    retried
}

/// Has every writer append `per` events on a thread of its own, all at
/// once: how many appends they retried.
fn soak(path: &Path, per: usize) -> usize {
    let retried = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for w in 0..WRITERS {
            let retried = &retried;
            scope.spawn(move || retried.fetch_add(append_all(path, w, per), Ordering::Relaxed));
        }
    });
    retried.into_inner()
}

#[test]
fn fifty_writers_appending_at_once_keep_every_log_gap_free_and_every_address_unique() {
    let total = std::env::var("PARINN_SOAK_EVENTS")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(EVENTS);
    let per = total / usize::from(WRITERS);
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    Store::open(&path).unwrap();

    let retried = soak(&path, per);
    println!(
        "{total} events by {WRITERS} writers, {retried} appends retried past the busy deadline"
    );

    let store = Store::open(&path).unwrap();
    let mut addresses: BTreeSet<Address> = BTreeSet::new();
    let mut record = BTreeMap::new();
    for w in store.writers().unwrap() {
        let links: Vec<_> = store.log(w).unwrap().into_iter().map(|s| s.link).collect();
        let seqs: Vec<u64> = links.iter().map(|l| l.header.address.seq.get()).collect();
        assert_eq!(seqs, (1..=per as u64).collect::<Vec<_>>(), "{w}");
        assert_eq!(chain::check_writer(w, &links), Ok(()), "{w}");
        for l in &links {
            assert!(
                addresses.insert(l.header.address),
                "{:?} twice",
                l.header.address
            );
        }
        record.insert(w, links);
    }
    assert_eq!(record.len(), usize::from(WRITERS));
    assert_eq!(addresses.len(), per * usize::from(WRITERS));
    assert_eq!(chain::check_heads(&record), Ok(()));
}
