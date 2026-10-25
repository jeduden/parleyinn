//! NFR-03's search latency, the evidence behind the bounded ranking
//! (S2, RCL-02): a store of events (default 1,000,000; `PARINN_SEARCH_EVENTS`
//! sets another number), each searched for a word nearly every event
//! holds, a word a third of them hold and a word one in a thousand
//! holds. The test is `#[ignore]` because it builds a large store; run
//! it with `cargo test --release -p store --test search_latency --
//! --ignored --nocapture`. The events are written by one SQL statement
//! beside the real appends, since a search reads no chain.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::time::{Duration, Instant};

use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, SeatId, Seq, WriterId};
use search::Request;
use serde_json::json;
use store::search_index::{SLOT_BITS, row_id_of};
use store::{NewEvent, Search, Store, StoredSeat};
use testkit::TempDir;

/// NFR-03's bound on a search at 10M events, p95.
const NFR_03: Duration = Duration::from_millis(200);

/// How many searches of each kind are timed.
const RUNS: usize = 50;

fn room() -> RoomId {
    RoomId::derive(&[1; 32], &[2; 16])
}

fn events() -> u64 {
    std::env::var("PARINN_SEARCH_EVENTS")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(1_000_000)
}

/// A store holding `count` events of one seated writer.
fn build(dir: &TempDir, count: u64) -> Store {
    let path = dir.path().join("store.db");
    let mut store = Store::open(&path).unwrap();
    let writer = WriterId::derive(&[1; 32]);
    store
        .add_seat(&StoredSeat {
            writer,
            seat: SeatId::derive(&room(), &[1; 32]),
            room: room(),
            certificate: json!({}),
        })
        .unwrap();
    let first = NewEvent {
        kind: EventKind::TranscriptLine,
        provenance: Provenance::HarnessText,
        content: json!({"line": 1, "text": "message first"}),
        commitment_key: CommitmentKey::from_bytes([3; 32]),
        transcript_timestamp: None,
    };
    store.append(writer, &[first]).unwrap();
    let row_of_first = row_id_of(Address::new(writer, Seq::FIRST)).unwrap();
    drop(store);

    let mut conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("PRAGMA synchronous = OFF").unwrap();
    conn.execute(
        "ATTACH DATABASE ?1 AS idx",
        [dir.path().join("search_index").to_str().unwrap()],
    )
    .unwrap();
    let slot_part = row_of_first - (1_i64 << SLOT_BITS);
    let text = "'message line ' || (i % 5000) || CASE WHEN i % 3 = 0 THEN ' deploy' ELSE '' END
                || CASE WHEN i % 1000 = 0 THEN ' rarity' ELSE '' END";
    let tx = conn.transaction().unwrap();
    let numbers = "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < ?1)";
    tx.execute(
        &format!(
            "{numbers} INSERT INTO events (writer, seq, kind, provenance, prev_hash, commitment, hash,
                                            content, commitment_key)
             SELECT ?2, 1 + i, 'transcript_line', 'harness_text', printf('%064x', i),
                    printf('%064x', i), printf('%064x', i),
                    json_object('line', i, 'text', {text}), printf('%064x', i)
             FROM n"
        ),
        rusqlite::params![count as i64, writer.to_string()],
    )
    .unwrap();
    tx.execute(
        &format!(
            "{numbers} INSERT INTO idx.events_fts (rowid, text)
             SELECT ((1 + i) << ?2) + ?3, {text} FROM n"
        ),
        rusqlite::params![count as i64, i64::from(SLOT_BITS), slot_part],
    )
    .unwrap();
    tx.execute(
        "UPDATE idx.search_writers SET indexed_to = 1 + ?1 WHERE writer = ?2",
        rusqlite::params![count as i64, writer.to_string()],
    )
    .unwrap();
    tx.commit().unwrap();

    Store::open(&path).unwrap()
}

#[expect(
    clippy::disallowed_methods,
    reason = "a test of time reads the clock to bound its own run"
)]
/// The p50 and p95 of `RUNS` searches for `word`.
fn time(store: &Store, word: &str, k: usize) -> (Duration, Duration, usize) {
    let query = search::compile(&Request {
        text: word,
        ..Request::default()
    })
    .unwrap();
    let search = Search {
        query: &query,
        room: room(),
        writer: None,
        provenance: &[],
        kinds: &[],
        since: None,
        until: None,
        k,
        deadline: Duration::from_secs(30),
    };
    let mut took = Vec::new();
    let mut hits = 0;
    for _ in 0..RUNS {
        let started = Instant::now();
        hits = store.search(&search).unwrap().len();
        took.push(started.elapsed());
    }
    took.sort();

    (took[RUNS / 2], took[RUNS * 95 / 100], hits)
}

#[expect(
    clippy::disallowed_methods,
    reason = "a test of time reads the clock to bound its own run"
)]
#[test]
#[ignore = "builds a store of a million events"]
fn a_search_ranks_the_newest_matches_within_nfr_03() {
    let dir = TempDir::new().unwrap();
    let count = events();
    let started = Instant::now();
    let store = build(&dir, count);
    println!("built {count} events in {:?}", started.elapsed());

    for (word, k) in [
        ("message", 10),
        ("deploy", 10),
        ("rarity", 10),
        ("message", 50),
    ] {
        let (p50, p95, hits) = time(&store, word, k);
        println!("{count} events: {word:?} k={k}: {hits} hits, p50 {p50:?}, p95 {p95:?}");
        assert!(p95 <= NFR_03, "{word:?}: p95 {p95:?} is past {NFR_03:?}");
    }
}
