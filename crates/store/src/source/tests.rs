use super::*;
use crate::{NewEvent, Store};
use record::commitment::CommitmentKey;
use record::event::Provenance;
use serde_json::json;
use testkit::TempDir;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn open(dir: &TempDir) -> Store {
    Store::open(&dir.path().join("store.db")).unwrap()
}

fn event(kind: EventKind, n: u64) -> NewEvent {
    NewEvent {
        kind,
        provenance: Provenance::HarnessText,
        content: json!({"line": n, "text": format!("line {n}")}),
        commitment_key: CommitmentKey::from_bytes([n as u8; 32]),
        transcript_timestamp: None,
    }
}

fn place(line: u64, uuid: Option<&str>) -> Place {
    Place {
        line,
        offset: (line - 1) * 10,
        uuid: uuid.map(str::to_owned),
    }
}

fn position(offset: u64, lines: u64) -> Position {
    Position {
        offset,
        prefix: Digest::of(&[b"prefix"]),
        lines,
        seen: offset,
    }
}

#[test]
fn a_new_position_has_read_nothing_and_is_complete() {
    let start = Position::start();

    assert_eq!(start.offset, 0);
    assert_eq!(start.prefix, Digest::of(&[]));
    assert_eq!(start.lines, 0);
    assert!(start.complete());
    assert!(!Position { seen: 5, ..start }.complete());
}

#[test]
fn a_source_starts_at_generation_1_and_each_new_generation_counts_on() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    assert_eq!(store.source("/t/a.jsonl").unwrap(), None);

    let first = store.start_generation("/t/a.jsonl", "/t").unwrap();
    let second = store.start_generation("/t/a.jsonl", "/t").unwrap();
    let other = store.start_generation("/t/b.jsonl", "/t").unwrap();

    assert_eq!(first.generation, 1);
    assert_eq!(first.position, Position::start());
    assert_eq!(first.directory, "/t");
    assert!(!first.lost && first.held_since.is_none());
    assert_eq!(second.generation, 2);
    assert_eq!(other.generation, 1);
    assert_eq!(store.source("/t/a.jsonl").unwrap(), Some(second.clone()));
    assert_eq!(
        store.generations("/t/a.jsonl").unwrap(),
        vec![first.clone(), second.clone()]
    );
    assert_eq!(store.sources().unwrap(), vec![second, other]);
}

#[test]
fn appended_lines_keep_their_places_and_move_the_position_with_them() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let source = store.start_generation("/t/a.jsonl", "/t").unwrap();
    let events = [
        event(EventKind::TranscriptLine, 1),
        event(EventKind::TranscriptLine, 2),
    ];
    let places = [place(1, Some("u1")), place(2, None)];

    let out = store
        .append_lines(
            writer(1),
            &events,
            &places,
            &[false, true],
            source.id,
            &position(20, 2),
        )
        .unwrap();

    assert_eq!(out.len(), 2);
    let placed = store.placed(source.id).unwrap();
    assert_eq!(placed.len(), 2);
    assert_eq!(placed[0].place, places[0]);
    assert_eq!(placed[1].address, out[1]);
    assert_eq!(placed[1].kind, EventKind::TranscriptLine);
    assert_eq!(
        store.source("/t/a.jsonl").unwrap().unwrap().position,
        position(20, 2)
    );
    assert!(store.segments(writer(1)).unwrap()[0].closed);
}

#[test]
fn an_append_that_fails_leaves_the_position_where_it_was() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let source = store.start_generation("/t/a.jsonl", "/t").unwrap();
    let mut bad = event(EventKind::TranscriptLine, 1);
    bad.content = json!({"x": 0.5});

    let err = store
        .append_lines(
            writer(1),
            &[bad],
            &[place(1, None)],
            &[true],
            source.id,
            &position(10, 1),
        )
        .unwrap_err();

    assert!(matches!(err, crate::StoreError::Encoding(_)), "{err}");
    let now = store.source("/t/a.jsonl").unwrap().unwrap();
    assert_eq!(now.position, Position::start());
    assert!(store.placed(source.id).unwrap().is_empty());
}

#[test]
fn an_append_behind_a_position_another_ingest_moved_is_refused_whole() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let source = store.start_generation("/t/a.jsonl", "/t").unwrap();
    store
        .append_lines(
            writer(1),
            &[event(EventKind::TranscriptLine, 1)],
            &[place(1, None)],
            &[true],
            source.id,
            &position(10, 1),
        )
        .unwrap();

    // A second ingest read the source before the first appended, and
    // starts again at line 1.
    let err = store
        .append_lines(
            writer(2),
            &[
                event(EventKind::TranscriptLine, 1),
                event(EventKind::TranscriptLine, 2),
            ],
            &[place(1, None), place(2, None)],
            &[false, true],
            source.id,
            &position(20, 2),
        )
        .unwrap_err();

    assert!(
        matches!(err, crate::StoreError::PositionMoved { source: id } if id == source.id),
        "{err}"
    );
    assert!(err.to_string().contains("moved"), "{err}");
    assert!(store.log(writer(2)).unwrap().is_empty());
    assert_eq!(store.placed(source.id).unwrap().len(), 1);
    assert_eq!(
        store.source("/t/a.jsonl").unwrap().unwrap().position,
        position(10, 1)
    );
}

#[test]
fn a_position_moves_only_forward() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let source = store.start_generation("/t/a.jsonl", "/t").unwrap();

    store.advance(source.id, &position(20, 2)).unwrap();
    store.advance(source.id, &position(10, 1)).unwrap();

    assert_eq!(
        store.source("/t/a.jsonl").unwrap().unwrap().position,
        position(20, 2),
        "an ingest that read less leaves the position another moved"
    );
}

#[test]
fn advance_hold_and_lose_change_only_the_state() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let source = store.start_generation("/t/a.jsonl", "/t").unwrap();

    store.advance(source.id, &position(7, 1)).unwrap();
    store.hold(source.id, Some(1234)).unwrap();
    let held = store.source("/t/a.jsonl").unwrap().unwrap();
    store.hold(source.id, None).unwrap();
    store.lose(source.id).unwrap();
    let lost = store.source("/t/a.jsonl").unwrap().unwrap();

    assert_eq!(held.position, position(7, 1));
    assert_eq!(held.held_since, Some(1234));
    assert_eq!(lost.held_since, None);
    assert!(lost.lost);
}

#[test]
fn a_line_owner_is_the_first_event_of_another_source_in_the_directory() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let a = store.start_generation("/t/a.jsonl", "/t").unwrap();
    let b = store.start_generation("/t/b.jsonl", "/t").unwrap();
    let elsewhere = store.start_generation("/u/c.jsonl", "/u").unwrap();
    let first = store
        .append_lines(
            writer(1),
            &[event(EventKind::TranscriptLine, 1)],
            &[place(1, Some("u1"))],
            &[true],
            a.id,
            &position(10, 1),
        )
        .unwrap();
    store
        .append_lines(
            writer(2),
            &[event(EventKind::RepeatReference, 1)],
            &[place(1, Some("u2"))],
            &[true],
            b.id,
            &position(10, 1),
        )
        .unwrap();
    store
        .append_lines(
            writer(3),
            &[event(EventKind::TranscriptLine, 1)],
            &[place(1, Some("u3"))],
            &[true],
            elsewhere.id,
            &position(10, 1),
        )
        .unwrap();

    assert_eq!(
        store.line_owner("/t", "u1", "/t/b.jsonl").unwrap(),
        Some(first[0])
    );
    // Its own source, a repeat reference and another directory own none.
    assert_eq!(store.line_owner("/t", "u1", "/t/a.jsonl").unwrap(), None);
    assert_eq!(store.line_owner("/t", "u2", "/t/a.jsonl").unwrap(), None);
    assert_eq!(store.line_owner("/t", "u3", "/t/a.jsonl").unwrap(), None);
}

#[test]
fn the_ingested_run_of_a_source_and_agent_id_is_the_one_kept_last() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let main = IngestedRun {
        writer: writer(1),
        path: "/t/s.jsonl".into(),
        harness_session: "s".into(),
        agent_id: None,
    };
    let sub = IngestedRun {
        writer: writer(2),
        agent_id: Some("a1".into()),
        ..main.clone()
    };

    store.add_run(&main).unwrap();
    store.add_run(&sub).unwrap();
    let twice = store.add_run(&main);
    let later = IngestedRun {
        writer: writer(3),
        ..main.clone()
    };
    store.add_run(&later).unwrap();

    assert!(twice.is_err());
    assert_eq!(
        store.ingested_run("/t/s.jsonl", None).unwrap(),
        Some(later.clone())
    );
    assert_eq!(
        store.ingested_run("/t/s.jsonl", Some("a1")).unwrap(),
        Some(sub.clone())
    );
    assert_eq!(store.ingested_run("/t/s.jsonl", Some("a2")).unwrap(), None);
    assert_eq!(store.ingested_runs().unwrap(), vec![main, sub, later]);
}

#[test]
fn a_corrupt_row_names_what_does_not_parse() {
    let row = |prefix: &str| -> SourceRow {
        (
            1,
            "/t/a.jsonl".into(),
            1,
            "/t".into(),
            -1,
            prefix.into(),
            0,
            0,
            None,
            false,
        )
    };
    let err = source(row("zz")).unwrap_err();
    assert!(
        err.to_string().contains("prefix hash of transcript source"),
        "{err}"
    );
    // A negative count reads as none.
    let ok = source(row(&Digest::ZERO.to_hex())).unwrap();
    assert_eq!(ok.position.offset, 0);
    assert_eq!(sql_count(u64::MAX), i64::MAX);
    assert!(address("x", 1).is_err());
    assert!(address(&writer(1).to_string(), 0).is_err());
}

#[test]
fn a_place_whose_kind_does_not_parse_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let a = store.start_generation("/t/a.jsonl", "/t").unwrap();
    store
        .append_lines(
            writer(1),
            &[event(EventKind::TranscriptLine, 1)],
            &[place(1, None)],
            &[true],
            a.id,
            &position(10, 1),
        )
        .unwrap();
    store
        .conn
        .execute("UPDATE events SET kind = 'nothing'", [])
        .unwrap();

    assert!(store.placed(a.id).is_err());
}

#[test]
fn a_run_whose_writer_does_not_parse_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    store
        .conn
        .execute(
            "INSERT INTO ingested_runs (writer, path, harness_session) VALUES ('x', 'p', 's')",
            [],
        )
        .unwrap();

    assert!(store.ingested_runs().is_err());
}
