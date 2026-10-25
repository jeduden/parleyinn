use super::*;
use crate::{NewEvent, SeatId, Store, StoredSeat};
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::{RoomId, WriterId};
use serde_json::json;
use testkit::TempDir;

fn open(dir: &TempDir) -> Store {
    Store::open(&dir.path().join("store.db")).unwrap()
}

fn with_events(dir: &TempDir, texts: &[&str]) -> Store {
    let mut store = open(dir);
    let (room, writer) = (
        RoomId::derive(&[1; 32], &[2; 16]),
        WriterId::derive(&[1; 32]),
    );
    store
        .add_seat(&StoredSeat {
            writer,
            seat: SeatId::derive(&room, writer.digest().as_bytes()),
            room,
            certificate: json!({}),
        })
        .unwrap();
    let events: Vec<NewEvent> = texts
        .iter()
        .zip(1..)
        .map(|(text, n)| NewEvent {
            kind: EventKind::TranscriptLine,
            provenance: Provenance::Unparsed,
            content: json!({"line": n, "text": text}),
            commitment_key: CommitmentKey::from_bytes([n as u8; 32]),
            transcript_timestamp: None,
        })
        .collect();
    store.append(writer, &events).unwrap();
    store
}

fn raw(dir: &TempDir) -> Connection {
    let conn = Connection::open(dir.path().join("store.db")).unwrap();
    conn.execute(
        "ATTACH DATABASE ?1 AS idx",
        [dir.path().join(search_index::FILE).to_str().unwrap()],
    )
    .unwrap();
    conn
}

#[test]
fn a_store_made_by_this_build_holds_no_foreign_object() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &["one"]);

    assert_eq!(
        store.foreign_schema_objects().unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn every_lazily_made_table_is_shipped_too() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &["one"]);
    for schema in [origin::SCHEMA, refusal::SCHEMA, purge::SCHEMA] {
        store.conn.execute_batch(schema).unwrap();
    }
    for schema in repository::schemas() {
        store.conn.execute_batch(&schema).unwrap();
    }

    assert_eq!(
        store.foreign_schema_objects().unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn an_object_this_build_does_not_ship_is_named_in_either_file() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &["one"]);
    store
        .conn
        .execute_batch("CREATE TABLE exfil (x TEXT); CREATE TABLE idx.extra (y TEXT);")
        .unwrap();

    assert_eq!(
        store.foreign_schema_objects().unwrap(),
        ["idx.extra", "main.exfil"]
    );
}

#[test]
fn a_new_index_ranks_as_shipped_and_an_altered_ranking_does_not() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &["one"]);
    assert!(store.search_index_ranking_is_shipped().unwrap());

    store
        .conn
        .execute(
            "INSERT INTO idx.events_fts (events_fts, rank) VALUES ('rank', 'bm25(0.0)')",
            [],
        )
        .unwrap();

    assert!(!store.search_index_ranking_is_shipped().unwrap());
    assert_eq!(ranking(&raw(&dir)).unwrap().len(), 1);
}

#[test]
fn an_index_that_the_record_derives_is_derived_and_a_changed_entry_is_not() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &["alpha beta", "gamma"]);
    assert!(store.search_index_is_derived(&|| true).unwrap());
    let row = search_index::row_id_of(record::id::Address::new(
        WriterId::derive(&[1; 32]),
        record::id::Seq::new(1).unwrap(),
    ))
    .unwrap();
    store
        .conn
        .execute("DELETE FROM idx.events_fts WHERE rowid = ?1", [row])
        .unwrap();
    store
        .conn
        .execute(
            "INSERT INTO idx.events_fts (rowid, text) VALUES (?1, 'zebra')",
            [row],
        )
        .unwrap();

    assert!(!store.search_index_is_derived(&|| true).unwrap());
    // The check changed nothing: the entry is still the wrong one.
    assert!(!store.search_index_is_derived(&|| true).unwrap());
}

/// The row id of event `seq` of the writer [`with_events`] appends to.
fn row(seq: u64) -> i64 {
    search_index::row_id_of(record::id::Address::new(
        WriterId::derive(&[1; 32]),
        record::id::Seq::new(seq).unwrap(),
    ))
    .unwrap()
}

/// Puts `text` in the index as event `seq`'s entry, in place of the one
/// it has.
fn forge(store: &Store, seq: u64, text: &str) {
    store
        .conn
        .execute("DELETE FROM idx.events_fts WHERE rowid = ?1", [row(seq)])
        .unwrap();
    store
        .conn
        .execute(
            "INSERT INTO idx.events_fts (rowid, text) VALUES (?1, ?2)",
            rusqlite::params![row(seq), text],
        )
        .unwrap();
}

/// Journals the range `first`-`last` of the writer [`with_events`]
/// appends to as a purge's step still to take out of the index.
fn pending(store: &Store, first: u64, last: u64) {
    store.conn.execute_batch(crate::purge::SCHEMA).unwrap();
    store
        .conn
        .execute(
            "INSERT INTO purge_journal (kind, target) VALUES ('index', ?1)",
            [format!("{}:{first}-{last}", WriterId::derive(&[1; 32]))],
        )
        .unwrap();
}

#[test]
fn an_entry_inside_a_pending_purges_range_is_not_compared() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &["alpha", "beta", "gamma"]);
    forge(&store, 2, "residue");
    assert!(!store.search_index_is_derived(&|| true).unwrap());

    pending(&store, 2, 2);

    assert!(store.search_index_is_derived(&|| true).unwrap());
}

#[test]
fn an_entry_outside_a_pending_purges_range_is_still_compared() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &["alpha", "beta", "gamma"]);
    pending(&store, 2, 2);
    forge(&store, 2, "residue");
    assert!(store.search_index_is_derived(&|| true).unwrap());

    forge(&store, 3, "zebra");

    assert!(!store.search_index_is_derived(&|| true).unwrap());
}

#[test]
fn a_pending_range_of_a_writer_without_its_slot_leaves_every_entry_compared() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &["alpha", "beta"]);
    store.conn.execute_batch(crate::purge::SCHEMA).unwrap();
    // Another writer: it holds no slot, so its range covers no entry,
    // even one at the slot it would take.
    store
        .conn
        .execute(
            "INSERT INTO purge_journal (kind, target) VALUES ('index', ?1)",
            [format!("{}:1-2", WriterId::derive(&[9; 32]))],
        )
        .unwrap();
    forge(&store, 2, "zebra");

    assert!(!store.search_index_is_derived(&|| true).unwrap());
}

#[test]
fn a_store_without_its_index_file_attached_has_only_its_own_objects() {
    let dir = TempDir::new().unwrap();
    drop(with_events(&dir, &["one"]));
    std::fs::remove_file(dir.path().join(search_index::FILE)).unwrap();
    let store = Store::open_read_only_made(&dir.path().join("store.db"))
        .unwrap()
        .unwrap();

    assert_eq!(
        store.foreign_schema_objects().unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn a_store_without_an_index_has_nothing_to_check() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    store.conn.execute_batch("DETACH DATABASE idx").unwrap();

    assert!(matches!(
        store.search_index_ranking_is_shipped(),
        Err(StoreError::NoIndex)
    ));
    assert!(matches!(
        store.search_index_is_derived(&|| true),
        Err(StoreError::NoIndex)
    ));
}

#[test]
fn a_seat_without_events_leaves_the_index_derived() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &[]);

    assert!(store.search_index_is_derived(&|| true).unwrap());
}

#[test]
fn another_connection_writes_both_files_while_the_index_is_derived_again() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &["alpha", "beta"]);
    let wrote = std::cell::Cell::new(0);
    // Asked between chunks whether time is left: a writer that waits for
    // no lock must get both files' write lock then (I9, ENG-03).
    let time_left = || {
        let other = rusqlite::Connection::open(dir.path().join("store.db")).unwrap();
        other.busy_timeout(std::time::Duration::ZERO).unwrap();
        let index = dir.path().join(search_index::FILE);
        other
            .execute("ATTACH DATABASE ?1 AS idx", [index.to_string_lossy()])
            .unwrap();
        let took = other
            .execute_batch(
                "BEGIN IMMEDIATE; CREATE TABLE IF NOT EXISTS main.t (x); \
                 CREATE TABLE IF NOT EXISTS idx.t (x); ROLLBACK;",
            )
            .is_ok();
        wrote.set(wrote.get() + usize::from(took));
        true
    };

    assert!(store.search_index_is_derived(&time_left).unwrap());
    assert!(wrote.get() > 0, "another connection could not write");
}

#[test]
fn an_index_check_out_of_time_is_cancelled_and_changes_nothing() {
    let dir = TempDir::new().unwrap();
    let store = with_events(&dir, &["alpha", "beta"]);
    let kept = store.search_index_digest().unwrap();

    assert!(matches!(
        store.search_index_is_derived(&|| false),
        Err(StoreError::Cancelled)
    ));
    assert_eq!(store.search_index_digest().unwrap(), kept);
}
