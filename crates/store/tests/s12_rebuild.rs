//! Spike S12's rebuild test in the store M1 ships (plan 2610012322, M1's
//! exit in §12.2): two writers' logs, appended in either order or
//! interleaved, keep one search index identical in content to the one
//! a rebuild from nothing derives (I10, ADR-2610112004). Each row's id
//! derives from its event's address, never from arrival. A purge turns
//! on FTS5's `secure-delete`, has SQLite overwrite what it frees,
//! truncates both write-ahead logs, and leaves no file holding the
//! purged text (ADM-07, SEC-14). The third finding, that a `user`
//! event's trust needs the deployment mode, is the trust policy's: its
//! unit test `a_user_event_is_trusted_only_when_every_condition_holds`
//! proves it, and `under_m1_no_user_event_is_trusted_whatever_the_caller_says`
//! that no `user` event is trusted before OQ-40.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::path::Path;
use std::time::Duration;

use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, SeatId, Seq, WriterId};
use search::Request;
use serde_json::json;
use store::search_index::row_id_of;
use store::{Erase, NewEvent, Search, Store, StoredSeat};
use testkit::TempDir;

/// The index's file name, beside `store.db`.
const INDEX_FILE: &str = "search_index";

/// A word only the purged event holds.
const PURGED_WORD: &str = "quokka7781secret";

/// The words the searches look for.
const WORDS: [&str; 4] = ["deploy", "staging", "wombat", PURGED_WORD];

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn room() -> RoomId {
    RoomId::derive(&[1; 32], &[2; 16])
}

/// Writer `n`'s appends: three batches of text, some words shared with
/// the other writer, so ranking compares across both logs.
fn appends(n: u8) -> Vec<Vec<NewEvent>> {
    let texts: [&[&str]; 3] = if n == 1 {
        [
            &["deploy to staging", "wombat says hello"],
            &["staging rolled back the deploy"],
            &["deploy deploy staging", "no match here", "wombat staging"],
        ]
    } else {
        [
            &[
                "wombat deploy",
                &format!("the {PURGED_WORD} token was seen"),
            ],
            &["staging", "deploy wombat wombat"],
            &["unrelated words"],
        ]
    };
    texts
        .iter()
        .enumerate()
        .map(|(b, batch)| {
            batch
                .iter()
                .enumerate()
                .map(|(i, text)| NewEvent {
                    kind: EventKind::UnparsedLine,
                    provenance: Provenance::Unparsed,
                    content: json!({"text": text}),
                    commitment_key: CommitmentKey::from_bytes([n * 16 + (b * 4 + i) as u8; 32]),
                    transcript_timestamp: None,
                })
                .collect()
        })
        .collect()
}

/// A store in `dir` that took the appends in `order`, each a writer
/// and the index of one of its batches.
fn arrived(dir: &Path, order: &[(u8, usize)]) -> Store {
    let mut store = Store::open(&dir.join("store.db")).unwrap();
    for n in [1, 2] {
        store
            .add_seat(&StoredSeat {
                writer: writer(n),
                seat: SeatId::derive(&room(), &[n; 32]),
                room: room(),
                certificate: json!({}),
            })
            .unwrap();
    }
    let (a, b) = (appends(1), appends(2));
    for &(n, i) in order {
        let batch = if n == 1 { &a[i] } else { &b[i] };
        store.append(writer(n), batch).unwrap();
    }
    store
}

const A_THEN_B: [(u8, usize); 6] = [(1, 0), (1, 1), (1, 2), (2, 0), (2, 1), (2, 2)];
const B_THEN_A: [(u8, usize); 6] = [(2, 0), (2, 1), (2, 2), (1, 0), (1, 1), (1, 2)];
const INTERLEAVED: [(u8, usize); 6] = [(2, 0), (1, 0), (1, 1), (2, 1), (1, 2), (2, 2)];

/// What a search for each word finds, with scores.
fn answers(store: &Store) -> Vec<Vec<(String, u64)>> {
    WORDS
        .iter()
        .map(|word| {
            let query = search::compile(&Request {
                text: word,
                ..Request::default()
            })
            .unwrap();
            store
                .search(&Search {
                    query: &query,
                    rooms: &[room()],
                    writers: &[],
                    provenance: &[],
                    kinds: &[],
                    since: None,
                    until: None,
                    k: 50,
                    deadline: Duration::from_secs(60),
                })
                .unwrap()
                .iter()
                .map(|h| (h.address.to_string(), h.score.to_bits()))
                .collect()
        })
        .collect()
}

/// The row ids the index file holds, sorted.
fn row_ids(dir: &Path) -> Vec<i64> {
    let index = rusqlite::Connection::open(dir.join(INDEX_FILE)).unwrap();
    let mut rows = index
        .prepare("SELECT rowid FROM events_fts ORDER BY rowid")
        .unwrap();
    rows.query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// The row ids every event of both writers derives from its address.
fn derived_row_ids() -> Vec<i64> {
    let mut ids: Vec<i64> = [1u8, 2]
        .iter()
        .flat_map(|&n| {
            let events = appends(n).concat().len() as u64;
            (1..=events).map(move |seq| Address {
                writer: writer(n),
                seq: Seq::new(seq).unwrap(),
            })
        })
        .filter_map(row_id_of)
        .collect();
    ids.sort_unstable();
    ids
}

#[test]
fn two_writers_logs_rebuild_one_search_index_identical_in_content_in_either_order() {
    let dirs = [
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    ];
    let mut stores: Vec<Store> = [A_THEN_B, B_THEN_A, INTERLEAVED]
        .iter()
        .zip(&dirs)
        .map(|(order, dir)| arrived(dir.path(), order))
        .collect();

    let kept = stores[0].search_index_digest().unwrap();
    let found = answers(&stores[0]);
    assert!(found.iter().all(|hits| !hits.is_empty()), "{found:?}");
    for (store, order) in stores
        .iter_mut()
        .zip(["a then b", "b then a", "interleaved"])
    {
        assert_eq!(store.search_index_digest().unwrap(), kept, "{order}");
        assert_eq!(answers(store), found, "{order}");
        store.rebuild_search_index().unwrap();
        assert_eq!(
            store.search_index_digest().unwrap(),
            kept,
            "{order}, rebuilt"
        );
        assert_eq!(answers(store), found, "{order}, rebuilt");
    }
    drop(stores);
    for dir in &dirs {
        assert_eq!(
            row_ids(dir.path()),
            derived_row_ids(),
            "row ids from addresses"
        );
    }
}

#[test]
fn a_purge_turns_on_secure_delete_truncates_both_logs_and_leaves_no_file_holding_the_text() {
    let dir = TempDir::new().unwrap();
    let mut store = arrived(dir.path(), &INTERLEAVED);
    let purged = Address {
        writer: writer(2),
        seq: Seq::new(2).unwrap(),
    };
    assert_eq!(answers(&store)[3].len(), 1, "the word is found before");
    assert!(
        !holding(dir.path()).is_empty(),
        "a file holds the text before"
    );

    store
        .purge(
            writer(1),
            &[],
            &[Erase {
                writer: purged.writer,
                first: purged.seq,
                last: purged.seq,
            }],
        )
        .unwrap();

    assert!(answers(&store)[3].is_empty(), "the purged word is no hit");
    let kept = store.search_index_digest().unwrap();
    store.rebuild_search_index().unwrap();
    assert_eq!(
        store.search_index_digest().unwrap(),
        kept,
        "kept equals rebuilt"
    );
    let index = rusqlite::Connection::open(dir.path().join(INDEX_FILE)).unwrap();
    let secure: i64 = index
        .query_row(
            "SELECT v FROM events_fts_config WHERE k = 'secure-delete'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(secure, 1, "FTS5's secure-delete is on");
    drop(index);
    drop(store);
    for log in ["store.db-wal", "search_index-wal"] {
        let path = dir.path().join(log);
        let len = std::fs::metadata(&path).map_or(0, |m| m.len());
        assert_eq!(len, 0, "{log} is truncated");
    }
    assert_eq!(
        holding(dir.path()),
        Vec::<String>::new(),
        "files holding the purged text"
    );
}

/// The files in `dir` whose bytes hold [`PURGED_WORD`].
fn holding(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
        .filter(|p| {
            let bytes = std::fs::read(p).unwrap();
            bytes
                .windows(PURGED_WORD.len())
                .any(|w| w == PURGED_WORD.as_bytes())
        })
        .map(|p| p.display().to_string())
        .collect()
}
