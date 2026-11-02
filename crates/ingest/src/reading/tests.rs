use super::*;
use key::random::OsEntropy;
use testkit::TempDir;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

#[test]
fn a_failed_lookup_of_the_gaps_is_said_beside_the_ingest_not_as_its_failure() {
    let gap = store::IndexGap {
        writer: writer(1),
        class: store::GapClass::WriterUnindexed,
    };

    assert_eq!(gaps(Ok(vec![gap])), (vec![gap], None));
    assert_eq!(
        gaps(Err(store::StoreError::NoIndex)),
        (
            Vec::new(),
            Some("the store has no search index: open it for writing once".to_owned())
        )
    );
}

#[test]
fn an_error_is_said_with_each_cause_behind_it() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
    let failed = store::StoreError::Io(io);

    let said = with_causes(&failed);

    assert!(said.starts_with(&failed.to_string()), "{said}");
    assert!(said.ends_with("no such file"), "{said}");
}

#[test]
fn a_transaction_holds_one_writer_s_consecutive_lines() {
    let (a, b) = (writer(1), writer(2));
    let ends = [
        (a, false, 1),
        (a, true, 2),
        (a, false, 3),
        (b, false, 4),
        (a, false, 5),
    ];

    assert_eq!(transactions(&ends, &BTreeSet::new()), [0..3, 3..4, 4..5]);
    assert_eq!(last_of_each_writer(&ends)[&a], 4);
    assert_eq!(last_of_each_writer(&ends)[&b], 3);
    assert!(transactions(&[], &BTreeSet::new()).is_empty());
}

#[test]
fn a_transaction_starts_at_each_index_an_observation_lands_before() {
    let ends: Vec<_> = (0..5).map(|i| (writer(1), false, i)).collect();

    assert_eq!(
        transactions(&ends, &BTreeSet::from([2, 4])),
        [0..2, 2..4, 4..5]
    );
}

#[test]
fn a_transaction_holds_at_most_a_batch() {
    let ends: Vec<_> = (0..250).map(|i| (writer(1), false, i)).collect();

    assert_eq!(
        transactions(&ends, &BTreeSet::new()),
        [0..100, 100..200, 200..250]
    );
}

#[test]
fn a_source_continues_while_its_consumed_prefix_stands_its_purged_lines_zeroed() {
    let mut source = Source {
        id: 1,
        path: "/t/a.jsonl".to_owned(),
        generation: 1,
        directory: "/t".to_owned(),
        position: Position {
            offset: 2,
            prefix: Digest::of(&[b"ab"]),
            lines: 1,
            seen: 2,
        },
        purged: Vec::new(),
        held_since: None,
        lost: false,
        marked_at: 0,
    };

    assert!(continues(&source, b"abc"));
    assert!(!continues(&source, b"xbc"));
    assert!(!continues(&source, b"a"));
    assert_eq!(as_index(3), 3);

    // A purge zeroed the bytes of the second line: the file still holds
    // them, and they count as zeros.
    source.purged = vec![store::source::Span {
        offset: 1,
        length: 1,
    }];
    source.position.prefix = Digest::of(&[b"a\0"]);
    assert!(continues(&source, b"abc"));
    assert!(!continues(&source, b"xbc"));
}

#[test]
fn a_run_whose_writer_has_no_seat_is_corrupt() {
    let err = no_seat(writer(1));

    assert!(err.to_string().contains("has no seat"), "{err}");
}

fn home(dir: &testkit::TempDir) -> Home {
    Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap()
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

fn transcript(dir: &testkit::TempDir, text: &str) -> std::path::PathBuf {
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, text).unwrap();
    path
}

fn numbered(session: &str, from: u64, to: u64) -> String {
    (from..=to)
        .map(|n| {
            format!(
                "{}\n",
                serde_json::json!({"type": "x", "uuid": format!("u{n}"), "sessionId": session, "n": n})
            )
        })
        .collect()
}

/// Erases the content of the first `count` events of `writer`, as a
/// purge does: the rows and their places stay.
fn purged(home: &Home, writer: record::id::WriterId, count: u64) {
    let mut store = Store::open(&home.store()).unwrap();
    let device = administration::node::Node::find(home, &store)
        .unwrap()
        .device_writer();
    let erase = store::Erase {
        writer,
        first: record::id::Seq::FIRST,
        last: record::id::Seq::new(count).unwrap(),
    };
    store.purge(device, &[], &[erase]).unwrap();
}

#[test]
fn a_line_whose_uuid_names_a_purged_event_is_refused_and_ingest_moves_past_it() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let parent = transcript(&dir, &numbered("p", 1, 5));
    let fork = dir.path().join("fork.jsonl");
    let mut text = numbered("f", 1, 3);
    text.push_str(&numbered("f", 100, 101));
    std::fs::write(&fork, text).unwrap();
    let owner = crate::ingest_path(&home, &parent, &redactor(), &mut OsEntropy).unwrap();
    purged(&home, owner.writer, 3);

    let late = crate::ingest_path(&home, &fork, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((late.refused, late.events, late.references), (3, 2, 0));
    assert_eq!(late.conflicts, 0, "no other content named the uuid");
    let store = Store::open(&home.store()).unwrap();
    assert_eq!(store.log(late.writer).unwrap().len(), 2);
    let again = crate::ingest_path(&home, &fork, &redactor(), &mut OsEntropy).unwrap();
    assert_eq!((again.refused, again.events), (0, 0));
}

#[test]
fn a_refusal_no_recorded_line_follows_still_moves_the_ingest_position() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let parent = transcript(&dir, &numbered("p", 1, 3));
    let fork = dir.path().join("fork.jsonl");
    std::fs::write(&fork, numbered("f", 1, 3)).unwrap();
    let owner = crate::ingest_path(&home, &parent, &redactor(), &mut OsEntropy).unwrap();
    purged(&home, owner.writer, 3);

    let late = crate::ingest_path(&home, &fork, &redactor(), &mut OsEntropy).unwrap();
    let again = crate::ingest_path(&home, &fork, &redactor(), &mut OsEntropy).unwrap();
    std::fs::write(
        &fork,
        format!("{}{}", numbered("f", 1, 3), numbered("f", 7, 7)),
    )
    .unwrap();
    let grown = crate::ingest_path(&home, &fork, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((late.refused, late.events), (3, 0));
    assert_eq!((again.refused, again.events), (0, 0));
    assert_eq!((grown.refused, grown.events), (0, 1));
    assert_eq!(grown.generation, late.generation, "a growth, not a rewrite");
}

#[test]
fn a_refusal_between_recorded_lines_keeps_the_prefix_the_position_digests() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let parent = transcript(&dir, &numbered("p", 1, 3));
    let fork = dir.path().join("fork.jsonl");
    let mut text = numbered("f", 100, 100);
    text.push_str(&numbered("f", 2, 2));
    text.push_str(&numbered("f", 101, 101));
    std::fs::write(&fork, &text).unwrap();
    let owner = crate::ingest_path(&home, &parent, &redactor(), &mut OsEntropy).unwrap();
    purged(&home, owner.writer, 3);

    let late = crate::ingest_path(&home, &fork, &redactor(), &mut OsEntropy).unwrap();
    text.push_str(&numbered("f", 102, 102));
    std::fs::write(&fork, &text).unwrap();
    let grown = crate::ingest_path(&home, &fork, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((late.refused, late.events), (1, 2));
    assert_eq!(
        grown.generation, late.generation,
        "the prefix still matches"
    );
    assert_eq!((grown.refused, grown.events), (0, 1));
}
