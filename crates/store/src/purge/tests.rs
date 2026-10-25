use super::*;
use crate::{Search, SeatId, StoredSeat};
use record::chain;
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::RoomId;
use search::Request;
use serde_json::json;
use std::path::Path;
use testkit::TempDir;

const MARKER: &str = "zebrapurge7";

fn room() -> RoomId {
    RoomId::derive(&[1; 32], &[2; 16])
}

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

fn open(dir: &TempDir) -> Store {
    Store::open(&dir.path().join("store.db")).unwrap()
}

fn seated(store: &mut Store, n: u8) {
    store
        .add_seat(&StoredSeat {
            writer: writer(n),
            seat: SeatId::derive(&room(), &[n; 32]),
            room: room(),
            certificate: json!({}),
        })
        .unwrap();
}

/// A transcript line of `text`, keyed by `n`, stamped with a time.
fn line(n: u64, text: &str) -> NewEvent {
    NewEvent {
        kind: EventKind::TranscriptLine,
        provenance: Provenance::Web,
        content: json!({"line": n, "text": text}),
        commitment_key: CommitmentKey::from_bytes([n as u8; 32]),
        transcript_timestamp: Some("2026-01-01T00:00:00Z".to_owned()),
    }
}

/// A tombstone-like event the caller records with the erasure.
fn recorded(n: u64) -> NewEvent {
    NewEvent {
        kind: EventKind::Tombstone,
        provenance: Provenance::Structural,
        content: json!({"count": n}),
        commitment_key: CommitmentKey::from_bytes([200 + n as u8; 32]),
        transcript_timestamp: None,
    }
}

/// A store with writer 1 holding `lines`, and writer 2 the device seat.
fn filled(dir: &TempDir, lines: &[&str]) -> Store {
    let mut store = open(dir);
    seated(&mut store, 1);
    seated(&mut store, 2);
    let events: Vec<NewEvent> = lines.iter().zip(1..).map(|(t, n)| line(n, t)).collect();
    store.append(writer(1), &events).unwrap();
    store
}

/// A store as `filled`, the device seat holding an event already.
fn device_filled(dir: &TempDir, lines: &[&str]) -> Store {
    let mut store = filled(dir, lines);
    store.append(writer(2), &[line(50, "device")]).unwrap();
    store
}

fn range(first: u64, last: u64) -> Erase {
    Erase {
        writer: writer(1),
        first: seq(first),
        last: seq(last),
    }
}

fn find(store: &Store, text: &str) -> usize {
    let query = search::compile(&Request {
        text,
        ..Request::default()
    })
    .unwrap();
    let search = Search {
        query: &query,
        rooms: &[],
        writers: &[],
        provenance: &[],
        kinds: &[],
        since: None,
        until: None,
        k: 10,
        deadline: Duration::from_secs(2),
    };

    store.search(&search).unwrap().len()
}

use std::time::Duration;

#[test]
fn a_purge_erases_what_confirms_a_guess_and_keeps_the_chain() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["one", MARKER, "three"]);
    let before = store.log(writer(1)).unwrap();

    let purged = store
        .purge(writer(2), &[recorded(1)], &[range(2, 2)])
        .unwrap();

    let after = store.log(writer(1)).unwrap();
    assert_eq!(purged.events, 1);
    assert_eq!(purged.appended, [Address::new(writer(2), seq(1))]);
    assert!(purged.finished.vacuumed && !purged.finished.nothing);
    assert_eq!(after[1].content, None);
    assert_eq!(after[1].commitment_key, None);
    assert_eq!(after[1].payload, None);
    assert_eq!(after[1].transcript_timestamp, None);
    assert_eq!(after[1].link, before[1].link, "the header and hash stay");
    assert_eq!(after[0], before[0]);
    assert_eq!(after[2], before[2]);
    let links: Vec<_> = after.iter().map(|s| s.link.clone()).collect();
    chain::check_writer(writer(1), &links).unwrap();
}

#[test]
fn what_a_purge_records_lands_in_the_device_seat_s_log() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["one", "two"]);

    store
        .purge(writer(2), &[recorded(1), recorded(2)], &[range(1, 2)])
        .unwrap();

    let log = store.log(writer(2)).unwrap();
    assert_eq!(log.len(), 2);
    assert_eq!(log[1].content, Some(json!({"count": 2})));
    assert!(store.segments(writer(2)).unwrap().last().unwrap().closed);
}

#[test]
fn a_purge_that_cannot_record_erases_nothing() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["one", MARKER]);
    let mut bad = recorded(1);
    bad.content = json!({"x": 0.5});

    let err = store.purge(writer(2), &[bad], &[range(2, 2)]).unwrap_err();

    assert!(matches!(err, StoreError::Encoding(_)), "{err}");
    assert!(store.log(writer(1)).unwrap()[1].content.is_some());
    assert!(!store.purge_pending().unwrap());
}

#[test]
fn a_purged_event_is_no_search_hit_and_the_index_matches_a_rebuild() {
    let dir = TempDir::new().unwrap();
    let mut store = device_filled(&dir, &["one", MARKER, "three"]);
    assert_eq!(find(&store, MARKER), 1);

    let purged = store.purge(writer(2), &[], &[range(2, 2)]).unwrap();

    assert_eq!(purged.appended, []);
    assert_eq!(find(&store, MARKER), 0);
    assert_eq!(find(&store, "three"), 1);
    let kept = store.search_index_digest().unwrap();
    store.rebuild_search_index().unwrap();
    assert_eq!(store.search_index_digest().unwrap(), kept);
}

/// Every file under `dir`, with its bytes.
fn files(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(files(&path));
        } else {
            out.push((path.display().to_string(), std::fs::read(&path).unwrap()));
        }
    }

    out
}

fn holds(bytes: &[u8], needle: &[u8]) -> bool {
    bytes.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn nothing_of_a_purged_event_stays_in_any_file() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["one", MARKER]);
    let big = NewEvent {
        content: json!({"text": format!("{MARKER} {}", "lorem ".repeat(2000))}),
        ..line(3, MARKER)
    };
    store.append(writer(1), &[big]).unwrap();
    let key = CommitmentKey::from_bytes([3; 32]);
    let payload = store.log(writer(1)).unwrap()[2]
        .payload
        .clone()
        .unwrap()
        .name;
    assert!(dir.path().join("payloads").join(&payload).exists());
    assert!(
        files(dir.path())
            .iter()
            .any(|(_, b)| holds(b, MARKER.as_bytes()))
    );

    store
        .purge(writer(2), &[recorded(1)], &[range(2, 3)])
        .unwrap();

    assert!(!store.purge_pending().unwrap());
    for (path, bytes) in files(dir.path()) {
        assert!(
            !holds(&bytes, MARKER.as_bytes()),
            "{path} holds the content"
        );
        assert!(
            !holds(&bytes, key.to_hex().as_bytes()),
            "{path} holds the key"
        );
    }
    assert!(!dir.path().join("payloads").join(&payload).exists());
    assert!(store.log(writer(1)).unwrap()[0].content.is_some());
}

#[test]
fn the_prompt_commitment_a_purged_line_used_up_goes_with_it() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["a prompt", "other"]);
    for id in ["p-1", "p-2"] {
        store
            .conn
            .execute(
                "INSERT INTO prompt_commitments VALUES ('s', ?1, ?2, 'c', 1)",
                params![id, "k".repeat(64)],
            )
            .unwrap();
    }
    let matched = |event: Address, id: &str| NewEvent {
        kind: EventKind::PromptMatch,
        provenance: Provenance::Structural,
        content: json!({"event": event.to_string(), "prompt_id": id, "result": "matched"}),
        commitment_key: CommitmentKey::from_bytes([9; 32]),
        transcript_timestamp: None,
    };
    store
        .append(
            writer(2),
            &[
                matched(Address::new(writer(1), seq(1)), "p-1"),
                matched(Address::new(writer(1), seq(2)), "p-2"),
                // A match naming no address, or one of another writer.
                matched(Address::new(writer(2), seq(1)), "p-2"),
            ],
        )
        .unwrap();
    store
        .append(
            writer(2),
            &[NewEvent {
                content: json!({"event": 7, "prompt_id": 7}),
                ..matched(Address::new(writer(2), seq(1)), "x")
            }],
        )
        .unwrap();

    store.purge(writer(2), &[], &[range(1, 1)]).unwrap();

    let kept: Vec<(String, String)> = store
        .conn
        .prepare("SELECT prompt_id, commitment_key FROM prompt_commitments ORDER BY prompt_id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(
        kept,
        [
            ("p-1".to_owned(), String::new()),
            ("p-2".to_owned(), "k".repeat(64))
        ]
    );
}

/// A disk whose removals fail while `broken` holds.
struct Breaking(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl Disk for Breaking {
    fn write_synced(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        crate::OsDisk.write_synced(path, bytes)
    }
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        crate::OsDisk.rename(from, to)
    }
    fn sync_dir(&self, dir: &Path) -> std::io::Result<()> {
        crate::OsDisk.sync_dir(dir)
    }
    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        crate::OsDisk.read(path)
    }
    fn remove(&self, path: &Path) -> std::io::Result<()> {
        if self.0.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(std::io::Error::other("the disk is gone"));
        }
        crate::OsDisk.remove(path)
    }
}

use crate::Disk;

#[test]
fn a_purge_cut_short_is_finished_by_the_next_attempt() {
    use std::sync::atomic::Ordering;
    let dir = TempDir::new().unwrap();
    let broken = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let mut store = filled(&dir, &["one"]).with_disk(Box::new(Breaking(broken.clone())));
    let big = NewEvent {
        content: json!({"text": format!("{MARKER} {}", "lorem ".repeat(2000))}),
        ..line(2, MARKER)
    };
    store.append(writer(1), &[big]).unwrap();

    let err = store
        .purge(writer(2), &[recorded(1)], &[range(2, 2)])
        .unwrap_err();

    assert!(matches!(err, StoreError::Io(_)), "{err}");
    assert_eq!(
        store.log(writer(1)).unwrap()[1].content,
        None,
        "the erasure stands"
    );
    assert!(store.purge_pending().unwrap());
    assert_eq!(
        store.log(writer(2)).unwrap().len(),
        1,
        "and so does its record"
    );
    drop(store);
    broken.store(false, Ordering::SeqCst);
    let mut store = open(&dir).with_disk(Box::new(Breaking(broken)));

    let done = store.finish_purge().unwrap();

    assert_eq!(
        (done.payloads, done.vacuumed, done.nothing),
        (1, true, false)
    );
    assert!(!store.purge_pending().unwrap());
    for (path, bytes) in files(dir.path()) {
        assert!(!holds(&bytes, MARKER.as_bytes()), "{path}");
    }
    assert!(store.finish_purge().unwrap().nothing);
}

#[test]
fn the_ranges_a_purge_left_unfinished_are_named_until_it_finishes() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["one", "two", "three"]);
    assert_eq!(
        store.unfinished_purges().unwrap(),
        Vec::<String>::new(),
        "no journal yet"
    );
    store
        .purge(writer(2), &[recorded(1)], &[range(2, 3)])
        .unwrap();
    assert_eq!(store.unfinished_purges().unwrap(), Vec::<String>::new());
    store
        .conn
        .execute_batch(&format!(
            "INSERT INTO purge_journal (kind, target) VALUES ('index', '{0}:3-3'), \
             ('index', '{0}:2-2'), ('vacuum', '')",
            writer(1)
        ))
        .unwrap();

    assert_eq!(
        store.unfinished_purges().unwrap(),
        [format!("{}:2-2", writer(1)), format!("{}:3-3", writer(1))]
    );
    store.finish_purge().unwrap();
    assert_eq!(store.unfinished_purges().unwrap(), Vec::<String>::new());
}

#[test]
fn a_store_that_never_purged_has_nothing_to_finish() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["one"]);

    assert!(!store.purge_pending().unwrap());
    assert_eq!(
        store.finish_purge().unwrap(),
        Finished {
            nothing: true,
            ..Finished::default()
        }
    );
}

#[test]
fn a_log_another_connection_holds_open_leaves_the_vacuum_for_later() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["one", MARKER]);
    store.conn.busy_timeout(Duration::ZERO).unwrap();
    let reader = Store::open_read_only(&dir.path().join("store.db")).unwrap();
    reader.conn.execute_batch("BEGIN").unwrap();
    reader
        .conn
        .query_row("SELECT count(*) FROM events", [], |r| r.get::<_, i64>(0))
        .unwrap();
    store
        .append(writer(1), &[line(3, "after the reader began")])
        .unwrap();

    store.purge(writer(2), &[], &[range(2, 2)]).unwrap();

    assert!(store.purge_pending().unwrap());
    reader.conn.execute_batch("COMMIT").unwrap();
    let done = store.finish_purge().unwrap();
    assert!(done.vacuumed);
    assert!(!store.purge_pending().unwrap());
}

#[test]
fn a_journalled_range_that_does_not_parse_is_dropped() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["one"]);
    store.conn.execute_batch(SCHEMA).unwrap();
    journal(&store.conn, INDEX, "not a range").unwrap();

    let done = store.finish_purge().unwrap();

    assert_eq!(done.postings, 0);
    assert!(!store.purge_pending().unwrap());
}

#[test]
fn a_range_in_a_store_without_an_index_has_no_postings() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["one", "two"]);
    std::fs::remove_file(dir.path().join("search_index")).ok();
    store.conn.execute_batch("DETACH DATABASE idx").unwrap();

    store.purge(writer(2), &[], &[range(1, 2)]).unwrap();

    assert!(!store.purge_pending().unwrap());
}

#[test]
fn a_writer_that_does_not_hold_its_slot_has_no_postings_to_remove() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["one"]);
    let stranger = WriterId::derive(&[9; 32]);

    let removed = search_index::remove(&store.conn, stranger, seq(1), seq(5)).unwrap();

    assert_eq!(removed, 0);
    store
        .purge(
            writer(2),
            &[],
            &[Erase {
                writer: stranger,
                ..range(1, 5)
            }],
        )
        .unwrap();
}

#[test]
fn an_address_names_an_event_of_its_range_alone() {
    let r = range(2, 4);
    let a = |w: u8, n: u64| json!(Address::new(writer(w), seq(n)).to_string());

    assert!(names(&a(1, 2), &r) && names(&a(1, 4), &r));
    assert!(!names(&a(1, 1), &r) && !names(&a(1, 5), &r));
    assert!(!names(&a(2, 3), &r));
    for bad in [
        json!(7),
        json!("no colon"),
        json!("x:y"),
        json!(format!("{}:0", writer(1))),
    ] {
        assert!(!names(&bad, &r), "{bad}");
    }
}

#[test]
fn a_journalled_range_is_read_back() {
    let target = format!("{}:2-9", writer(1));

    assert_eq!(parse_range(&target), Some((writer(1), seq(2), seq(9))));
    for bad in [
        "",
        "x",
        &format!("{}:2", writer(1)),
        &format!("{}:a-b", writer(1)),
        "w:2-9",
    ] {
        assert_eq!(parse_range(bad), None, "{bad}");
    }
}

#[test]
fn a_prompt_match_of_a_purged_event_without_a_prompt_id_forgets_nothing() {
    let dir = TempDir::new().unwrap();
    let mut store = filled(&dir, &["a prompt"]);
    let named = Address::new(writer(1), seq(1)).to_string();
    let no_id = NewEvent {
        kind: EventKind::PromptMatch,
        provenance: Provenance::Structural,
        content: json!({"event": named, "prompt_id": 7}),
        commitment_key: CommitmentKey::from_bytes([9; 32]),
        transcript_timestamp: None,
    };
    store.append(writer(2), &[no_id]).unwrap();

    let purged = store.purge(writer(2), &[], &[range(1, 1)]).unwrap();

    assert_eq!(purged.events, 1);
}
