use super::*;
use crate::home::Home;
use crate::node_identity::IdentitySource;
use key::random::OsEntropy;
use testkit::TempDir;

const T0: u64 = 1_760_000_000;

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

struct Fixture {
    _dir: TempDir,
    store: Store,
    node: Node,
    ingest: store::StoredSeat,
}

fn line(n: u8, class: Provenance, kind: EventKind) -> NewEvent {
    NewEvent {
        kind,
        provenance: class,
        content: json!({"text": format!("line {n}")}),
        commitment_key: CommitmentKey::from_bytes([n; 32]),
        transcript_timestamp: None,
    }
}

/// A node with an ingest seat; no event yet.
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

    Fixture {
        _dir: dir,
        store,
        node,
        ingest,
    }
}

impl Fixture {
    fn lines(&mut self, classes: &[Provenance]) {
        let events: Vec<NewEvent> = classes
            .iter()
            .zip(1..)
            .map(|(c, n)| line(n, c.clone(), EventKind::TranscriptLine))
            .collect();
        self.store.append(self.ingest.writer, &events).unwrap();
    }

    fn mark(&mut self, now: u64) -> Option<Address> {
        mark(&mut self.store, &self.node, now, &mut OsEntropy).unwrap()
    }

    fn aged(&self, policies: &[Policy], now: u64) -> Vec<(u64, u64, String)> {
        aged(&self.store, self.node.personal_room(), policies, now)
            .unwrap()
            .into_iter()
            .filter(|a| a.writer == self.ingest.writer)
            .map(|a| (a.first.get(), a.last.get(), a.policy))
            .collect()
    }
}

fn policy(class: Provenance, days: u64) -> Policy {
    Policy {
        key: format!("retention_policy.*.{}", class.name()),
        room: None,
        class,
        days,
    }
}

#[test]
fn a_mark_is_recorded_when_none_is_or_the_last_is_an_interval_old() {
    let mut f = fixture();

    let first = f.mark(T0).unwrap();
    assert_eq!(first.writer, f.node.device_writer());
    assert_eq!(f.mark(T0 + MARK_EVERY - 1), None);
    assert!(f.mark(T0 + MARK_EVERY).is_some());
    assert_eq!(f.mark(T0 + 5), None, "a clock gone back records none");

    let log = f.store.log(f.node.device_writer()).unwrap();
    let marks: Vec<&store::Stored> = log
        .iter()
        .filter(|s| s.link.header.kind == EventKind::TimeMark)
        .collect();
    assert_eq!(marks.len(), 2);
    assert_eq!(marks[0].link.header.provenance, Provenance::Structural);
    assert_eq!(marks[1].content, Some(json!({"time": T0 + MARK_EVERY})));
}

#[test]
fn an_event_ages_from_the_first_mark_after_it_not_from_when_it_was_recorded() {
    let mut f = fixture();
    f.lines(&[Provenance::Web, Provenance::Web]);
    let policies = [policy(Provenance::Web, 30)];
    // No mark follows the events yet: they have no age.
    assert!(f.aged(&policies, T0 + 90 * DAY).is_empty());
    f.mark(T0);
    f.store
        .append(
            f.ingest.writer,
            &[line(3, Provenance::Web, EventKind::TranscriptLine)],
        )
        .unwrap();

    assert!(
        f.aged(&policies, T0 + 30 * DAY).is_empty(),
        "not past the window"
    );
    assert_eq!(
        f.aged(&policies, T0 + 30 * DAY + 1),
        [(1, 2, "retention_policy.*.web".to_owned())],
        "the third follows no mark yet"
    );
    f.mark(T0 + 40 * DAY);
    assert_eq!(f.aged(&policies, T0 + 40 * DAY).len(), 1);
    assert_eq!(f.aged(&policies, T0 + 70 * DAY + 1)[0].1, 3);
}

#[test]
fn only_a_covered_class_ages_and_a_run_ends_where_the_class_changes() {
    let mut f = fixture();
    f.lines(&[
        Provenance::Web,
        Provenance::Web,
        Provenance::User,
        Provenance::Web,
        Provenance::File,
    ]);
    f.mark(T0);
    let policies = [policy(Provenance::Web, 1), policy(Provenance::File, 1)];

    let aged = f.aged(&policies, T0 + 2 * DAY);

    let ranges: Vec<(u64, u64)> = aged.iter().map(|(a, b, _)| (*a, *b)).collect();
    assert_eq!(ranges, [(1, 2), (4, 4), (5, 5)]);
    assert!(
        f.aged(&[], T0 + 2 * DAY).is_empty(),
        "the default keeps everything"
    );
}

#[test]
fn a_room_s_own_policy_wins_over_the_one_for_every_room_and_another_room_has_none() {
    let mut f = fixture();
    f.lines(&[Provenance::Web]);
    f.mark(T0);
    let room = f.ingest.room;
    let other = RoomId::derive(&[9; 32], &[9; 16]);
    let every = policy(Provenance::Web, 1);
    let own = Policy {
        key: "own".to_owned(),
        room: Some(room),
        days: 100,
        ..policy(Provenance::Web, 1)
    };
    let elsewhere = Policy {
        key: "elsewhere".to_owned(),
        room: Some(other),
        days: 1,
        ..policy(Provenance::Web, 1)
    };
    let now = T0 + 2 * DAY;

    assert_eq!(f.aged(&[every.clone(), own.clone()], now), []);
    assert_eq!(f.aged(&[every, own], T0 + 101 * DAY)[0].2, "own");
    assert_eq!(
        f.aged(&[elsewhere], now),
        [],
        "a policy of another room covers none"
    );
}

#[test]
fn a_purge_may_not_erase_what_it_keeps_nor_an_erased_event_again() {
    let mut f = fixture();
    f.store
        .append(
            f.ingest.writer,
            &[
                line(1, Provenance::Web, EventKind::TranscriptLine),
                line(2, Provenance::Web, EventKind::RepeatReference),
                line(3, Provenance::Web, EventKind::UnparsedLine),
            ],
        )
        .unwrap();
    f.mark(T0);
    let policies = [policy(Provenance::Web, 1)];
    let now = T0 + 2 * DAY;
    assert_eq!(
        f.aged(&policies, now).len(),
        2,
        "the reference ends the run"
    );

    let planned = crate::purge::plan(
        &f.store,
        f.ingest.writer,
        seq(3),
        seq(3),
        &crate::purge::Reason::Purged,
    )
    .unwrap();
    let crate::purge::Planned::Ready(plan) = planned else {
        panic!("refused");
    };
    crate::purge::carry_out(&mut f.store, &f.node, &plan, None, &mut OsEntropy).unwrap();

    assert_eq!(
        f.aged(&policies, now).len(),
        1,
        "an erased event ages no more"
    );
}

#[test]
fn the_device_seat_s_own_events_age_from_the_mark_after_them() {
    let mut f = fixture();
    let device = f.node.device_writer();
    f.store
        .append(device, &[line(1, Provenance::Assistant, EventKind::Recall)])
        .unwrap();
    f.mark(T0);
    let policies = [policy(Provenance::Assistant, 1)];

    let found = aged(&f.store, f.node.personal_room(), &policies, T0 + 2 * DAY).unwrap();

    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].writer, found[0].first),
        (
            device,
            seq(f.store.head(device).unwrap().unwrap().get() - 1)
        )
    );
}

#[test]
fn a_citation_no_mark_follows_dates_nothing() {
    let mut f = fixture();
    f.lines(&[Provenance::Web]);
    f.store
        .append(
            f.node.device_writer(),
            &[line(9, Provenance::Structural, EventKind::Recall)],
        )
        .unwrap();

    assert!(
        f.aged(&[policy(Provenance::Web, 1)], T0 + 9 * DAY)
            .is_empty()
    );
}

#[test]
fn applying_marks_the_time_then_purges_what_a_window_has_passed_naming_the_policy() {
    let mut f = fixture();
    f.lines(&[Provenance::Web, Provenance::User]);
    let policies = [policy(Provenance::Web, 30)];

    let first = apply(&mut f.store, &f.node, &policies, T0, &mut OsEntropy).unwrap();
    assert_eq!(
        first,
        Applied::default(),
        "the mark dates the events; nothing is old yet"
    );
    let later = apply(
        &mut f.store,
        &f.node,
        &policies,
        T0 + 31 * DAY,
        &mut OsEntropy,
    )
    .unwrap();

    assert_eq!(later.purged.len(), 1);
    let (range, done) = &later.purged[0];
    assert_eq!((range.first.get(), range.last.get()), (1, 1));
    assert_eq!(done.act, None);
    let tombstone = f.store.event(done.tombstone).unwrap().unwrap();
    let tombstone = crate::purge::Tombstone::from_value(&tombstone.content.unwrap()).unwrap();
    assert_eq!(tombstone.reason, "retention");
    assert_eq!(tombstone.policy.as_deref(), Some("retention_policy.*.web"));
    let log = f.store.log(f.ingest.writer).unwrap();
    assert!(
        log[0].content.is_none() && log[1].content.is_some(),
        "a user line is kept"
    );
    assert!(crate::verify::verify(&f.store, &f.node.device().public()).is_ok());
}

#[test]
fn with_no_policy_applying_records_no_mark_and_removes_nothing() {
    let mut f = fixture();
    f.lines(&[Provenance::Web]);
    let before = f.store.head(f.node.device_writer()).unwrap();

    let applied = apply(&mut f.store, &f.node, &[], T0 + 900 * DAY, &mut OsEntropy).unwrap();

    assert_eq!(applied, Applied::default());
    assert_eq!(f.store.head(f.node.device_writer()).unwrap(), before);
    assert!(f.store.log(f.ingest.writer).unwrap()[0].content.is_some());
}

#[test]
fn a_range_a_purge_refuses_is_reported_not_purged() {
    let mut f = fixture();
    f.lines(&[Provenance::Web]);
    f.store
        .add_run(&store::IngestedRun {
            writer: f.ingest.writer,
            path: "/t/a.jsonl".to_owned(),
            harness_session: "s".to_owned(),
            agent_id: None,
        })
        .unwrap();
    let source = f.store.start_generation("/t/a.jsonl", "/t").unwrap();
    let mut position = store::Position::start();
    position.seen = 10;
    f.store.advance(source.id, &position).unwrap();
    let policies = [policy(Provenance::Web, 1)];
    apply(&mut f.store, &f.node, &policies, T0, &mut OsEntropy).unwrap();

    let applied = apply(
        &mut f.store,
        &f.node,
        &policies,
        T0 + 3 * DAY,
        &mut OsEntropy,
    )
    .unwrap();

    assert!(applied.purged.is_empty());
    assert_eq!(
        applied.refused[0].1,
        [Refusal::PendingIngest {
            writer: f.ingest.writer
        }]
    );
    assert!(f.store.log(f.ingest.writer).unwrap()[0].content.is_some());
}

#[test]
fn a_failure_to_draw_a_key_stops_the_application() {
    struct Dry;
    impl Entropy for Dry {
        fn fill(&mut self, _: &mut [u8]) -> Result<(), key::KeyError> {
            Err(key::KeyError::Malformed("entropy".into()))
        }
    }
    let mut f = fixture();

    assert!(
        apply(
            &mut f.store,
            &f.node,
            &[policy(Provenance::Web, 1)],
            T0,
            &mut Dry
        )
        .is_err()
    );
}
