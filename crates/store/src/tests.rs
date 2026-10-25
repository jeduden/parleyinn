use super::*;
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use serde_json::json;
use testkit::TempDir;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn new_event(n: u64) -> NewEvent {
    NewEvent {
        kind: EventKind::UnparsedLine,
        provenance: Provenance::Unparsed,
        content: json!({"text": format!("line {n}")}),
        commitment_key: CommitmentKey::from_bytes([n as u8; 32]),
        transcript_timestamp: None,
    }
}

fn new_events(n: u64) -> Vec<NewEvent> {
    (1..=n).map(new_event).collect()
}

fn open(dir: &TempDir) -> Store {
    Store::open(&dir.path().join("store.db")).unwrap()
}

#[test]
fn a_new_store_records_its_version_and_the_header_field_set() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);

    assert_eq!(store.meta("schema_version").unwrap().as_deref(), Some("1"));
    assert_eq!(
        store.meta("header_field_set").unwrap().as_deref(),
        Some(FIELD_SET)
    );
    assert_eq!(
        store.meta("header_encoding").unwrap().as_deref(),
        Some(ENCODING)
    );
    assert_eq!(store.meta("nothing").unwrap(), None);
    assert!(store.writers().unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn the_store_file_is_readable_only_by_its_owner() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    drop(open(&dir));

    let mode = std::fs::metadata(dir.path().join("store.db"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[cfg(unix)]
#[test]
fn a_store_file_others_can_read_is_refused_and_left_as_it_is() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    drop(open(&dir));
    let path = dir.path().join("store.db");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    let err = Store::open(&path).unwrap_err();

    assert!(
        matches!(err, StoreError::Permissions { mode: 0o644 }),
        "{err}"
    );
    assert_eq!(
        err.to_string(),
        "the store file can be read by others (mode 644): fix its permissions to 600 (SEC-02)"
    );
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o644);
}

#[test]
fn a_store_reopens_with_what_it_held() {
    let dir = TempDir::new().unwrap();
    open(&dir).append(writer(1), &new_events(3)).unwrap();

    let store = open(&dir);

    assert_eq!(store.log(writer(1)).unwrap().len(), 3);
}

#[test]
fn a_store_closed_without_a_checkpoint_leaves_its_log_to_the_next_opener() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    let wal = dir.path().join("store.db-wal");
    drop(open(&dir));
    assert!(
        !wal.exists(),
        "a plain close checkpoints and removes the log"
    );

    let mut store = open(&dir).without_checkpoint_on_close().unwrap();
    store.append(writer(1), &new_events(2)).unwrap();
    drop(store);

    assert!(std::fs::metadata(&wal).unwrap().len() > 0);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(reopened.log(writer(1)).unwrap().len(), 2);
}

#[test]
fn a_store_of_a_newer_version_is_refused_and_named() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    store
        .conn
        .execute(
            "UPDATE meta SET value = '2' WHERE key = 'schema_version'",
            [],
        )
        .unwrap();
    drop(store);

    let err = Store::open(&dir.path().join("store.db")).unwrap_err();

    assert!(matches!(err, StoreError::Newer { found: 2 }), "{err}");
    assert_eq!(
        err.to_string(),
        "the store is schema version 2, newer than this parinn's 1"
    );
}

#[test]
fn a_store_whose_version_does_not_parse_is_refused() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    store
        .conn
        .execute(
            "UPDATE meta SET value = 'x' WHERE key = 'schema_version'",
            [],
        )
        .unwrap();
    drop(store);

    let err = Store::open(&dir.path().join("store.db")).unwrap_err();

    assert!(
        matches!(err, StoreError::Corrupt { address: None, .. }),
        "{err}"
    );
}

#[test]
fn a_file_that_is_no_database_is_refused() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("store.db"),
        "not sqlite at all, but long enough to read a header from it",
    )
    .unwrap();
    owner_only_file(&dir.path().join("store.db"));

    let err = Store::open(&dir.path().join("store.db")).unwrap_err();

    assert!(matches!(err, StoreError::Sqlite(_)), "{err}");
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn append_numbers_each_writer_from_1_and_chains_its_events() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);

    let a = store.append(writer(1), &new_events(3)).unwrap();
    let b = store.append(writer(1), &new_events(2)).unwrap();

    let seqs: Vec<u64> = a.iter().chain(&b).map(|x| x.seq.get()).collect();
    assert_eq!(seqs, [1, 2, 3, 4, 5]);
    let log = store.log(writer(1)).unwrap();
    let links: Vec<Link> = log.iter().map(|s| s.link.clone()).collect();
    assert_eq!(record::chain::check_writer(writer(1), &links), Ok(()));
    assert_eq!(log[0].link.header.prev_hash, Digest::ZERO);
    assert_eq!(log[1].link.header.prev_hash, log[0].link.hash);
}

#[test]
fn an_event_keeps_its_content_and_commitment_key_beside_the_header() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let [address] = store.append(writer(1), &[new_event(7)]).unwrap()[..] else {
        panic!("one address")
    };

    let stored = store.event(address).unwrap().unwrap();

    let d = new_event(7);
    assert_eq!(stored.content, Some(d.content.clone()));
    assert_eq!(stored.commitment_key, Some(d.commitment_key.clone()));
    assert_eq!(
        stored.link.header.commitment,
        record::commitment::commit(&d.commitment_key, &d.content).unwrap()
    );
    assert_eq!(stored.link.header.kind, EventKind::UnparsedLine);
    assert_eq!(stored.link.header.provenance, Provenance::Unparsed);
    assert_eq!(
        store
            .event(Address::new(writer(1), Seq::new(2).unwrap()))
            .unwrap(),
        None
    );
}

#[test]
fn the_first_event_of_a_transaction_cites_the_heads_that_advanced_since_its_writer_last_cited() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let (a, b) = (writer(1), writer(2));

    store.append(a, &new_events(2)).unwrap();
    store.append(b, &new_events(2)).unwrap();
    store.append(b, &new_events(1)).unwrap();
    store.append(a, &new_events(2)).unwrap();
    store.append(a, &new_events(1)).unwrap();

    let la = store.log(a).unwrap();
    let lb = store.log(b).unwrap();
    let cites = |s: &Stored| s.link.header.heads.clone();
    assert_eq!(
        cites(&lb[0]),
        [SeenHead::new(la[1].link.header.address, la[1].link.hash)]
    );
    assert!(cites(&lb[1]).is_empty());
    assert!(cites(&lb[2]).is_empty(), "a's head did not advance");
    assert_eq!(
        cites(&la[2]),
        [SeenHead::new(lb[2].link.header.address, lb[2].link.hash)]
    );
    assert!(cites(&la[3]).is_empty());
    assert!(cites(&la[4]).is_empty(), "b's head did not advance");
    let mut all = std::collections::BTreeMap::new();
    all.insert(a, la.iter().map(|s| s.link.clone()).collect());
    all.insert(b, lb.iter().map(|s| s.link.clone()).collect());
    assert_eq!(record::chain::check_heads(&all), Ok(()));
}

#[test]
fn a_new_event_without_a_canonical_encoding_rolls_back_its_transaction_alone() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let mut batch = new_events(250);
    // Event 160 fails: a hundred to a transaction keeps 100, where fifty
    // would keep 150 and two hundred none.
    batch[159].content = json!({"x": 0.5});

    let err = store.append(writer(1), &batch).unwrap_err();

    assert!(matches!(err, StoreError::Encoding(_)), "{err}");
    assert_eq!(store.log(writer(1)).unwrap().len(), 100);
}

#[test]
fn seats_are_kept_by_writer() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let room = RoomId::derive(&[1; 32], &[2; 16]);
    let seat = StoredSeat {
        writer: writer(1),
        seat: SeatId::derive(&room, &[1; 32]),
        room,
        certificate: json!({"any": "value"}),
    };

    store.add_seat(&seat).unwrap();

    assert_eq!(store.seat(writer(1)).unwrap(), Some(seat.clone()));
    assert_eq!(store.seat(writer(2)).unwrap(), None);
    assert_eq!(store.seats().unwrap(), std::slice::from_ref(&seat));
    assert!(matches!(store.add_seat(&seat), Err(StoreError::Sqlite(_))));
}

#[test]
fn writers_lists_every_writer_with_an_event_or_a_seat() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(2), &new_events(1)).unwrap();
    store.append(writer(1), &new_events(1)).unwrap();

    let mut want = vec![writer(1), writer(2)];
    want.sort();
    assert_eq!(store.writers().unwrap(), want);
}

/// Sets one column of one stored event to `value`.
fn set(store: &Store, column: &str, value: &str) {
    store
        .conn
        .execute(
            &format!("UPDATE events SET {column} = ?1 WHERE seq = 2"),
            [value],
        )
        .unwrap();
}

#[test]
fn a_stored_row_that_does_not_parse_is_corrupt_and_names_its_event() {
    for (column, value) in [
        ("kind", "message"),
        ("provenance", "system"),
        ("prev_hash", "x"),
        ("commitment", "x"),
        ("hash", "x"),
        ("content", "{"),
        ("commitment_key", "x"),
    ] {
        let dir = TempDir::new().unwrap();
        let mut store = open(&dir);
        store.append(writer(1), &new_events(3)).unwrap();
        set(&store, column, value);

        let err = store.log(writer(1)).unwrap_err();

        let at = Address::new(writer(1), Seq::new(2).unwrap());
        match &err {
            StoreError::Corrupt {
                address: Some(a),
                what,
            } => {
                assert_eq!(*a, at, "{column}");
                assert!(what.contains(column), "{column}: {what}");
            }
            other => panic!("{column}: {other}"),
        }
        assert!(
            err.to_string()
                .starts_with(&format!("the stored event {at} is corrupt: ")),
            "{err}"
        );
        assert!(store.event(at).is_err(), "{column}");
    }
}

#[test]
fn a_cited_head_that_does_not_parse_is_corrupt_naming_its_column() {
    for (column, value, what) in [
        ("other_hash", "'x'", "cited head hash"),
        ("other_writer", "'x'", "cited head writer"),
        ("other_seq", "0", "cited head seq"),
    ] {
        let dir = TempDir::new().unwrap();
        let mut store = open(&dir);
        store.append(writer(1), &new_events(1)).unwrap();
        store.append(writer(2), &new_events(1)).unwrap();
        store
            .conn
            .execute(&format!("UPDATE event_heads SET {column} = {value}"), [])
            .unwrap();

        let err = store.log(writer(2)).unwrap_err();

        let at = Address::new(writer(2), Seq::FIRST);
        assert!(
            matches!(&err, StoreError::Corrupt { address: Some(a), what: w } if *a == at && w == what),
            "{column}: {err}"
        );
    }
}

#[test]
fn a_seat_row_or_writer_id_that_does_not_parse_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let room = RoomId::derive(&[1; 32], &[2; 16]);
    store
        .add_seat(&StoredSeat {
            writer: writer(1),
            seat: SeatId::derive(&room, &[1; 32]),
            room,
            certificate: json!({}),
        })
        .unwrap();
    for (column, value) in [
        ("certificate", "{"),
        ("room", "x"),
        ("seat", "x"),
        ("writer", "x"),
    ] {
        store
            .conn
            .execute(&format!("UPDATE seats SET {column} = ?1"), [value])
            .unwrap();
        let err = store.seats().unwrap_err();
        assert!(
            matches!(&err, StoreError::Corrupt { address: None, what } if *what == format!("seat {column}")),
            "{column}: {err}"
        );
    }
    store.append(writer(1), &new_events(1)).unwrap();
    store
        .conn
        .execute("UPDATE events SET writer = 'x'", [])
        .unwrap();
    assert!(matches!(store.writers(), Err(StoreError::Corrupt { .. })));
}

#[test]
fn a_purged_event_keeps_its_header_without_content() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(1)).unwrap();
    store
        .conn
        .execute(
            "UPDATE events SET content = NULL, commitment_key = NULL",
            [],
        )
        .unwrap();

    let stored = store.log(writer(1)).unwrap().remove(0);

    assert_eq!((stored.content, stored.commitment_key), (None, None));
}

#[test]
fn errors_explain_themselves() {
    let corrupt = StoreError::Corrupt {
        address: None,
        what: "x".into(),
    };
    assert_eq!(corrupt.to_string(), "the store is corrupt: x");
    let enc = record::canonical::Error::Number("0.5".into());
    assert_eq!(
        StoreError::Encoding(enc.clone()).to_string(),
        enc.to_string()
    );
    assert!(std::error::Error::source(&StoreError::Encoding(enc)).is_some());
    assert!(std::error::Error::source(&corrupt).is_none());
    let full = StoreError::Full(writer(1));
    assert_eq!(
        full.to_string(),
        format!("writer {} has no seq left", writer(1))
    );
    let io = StoreError::Io(std::io::Error::other("x"));
    assert_eq!(io.to_string(), "the store file: x");
    assert!(std::error::Error::source(&io).is_some());
}

#[test]
fn a_stored_seq_outside_the_seqs_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(1)).unwrap();
    store
        .conn
        .execute("UPDATE events SET seq = -1", [])
        .unwrap();

    let err = store.log(writer(1)).unwrap_err();

    assert!(
        matches!(&err, StoreError::Corrupt { address: None, what } if what.contains("seq -1")),
        "{err}"
    );
}

#[test]
fn an_append_after_a_corrupt_newest_hash_is_refused() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(2)).unwrap();
    store
        .conn
        .execute("UPDATE events SET hash = 'x' WHERE seq = 2", [])
        .unwrap();

    let err = store.append(writer(1), &new_events(1)).unwrap_err();

    let at = Address::new(writer(1), Seq::new(2).unwrap());
    assert!(
        matches!(err, StoreError::Corrupt { address: Some(a), .. } if a == at),
        "{err}"
    );
}

#[test]
fn an_append_that_would_cite_a_corrupt_head_is_refused() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(1)).unwrap();
    store
        .conn
        .execute("UPDATE events SET hash = 'x'", [])
        .unwrap();

    let err = store.append(writer(2), &new_events(1)).unwrap_err();

    let at = Address::new(writer(1), Seq::FIRST);
    assert!(
        matches!(err, StoreError::Corrupt { address: Some(a), .. } if a == at),
        "{err}"
    );
}

#[test]
fn a_store_shows_its_path_and_a_sqlite_error_its_cause() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);

    assert!(format!("{store:?}").contains("store.db"));
    let err = StoreError::Sqlite(rusqlite::Error::InvalidQuery);
    assert!(err.to_string().starts_with("the store: "), "{err}");
}

fn seat_in(room: RoomId, n: u8) -> StoredSeat {
    StoredSeat {
        writer: writer(n),
        seat: SeatId::derive(&room, &[n; 32]),
        room,
        certificate: json!({"n": n}),
    }
}

#[test]
fn an_event_cites_only_the_heads_of_writers_in_its_own_room() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let (here, there) = (
        RoomId::derive(&[1; 32], &[1; 16]),
        RoomId::derive(&[2; 32], &[2; 16]),
    );
    for seat in [seat_in(here, 1), seat_in(here, 2), seat_in(there, 3)] {
        store.add_seat(&seat).unwrap();
    }
    store.append(writer(1), &new_events(1)).unwrap();
    store.append(writer(3), &new_events(1)).unwrap();

    store.append(writer(2), &new_events(1)).unwrap();

    let cited: Vec<WriterId> = store.log(writer(2)).unwrap()[0]
        .link
        .header
        .heads
        .iter()
        .map(|s| s.address.writer)
        .collect();
    assert_eq!(cited, [writer(1)]);
}

#[test]
fn a_store_whose_header_field_set_or_encoding_differs_is_refused() {
    for key in ["header_field_set", "header_encoding"] {
        let dir = TempDir::new().unwrap();
        let store = open(&dir);
        store
            .conn
            .execute("UPDATE meta SET value = 'other' WHERE key = ?1", [key])
            .unwrap();
        drop(store);

        let err = Store::open(&dir.path().join("store.db")).unwrap_err();

        assert!(
            matches!(&err, StoreError::Format { key: k, found } if k == key && found == "other"),
            "{err}"
        );
        assert_eq!(
            err.to_string(),
            format!("the store records {key} \"other\", which this parinn does not write")
        );
    }
}

#[cfg(unix)]
#[test]
fn a_new_store_file_is_created_readable_only_by_its_owner_before_sqlite_opens_it() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");

    create_owner_only(&path).unwrap();
    create_owner_only(&path).unwrap();

    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
    assert!(matches!(
        create_owner_only(&dir.path().join("no/such/dir/store.db")),
        Err(StoreError::Io(_))
    ));
}

#[test]
fn a_snapshot_reads_the_store_as_it_stood_when_it_began() {
    let dir = TempDir::new().unwrap();
    let mut reader = open(&dir);
    let mut writer_side = open(&dir);
    reader.append(writer(1), &new_events(1)).unwrap();

    let seen = reader
        .snapshot(|store| {
            writer_side.append(writer(2), &new_events(1))?;
            store.writers()
        })
        .unwrap();

    assert_eq!(seen, [writer(1)]);
    assert_eq!(reader.writers().unwrap().len(), 2);
}

#[test]
fn a_snapshot_passes_on_its_closure_s_error() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);

    let err = store
        .snapshot(|_| -> Result<(), StoreError> { Err(StoreError::Full(writer(1))) })
        .unwrap_err();

    assert!(matches!(err, StoreError::Full(_)));
    assert!(store.writers().is_ok(), "the snapshot ended");
}

#[test]
fn a_seat_and_its_writer_s_first_events_land_in_one_transaction() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let room = RoomId::derive(&[1; 32], &[1; 16]);
    let seat = seat_in(room, 1);
    let mut bad = new_events(2);
    bad[1].content = json!({"x": 0.5});

    let err = store
        .add_seat_and_append(&seat, writer(9), &bad)
        .unwrap_err();

    assert!(matches!(err, StoreError::Encoding(_)));
    assert_eq!(store.seats().unwrap(), []);
    assert!(store.log(writer(9)).unwrap().is_empty());

    let addresses = store
        .add_seat_and_append(&seat, writer(9), &new_events(2))
        .unwrap();

    assert_eq!(addresses.len(), 2);
    assert_eq!(store.seats().unwrap(), std::slice::from_ref(&seat));
    assert!(matches!(
        store.add_seat_and_append(&seat, writer(9), &new_events(1)),
        Err(StoreError::Sqlite(_))
    ));
    assert_eq!(store.log(writer(9)).unwrap().len(), 2);
}

#[test]
fn a_writer_at_the_last_seq_appends_nothing_more() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store.append(writer(1), &new_events(1)).unwrap();
    store
        .conn
        .execute(
            &format!("UPDATE events SET seq = {}", record::canonical::MAX_SAFE),
            [],
        )
        .unwrap();

    let err = store.append(writer(1), &new_events(1)).unwrap_err();

    assert!(
        matches!(err, StoreError::Full(w) if w == writer(1)),
        "{err}"
    );
}

/// Makes the file at `path` readable by its owner alone, as a store file
/// must be before it is opened (SEC-02).
fn owner_only_file(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// An event whose content is `bytes` of text long.
fn sized_event(n: u8, bytes: usize) -> NewEvent {
    NewEvent {
        kind: EventKind::TranscriptLine,
        provenance: Provenance::ToolResult("Bash".into()),
        content: json!({"line": n, "text": "x".repeat(bytes)}),
        commitment_key: CommitmentKey::from_bytes([n; 32]),
        transcript_timestamp: None,
    }
}

#[test]
fn content_above_the_threshold_goes_to_the_payload_store_and_reads_back_whole() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let big = sized_event(1, 20 << 10);

    let [address] = store.append(writer(1), std::slice::from_ref(&big)).unwrap()[..] else {
        panic!("one address")
    };

    let stored = store.event(address).unwrap().unwrap();
    assert_eq!(stored.content.as_ref(), Some(&big.content));
    let payload = stored.payload.clone().unwrap();
    let canonical = record::canonical::encode(&big.content).unwrap();
    assert_eq!(payload.name, payload::name(&big.commitment_key));
    assert_eq!(payload.size, canonical.len() as u64);
    assert_eq!(payload.preview, "x".repeat(512));
    assert_eq!(
        stored.link.header.commitment,
        record::commitment::commit(&big.commitment_key, &big.content).unwrap()
    );
    let file = dir.path().join("payloads").join(&payload.name);
    assert_eq!(std::fs::read_to_string(file).unwrap(), canonical);
    let inline: Option<String> = store
        .conn
        .query_row("SELECT content FROM events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(inline, None, "the row keeps no copy of the content");
}

#[test]
fn content_at_or_below_the_threshold_stays_inline() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let small = sized_event(1, 2 << 10);

    store
        .append(writer(1), std::slice::from_ref(&small))
        .unwrap();

    let stored = store.log(writer(1)).unwrap().remove(0);
    assert_eq!(stored.content, Some(small.content));
    assert_eq!(stored.payload, None);
    assert!(!dir.path().join("payloads").exists());
}

#[test]
fn the_payload_threshold_is_set_per_store() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir).with_payload_threshold(100);
    let canonical = record::canonical::encode(&sized_event(1, 90).content).unwrap();
    assert!(canonical.len() > 100);

    store
        .append(writer(1), &[sized_event(1, 90), sized_event(2, 10)])
        .unwrap();

    let log = store.log(writer(1)).unwrap();
    assert!(log[0].payload.is_some());
    assert!(log[1].payload.is_none());
    assert_eq!(DEFAULT_PAYLOAD_THRESHOLD, 8192);
}

#[test]
fn a_payload_goes_through_the_store_s_disk() {
    use std::sync::{Arc, Mutex};
    #[derive(Clone, Default)]
    struct Count(Arc<Mutex<usize>>);
    impl Disk for Count {
        fn write_synced(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            *self.0.lock().unwrap() += 1;
            OsDisk.write_synced(path, bytes)
        }
        fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
            OsDisk.rename(from, to)
        }
        fn sync_dir(&self, dir: &Path) -> std::io::Result<()> {
            OsDisk.sync_dir(dir)
        }
        fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            OsDisk.read(path)
        }
    }
    let dir = TempDir::new().unwrap();
    let count = Count::default();
    let mut store = open(&dir).with_disk(Box::new(count.clone()));

    store.append(writer(1), &[sized_event(1, 9000)]).unwrap();

    assert_eq!(*count.0.lock().unwrap(), 1);
}

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
fn a_payload_that_is_not_utf_8_is_corrupt() {
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
        matches!(&err, StoreError::Corrupt { address: Some(_), what } if what.contains("not UTF-8")),
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
    let mut spilled = open(&TempDir::new().unwrap()).with_payload_threshold(10);
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
