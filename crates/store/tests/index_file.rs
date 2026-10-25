//! The search index is a derived artifact kept in its own file beside the
//! record (I10, §8.2, NFR-09): `store.db` holds the record alone, the
//! index file can be deleted, and the next opener derives it again from
//! the record, with the same digest and the same answers.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::path::Path;
use std::time::Duration;

use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::{RoomId, SeatId, WriterId};
use search::Request;
use serde_json::json;
use store::{NewEvent, Search, Store, StoredSeat};
use testkit::TempDir;

/// The index's file name, beside `store.db`.
const INDEX_FILE: &str = "search_index";

fn writer() -> WriterId {
    WriterId::derive(&[1; 32])
}

fn room() -> RoomId {
    RoomId::derive(&[1; 32], &[2; 16])
}

/// A store holding two appends of text, and its directory.
fn filled(dir: &Path) -> Store {
    let mut store = Store::open(&dir.join("store.db")).unwrap();
    store
        .add_seat(&StoredSeat {
            writer: writer(),
            seat: SeatId::derive(&room(), &[1; 32]),
            room: room(),
            certificate: json!({}),
        })
        .unwrap();
    for (n, text) in ["deploy to staging", "the wombat rolled back the deploy"]
        .iter()
        .enumerate()
    {
        let mut key = [0; 32];
        key[0] = n as u8;
        store
            .append(
                writer(),
                &[NewEvent {
                    kind: EventKind::UnparsedLine,
                    provenance: Provenance::Unparsed,
                    content: json!({"text": text}),
                    commitment_key: CommitmentKey::from_bytes(key),
                    transcript_timestamp: None,
                }],
            )
            .unwrap();
    }
    store
}

/// The addresses a search for `word` finds, with scores.
fn found(store: &Store, word: &str) -> Vec<(String, u64)> {
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
        .iter()
        .map(|h| (h.address.to_string(), h.score.to_bits()))
        .collect()
}

/// Removes the index file and its journal files.
fn delete_index(dir: &Path) {
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(dir.join(format!("{INDEX_FILE}{suffix}")));
    }
}

#[test]
fn the_record_file_holds_no_search_table_and_the_index_has_a_file_of_its_own() {
    let dir = TempDir::new().unwrap();
    drop(filled(dir.path()));

    assert!(dir.path().join(INDEX_FILE).exists());
    let record = rusqlite::Connection::open(dir.path().join("store.db")).unwrap();
    let tables: i64 = record
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE name LIKE 'events_fts%' OR name LIKE 'events_vocab%' OR name = 'search_writers'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tables, 0);
}

#[test]
fn a_deleted_index_file_is_derived_again_with_the_same_digest_and_answers() {
    let dir = TempDir::new().unwrap();
    let store = filled(dir.path());
    let (digest, deploy, wombat) = (
        store.search_index_digest().unwrap(),
        found(&store, "deploy"),
        found(&store, "wombat"),
    );
    drop(store);
    delete_index(dir.path());

    let store = Store::open(&dir.path().join("store.db")).unwrap();

    assert_eq!(store.search_index_digest().unwrap(), digest);
    assert_eq!(found(&store, "deploy"), deploy);
    assert_eq!(found(&store, "wombat"), wombat);
    assert_eq!(deploy.len(), 2);
}

#[test]
fn an_index_file_that_lags_the_record_is_brought_up_to_it_by_the_next_opener() {
    let dir = TempDir::new().unwrap();
    let store = filled(dir.path());
    let digest = store.search_index_digest().unwrap();
    drop(store);
    // An index file from before the record's last append, as a crash
    // between the two files' commits leaves it.
    let index = rusqlite::Connection::open(dir.path().join(INDEX_FILE)).unwrap();
    index
        .execute("UPDATE search_writers SET indexed_to = 1", [])
        .unwrap();
    index
        .execute("DELETE FROM events_fts WHERE rowid >> 35 = 2", [])
        .unwrap();
    drop(index);

    let store = Store::open(&dir.path().join("store.db")).unwrap();

    assert_eq!(store.search_index_digest().unwrap(), digest);
}
