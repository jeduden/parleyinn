//! REC-11's indexing cap, the evidence that freezes it (§12.3): an
//! event's text is indexed up to 1 MiB, and indexing one event at the
//! cap costs little time and index. The numbers print with
//! `cargo test -p store --test index_cap -- --nocapture`.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::time::{Duration, Instant};

use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::{RoomId, SeatId, WriterId};
use search::Request;
use serde_json::json;
use store::search_index::DEFAULT_CAP;
use store::{NewEvent, Search, Store, StoredSeat};
use testkit::TempDir;

/// What indexing one event at the cap may take, and the most the store
/// may grow, as a multiple of the text indexed: loose bounds, set well
/// above what is measured (a debug build compiles SQLite unoptimized, so
/// the time is measured in a release build), so only a regression in
/// kind fails.
const MOST_TIME: Duration = Duration::from_secs(60);
const MOST_GROWTH_PER_INDEXED_BYTE: f64 = 4.0;

fn room() -> RoomId {
    RoomId::derive(&[1; 32], &[2; 16])
}

fn seated(dir: &TempDir) -> (Store, WriterId) {
    let mut store = Store::open(&dir.path().join("store.db")).unwrap();
    let writer = WriterId::derive(&[1; 32]);
    store
        .add_seat(&StoredSeat {
            writer,
            seat: SeatId::derive(&room(), &[1; 32]),
            room: room(),
            certificate: json!({}),
        })
        .unwrap();

    (store, writer)
}

fn event(text: String) -> NewEvent {
    NewEvent {
        kind: EventKind::TranscriptLine,
        provenance: Provenance::HarnessText,
        content: json!({"line": 1, "text": text}),
        commitment_key: CommitmentKey::from_bytes([7; 32]),
        transcript_timestamp: None,
    }
}

fn found(store: &Store, word: &str) -> usize {
    let query = search::compile(&Request {
        text: word,
        ..Request::default()
    })
    .unwrap();

    store
        .search(&Search {
            query: &query,
            room: room(),
            writer: None,
            provenance: &[],
            kinds: &[],
            since: None,
            until: None,
            k: 10,
            deadline: Duration::from_secs(10),
        })
        .unwrap()
        .len()
}

/// Words of 7 bytes with their space: `w00000 `, distinct up to a million.
fn words(bytes: usize) -> String {
    (0..bytes / 8).map(|n| format!("w{n:06} ")).collect()
}

fn size(dir: &TempDir) -> u64 {
    ["store.db", "store.db-wal"]
        .iter()
        .map(|f| std::fs::metadata(dir.path().join(f)).map_or(0, |m| m.len()))
        .sum()
}

#[test]
fn the_default_cap_is_one_mebibyte() {
    assert_eq!(DEFAULT_CAP, 1 << 20);
}

#[expect(
    clippy::disallowed_methods,
    reason = "a test of time reads the clock to bound its own run"
)]
#[test]
fn text_past_the_cap_is_not_indexed_and_an_event_at_the_cap_is_cheap_to_index() {
    let dir = TempDir::new().unwrap();
    let (mut store, writer) = seated(&dir);
    let text = format!("{}tail-marker {}", words(DEFAULT_CAP), words(DEFAULT_CAP));
    let before = size(&dir);

    let started = Instant::now();
    store.append(writer, &[event(text)]).unwrap();
    let took = started.elapsed();

    let held = size(&dir) - before;
    let ratio = held as f64 / DEFAULT_CAP as f64;
    println!(
        "one event with 2 MiB of text: indexed to the cap in {took:?}; the store grew {held} bytes, {ratio:.2} per indexed byte"
    );
    assert_eq!(found(&store, "w000000"), 1, "the start is indexed");
    assert_eq!(
        found(&store, "w131071"),
        1,
        "the last word before the cap is"
    );
    assert_eq!(found(&store, "tail-marker"), 0, "text past the cap is not");
    assert!(took < MOST_TIME, "indexing one capped event took {took:?}");
    assert!(
        ratio < MOST_GROWTH_PER_INDEXED_BYTE,
        "the store grew {ratio:.2} bytes per indexed byte"
    );
}
