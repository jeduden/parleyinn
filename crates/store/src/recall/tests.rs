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

    let first = store.counts(&Reach::Writers(vec![writer(1)])).unwrap();
    let by_room = store.counts(&Reach::Rooms(vec![room(2)])).unwrap();
    let both = store.counts(&Reach::Rooms(vec![room(1), room(2)])).unwrap();
    let none = store.counts(&Reach::Writers(vec![])).unwrap();

    assert_eq!(
        first,
        Counts {
            events: 7,
            recalls: 1,
            compactions: 1
        }
    );
    assert_eq!(
        by_room,
        Counts {
            events: 1,
            recalls: 0,
            compactions: 0
        }
    );
    assert_eq!(both.events, 8);
    assert_eq!(none, Counts::default());
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
