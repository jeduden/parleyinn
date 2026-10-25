use super::*;
use crate::act::{Act, record};
use crate::home::Home;
use crate::node::Node;
use crate::node_identity::IdentitySource;
use key::random::OsEntropy;
use record::commitment::CommitmentKey;
use serde_json::json;
use store::{NewEvent, StoredSeat};
use testkit::TempDir;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

fn room(n: u8) -> RoomId {
    RoomId::derive(&[n; 32], &[2; 16])
}

fn seat(n: u8) -> SeatId {
    SeatId::derive(&room(1), &[n; 32])
}

fn facts() -> WriterFacts {
    WriterFacts {
        writer: writer(1),
        room: room(1),
        seat: seat(1),
        run: Some("session-1/agent-2".to_owned()),
    }
}

fn event<'a>(
    n: u64,
    class: &'a Provenance,
    stamp: Option<&'a str>,
    flags: &'a [String],
) -> EventFacts<'a> {
    EventFacts {
        seq: seq(n),
        provenance: class,
        timestamp: stamp,
        flags,
    }
}

#[test]
fn every_selector_round_trips_through_the_act_that_records_it() {
    for selector in [
        Selector::Range {
            writer: writer(1),
            first: seq(100),
            last: seq(200),
        },
        Selector::run("session-1/agent-2").unwrap(),
        Selector::Author(seat(1)),
        Selector::Room(room(1)),
        Selector::Writer(writer(2)),
        Selector::provenance("tool_result:Bash").unwrap(),
        Selector::flag("instruction_like").unwrap(),
        Selector::since("2026-01-01T00:00:00Z").unwrap(),
    ] {
        assert_eq!(
            Selector::from_value(&selector.to_value()),
            Some(selector.clone())
        );
        assert!(!selector.describe().is_empty());
    }
}

#[test]
fn a_value_that_is_no_selector_is_none() {
    for bad in [
        json!(null),
        json!({}),
        json!({"by": "nothing"}),
        json!({"by": "range", "writer": "x", "first": 1, "last": 2}),
        json!({"by": "range", "writer": writer(1).to_string(), "first": 0, "last": 2}),
        json!({"by": "range", "writer": writer(1).to_string(), "first": 1}),
        json!({"by": "run", "run": ""}),
        json!({"by": "author", "seat": "x"}),
        json!({"by": "room", "room": 3}),
        json!({"by": "writer", "writer": "x"}),
        json!({"by": "provenance", "class": "nonsense"}),
        json!({"by": "flag", "flag": "Not Plain"}),
        json!({"by": "since", "time": "yesterday"}),
    ] {
        assert_eq!(Selector::from_value(&bad), None, "{bad}");
    }
}

#[test]
fn a_selector_value_that_is_not_plain_is_refused_with_its_reason() {
    assert_eq!(
        Selector::run("two words").unwrap_err().to_string(),
        "a run name is letters, digits and - _ . / :"
    );
    assert!(Selector::run(&"x".repeat(201)).is_err());
    assert!(Selector::flag("UPPER").is_err());
    assert!(Selector::flag("").is_err());
    assert_eq!(
        Selector::provenance("web:x").unwrap_err().to_string(),
        "not a provenance class"
    );
    for bad in ["", "2026", "2026/01/01", "abcd-ef-gh", "2026-01-01 00:00"] {
        assert_eq!(
            Selector::since(bad).unwrap_err().to_string(),
            "a time is RFC 3339, such as 2026-01-01T00:00:00Z",
            "{bad}"
        );
    }
    assert!(std::error::Error::source(&Invalid("x")).is_none());
}

/// Whether `selector` holds a web event of seq `n` in `writer`.
fn holds(selector: &Selector, writer: &WriterFacts, n: u64, flags: &[String]) -> bool {
    let web = Provenance::Web;
    let stamp = Some("2026-03-04T05:06:07.250Z");

    selector.holds(writer, &event(n, &web, stamp, flags))
}

#[test]
fn a_range_holds_its_writer_s_events_from_first_to_last() {
    let range = |a, b| Selector::Range {
        writer: writer(1),
        first: seq(a),
        last: seq(b),
    };
    let there = |s: &Selector, n| holds(s, &facts(), n, &[]);

    assert!(there(&range(100, 200), 100) && there(&range(100, 200), 200));
    assert!(!there(&range(100, 200), 201) && !there(&range(100, 200), 99));
    let other = WriterFacts {
        writer: writer(2),
        ..facts()
    };
    assert!(!holds(&range(100, 200), &other, 150, &[]));
}

#[test]
fn a_run_an_author_a_room_and_a_writer_hold_what_they_name() {
    let here = facts();
    let on = |s: &Selector| holds(s, &here, 1, &[]);

    assert!(on(&Selector::run("session-1/agent-2").unwrap()));
    assert!(!on(&Selector::run("session-1").unwrap()));
    assert!(!holds(
        &Selector::run("session-1/agent-2").unwrap(),
        &WriterFacts {
            run: None,
            ..facts()
        },
        1,
        &[]
    ));
    assert!(on(&Selector::Author(seat(1))) && !on(&Selector::Author(seat(2))));
    assert!(on(&Selector::Room(room(1))) && !on(&Selector::Room(room(2))));
    assert!(on(&Selector::Writer(writer(1))) && !on(&Selector::Writer(writer(2))));
}

#[test]
fn a_provenance_class_and_a_flag_hold_the_events_that_carry_them() {
    let here = facts();
    let flags = vec!["instruction_like".to_owned()];
    let flagged = Selector::flag("instruction_like").unwrap();

    assert!(holds(&Selector::Provenance(Provenance::Web), &here, 1, &[]));
    assert!(!holds(
        &Selector::Provenance(Provenance::File),
        &here,
        1,
        &[]
    ));
    assert!(holds(&flagged, &here, 1, &flags));
    assert!(!holds(&flagged, &here, 1, &[]));
}

#[test]
fn a_time_window_compares_to_the_second_and_holds_no_event_without_a_time() {
    let class = Provenance::Web;
    let since = |t| Selector::since(t).unwrap();
    let stamped = |t| event(1, &class, Some(t), &[]);

    assert!(since("2026-01-01T00:00:00Z").holds(&facts(), &stamped("2026-01-01T00:00:00.500Z")));
    assert!(since("2026-01-01T00:00:00.900Z").holds(&facts(), &stamped("2026-01-01T00:00:00Z")));
    assert!(since("2026-01-01T00:00:00Z").holds(&facts(), &stamped("2027-01-01T00:00:00Z")));
    assert!(!since("2026-01-01T00:00:01Z").holds(&facts(), &stamped("2026-01-01T00:00:00.999Z")));
    assert!(since("2026-01-01").holds(&facts(), &stamped("2026-01-01T00:00:00Z")));
    assert!(!since("2026-01-01T00:00:00Z").holds(&facts(), &event(1, &class, None, &[])));
}

struct Fixture {
    dir: TempDir,
    home: Home,
    store: Store,
    node: Node,
    ingest: StoredSeat,
}

fn line(n: u8, class: Provenance, stamp: &str) -> NewEvent {
    NewEvent {
        kind: EventKind::TranscriptLine,
        provenance: class,
        content: json!({"text": format!("line {n}")}),
        commitment_key: CommitmentKey::from_bytes([n; 32]),
        transcript_timestamp: Some(stamp.to_owned()),
    }
}

/// A node with an ingest seat holding two lines, the second a web
/// result, and a run recorded for it.
fn fixture() -> Fixture {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("machine-id"), "first").unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![dir.path().join("machine-id")]));
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let ingest = node
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();
    store
        .append(
            ingest.writer,
            &[
                line(1, Provenance::User, "2025-12-31T23:59:59Z"),
                line(2, Provenance::Web, "2026-02-01T00:00:00Z"),
            ],
        )
        .unwrap();
    store
        .add_run(&store::IngestedRun {
            writer: ingest.writer,
            path: "/t/session-9.jsonl".to_owned(),
            harness_session: "session-9".to_owned(),
            agent_id: None,
        })
        .unwrap();

    Fixture {
        dir,
        home,
        store,
        node,
        ingest,
    }
}

impl Fixture {
    fn act(&mut self, act: &Act) {
        record(&mut self.store, &self.node, act, &mut OsEntropy).unwrap();
    }

    fn at(&self, n: u64) -> Address {
        Address::new(self.ingest.writer, seq(n))
    }

    fn quarantine(&self) -> Quarantine<'_> {
        Quarantine::load(&self.store, self.node.personal_room()).unwrap()
    }
}

#[test]
fn nothing_is_quarantined_until_an_act_adds_a_selector() {
    let f = fixture();

    let q = f.quarantine();

    assert!(q.set().is_empty());
    assert!(!q.holds(f.at(1)).unwrap());
    assert!(!q.holds(f.at(2)).unwrap());
}

#[test]
fn an_added_selector_holds_its_events_and_those_recorded_later() {
    let mut f = fixture();
    f.act(&Act::QuarantineAdd {
        selector: Selector::Provenance(Provenance::Web),
        reason: "suspect content".to_owned(),
    });
    f.act(&Act::QuarantineAdd {
        selector: Selector::run("session-9").unwrap(),
        reason: "a bad run".to_owned(),
    });
    let q = f.quarantine();
    assert_eq!(q.set().active().len(), 2);
    assert_eq!(q.set().active()[0].reason, "suspect content");
    // The run selector holds every event of the run's writer.
    assert!(q.holds(f.at(1)).unwrap() && q.holds(f.at(2)).unwrap());
    drop(q);

    f.act(&Act::QuarantineRelease {
        selector: Selector::run("session-9").unwrap(),
    });
    f.store
        .append(
            f.ingest.writer,
            &[
                line(3, Provenance::Web, "2026-03-01T00:00:00Z"),
                line(4, Provenance::User, "2026-03-01T00:00:00Z"),
            ],
        )
        .unwrap();

    let q = f.quarantine();
    assert_eq!(q.set().active().len(), 1);
    assert!(!q.holds(f.at(1)).unwrap(), "released with its selector");
    assert!(q.holds(f.at(2)).unwrap());
    assert!(q.holds(f.at(3)).unwrap(), "recorded after the act");
    assert!(!q.holds(f.at(4)).unwrap());
    assert!(!q.holds(f.at(9)).unwrap(), "an event that is not there");
    let nowhere = Address::new(writer(9), seq(1));
    assert!(!q.holds(nowhere).unwrap(), "a writer with no seat");
}

#[test]
fn an_act_that_does_not_parse_adds_and_releases_nothing() {
    let mut f = fixture();
    let device = f.node.device_writer();
    let odd = |content: Value| NewEvent {
        kind: EventKind::PrincipalAct,
        provenance: Provenance::Operator,
        content,
        commitment_key: CommitmentKey::from_bytes([77; 32]),
        transcript_timestamp: None,
    };
    f.store
        .append(
            device,
            &[
                odd(json!({"act": ADD, "selector": {"by": "nothing"}})),
                odd(json!({"act": "counter_ack", "selector": Selector::Room(room(1)).to_value()})),
                odd(json!({"act": ADD, "selector": Selector::Writer(writer(1)).to_value()})),
                odd(json!({"act": RELEASE, "selector": Selector::Writer(writer(2)).to_value()})),
            ],
        )
        .unwrap();

    let set = f.quarantine().set().clone();

    assert_eq!(set.active().len(), 1);
    assert_eq!(set.active()[0].reason, "");
}

#[test]
fn a_quarantine_changes_no_origin() {
    let mut f = fixture();
    let before = crate::origin::derive(&f.store, &f.node.device().public()).unwrap();

    f.act(&Act::QuarantineAdd {
        selector: Selector::Writer(f.ingest.writer),
        reason: "forensics".to_owned(),
    });

    assert_eq!(
        crate::origin::derive(&f.store, &f.node.device().public()).unwrap(),
        before
    );
}

#[test]
fn a_node_identity_change_keeps_the_acts_of_the_device_seat_it_replaced() {
    let mut f = fixture();
    let old = f.node.device_writer();
    let selector = Selector::Provenance(Provenance::Web);
    f.act(&Act::QuarantineAdd {
        selector: selector.clone(),
        reason: "before".to_owned(),
    });
    std::fs::write(f.dir.path().join("machine-id"), "second").unwrap();
    f.node = Node::open(&f.home, &mut f.store, &mut OsEntropy).unwrap();
    assert_ne!(f.node.device_writer(), old);

    let devices = device_writers(&f.store, f.node.personal_room()).unwrap();

    assert_eq!(devices, [old, f.node.device_writer()]);
    assert!(f.quarantine().holds(f.at(2)).unwrap(), "still quarantined");
    f.act(&Act::QuarantineRelease { selector });
    assert!(
        !f.quarantine().holds(f.at(2)).unwrap(),
        "released by the new seat"
    );
}

#[test]
fn a_device_seat_of_another_room_holds_no_quarantine_of_this_node() {
    let f = fixture();

    let devices = device_writers(&f.store, room(7)).unwrap();

    assert!(devices.is_empty());
}
