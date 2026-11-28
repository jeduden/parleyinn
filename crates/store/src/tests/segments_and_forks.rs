//! The store's payloads, segments, previews and forks: the second half of
//! the store's tests, moved from `tests.rs` to keep each file in its limit.

use super::*;
use record::event::{EventKind, Provenance};
use serde_json::json;
use testkit::TempDir;

#[test]
fn a_payload_that_cannot_be_read_names_its_event() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &[sized_event(1, 9000)]).unwrap();
    let name = store.log(writer(1)).unwrap()[0]
        .payload
        .clone()
        .unwrap()
        .name;
    std::fs::remove_file(dir.path().join("payloads").join(name)).unwrap();

    let err = store.log(writer(1)).unwrap_err();

    let at = Address::new(writer(1), Seq::FIRST);
    assert!(
        matches!(&err, StoreError::Payload { address, .. } if *address == at),
        "{err}"
    );
    assert!(
        err.to_string()
            .starts_with(&format!("the payload of {at}: ")),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn a_payload_that_does_not_parse_or_whose_row_is_off_is_corrupt() {
    for column in ["payload_size", "preview", "payload"] {
        let dir = TempDir::new().unwrap();
        let mut store = open(&dir);
        store.append(writer(1), &[sized_event(1, 9000)]).unwrap();
        if column == "payload_size" {
            store
                .conn
                .execute("UPDATE events SET payload_size = -1", [])
                .unwrap();
        } else {
            store
                .conn
                .execute(&format!("UPDATE events SET {column} = NULL"), [])
                .unwrap();
        }

        let err = store.log(writer(1)).unwrap_err();

        assert!(
            matches!(
                err,
                StoreError::Corrupt {
                    address: Some(_),
                    ..
                }
            ),
            "{column}: {err}"
        );
    }
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &[sized_event(1, 9000)]).unwrap();
    let name = store.log(writer(1)).unwrap()[0]
        .payload
        .clone()
        .unwrap()
        .name;
    std::fs::write(dir.path().join("payloads").join(name), "{").unwrap();
    assert!(matches!(
        store.log(writer(1)),
        Err(StoreError::Corrupt {
            address: Some(_),
            ..
        })
    ));
}

#[test]
fn a_failed_payload_write_rolls_back_its_transaction() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("payloads"), "a file in its place").unwrap();
    let mut store = open(&dir);

    let err = store
        .append(writer(1), &[sized_event(1, 9000)])
        .unwrap_err();

    assert!(matches!(err, StoreError::Payload { .. }), "{err}");
    assert!(store.log(writer(1)).unwrap().is_empty());
}

#[test]
fn a_transcript_timestamp_is_kept_as_metadata_and_never_made_up() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let mut stamped = new_event(1);
    stamped.transcript_timestamp = Some("2026-10-17T12:00:00.000Z".into());

    store.append(writer(1), &[stamped, new_event(2)]).unwrap();

    let log = store.log(writer(1)).unwrap();
    assert_eq!(
        log[0].transcript_timestamp.as_deref(),
        Some("2026-10-17T12:00:00.000Z")
    );
    assert_eq!(log[1].transcript_timestamp, None);
    assert!(
        !log[0].link.header.canonical().contains("2026"),
        "the header never carries it"
    );
}

fn segment(w: WriterId, first: u64, last: u64, closed: bool) -> Segment {
    Segment {
        writer: w,
        first: Seq::new(first).unwrap(),
        last: Seq::new(last).unwrap(),
        format_version: SEGMENT_VERSION,
        closed,
    }
}

#[test]
fn a_core_append_closes_the_segment_it_ends() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);

    store.append(writer(1), &new_events(3)).unwrap();
    store.append(writer(1), &new_events(2)).unwrap();

    assert_eq!(
        store.segments(writer(1)).unwrap(),
        [
            segment(writer(1), 1, 3, true),
            segment(writer(1), 4, 5, true)
        ]
    );
    assert!(store.open_segments().unwrap().is_empty());
}

#[test]
fn an_open_append_extends_the_open_segment_until_it_closes() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);

    store.append_open(writer(1), &new_events(2)).unwrap();
    store.append_open(writer(1), &new_events(250)).unwrap();
    assert_eq!(
        store.open_segments().unwrap(),
        [segment(writer(1), 1, 252, false)]
    );
    store.append(writer(1), &new_events(1)).unwrap();
    store.append_open(writer(1), &new_events(1)).unwrap();
    store.close_segment(writer(1)).unwrap();
    store.close_segment(writer(1)).unwrap();
    store.close_segment(writer(2)).unwrap();

    assert_eq!(
        store.segments(writer(1)).unwrap(),
        [
            segment(writer(1), 1, 253, true),
            segment(writer(1), 254, 254, true)
        ]
    );
    assert!(store.segments(writer(2)).unwrap().is_empty());
}

#[test]
fn a_rolled_back_append_leaves_its_segment_as_it_stood() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append_open(writer(1), &new_events(2)).unwrap();
    let mut bad = new_events(2);
    bad[1].content = json!({"x": 0.5});

    assert!(store.append(writer(1), &bad).is_err());

    assert_eq!(
        store.segments(writer(1)).unwrap(),
        [segment(writer(1), 1, 2, false)]
    );
}

#[test]
fn a_seat_s_first_events_close_their_segment() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let room = RoomId::derive(&[1; 32], &[1; 16]);

    store
        .add_seat_and_append(&seat_in(room, 1), writer(1), &new_events(2))
        .unwrap();

    assert_eq!(
        store.segments(writer(1)).unwrap(),
        [segment(writer(1), 1, 2, true)]
    );
}

/// Marks writer 1's segment starting at `first` as of format version
/// `version`.
fn set_segment_version(store: &Store, first: i64, version: i64) {
    store
        .conn
        .execute(
            "UPDATE segments SET format_version = ?1 WHERE first_seq = ?2",
            [version, first],
        )
        .unwrap();
}

#[test]
fn a_segment_of_a_newer_format_version_is_refused_named_and_left_unchanged() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(3)).unwrap();
    store.append(writer(1), &new_events(2)).unwrap();
    store.append(writer(2), &new_events(1)).unwrap();
    set_segment_version(&store, 4, 2);
    let mut newer = segment(writer(1), 4, 5, true);
    newer.format_version = 2;

    for err in [
        store.log(writer(1)).unwrap_err(),
        store
            .event(Address::new(writer(1), Seq::FIRST))
            .unwrap_err(),
        store.append(writer(1), &new_events(1)).unwrap_err(),
        store.append_open(writer(1), &new_events(1)).unwrap_err(),
        store.close_segment(writer(1)).unwrap_err(),
    ] {
        assert!(
            matches!(&err, StoreError::NewerSegment(s) if *s == newer),
            "{err}"
        );
        assert_eq!(
            err.to_string(),
            format!(
                "the segment of writer {} from seq 4 to 5 is format version 2, newer than this parinn's 1",
                writer(1)
            )
        );
    }
    assert_eq!(store.newer_segments().unwrap(), [newer.clone()]);
    assert_eq!(
        store.log(writer(2)).unwrap().len(),
        1,
        "another writer reads on"
    );
    assert_eq!(
        store.segments(writer(1)).unwrap()[1],
        newer,
        "left unchanged"
    );
}

#[test]
fn a_segment_row_that_does_not_parse_is_corrupt() {
    for (column, value) in [("first_seq", -1), ("last_seq", 0), ("format_version", -1)] {
        let dir = TempDir::new().unwrap();
        let mut store = open(&dir);
        store.append(writer(1), &new_events(1)).unwrap();
        store
            .conn
            .execute(&format!("UPDATE segments SET {column} = ?1"), [value])
            .unwrap();

        let err = store.segments(writer(1)).unwrap_err();

        assert!(matches!(err, StoreError::Corrupt { .. }), "{column}: {err}");
    }
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(1)).unwrap();
    store
        .conn
        .execute("UPDATE segments SET writer = 'x', closed = 0", [])
        .unwrap();
    assert!(matches!(
        store.open_segments(),
        Err(StoreError::Corrupt { .. })
    ));
}

#[test]
fn a_newer_store_is_refused_before_anything_in_it_changes() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    let store = open(&dir);
    store
        .conn
        .pragma_update(None, "journal_mode", "DELETE")
        .unwrap();
    store
        .conn
        .execute(
            "UPDATE meta SET value = '2' WHERE key = 'schema_version'",
            [],
        )
        .unwrap();
    drop(store);
    let before = std::fs::read(&path).unwrap();

    assert!(matches!(
        Store::open(&path),
        Err(StoreError::Newer { found: 2 })
    ));
    assert!(matches!(
        Store::open_read_only(&path),
        Err(StoreError::Newer { found: 2 })
    ));

    assert_eq!(std::fs::read(&path).unwrap(), before);
    let mode: String = rusqlite::Connection::open(&path)
        .unwrap()
        .pragma_query_value(None, "journal_mode", |r| r.get(0))
        .unwrap();
    assert_eq!(
        mode, "delete",
        "the refused store's journal is left as it was"
    );
}

#[test]
fn a_store_whose_first_opener_made_no_schema_reads_as_none_yet() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    std::fs::write(&path, "").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    assert!(Store::open_read_only_made(&path).unwrap().is_none());
    assert!(matches!(
        Store::open_read_only(&path),
        Err(StoreError::Corrupt { what, .. }) if what == "it holds no schema"
    ));
    open(&dir);
    assert!(Store::open_read_only_made(&path).unwrap().is_some());
}

#[test]
fn a_read_only_store_reads_what_the_store_holds_and_makes_nothing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");

    assert!(Store::open_read_only(&path).is_err());
    assert!(!path.exists(), "a read creates no store");

    open(&dir).append_open(writer(1), &new_events(2)).unwrap();
    let store = Store::open_read_only(&path).unwrap();

    assert_eq!(
        store.open_segments().unwrap(),
        [segment(writer(1), 1, 2, false)]
    );
    assert_eq!(store.meta("schema_version").unwrap().as_deref(), Some("1"));
}

#[cfg(unix)]
#[test]
fn a_read_only_store_others_can_read_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    drop(open(&dir));
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    assert!(matches!(
        Store::open_read_only(&path),
        Err(StoreError::Permissions { .. })
    ));
}

#[test]
fn a_read_only_store_without_its_schema_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    drop(rusqlite::Connection::open(&path).unwrap());
    owner_only_file(&path);

    assert!(matches!(
        Store::open_read_only(&path),
        Err(StoreError::Corrupt { address: None, .. })
    ));
}

#[test]
fn a_payload_that_is_not_utf_8_is_a_payload_fault() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &[sized_event(1, 9000)]).unwrap();
    let name = store.log(writer(1)).unwrap()[0]
        .payload
        .clone()
        .unwrap()
        .name;
    std::fs::write(dir.path().join("payloads").join(name), [0xff, 0xfe]).unwrap();

    let err = store.log(writer(1)).unwrap_err();

    assert!(
        matches!(&err, StoreError::Payload { source, .. } if source.to_string().contains("not UTF-8")),
        "{err}"
    );
}

#[test]
fn a_segment_that_cannot_be_recorded_rolls_back_its_append() {
    for (setup, trigger) in [
        (0, "BEFORE INSERT ON segments"),
        (2, "BEFORE UPDATE ON segments"),
    ] {
        let dir = TempDir::new().unwrap();
        let mut store = open(&dir);
        if setup > 0 {
            store.append_open(writer(1), &new_events(setup)).unwrap();
        }
        store
            .conn
            .execute_batch(&format!(
                "CREATE TRIGGER refuse {trigger} BEGIN SELECT RAISE(ABORT, 'refused'); END"
            ))
            .unwrap();

        let err = store.append(writer(1), &new_events(1)).unwrap_err();

        assert!(matches!(err, StoreError::Sqlite(_)), "{trigger}: {err}");
        assert_eq!(store.log(writer(1)).unwrap().len(), setup as usize);
    }
}

#[test]
fn a_store_whose_segments_cannot_be_read_fails() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(1)).unwrap();
    store.conn.execute_batch("DROP TABLE segments").unwrap();

    assert!(matches!(store.log(writer(1)), Err(StoreError::Sqlite(_))));
    assert!(matches!(
        store.segments(writer(1)),
        Err(StoreError::Sqlite(_))
    ));
    assert!(matches!(
        store.close_segment(writer(1)),
        Err(StoreError::Sqlite(_))
    ));
}

#[test]
fn a_newer_segment_row_that_does_not_parse_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(1)).unwrap();
    store
        .conn
        .execute("UPDATE segments SET format_version = 2, first_seq = 0", [])
        .unwrap();

    let err = store.log(writer(1)).unwrap_err();

    assert!(
        matches!(&err, StoreError::Corrupt { address: None, what } if what.contains("first seq 0")),
        "{err}"
    );
}

#[test]
fn a_seat_recorded_with_no_event_opens_no_segment() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let room = RoomId::derive(&[1; 32], &[1; 16]);

    let addresses = store
        .add_seat_and_append(&seat_in(room, 1), writer(1), &[])
        .unwrap();

    assert!(addresses.is_empty());
    assert_eq!(store.seats().unwrap().len(), 1);
    assert!(store.segments(writer(1)).unwrap().is_empty());
}

fn address(seq: u64) -> Address {
    Address::new(writer(1), Seq::new(seq).unwrap())
}

#[test]
fn a_preview_is_the_start_of_the_text_and_the_classes_are_the_events() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(1)).unwrap();

    assert_eq!(
        store.preview(address(1)).unwrap().as_deref(),
        Some("line 1")
    );
    assert_eq!(
        store.kind_and_provenance(address(1)).unwrap(),
        Some((EventKind::UnparsedLine, Provenance::Unparsed))
    );
}

#[test]
fn a_long_text_is_previewed_to_512_bytes_inline_or_in_the_payload_store() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let long = |n| NewEvent {
        content: json!({"text": "é".repeat(n)}),
        ..new_event(1)
    };
    store.append(writer(1), &[long(300)]).unwrap();
    let spilled_dir = TempDir::new().unwrap();
    let mut spilled = open(&spilled_dir).with_payload_threshold(10);
    spilled.append(writer(1), &[long(2000)]).unwrap();

    assert_eq!(store.preview(address(1)).unwrap().unwrap().len(), 512);
    assert_eq!(
        spilled.preview(address(1)).unwrap().unwrap().len(),
        payload::PREVIEW_BYTES
    );
}

#[test]
fn the_preview_of_content_with_no_text_is_its_encoding() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let bare = NewEvent {
        content: json!({"k": 1}),
        ..new_event(1)
    };
    store.append(writer(1), &[bare]).unwrap();

    assert_eq!(
        store.preview(address(1)).unwrap().as_deref(),
        Some(r#"{"k":1}"#)
    );
}

#[test]
fn an_event_that_is_not_there_has_no_preview_and_one_without_content_none_to_show() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(1)).unwrap();
    store
        .conn
        .execute("UPDATE events SET content = NULL", [])
        .unwrap();

    assert!(store.preview(address(2)).unwrap().is_none());
    assert!(store.kind_and_provenance(address(2)).unwrap().is_none());
    assert_eq!(store.preview(address(1)).unwrap().as_deref(), Some(""));
}

#[test]
fn a_row_that_does_not_parse_is_corrupt_for_a_preview_or_its_classes() {
    for (column, value) in [("content", "{"), ("kind", "bad"), ("provenance", "bad")] {
        let dir = TempDir::new().unwrap();
        let mut store = open(&dir);
        store.append(writer(1), &new_events(1)).unwrap();
        store
            .conn
            .execute(&format!("UPDATE events SET {column} = '{value}'"), [])
            .unwrap();

        let err = if column == "content" {
            store.preview(address(1)).unwrap_err()
        } else {
            store.kind_and_provenance(address(1)).unwrap_err()
        };

        assert!(matches!(err, StoreError::Corrupt { .. }), "{column}: {err}");
    }
}

#[test]
fn a_search_error_reads_as_fixed_text() {
    assert_eq!(
        StoreError::NoIndex.to_string(),
        "the store has no search index: open it for writing once"
    );
    assert_eq!(
        StoreError::Cancelled.to_string(),
        "the search ran past its deadline and was cancelled"
    );
}

/// Two stores whose writer 1 recorded `shared` same events, then the
/// first `held` more and the second `other` more, of other content.
fn two_histories(shared: u64, held: u64, other: u64) -> (TempDir, Store, TempDir, Store) {
    let (da, db) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let (mut a, mut b) = (open(&da), open(&db));
    a.append(writer(1), &new_events(shared)).unwrap();
    b.append(writer(1), &new_events(shared)).unwrap();
    let more = |from: u64, n: u64, tag: &str| -> Vec<NewEvent> {
        (from..from + n)
            .map(|k| NewEvent {
                content: json!({"text": format!("{tag} {k}")}),
                ..new_event(k)
            })
            .collect()
    };
    a.append(writer(1), &more(shared + 1, held, "a")).unwrap();
    b.append(writer(1), &more(shared + 1, other, "b")).unwrap();

    (da, a, db, b)
}

fn links(store: &Store, w: WriterId) -> Vec<Link> {
    store.log(w).unwrap().into_iter().map(|s| s.link).collect()
}

#[test]
fn a_second_history_of_one_writer_is_refused_and_the_first_head_stays() {
    let (_da, mut a, _db, b) = two_histories(2, 3, 2);
    let first = links(&a, writer(1));

    let new = a.refuse_forks(&links(&b, writer(1))).unwrap();

    assert_eq!(new.len(), 1);
    let fork = new[0];
    assert_eq!(fork.writer, writer(1));
    assert_eq!((fork.first.get(), fork.last.get()), (3, 4));
    assert_eq!(fork.held, first[2].hash);
    assert_eq!(fork.offered, links(&b, writer(1))[2].hash);
    assert_eq!(links(&a, writer(1)), first);
    assert_eq!(a.refused_forks().unwrap(), new);
}

#[test]
fn a_fork_refused_twice_is_new_once() {
    let (_da, mut a, _db, b) = two_histories(1, 2, 2);
    let offered = links(&b, writer(1));

    assert_eq!(a.refuse_forks(&offered).unwrap().len(), 1);
    assert_eq!(a.refuse_forks(&offered).unwrap(), vec![]);
    assert_eq!(a.refused_forks().unwrap().len(), 1);
}

#[test]
fn a_fork_that_cannot_be_recorded_fails_the_store_and_keeps_no_refusal() {
    let (_da, mut a, _db, b) = two_histories(2, 3, 2);
    a.conn
        .execute_batch(&format!(
            "{} CREATE TRIGGER refuse BEFORE INSERT ON fork_refusals
             BEGIN SELECT RAISE(ABORT, 'refused'); END;",
            refusal::SCHEMA
        ))
        .unwrap();

    let err = a.refuse_forks(&links(&b, writer(1))).unwrap_err();

    assert!(matches!(err, StoreError::Sqlite(_)), "{err}");
    assert_eq!(a.refused_forks().unwrap(), vec![]);
}

#[test]
fn refusals_that_cannot_be_read_fail_the_store() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    store
        .conn
        .execute_batch("CREATE TABLE fork_refusals (unrelated TEXT)")
        .unwrap();

    assert!(matches!(store.refused_forks(), Err(StoreError::Sqlite(_))));
}

#[test]
fn a_copy_of_the_same_history_refuses_nothing_and_makes_no_table() {
    let (_da, mut a, _db, b) = two_histories(3, 0, 2);

    assert_eq!(a.refuse_forks(&links(&b, writer(1))).unwrap(), vec![]);
    assert_eq!(a.refused_forks().unwrap(), vec![]);
    let table: i64 = a
        .conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE name = 'fork_refusals'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(table, 0);
}

#[test]
fn the_kept_origins_are_empty_until_written_and_then_replaced_whole() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    assert!(store.origins().unwrap().is_empty());

    let first: BTreeMap<_, _> = [(writer(1), "witnessed".to_owned())].into();
    store.replace_origins(&first).unwrap();
    assert_eq!(store.origins().unwrap(), first);

    let second: BTreeMap<_, _> = [(writer(2), "peer".to_owned())].into();
    store.replace_origins(&second).unwrap();
    assert_eq!(store.origins().unwrap(), second);
}

#[test]
fn a_store_keeps_the_identity_a_worktree_resolves_to_and_replaces_them_all() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);

    assert_eq!(store.worktree_identity("w").unwrap(), None);
    assert!(store.worktree_identities().unwrap().is_empty());
    store.set_worktree_identity("w", "h").unwrap();
    store.set_worktree_mark("reported:w", "m").unwrap();
    assert_eq!(store.worktree_identity("w").unwrap().as_deref(), Some("h"));

    let derived = BTreeMap::from([("v".to_owned(), "i".to_owned())]);
    store.replace_worktree_identities(&derived).unwrap();
    assert_eq!(store.worktree_identities().unwrap(), derived);
    // The marks are the node's own, and stay.
    assert_eq!(
        store.worktree_mark("reported:w").unwrap().as_deref(),
        Some("m")
    );
    store.forget_worktree_mark("reported:w").unwrap();
    assert_eq!(store.worktree_mark("reported:w").unwrap(), None);
}

#[test]
fn a_bind_appends_its_event_and_keeps_its_identity_in_one_unit_or_neither() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let w = writer(1);

    // Nothing is kept for the worktree: the bind lands whole.
    let bound = store
        .bind_worktree(w, &new_event(1), "wt", None, "one")
        .unwrap();
    assert!(bound.is_some());
    assert_eq!(
        store.worktree_identity("wt").unwrap().as_deref(),
        Some("one")
    );
    assert_eq!(store.log(w).unwrap().len(), 1);

    // Another process bound it since the caller looked: nothing changes.
    let raced = store
        .bind_worktree(w, &new_event(2), "wt", None, "two")
        .unwrap();
    assert_eq!(raced, None);
    assert_eq!(
        store.worktree_identity("wt").unwrap().as_deref(),
        Some("one")
    );
    assert_eq!(store.log(w).unwrap().len(), 1);

    // The caller saw what is kept: the bind replaces it.
    store
        .bind_worktree(w, &new_event(3), "wt", Some("one"), "three")
        .unwrap();
    assert_eq!(
        store.worktree_identity("wt").unwrap().as_deref(),
        Some("three")
    );
    assert_eq!(store.log(w).unwrap().len(), 2);

    // A table that takes no write rolls the event back with it.
    store.conn.execute_batch("DROP TABLE repository_worktrees; CREATE TABLE repository_worktrees (worktree TEXT PRIMARY KEY, identity TEXT NOT NULL CHECK (identity <> 'bad'));").unwrap();
    let refused = store.bind_worktree(w, &new_event(4), "wt2", None, "bad");
    assert!(refused.is_err());
    assert_eq!(store.log(w).unwrap().len(), 2);
}
