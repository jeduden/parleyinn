use super::*;

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
fn a_source_continues_while_its_consumed_prefix_stands() {
    let position = Position {
        offset: 2,
        prefix: Digest::of(&[b"ab"]),
        lines: 1,
        seen: 2,
    };

    assert!(continues(&position, b"abc"));
    assert!(!continues(&position, b"xbc"));
    assert!(!continues(&position, b"a"));
    assert_eq!(as_index(3), 3);
}

#[test]
fn the_directory_s_other_transcripts_give_their_uuids() {
    let dir = testkit::TempDir::new().unwrap();
    let own = dir.path().join("own.jsonl");
    std::fs::write(&own, "{\"uuid\":\"mine\"}\n").unwrap();
    std::fs::write(
        dir.path().join("other.jsonl"),
        "{\"uuid\":\"u1\"}\nnot json\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("notes.txt"), "{\"uuid\":\"u2\"}\n").unwrap();
    std::fs::create_dir(dir.path().join("sub.jsonl")).unwrap();

    let found = sibling_uuids(&dir.path().to_string_lossy(), &own.to_string_lossy());

    assert_eq!(found, BTreeSet::from(["u1".to_owned()]));
    assert!(sibling_uuids(&dir.path().join("none").to_string_lossy(), "").is_empty());
}

#[test]
fn a_run_whose_writer_has_no_seat_is_corrupt() {
    let err = no_seat(writer(1));

    assert!(err.to_string().contains("has no seat"), "{err}");
}
