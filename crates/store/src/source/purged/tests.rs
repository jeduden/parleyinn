use super::*;
use crate::source::Zeroed;
use crate::{NewEvent, Place, Position, Store};
use record::commitment::CommitmentKey;
use record::digest::Digest;
use record::event::{EventKind, Provenance};
use record::id::{Seq, WriterId};
use serde_json::json;
use testkit::TempDir;

/// Five lines of four bytes each, a line break included.
const BODY: &[u8] = b"aaa\nbbb\nccc\nddd\neee\n";

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

fn range(writer: WriterId, first: u64, last: u64) -> Erase {
    Erase {
        writer,
        first: seq(first),
        last: seq(last),
    }
}

fn read(
    path: &Path,
    offset: u64,
    first: &[Span],
    second: &[Span],
) -> io::Result<Option<(Digest, Digest)>> {
    let bytes = std::fs::read(path)?;
    Ok(crate::source::prefix_digest(&bytes, offset, first)
        .zip(crate::source::prefix_digest(&bytes, offset, second)))
}

fn unreadable(_: &Path, _: u64, _: &[Span], _: &[Span]) -> io::Result<Option<(Digest, Digest)>> {
    Err(io::Error::other("unreadable"))
}

fn zeroed(lines: &[usize]) -> Vec<u8> {
    let mut bytes = BODY.to_vec();
    for line in lines {
        bytes[(line - 1) * 4..line * 4].fill(0);
    }

    bytes
}

fn digest(bytes: &[u8]) -> Digest {
    Digest::of(&[bytes])
}

/// A store holding `BODY` of the file `a.jsonl` in `dir`, ingested as
/// five lines of `writer`.
fn ingested(dir: &TempDir, writer: WriterId, name: &str) -> (Store, Source, String) {
    let path = dir.path().join(name);
    std::fs::write(&path, BODY).unwrap();
    let path = path.to_string_lossy().into_owned();
    let mut store = Store::open(&dir.path().join("store.db")).unwrap();
    let source = store.start_generation(&path, "/t").unwrap();
    let events: Vec<NewEvent> = (1..=5)
        .map(|n| NewEvent {
            kind: EventKind::TranscriptLine,
            provenance: Provenance::HarnessText,
            content: json!({"line": n}),
            commitment_key: CommitmentKey::from_bytes([n as u8; 32]),
            transcript_timestamp: None,
        })
        .collect();
    let places: Vec<Place> = (1..=5)
        .map(|n| Place {
            line: n,
            offset: (n - 1) * 4,
            length: 4,
            uuid: Some(format!("u{n}")),
        })
        .collect();
    let position = Position {
        offset: 20,
        prefix: digest(BODY),
        lines: 5,
        seen: 20,
    };
    store
        .append_lines(writer, &events, &places, &[false; 5], source.id, &position)
        .unwrap();
    let source = store.source(&path).unwrap().unwrap();

    (store, source, path)
}

#[test]
fn zeroing_hashes_the_bytes_of_each_span_as_zeros_whichever_way_they_arrive() {
    let spans = [
        Span {
            offset: 4,
            length: 4,
        },
        Span {
            offset: 10,
            length: 5,
        },
    ];
    let mut expected = BODY.to_vec();
    expected[4..8].fill(0);
    expected[10..15].fill(0);
    for cut in 0..=BODY.len() {
        let mut hash = Zeroed::new(&spans);
        hash.update(&BODY[..cut]);
        hash.update(&BODY[cut..]);
        assert_eq!(hash.digest(), digest(&expected), "cut at {cut}");
    }
    let mut bytewise = Zeroed::new(&spans);
    for byte in BODY {
        bytewise.update(&[*byte]);
    }
    assert_eq!(bytewise.digest(), digest(&expected));
    assert_eq!(Zeroed::new(&[]).digest(), digest(&[]));
}

#[test]
fn a_prefix_digest_wants_the_bytes_it_covers() {
    let span = [Span {
        offset: 4,
        length: 4,
    }];

    assert_eq!(
        crate::source::prefix_digest(BODY, 8, &span),
        Some(digest(&zeroed(&[2])[..8]))
    );
    assert_eq!(
        crate::source::prefix_digest(BODY, 0, &[]),
        Some(digest(&[]))
    );
    assert_eq!(crate::source::prefix_digest(BODY, 21, &[]), None);
    assert_ne!(unconfirmable(), digest(&[]));
}

#[test]
fn a_purge_zeroes_its_lines_and_keeps_only_their_place_length_and_harness_uuid() {
    let dir = TempDir::new().unwrap();
    let (mut store, source, path) = ingested(&dir, writer(1), "a.jsonl");
    store = store.with_transcripts(read);

    store
        .purge(writer(2), &[], &[range(writer(1), 2, 3)])
        .unwrap();

    let now = store.source(&path).unwrap().unwrap();
    assert_eq!(now.position.prefix, digest(&zeroed(&[2, 3])));
    assert_eq!(
        now.purged,
        [
            Span {
                offset: 4,
                length: 4
            },
            Span {
                offset: 8,
                length: 4
            }
        ]
    );
    assert_eq!(now.position.offset, source.position.offset);
    let kept: Vec<(u64, u64)> = store
        .placed(source.id)
        .unwrap()
        .iter()
        .map(|p| (p.place.offset, p.place.length))
        .collect();
    assert_eq!(kept, [(0, 4), (12, 4), (16, 4)]);
    let residue: Vec<(u64, u64, Option<String>)> = store
        .purged_places(source.id)
        .unwrap()
        .into_iter()
        .map(|p| (p.offset, p.length, p.uuid))
        .collect();
    assert_eq!(
        residue,
        [(4, 4, Some("u2".to_owned())), (8, 4, Some("u3".to_owned()))]
    );
    let associations: i64 = store
        .conn
        .query_row(
            "SELECT count(*) FROM transcript_places p JOIN events e ON e.rowid = p.event
             WHERE e.writer = ?1 AND e.seq BETWEEN 2 AND 3",
            [writer(1).to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(associations, 0, "purged residue names no event row");
    assert_eq!(std::fs::read(&path).unwrap(), BODY);
}

#[test]
fn a_second_purge_and_a_repeated_range_zero_each_line_once() {
    let dir = TempDir::new().unwrap();
    let (store, _, path) = ingested(&dir, writer(1), "a.jsonl");
    let mut store = store.with_transcripts(read);

    store
        .purge(writer(2), &[], &[range(writer(1), 2, 2)])
        .unwrap();
    store
        .purge(
            writer(2),
            &[],
            &[range(writer(1), 2, 2), range(writer(1), 5, 5)],
        )
        .unwrap();

    let now = store.source(&path).unwrap().unwrap();
    assert_eq!(now.position.prefix, digest(&zeroed(&[2, 5])));
    assert_eq!(now.purged.len(), 2);
}

#[test]
fn ranges_of_two_sources_each_recompute_their_own_hash() {
    let dir = TempDir::new().unwrap();
    let (store, _, first) = ingested(&dir, writer(1), "a.jsonl");
    drop(store);
    let path = dir.path().join("b.jsonl");
    std::fs::write(&path, BODY).unwrap();
    let mut store = Store::open(&dir.path().join("store.db"))
        .unwrap()
        .with_transcripts(read);
    let second = store
        .start_generation(&path.to_string_lossy(), "/t")
        .unwrap();
    let event = NewEvent {
        kind: EventKind::TranscriptLine,
        provenance: Provenance::HarnessText,
        content: json!({"line": 1}),
        commitment_key: CommitmentKey::from_bytes([9; 32]),
        transcript_timestamp: None,
    };
    let position = Position {
        offset: 20,
        prefix: digest(BODY),
        lines: 5,
        seen: 20,
    };
    let place = Place {
        line: 1,
        offset: 0,
        length: 4,
        uuid: None,
    };
    store
        .append_lines(
            writer(3),
            &[event],
            &[place],
            &[false],
            second.id,
            &position,
        )
        .unwrap();

    store
        .purge(
            writer(2),
            &[],
            &[range(writer(1), 1, 1), range(writer(3), 1, 1)],
        )
        .unwrap();

    let a = store.source(&first).unwrap().unwrap();
    let b = store.source(&path.to_string_lossy()).unwrap().unwrap();
    assert_eq!(a.position.prefix, digest(&zeroed(&[1])));
    assert_eq!(b.position.prefix, digest(&zeroed(&[1])));
}

#[test]
fn a_purge_that_cannot_read_the_transcript_leaves_a_hash_that_confirms_nothing() {
    for reader in [None, Some(unreadable as Transcripts)] {
        let dir = TempDir::new().unwrap();
        let (store, _, path) = ingested(&dir, writer(1), "a.jsonl");
        let mut store = match reader {
            Some(reader) => store.with_transcripts(reader),
            None => store,
        };

        store
            .purge(writer(2), &[], &[range(writer(1), 2, 2)])
            .unwrap();

        let now = store.source(&path).unwrap().unwrap();
        assert_eq!(now.position.prefix, unconfirmable());
        assert_eq!(now.purged.len(), 1);
    }
}

#[test]
fn a_transcript_that_no_longer_holds_what_was_consumed_confirms_nothing() {
    let dir = TempDir::new().unwrap();
    let (store, _, path) = ingested(&dir, writer(1), "a.jsonl");
    let mut store = store.with_transcripts(read);
    for rewritten in [&BODY[..10], b"zzz\nbbb\nccc\nddd\neee\n"] {
        std::fs::write(&path, rewritten).unwrap();
        store
            .purge(writer(2), &[], &[range(writer(1), 2, 2)])
            .unwrap();

        let now = store.source(&path).unwrap().unwrap();
        assert_eq!(now.position.prefix, unconfirmable());
    }
}

#[test]
fn a_purge_that_races_an_ingest_that_moved_the_position_fails_whole() {
    let dir = TempDir::new().unwrap();
    let (store, source, _) = ingested(&dir, writer(1), "a.jsonl");
    let mut store = store.with_transcripts(read);
    let rehashes = plan(&store.conn, &[range(writer(1), 2, 2)], Some(read)).unwrap();
    store
        .conn
        .execute(
            "UPDATE transcript_sources SET prefix_hash = ?1 WHERE id = ?2",
            params![digest(b"moved").to_hex(), source.id],
        )
        .unwrap();

    let tx = store.conn.transaction().unwrap();
    let err = apply(&tx, &rehashes).unwrap_err();

    assert!(
        matches!(err, StoreError::PositionMoved { source: id } if id == source.id),
        "{err}"
    );
}

#[test]
fn a_store_that_cannot_mark_or_read_its_places_fails_each_step() {
    let dir = TempDir::new().unwrap();
    let (store, source, _) = ingested(&dir, writer(1), "a.jsonl");
    store
        .conn
        .execute_batch("DROP TABLE transcript_places")
        .unwrap();

    assert!(mark(&store.conn, &range(writer(1), 1, 1)).is_err());
    assert!(crate::source::spans(&store.conn, source.id).is_err());
}

#[test]
fn a_purge_whose_residue_cannot_be_copied_fails_whole() {
    let dir = TempDir::new().unwrap();
    let (store, _, _) = ingested(&dir, writer(1), "a.jsonl");
    mark(&store.conn, &range(writer(1), 1, 1)).unwrap();
    store
        .conn
        .execute_batch(
            "CREATE TRIGGER refuse_copy BEFORE INSERT ON transcript_purged_copies
             BEGIN SELECT RAISE(FAIL, 'refused'); END;",
        )
        .unwrap();

    assert!(mark(&store.conn, &range(writer(1), 2, 3)).is_err());
}

#[test]
fn a_purge_whose_places_cannot_be_released_fails_whole() {
    let dir = TempDir::new().unwrap();
    let (store, _, _) = ingested(&dir, writer(1), "a.jsonl");
    store
        .conn
        .execute_batch(
            "CREATE TRIGGER refuse_release BEFORE DELETE ON transcript_places
             BEGIN SELECT RAISE(ABORT, 'refused'); END;",
        )
        .unwrap();

    let failed = mark(&store.conn, &range(writer(1), 1, 3));

    assert!(matches!(failed, Err(crate::StoreError::Sqlite(_))));
}

#[test]
fn a_purge_whose_residue_table_cannot_be_made_fails_whole() {
    let dir = TempDir::new().unwrap();
    let (store, _, _) = ingested(&dir, writer(1), "a.jsonl");
    store
        .conn
        .execute_batch(
            "DROP INDEX transcript_purged_copies_uuid;
             DROP TABLE transcript_purged_copies;
             CREATE TABLE transcript_purged_copies (
                 source INTEGER PRIMARY KEY
             ) STRICT, WITHOUT ROWID;",
        )
        .unwrap();

    assert!(mark(&store.conn, &range(writer(1), 1, 1)).is_err());
}
