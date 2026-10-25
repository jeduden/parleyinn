use super::*;
use crate::{NewEvent, SeatId, Store, StoredSeat};
use record::commitment::CommitmentKey;
use record::event::Provenance;
use serde_json::json;
use testkit::TempDir;

fn room(n: u8) -> RoomId {
    RoomId::derive(&[n; 32], &[2; 16])
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

fn event(kind: EventKind, provenance: Provenance, content: Value) -> NewEvent {
    NewEvent {
        kind,
        provenance,
        content,
        commitment_key: CommitmentKey::from_bytes([9; 32]),
        transcript_timestamp: None,
    }
}

fn assign(writer: Option<WriterId>) -> NewEvent {
    let content = writer.map_or(json!({}), |w| json!({"writer": w.to_string()}));

    event(EventKind::WriterLabel, Provenance::Structural, content)
}

fn line(text: &str, provenance: Provenance) -> NewEvent {
    event(
        EventKind::TranscriptLine,
        provenance,
        json!({"line": 1, "text": text}),
    )
}

#[test]
fn a_label_is_the_rank_of_its_assignment() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, writer(9), room(1));
    store
        .append(
            writer(9),
            &[
                assign(Some(writer(1))),
                assign(Some(writer(2))),
                assign(Some(writer(1))),
                assign(None),
                assign(Some(writer(3))),
            ],
        )
        .unwrap();

    let labels = store.writer_labels(writer(9)).unwrap();

    let label = |n| Label::new(n).unwrap();
    assert_eq!(
        labels,
        BTreeMap::from([
            (writer(1), label(1)),
            (writer(2), label(2)),
            (writer(3), label(5)),
        ])
    );
    assert!(store.writer_labels(writer(8)).unwrap().is_empty());
}

#[test]
fn counts_cover_the_events_of_the_writers_or_the_rooms_reached() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, writer(1), room(1));
    seat(&mut store, writer(2), room(2));
    let boundary = json!({"type": "system", "subtype": "compact_boundary"}).to_string();
    let aside = json!({"type": "system", "subtype": "other"}).to_string();
    store
        .append(
            writer(1),
            &[
                line("a", Provenance::User),
                event(
                    EventKind::Recall,
                    Provenance::Assistant,
                    json!({"tool": "t"}),
                ),
                line(&boundary, Provenance::HarnessText),
                line(&aside, Provenance::HarnessText),
                line(&boundary, Provenance::Web),
                line("not json compact_boundary", Provenance::HarnessText),
                line("{\"compact_boundary\":1}", Provenance::HarnessText),
            ],
        )
        .unwrap();
    store
        .append(writer(2), &[line("b", Provenance::User)])
        .unwrap();

    let first = store.stats(&RecallReach::Writers(vec![writer(1)])).unwrap();
    let by_room = store.stats(&RecallReach::Rooms(vec![room(2)])).unwrap();
    let both = store
        .stats(&RecallReach::Rooms(vec![room(1), room(2)]))
        .unwrap();
    let none = store.stats(&RecallReach::Writers(vec![])).unwrap();

    assert_eq!(
        first,
        Stats {
            events: 7,
            recalls: 1,
            compactions: 1
        }
    );
    assert_eq!(
        by_room,
        Stats {
            events: 1,
            recalls: 0,
            compactions: 0
        }
    );
    assert_eq!(both.events, 8);
    assert_eq!(none, Stats::default());
}

#[test]
fn a_boundary_line_is_a_system_line_of_that_subtype_only() {
    let line = |l: &Value| json!({"line": 1, "text": l.to_string()}).to_string();

    assert!(is_compact_boundary(&line(
        &json!({"type": "system", "subtype": "compact_boundary"})
    )));
    assert!(!is_compact_boundary(&line(
        &json!({"type": "user", "subtype": "compact_boundary"})
    )));
    assert!(!is_compact_boundary("not json"));
    assert!(!is_compact_boundary(r#"{"line":1}"#));
    assert!(!is_compact_boundary(r#"{"text":"not json"}"#));
}

#[test]
fn certified_writers_come_in_the_order_the_device_seat_recorded_them() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, writer(9), room(1));
    let record = |w: Option<WriterId>| {
        let content = w.map_or(json!({}), |w| json!({"writer": w.to_string()}));
        event(
            EventKind::CertificateRecord,
            Provenance::Structural,
            content,
        )
    };
    store
        .append(
            writer(9),
            &[
                record(Some(writer(3))),
                record(None),
                record(Some(writer(1))),
                assign(Some(writer(2))),
            ],
        )
        .unwrap();

    assert_eq!(
        store.certified_writers(writer(9)).unwrap(),
        [writer(3), writer(1)]
    );
    assert!(store.certified_writers(writer(8)).unwrap().is_empty());
}

#[test]
fn the_head_of_a_log_is_its_last_seq_and_an_empty_log_has_none() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, writer(1), room(1));
    seat(&mut store, writer(2), room(1));
    store
        .append(
            writer(1),
            &[line("a", Provenance::User), line("b", Provenance::User)],
        )
        .unwrap();

    assert_eq!(store.head(writer(1)).unwrap(), Seq::new(2));
    assert_eq!(store.head(writer(2)).unwrap(), None);
    assert_eq!(store.head(writer(9)).unwrap(), None);
}

#[test]
fn the_contents_of_one_kind_come_in_seq_order_without_the_erased_or_unparsed() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, writer(1), room(1));
    let act = |n: u64| {
        event(
            EventKind::PrincipalAct,
            Provenance::Operator,
            json!({"n": n}),
        )
    };
    store
        .append(
            writer(1),
            &[
                act(1),
                line("other kind", Provenance::Web),
                act(2),
                act(3),
                act(4),
            ],
        )
        .unwrap();
    store
        .conn
        .execute("UPDATE events SET content = NULL WHERE seq = 3", [])
        .unwrap();
    store
        .conn
        .execute("UPDATE events SET content = 'not json' WHERE seq = 4", [])
        .unwrap();

    let found = store.contents(writer(1), EventKind::PrincipalAct).unwrap();

    let seqs: Vec<u64> = found.iter().map(|(s, _)| s.get()).collect();
    assert_eq!(seqs, [1, 5]);
    assert_eq!(found[0].1, json!({"n": 1}));
    assert!(
        store
            .contents(writer(2), EventKind::PrincipalAct)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_range_shows_each_event_without_its_content_and_whether_it_was_erased() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, writer(1), room(1));
    store
        .append(
            writer(1),
            &[
                line("a", Provenance::Web),
                assign(None),
                line("c", Provenance::Web),
            ],
        )
        .unwrap();
    store
        .conn
        .execute(
            "UPDATE events SET content = NULL, commitment_key = NULL WHERE seq = 2",
            [],
        )
        .unwrap();

    let seq = |n: u64| Seq::new(n).unwrap();
    let found = store.range_events(writer(1), seq(2), seq(9)).unwrap();

    let shown: Vec<(u64, EventKind, bool)> = found
        .iter()
        .map(|e| (e.seq.get(), e.kind, e.erased))
        .collect();
    assert_eq!(
        shown,
        [
            (2, EventKind::WriterLabel, true),
            (3, EventKind::TranscriptLine, false)
        ]
    );
    let held = store.log(writer(1)).unwrap();
    assert_eq!(found[1].commitment, held[2].link.header.commitment);
    assert!(
        store
            .range_events(writer(2), seq(1), seq(9))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn an_event_shows_its_classes_timestamp_and_erasure() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    seat(&mut store, writer(1), room(1));
    let mut stamped = line("a", Provenance::Web);
    stamped.transcript_timestamp = Some("2026-01-01T00:00:00Z".to_owned());
    store.append(writer(1), &[stamped]).unwrap();
    let at = |n: u64| Address::new(writer(1), Seq::new(n).unwrap());

    let facts = store.event_facts(at(1)).unwrap().unwrap();

    assert_eq!(facts.kind, EventKind::TranscriptLine);
    assert_eq!(facts.provenance, Provenance::Web);
    assert_eq!(facts.timestamp.as_deref(), Some("2026-01-01T00:00:00Z"));
    assert!(!facts.erased);
    assert_eq!(store.event_facts(at(2)).unwrap(), None);
    store
        .conn
        .execute("UPDATE events SET kind = 'nonsense'", [])
        .unwrap();
    assert!(store.event_facts(at(1)).is_err());
}
