use super::*;
use crate::act::{Act, event};
use crate::home::Home;
use crate::node_identity::IdentitySource;
use key::random::OsEntropy;
use serde_json::json;
use store::{Position, StoredSeat};
use testkit::TempDir;

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

struct Fixture {
    _dir: TempDir,
    store: Store,
    node: Node,
    ingest: StoredSeat,
}

fn line(n: u8, kind: EventKind) -> NewEvent {
    NewEvent {
        kind,
        provenance: Provenance::Web,
        content: json!({"text": format!("line {n}")}),
        commitment_key: CommitmentKey::from_bytes([n; 32]),
        transcript_timestamp: None,
    }
}

/// A node with an ingest seat holding four lines, the third a repeat
/// reference.
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
                line(1, EventKind::TranscriptLine),
                line(2, EventKind::UnparsedLine),
                line(3, EventKind::RepeatReference),
                line(4, EventKind::TranscriptLine),
            ],
        )
        .unwrap();

    Fixture {
        _dir: dir,
        store,
        node,
        ingest,
    }
}

impl Fixture {
    fn plan(&self, first: u64, last: u64) -> Planned {
        plan(
            &self.store,
            self.ingest.writer,
            seq(first),
            seq(last),
            &Reason::Purged,
        )
        .unwrap()
    }
}

fn ready(planned: Planned) -> Plan {
    match planned {
        Planned::Ready(plan) => plan,
        Planned::Refused(why) => panic!("refused: {why:?}"),
    }
}

#[test]
fn a_range_of_content_is_planned_with_its_tombstone() {
    let f = fixture();

    let plan = ready(f.plan(1, 2));

    assert_eq!(
        plan.erase,
        Erase {
            writer: f.ingest.writer,
            first: seq(1),
            last: seq(2)
        }
    );
    assert_eq!(plan.held, 2);
    let t = &plan.tombstone;
    assert_eq!((t.first, t.last, t.count), (seq(1), seq(2), 2));
    assert_eq!((t.reason.as_str(), t.policy.as_deref()), ("purged", None));
    let held = f.store.log(f.ingest.writer).unwrap();
    let hex: Vec<String> = held[..2]
        .iter()
        .map(|s| s.link.header.commitment.to_string())
        .collect();
    assert_eq!(t.commitments, commitments_digest(&hex));
}

#[test]
fn a_range_is_cut_to_the_events_there_are() {
    let f = fixture();

    let plan = ready(f.plan(4, 900));

    assert_eq!(
        (plan.tombstone.first, plan.tombstone.last),
        (seq(4), seq(4))
    );
    assert!(matches!(f.plan(9, 12), Planned::Refused(r) if r == [Refusal::NothingHeld]));
}

#[test]
fn a_retention_policy_is_named_in_the_tombstone() {
    let f = fixture();
    let reason = Reason::Retention {
        policy: "retention_policy.*.web".to_owned(),
    };

    let plan = ready(plan(&f.store, f.ingest.writer, seq(1), seq(1), &reason).unwrap());

    assert_eq!(plan.tombstone.reason, "retention");
    assert_eq!(
        plan.tombstone.policy.as_deref(),
        Some("retention_policy.*.web")
    );
    assert_eq!(
        Tombstone::from_value(&plan.tombstone.to_value()),
        Some(plan.tombstone)
    );
}

#[test]
fn a_range_holding_an_event_a_purge_keeps_is_refused_naming_it() {
    let f = fixture();

    let Planned::Refused(why) = f.plan(2, 4) else {
        panic!("planned");
    };

    assert_eq!(
        why,
        [Refusal::Kept {
            address: Address::new(f.ingest.writer, seq(3)),
            kind: EventKind::RepeatReference
        }]
    );
    assert_eq!(
        why[0].to_string(),
        format!(
            "{}:3 is a repeat_reference event, which a purge keeps: the key set or room state derives from it",
            f.ingest.writer
        )
    );
    let device = f.node.device_writer();
    let Planned::Refused(keys) = plan(&f.store, device, seq(1), seq(2), &Reason::Purged).unwrap()
    else {
        panic!("a device seat's key events are purged");
    };
    assert_eq!(keys.len(), 2);
}

#[test]
fn a_range_already_purged_holds_nothing_more() {
    let mut f = fixture();
    let plan = ready(f.plan(1, 2));
    carry_out(&mut f.store, &f.node, &plan, None, &mut OsEntropy).unwrap();

    assert!(matches!(f.plan(1, 2), Planned::Refused(r) if r == [Refusal::NothingHeld]));
    assert!(
        matches!(f.plan(1, 2), Planned::Refused(r) if r[0].detail() == "refused: the range holds no event")
    );
}

#[test]
fn a_run_with_lines_past_its_ingest_marker_or_held_observations_is_refused() {
    let mut f = fixture();
    f.store
        .add_run(&store::IngestedRun {
            writer: f.ingest.writer,
            path: "/t/a.jsonl".to_owned(),
            harness_session: "s".to_owned(),
            agent_id: None,
        })
        .unwrap();
    assert!(
        matches!(f.plan(1, 1), Planned::Ready(_)),
        "no marker, no pending ingest"
    );
    let source = f.store.start_generation("/t/a.jsonl", "/t").unwrap();
    let mut position = Position::start();
    position.seen = 10;
    f.store.advance(source.id, &position).unwrap();

    let Planned::Refused(why) = f.plan(1, 1) else {
        panic!("planned");
    };
    assert_eq!(
        why,
        [Refusal::PendingIngest {
            writer: f.ingest.writer
        }]
    );
    assert_eq!(
        why[0].to_string(),
        format!(
            "writer {} has a pending ingest marker: run parinn ingest to bring its run in first",
            f.ingest.writer
        )
    );

    position.offset = 10;
    f.store.advance(source.id, &position).unwrap();
    assert!(matches!(f.plan(1, 1), Planned::Ready(_)));
    f.store
        .hold_observation(
            &store::Observation {
                hook: "SessionStart".to_owned(),
                harness_session: "s".to_owned(),
                transcript_path: "/t/a.jsonl".to_owned(),
                cwd: "/w".to_owned(),
                offset: None,
                source: None,
                prompt_id: None,
                attended: false,
                ancestry: None,
                harness: None,
                at: 1,
            },
            None,
        )
        .unwrap();
    assert!(
        matches!(f.plan(1, 1), Planned::Refused(r) if r[0].detail().ends_with("pending ingest marker"))
    );
}

#[test]
fn a_purge_records_its_act_and_tombstone_and_erases_the_content() {
    let mut f = fixture();
    let plan = ready(f.plan(1, 2));
    let act = event(
        &Act::Purge {
            ranges: vec!["r".to_owned()],
            reason: None,
        },
        &mut OsEntropy,
    )
    .unwrap();

    let done = carry_out(&mut f.store, &f.node, &plan, Some(act), &mut OsEntropy).unwrap();

    assert_eq!(done.erased, 2);
    let device = f.node.device_writer();
    let act_at = done.act.unwrap();
    assert_eq!(act_at.writer, device);
    assert_eq!(
        done.tombstone,
        Address::new(device, act_at.seq.next().unwrap())
    );
    let tombstone = f.store.event(done.tombstone).unwrap().unwrap();
    assert_eq!(tombstone.link.header.kind, EventKind::Tombstone);
    assert_eq!(tombstone.link.header.provenance, Provenance::Structural);
    assert_eq!(
        Tombstone::from_value(&tombstone.content.unwrap()),
        Some(plan.tombstone)
    );
    let log = f.store.log(f.ingest.writer).unwrap();
    assert!(log[0].content.is_none() && log[1].content.is_none());
    assert!(log[2].content.is_some());
}

#[test]
fn a_retention_purge_records_a_tombstone_and_no_act() {
    let mut f = fixture();
    let plan = ready(f.plan(1, 1));

    let done = carry_out(&mut f.store, &f.node, &plan, None, &mut OsEntropy).unwrap();

    assert_eq!(done.act, None);
    let kinds: Vec<EventKind> = f
        .store
        .log(f.node.device_writer())
        .unwrap()
        .iter()
        .map(|s| s.link.header.kind)
        .rev()
        .take(1)
        .collect();
    assert_eq!(kinds, [EventKind::Tombstone]);
}

#[test]
fn a_purge_that_cannot_draw_a_key_changes_nothing() {
    struct Dry;
    impl Entropy for Dry {
        fn fill(&mut self, _: &mut [u8]) -> Result<(), key::KeyError> {
            Err(key::KeyError::Malformed("entropy".into()))
        }
    }
    let mut f = fixture();
    let plan = ready(f.plan(1, 1));

    assert!(carry_out(&mut f.store, &f.node, &plan, None, &mut Dry).is_err());
    assert!(f.store.log(f.ingest.writer).unwrap()[0].content.is_some());
}

#[test]
fn a_value_that_is_no_tombstone_is_none_and_one_covers_its_range_alone() {
    let f = fixture();
    let t = ready(f.plan(1, 2)).tombstone;
    let at = |n| Address::new(f.ingest.writer, seq(n));

    assert!(t.covers(at(1)) && t.covers(at(2)));
    assert!(!t.covers(at(3)));
    assert!(!t.covers(Address::new(f.node.device_writer(), seq(1))));
    for field in ["writer", "first", "last", "count", "reason", "commitments"] {
        let mut value = t.to_value();
        value.as_object_mut().unwrap().remove(field);
        assert_eq!(Tombstone::from_value(&value), None, "{field}");
    }
    let mut zero = t.to_value();
    zero["first"] = json!(0);
    assert_eq!(Tombstone::from_value(&zero), None);
    assert_eq!(Reason::Purged.as_str(), "purged");
}
