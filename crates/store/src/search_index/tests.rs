use super::*;
use crate::{NewEvent, SeatId, Store, StoredSeat};
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use search::Request;
use serde_json::json;
use testkit::TempDir;

fn room() -> RoomId {
    RoomId::derive(&[1; 32], &[2; 16])
}

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn open(dir: &TempDir) -> Store {
    Store::open(&dir.path().join("store.db")).unwrap()
}

fn seat(store: &mut Store, writer: WriterId, room: RoomId) {
    store
        .add_seat(&StoredSeat {
            writer,
            seat: SeatId::derive(&room, writer.digest().as_bytes()),
            room,
            certificate: json!({}),
        })
        .unwrap();
}

fn line(n: u64, text: &str, provenance: Provenance, time: Option<&str>) -> NewEvent {
    NewEvent {
        kind: EventKind::TranscriptLine,
        provenance,
        content: json!({"line": n, "text": text}),
        commitment_key: CommitmentKey::from_bytes([n as u8; 32]),
        transcript_timestamp: time.map(str::to_owned),
    }
}

fn texts(texts: &[&str]) -> Vec<NewEvent> {
    texts
        .iter()
        .zip(1..)
        .map(|(t, n)| line(n, t, Provenance::Unparsed, None))
        .collect()
}

fn compiled(text: &str) -> Compiled {
    search::compile(&Request {
        text,
        ..Request::default()
    })
    .unwrap()
}

fn find(store: &Store, text: &str) -> Vec<Hit> {
    let query = compiled(text);
    store.search(&base(&query)).unwrap()
}

fn base(query: &Compiled) -> Search<'_> {
    Search {
        query,
        room: room(),
        writer: None,
        provenance: &[],
        kinds: &[],
        since: None,
        until: None,
        k: 10,
        deadline: Duration::from_secs(60),
    }
}

/// A store with a seated writer 1 holding `events`.
fn with(dir: &TempDir, events: &[NewEvent]) -> Store {
    let mut store = open(dir);
    seat(&mut store, writer(1), room());
    store.append(writer(1), events).unwrap();
    store
}

fn seqs(hits: &[Hit]) -> Vec<u64> {
    hits.iter().map(|h| h.address.seq.get()).collect()
}

#[test]
fn text_is_the_contents_text_cut_at_the_cap_on_a_character_boundary() {
    let content = json!({"text": "aé"});

    assert_eq!(text_of(&content, 10), Some("aé"));
    assert_eq!(text_of(&content, 2), Some("a"));
    assert_eq!(text_of(&content, 0), Some(""));
    assert_eq!(text_of(&json!({"line": 1}), 10), None);
    assert_eq!(text_of(&json!({"text": 3}), 10), None);
    assert_eq!(text_of(&json!("bare"), 10), None);
}

#[test]
fn a_row_id_follows_the_address_and_a_newer_event_has_the_larger_one() {
    let slot = slot_of(writer(1));
    let (one, two) = (Seq::new(1).unwrap(), Seq::new(2).unwrap());

    assert_eq!(row_id(slot, one), row_id(slot_of(writer(1)), one));
    assert!(row_id(slot, two) > row_id(slot, one));
    assert!(row_id(slot_of(writer(2)), one) < row_id(slot, two));
    assert_eq!(row_id(slot, two) & SLOT_MASK, slot);
    assert!((0..=SLOT_MASK).contains(&slot));
}

#[test]
fn the_row_id_of_an_address_is_what_the_index_uses_and_none_past_the_limit() {
    let at = |seq| Address::new(writer(1), Seq::new(seq).unwrap());

    assert_eq!(
        row_id_of(at(7)),
        Some(row_id(slot_of(writer(1)), Seq::new(7).unwrap()))
    );
    assert_eq!(
        row_id_of(at(SEQ_LIMIT - 1)).unwrap() >> SLOT_BITS,
        (SEQ_LIMIT - 1) as i64
    );
    assert_eq!(row_id_of(at(SEQ_LIMIT)), None);
}

#[test]
fn a_new_store_has_its_index_and_records_how_it_is_built() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);

    assert!(exists(&store.conn).unwrap());
    let setting = |key: &str| -> Option<String> {
        store
            .conn
            .query_row(
                "SELECT value FROM idx.search_settings WHERE key = ?1",
                [key],
                |r| r.get(0),
            )
            .optional()
            .unwrap()
    };
    assert_eq!(setting("search_tokenizer").as_deref(), Some(TOKENIZER));
    assert_eq!(
        setting("search_index.indexing_cap"),
        Some(DEFAULT_CAP.to_string())
    );
}

#[test]
fn an_appended_event_is_found_by_the_words_of_its_text() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy the staging cluster", "lunch plans"]));

    assert_eq!(seqs(&find(&store, "staging")), [1]);
    assert_eq!(seqs(&find(&store, "STAGING deploy")), [1]);
    assert!(find(&store, "production").is_empty());
}

#[test]
fn a_hyphenated_term_matches_its_words_next_to_each_other() {
    let dir = TempDir::new().unwrap();
    let store = with(
        &dir,
        &texts(&["a quokka-early tool result", "early quokka"]),
    );

    assert_eq!(seqs(&find(&store, "quokka-early")), [1]);
}

#[test]
fn operators_in_caller_text_run_nothing() {
    let dir = TempDir::new().unwrap();
    let store = with(
        &dir,
        &texts(&["foo only", "foo OR bar together", "bar only"]),
    );

    assert_eq!(seqs(&find(&store, "foo OR bar")), [2]);
    assert!(find(&store, "foo NEAR(bar)").is_empty());
    assert_eq!(
        seqs(&find(&store, "bar -foo")),
        [2],
        "a minus is no negation"
    );
}

#[test]
fn text_beyond_the_indexing_cap_is_not_indexed() {
    let dir = TempDir::new().unwrap();
    let long = format!("quokka-early {} quokka-late", "x".repeat(DEFAULT_CAP));
    let store = with(&dir, &texts(&[&long]));

    assert_eq!(seqs(&find(&store, "quokka-early")), [1]);
    assert!(find(&store, "quokka-late").is_empty());
}

#[test]
fn payload_text_is_indexed_too() {
    let dir = TempDir::new().unwrap();
    let big = format!("{} needle", "padding ".repeat(2000));
    let store = with(&dir, &texts(&[&big]));

    let stored = store.log(writer(1)).unwrap();
    assert!(stored[0].payload.is_some());
    assert_eq!(seqs(&find(&store, "needle")), [1]);
}

#[test]
fn an_event_without_text_is_left_out_without_a_word_of_it() {
    let dir = TempDir::new().unwrap();
    let mut events = texts(&["one"]);
    events.push(NewEvent {
        kind: EventKind::KeyEvent,
        provenance: Provenance::Unparsed,
        content: json!({"key": "needle"}),
        commitment_key: CommitmentKey::from_bytes([9; 32]),
        transcript_timestamp: None,
    });
    let store = with(&dir, &events);

    assert!(find(&store, "needle").is_empty());
    assert_eq!(seqs(&find(&store, "one")), [1]);
}

#[test]
fn a_text_naming_a_word_five_times_ranks_above_one_naming_it_once() {
    let dir = TempDir::new().unwrap();
    let store = with(
        &dir,
        &texts(&[
            "wombat and four other words here",
            "wombat wombat wombat wombat wombat",
            "nothing relevant",
        ]),
    );

    let hits = find(&store, "wombat");

    assert_eq!(seqs(&hits), [2, 1]);
    assert!(hits[0].score > hits[1].score);
}

fn mixed(dir: &TempDir) -> Store {
    let mut store = open(dir);
    seat(&mut store, writer(1), room());
    seat(&mut store, writer(2), RoomId::derive(&[3; 32], &[4; 16]));
    store
        .append(
            writer(1),
            &[
                line(
                    1,
                    "deploy web",
                    Provenance::parse("web").unwrap(),
                    Some("2026-03-01T10:00:00Z"),
                ),
                line(
                    2,
                    "deploy file",
                    Provenance::parse("file").unwrap(),
                    Some("2026-07-01T10:00:00Z"),
                ),
                line(3, "deploy none", Provenance::parse("file").unwrap(), None),
            ],
        )
        .unwrap();
    store
        .append(
            writer(2),
            &[line(1, "deploy elsewhere", Provenance::Unparsed, None)],
        )
        .unwrap();
    store
}

#[test]
fn a_search_reaches_only_the_writers_of_its_room() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);

    let hits = find(&store, "deploy");

    assert!(hits.iter().all(|h| h.address.writer == writer(1)));
    assert_eq!(hits.len(), 3);
}

#[test]
fn a_run_filter_keeps_one_writer() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);
    let query = compiled("deploy");

    let none = store
        .search(&Search {
            writer: Some(writer(2)),
            ..base(&query)
        })
        .unwrap();
    let one = store
        .search(&Search {
            writer: Some(writer(1)),
            ..base(&query)
        })
        .unwrap();

    assert!(none.is_empty(), "writer 2 is in another room");
    assert_eq!(one.len(), 3);
}

#[test]
fn provenance_and_kind_filters_keep_the_events_they_name() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);
    let query = compiled("deploy");
    let with = |provenance: &[String], kinds: &[String]| {
        seqs(
            &store
                .search(&Search {
                    provenance,
                    kinds,
                    ..base(&query)
                })
                .unwrap(),
        )
    };
    let mut got = with(&["file".to_owned()], &[]);
    got.sort_unstable();

    assert_eq!(with(&["web".to_owned()], &[]), [1]);
    assert_eq!(got, [2, 3]);
    assert_eq!(with(&[], &["transcript_line".to_owned()]).len(), 3);
    assert!(with(&[], &["unparsed_line".to_owned()]).is_empty());
}

#[test]
fn a_time_range_keeps_events_whose_transcript_time_lies_in_it() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);
    let query = compiled("deploy");
    let range = |since, until| {
        let mut found = seqs(
            &store
                .search(&Search {
                    since,
                    until,
                    ..base(&query)
                })
                .unwrap(),
        );
        found.sort_unstable();
        found
    };

    assert_eq!(range(Some("2026-01-01"), None), [1, 2]);
    assert_eq!(range(None, Some("2026-06-30")), [1]);
    assert_eq!(range(Some("2026-03-01"), Some("2026-03-01")), [1]);
    assert_eq!(
        range(Some("2026-07-01T10:00:00Z"), Some("2026-07-01T10:00:00Z")),
        [2]
    );
    assert!(range(Some("2027-01-01"), None).is_empty());
}

#[test]
fn only_the_newest_matches_are_ranked() {
    let dir = TempDir::new().unwrap();
    let mut events = vec![line(
        1,
        "alpha alpha alpha alpha",
        Provenance::Unparsed,
        None,
    )];
    events.extend(
        (2..=RANKED_MATCHES as u64 + 1).map(|n| line(n, "alpha pad", Provenance::Unparsed, None)),
    );
    let store = with(&dir, &events);
    let query = compiled("alpha");

    let all = store
        .search(&Search {
            k: rank::MAX_HITS,
            ..base(&query)
        })
        .unwrap();

    assert!(
        !seqs(&all).contains(&1),
        "the oldest match lies outside the newest matches"
    );
    assert_eq!(all.len(), rank::MAX_HITS);
}

#[test]
fn a_search_past_its_deadline_is_cancelled_and_the_next_one_runs() {
    let dir = TempDir::new().unwrap();
    let events: Vec<NewEvent> = (1..=RANKED_MATCHES as u64)
        .map(|n| line(n, "alpha pad", Provenance::Unparsed, None))
        .collect();
    let store = with(&dir, &events);
    let query = compiled("alpha");

    let late = store.search(&Search {
        deadline: Duration::ZERO,
        ..base(&query)
    });
    let after = store.search(&base(&query));

    assert!(matches!(late, Err(StoreError::Cancelled)), "{late:?}");
    assert_eq!(after.unwrap().len(), 10);
}

#[test]
fn the_index_kept_per_append_has_the_digest_of_a_rebuild() {
    let dir = TempDir::new().unwrap();
    let mut store = mixed(&dir);
    let kept = store.search_index_digest().unwrap();

    let read = store.rebuild_search_index().unwrap();

    assert_eq!(read, 4);
    assert_eq!(store.search_index_digest().unwrap(), kept);
}

#[test]
fn two_stores_holding_the_same_logs_share_a_digest_whatever_arrived_first() {
    let appends = |order: [u8; 2]| {
        let dir = TempDir::new().unwrap();
        let mut store = open(&dir);
        for n in order {
            seat(&mut store, writer(n), room());
            store
                .append(writer(n), &texts(&["shared word", &format!("only {n}")]))
                .unwrap();
        }
        store.search_index_digest().unwrap()
    };

    assert_eq!(appends([1, 2]), appends([2, 1]));
    assert_ne!(appends([1, 2]), appends([1, 1 + 2]));
}

#[test]
fn a_different_text_is_a_different_digest() {
    let (one, two) = (TempDir::new().unwrap(), TempDir::new().unwrap());

    assert_ne!(
        with(&one, &texts(&["same"])).search_index_digest().unwrap(),
        with(&two, &texts(&["other"]))
            .search_index_digest()
            .unwrap()
    );
}

#[test]
fn a_store_made_before_the_index_gets_one_built_from_its_record() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy the cluster", "lunch"]));
    let kept = store.search_index_digest().unwrap();
    for table in ["events_vocab", "events_fts", "search_writers"] {
        store
            .conn
            .execute_batch(&format!("DROP TABLE {table}"))
            .unwrap();
    }
    drop(store);

    let store = open(&dir);

    assert_eq!(seqs(&find(&store, "cluster")), [1]);
    assert_eq!(store.search_index_digest().unwrap(), kept);
}

#[test]
fn a_store_opened_for_reading_with_no_index_says_so() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy"]));
    for table in ["events_vocab", "events_fts", "search_writers"] {
        store
            .conn
            .execute_batch(&format!("DROP TABLE {table}"))
            .unwrap();
    }
    drop(store);
    let mut read = Store::open_read_only(&dir.path().join("store.db")).unwrap();
    let query = compiled("deploy");

    assert!(matches!(
        read.search(&base(&query)),
        Err(StoreError::NoIndex)
    ));
    assert!(matches!(
        read.search_index_digest(),
        Err(StoreError::NoIndex)
    ));
    assert!(matches!(
        read.rebuild_search_index(),
        Err(StoreError::NoIndex)
    ));
}

/// A writer id that shares the slot of `writer(1)`: the same first five
/// bytes, then `tail`.
fn sharing_the_slot(tail: u8) -> WriterId {
    let mut bytes = *writer(1).digest().as_bytes();
    bytes[5..].fill(tail);

    WriterId::parse(&Digest::from_bytes(bytes).to_hex()).unwrap()
}

#[test]
fn of_two_writers_sharing_a_slot_the_smaller_id_is_indexed_whichever_arrived_first() {
    let (a, b) = (writer(1), sharing_the_slot(0));
    let (small, large) = if a < b { (a, b) } else { (b, a) };
    assert_eq!(slot_of(small), slot_of(large));
    let digests: Vec<_> = [[small, large], [large, small]]
        .into_iter()
        .map(|order| {
            let dir = TempDir::new().unwrap();
            let mut store = open(&dir);
            for w in order {
                seat(&mut store, w, room());
                store.append(w, &texts(&["shared word"])).unwrap();
            }
            let hits = find(&store, "shared");
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0].address.writer, small);
            assert_eq!(store.search_index_unindexed_writers().unwrap(), [large]);
            let kept = store.search_index_digest().unwrap();
            store.rebuild_search_index().unwrap();
            assert_eq!(store.search_index_digest().unwrap(), kept);
            kept
        })
        .collect();

    assert_eq!(digests[0], digests[1]);
}

#[test]
fn a_store_leaving_out_no_writer_names_none() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);

    assert!(store.search_index_unindexed_writers().unwrap().is_empty());
}

#[test]
fn an_unindexed_writer_id_that_does_not_parse_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);
    store
        .conn
        .execute(
            "UPDATE events SET writer = 'bad' WHERE writer = ?1",
            [writer(2).to_string()],
        )
        .unwrap();

    assert!(matches!(
        store.search_index_unindexed_writers(),
        Err(StoreError::Corrupt { .. })
    ));
}

#[test]
fn a_seq_past_the_limit_is_not_indexed() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy"]));
    let mut indexing = Indexing::open(&store.conn, writer(1)).unwrap();

    indexing
        .add(Seq::new(SEQ_LIMIT).unwrap(), &json!({"text": "beyond"}))
        .unwrap();

    assert!(find(&store, "beyond").is_empty());
}

#[test]
fn a_hit_naming_a_writer_that_is_no_id_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy"]));
    for table in ["search_writers", "seats", "events"] {
        store
            .conn
            .execute(&format!("UPDATE {table} SET writer = 'bad'"), [])
            .unwrap();
    }
    let query = compiled("deploy");

    assert!(matches!(
        store.search(&base(&query)),
        Err(StoreError::Corrupt { .. })
    ));
}

#[test]
fn a_rebuild_failing_midway_leaves_the_index_as_it_was() {
    let dir = TempDir::new().unwrap();
    let mut store = with(&dir, &texts(&["deploy", "lunch"]));
    let kept = store.search_index_digest().unwrap();
    store
        .conn
        .execute("UPDATE events SET kind = 'bad' WHERE seq = 2", [])
        .unwrap();

    assert!(store.rebuild_search_index().is_err());
    store
        .conn
        .execute(
            "UPDATE events SET kind = 'transcript_line' WHERE seq = 2",
            [],
        )
        .unwrap();

    assert_eq!(store.search_index_digest().unwrap(), kept);
}

/// A statement that runs for minutes.
const FOREVER: &str =
    "WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x + 1 FROM c WHERE x < 4000000000)
     SELECT count(*) FROM c";

#[expect(
    clippy::disallowed_methods,
    reason = "a test of time reads the clock to bound its own run"
)]
#[test]
fn work_that_outlasts_its_deadline_is_interrupted_and_cancelled() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    let started = std::time::Instant::now();

    let out = within(&store.conn, Duration::from_millis(50), || {
        Ok(store.conn.query_row(FOREVER, [], |r| r.get::<_, i64>(0))?)
    });

    assert!(matches!(out, Err(StoreError::Cancelled)), "{out:?}");
    assert!(started.elapsed() < Duration::from_secs(30));
}

#[test]
fn work_that_ends_in_time_returns_its_result_and_leaves_the_connection_alone() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);

    let out = within(&store.conn, Duration::from_secs(60), || Ok(7));

    assert_eq!(out.unwrap(), 7);
    assert_eq!(
        store
            .conn
            .query_row("SELECT 1", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn a_deadline_already_passed_cancels_the_work_before_it_starts() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    let mut ran = false;

    let out = within(&store.conn, Duration::ZERO, || {
        ran = true;
        Ok(())
    });

    assert!(matches!(out, Err(StoreError::Cancelled)));
    assert!(!ran);
}

#[test]
fn work_that_fails_for_another_reason_keeps_its_error() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);

    let out: Result<(), _> = within(&store.conn, Duration::from_secs(60), || {
        Err(StoreError::NoIndex)
    });

    assert!(matches!(out, Err(StoreError::NoIndex)));
}

#[test]
fn a_rebuild_leaves_out_an_event_whose_content_was_removed() {
    let dir = TempDir::new().unwrap();
    let mut store = with(&dir, &texts(&["deploy", "lunch"]));
    store
        .conn
        .execute("UPDATE events SET content = NULL WHERE seq = 1", [])
        .unwrap();

    assert_eq!(store.rebuild_search_index().unwrap(), 2);

    assert!(find(&store, "deploy").is_empty());
    assert_eq!(seqs(&find(&store, "lunch")), [2]);
}

#[test]
fn opening_a_store_twice_before_it_has_an_index_builds_it_once_whole() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy"]));
    let kept = store.search_index_digest().unwrap();
    store
        .conn
        .execute_batch("DROP TABLE events_vocab; DROP TABLE events_fts; DROP TABLE search_writers;")
        .unwrap();
    drop(store);

    drop(open(&dir));
    let again = open(&dir);

    assert_eq!(again.search_index_digest().unwrap(), kept);
    assert_eq!(seqs(&find(&again, "deploy")), [1]);
}

/// The index's view of the record `heads` describes.
fn current(store: &Store, heads: &[(WriterId, u64)]) -> bool {
    is_current(&store.conn, heads).unwrap()
}

#[test]
fn an_index_is_current_only_where_it_holds_every_writer_s_newest_event_and_no_more() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, writer(1), room());
    store.append(writer(1), &texts(&["a", "b"])).unwrap();

    assert!(current(&store, &[(writer(1), 2)]));
    assert!(!current(&store, &[(writer(1), 3)]), "behind the record");
    assert!(!current(&store, &[(writer(1), 1)]), "ahead of the record");
    assert!(!current(&store, &[]), "a writer the record lacks");
    assert!(current(&store, &[(writer(1), 2), (writer(2), 0)]));
    assert!(!current(&store, &[(writer(1), 2), (writer(2), 5)]));
}

#[test]
fn a_writer_that_lost_its_slot_to_a_smaller_id_leaves_the_index_current_and_one_that_did_not_does_not()
 {
    let (a, b) = (writer(1), sharing_the_slot(0));
    let (small, large) = if a < b { (a, b) } else { (b, a) };
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, small, room());
    store.append(small, &texts(&["shared"])).unwrap();

    assert!(current(&store, &[(small, 1), (large, 3)]), "the loser");
    store
        .conn
        .execute("UPDATE search_writers SET writer = ?1", [large.to_string()])
        .unwrap();
    assert!(
        !current(&store, &[(small, 1), (large, 1)]),
        "a larger id holds the slot a smaller one should"
    );
}

#[test]
fn a_reader_of_a_store_whose_index_file_is_gone_finds_no_index() {
    let dir = TempDir::new().unwrap();
    drop(with(&dir, &texts(&["deploy"])));
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(dir.path().join(format!("{FILE}{suffix}")));
    }

    let read = Store::open_read_only(&dir.path().join("store.db")).unwrap();

    assert!(!exists(&read.conn).unwrap());
    assert!(matches!(
        read.search(&base(&compiled("deploy"))),
        Err(StoreError::NoIndex)
    ));
}

#[cfg(unix)]
#[test]
fn an_index_file_whose_path_is_not_text_cannot_be_attached() {
    use std::os::unix::ffi::OsStrExt;
    let dir = TempDir::new().unwrap();
    let conn = Connection::open_in_memory().unwrap();
    let path = dir.path().join(std::ffi::OsStr::from_bytes(b"idx\xff"));

    assert!(matches!(attach(&conn, &path, true), Err(StoreError::Io(_))));
}

#[test]
fn a_writer_id_of_the_record_that_does_not_parse_makes_the_currency_check_corrupt() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy"]));
    store
        .conn
        .execute(
            "INSERT INTO seats (writer, seat, room, certificate) VALUES ('nope', 's', 'r', '{}')",
            [],
        )
        .unwrap();

    assert!(matches!(
        store.index_is_current(),
        Err(StoreError::Corrupt { .. })
    ));
}

#[test]
fn an_index_missing_its_tables_fails_to_take_settings_to_note_progress_or_to_digest() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy"]));
    let indexing = Indexing::open(&store.conn, writer(1)).unwrap();
    store
        .conn
        .execute_batch("DROP TABLE search_writers")
        .unwrap();
    assert!(indexing.done_to(1).is_err());
    store
        .conn
        .execute_batch("DROP TABLE search_settings")
        .unwrap();

    assert!(write_settings(&store.conn).is_err());
    assert!(digest(&store.conn).is_err());
}

fn colliding() -> (WriterId, WriterId) {
    let (a, b) = (writer(1), sharing_the_slot(0));

    if a < b { (a, b) } else { (b, a) }
}

fn gap(writer: WriterId, class: GapClass) -> IndexGap {
    IndexGap { writer, class }
}

#[test]
fn a_writer_without_a_slot_is_a_gap_and_a_writer_past_the_seq_limit_is_another() {
    let (small, large) = colliding();
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, small, room());
    store.append(small, &texts(&["shared"])).unwrap();
    let heads = [
        (small, 1),
        (large, 3),
        (writer(2), 0),
        (writer(3), SEQ_LIMIT),
    ];
    let mut found = gaps(&store.conn, &heads).unwrap();
    found.sort();
    let mut want = vec![
        gap(large, GapClass::WriterUnindexed),
        gap(writer(3), GapClass::WriterUnindexed),
        gap(writer(3), GapClass::SeqBeyondLimit),
    ];
    want.sort();

    assert_eq!(found, want);
}

#[test]
fn a_writer_whose_newest_seq_is_just_under_the_limit_is_no_gap() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, writer(1), room());
    store.append(writer(1), &texts(&["a"])).unwrap();

    assert!(gaps(&store.conn, &[(writer(1), 1)]).unwrap().is_empty());
    assert!(
        gaps(&store.conn, &[(writer(1), SEQ_LIMIT - 1)])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_store_names_the_writer_its_index_left_out_whichever_arrived_first() {
    let (small, large) = colliding();
    for order in [[small, large], [large, small]] {
        let dir = TempDir::new().unwrap();
        let mut store = open(&dir);
        for w in order {
            seat(&mut store, w, room());
            store.append(w, &texts(&["shared word"])).unwrap();
        }

        assert_eq!(
            store.search_index_gaps().unwrap(),
            [gap(large, GapClass::WriterUnindexed)]
        );
    }
}

#[test]
fn a_store_leaving_out_nothing_has_no_gap() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);

    assert!(store.search_index_gaps().unwrap().is_empty());
    assert!(store.unreported_search_index_gaps().unwrap().is_empty());
}

#[test]
fn a_gap_is_unreported_until_marked_and_stays_reported_through_a_rebuild() {
    let (small, large) = colliding();
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    for w in [small, large] {
        seat(&mut store, w, room());
        store.append(w, &texts(&["shared word"])).unwrap();
    }
    let found = store.unreported_search_index_gaps().unwrap();
    assert_eq!(found, [gap(large, GapClass::WriterUnindexed)]);

    store.mark_search_index_gaps_reported(&found).unwrap();
    store.mark_search_index_gaps_reported(&found).unwrap();
    assert!(store.unreported_search_index_gaps().unwrap().is_empty());
    store.rebuild_search_index().unwrap();
    store.append(large, &texts(&["more"])).unwrap();

    assert!(store.unreported_search_index_gaps().unwrap().is_empty());
    assert_eq!(store.search_index_gaps().unwrap().len(), 1);
}

#[test]
fn a_store_whose_index_lacks_the_note_of_reported_gaps_cannot_say_which_are() {
    let dir = TempDir::new().unwrap();
    let (small, large) = colliding();
    let mut store = open(&dir);
    for w in [small, large] {
        seat(&mut store, w, room());
        store.append(w, &texts(&["shared word"])).unwrap();
    }
    store
        .conn
        .execute_batch("DROP TABLE idx.search_gaps_reported")
        .unwrap();

    assert!(store.unreported_search_index_gaps().is_err());
    assert!(
        store
            .mark_search_index_gaps_reported(&[gap(large, GapClass::WriterUnindexed)])
            .is_err()
    );
    assert!(
        store
            .unmark_search_index_gaps_reported(&[gap(large, GapClass::WriterUnindexed)])
            .is_err()
    );
}

#[test]
fn a_gap_whose_report_did_not_reach_the_audit_log_can_be_unmarked_to_be_reported_again() {
    let (small, large) = colliding();
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    for w in [small, large] {
        seat(&mut store, w, room());
        store.append(w, &texts(&["shared word"])).unwrap();
    }
    let found = store.unreported_search_index_gaps().unwrap();
    store.mark_search_index_gaps_reported(&found).unwrap();

    store.unmark_search_index_gaps_reported(&found).unwrap();
    store.unmark_search_index_gaps_reported(&found).unwrap();

    assert_eq!(store.unreported_search_index_gaps().unwrap(), found);
}

#[test]
fn each_class_of_gap_has_its_counter_name() {
    assert_eq!(GapClass::WriterUnindexed.name(), "writer_unindexed");
    assert_eq!(GapClass::SeqBeyondLimit.name(), "seq_beyond_limit");
}

#[test]
fn a_search_over_the_matches_a_caller_keeps_chooses_them_before_the_cut() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);
    let query = compiled("deploy");
    let all = seqs(&store.search(&base(&query)).unwrap());
    let oldest = *all.iter().min().unwrap();

    let kept = store
        .search_where(
            &Search {
                k: 1,
                ..base(&query)
            },
            &|a| a.seq.get() == oldest,
        )
        .unwrap();

    assert_eq!(seqs(&kept), [oldest]);
}
