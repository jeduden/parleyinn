use administration::home::Home;
use administration::node::Node;
use key::random::{Entropy, OsEntropy};
use record::event::Provenance;
use store::Store;

use super::*;
use crate::tests::{DEV, NOW, Process, audited, transcript_in};
use crate::{EXIT_OK, settings};

const DAY: u64 = 86_400;

/// A policy file for every room's `unparsed` lines.
const EVERY_ROOM: &str = "[retention_policy.\"*\"]\nunparsed = 30\n";

fn counted(p: &Process, counter: &str) -> Vec<String> {
    audited(p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some(counter))
        .map(|e| e.detail)
        .collect()
}

fn open(p: &Process) -> (Home, Store, Node) {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let store = Store::open(&home.store()).unwrap();
    let node = Node::find(&home, &store).unwrap();

    (home, store, node)
}

/// The retention policies the process's settings hold.
fn policies_of(p: &Process) -> Vec<administration::retention::Policy> {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    p.with_context(&DEV, |ctx| {
        let in_force = settings::in_force(ctx, &home, &[]);

        policies(&in_force.resolved.settings)
    })
}

#[test]
fn the_policies_are_the_retention_keys_that_read_as_a_room_and_a_class() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let (_, _, node) = open(&p);
    let room = node.personal_room();
    p.manage(&format!(
        "[retention_policy.\"*\"]\nweb = 30\n\"tool_result:Bash\" = 7\n\
         [retention_policy.\"{room}\"]\nuser = 365\n"
    ));

    let found = policies_of(&p);

    let shown: Vec<(String, Option<RoomId>, String, u64)> = found
        .into_iter()
        .map(|p| (p.key, p.room, p.class.name(), p.days))
        .collect();
    assert_eq!(
        shown,
        [
            (
                "retention_policy.*.tool_result:Bash".to_owned(),
                None,
                "tool_result:Bash".to_owned(),
                7
            ),
            (
                "retention_policy.*.web".to_owned(),
                None,
                "web".to_owned(),
                30
            ),
            (
                format!("retention_policy.{room}.user"),
                Some(room),
                "user".to_owned(),
                365
            ),
        ]
    );
    let nothing = Process::new();
    assert!(policies_of(&nothing).is_empty());
}

#[test]
fn an_ingest_after_the_window_purges_the_aged_lines_naming_the_policy_audited_and_counted() {
    let mut p = Process::new();
    p.manage(EVERY_ROOM);
    transcript_in(&p, "-w-a", "a.jsonl", b"one\ntwo\n");
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    p.now = NOW + 31 * DAY;
    transcript_in(&p, "-w-b", "b.jsonl", b"three\n");

    let (code, _, err) = p.invoke(&["ingest", "--all"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let (_, store, _) = open(&p);
    let runs = store.ingested_runs().unwrap();
    let first = store.log(runs[0].writer).unwrap();
    assert!(
        first.iter().all(|s| s.content.is_none()),
        "the aged lines went"
    );
    let second = store.log(runs[1].writer).unwrap();
    assert!(
        second.iter().all(|s| s.content.is_some()),
        "the new line is kept"
    );
    let entry = audited(&p).into_iter().find(|e| e.kind == "purge").unwrap();
    assert_eq!(entry.counter.as_deref(), Some("retention_purged"));
    assert!(
        entry
            .detail
            .ends_with("(retention) by retention policy retention_policy.*.unparsed"),
        "{}",
        entry.detail
    );
    assert_eq!(counted(&p, "retention_purged").len(), 1);
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
}

#[test]
fn before_the_window_ends_and_with_no_policy_nothing_is_purged() {
    let mut p = Process::new();
    transcript_in(&p, "-w-a", "a.jsonl", b"one\n");
    p.manage(EVERY_ROOM);
    p.invoke(&["ingest", "--all"], &DEV);
    p.now = NOW + 30 * DAY;
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    let (_, store, _) = open(&p);
    assert!(
        store.log(store.ingested_runs().unwrap()[0].writer).unwrap()[0]
            .content
            .is_some()
    );
    drop(store);

    let none = Process::new();
    transcript_in(&none, "-w-a", "a.jsonl", b"one\n");
    none.invoke(&["ingest", "--all"], &DEV);
    let mut later = none;
    later.now = NOW + 900 * DAY;
    later.invoke(&["ingest", "--all"], &DEV);
    let (_, store, node) = open(&later);
    assert!(
        store.log(store.ingested_runs().unwrap()[0].writer).unwrap()[0]
            .content
            .is_some()
    );
    let marks = store
        .contents(node.device_writer(), record::event::EventKind::TimeMark)
        .unwrap();
    assert!(marks.is_empty(), "no policy, no mark");
}

struct Dry;

impl Entropy for Dry {
    fn fill(&mut self, _: &mut [u8]) -> Result<(), key::KeyError> {
        Err(key::KeyError::Malformed("entropy".into()))
    }
}

#[test]
fn a_failure_to_apply_the_policies_is_audited_and_counted_and_stops_nothing() {
    let mut p = Process::new();
    p.manage(EVERY_ROOM);
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let (home, mut store, node) = open(&p);
    // A mark is due again.
    p.now = NOW + DAY;

    p.with_context(&DEV, |ctx| {
        let in_force = settings::in_force(ctx, &home, &[]);
        apply(
            ctx,
            &mut store,
            &node,
            &in_force.resolved.settings,
            &mut Dry,
        );
    });

    assert_eq!(counted(&p, "retention_failed").len(), 1);
}

#[test]
fn a_range_the_purge_refuses_is_counted_and_never_erased() {
    let mut p = Process::new();
    p.manage(EVERY_ROOM);
    transcript_in(&p, "-w-a", "a.jsonl", b"one\n");
    p.invoke(&["ingest", "--all"], &DEV);
    let (home, mut store, node) = open(&p);
    let source = store.sources().unwrap().remove(0);
    let behind = store::Position {
        seen: source.position.offset + 10,
        ..source.position
    };
    store.advance(source.id, &behind).unwrap();
    p.now = NOW + 31 * DAY;

    p.with_context(&DEV, |ctx| {
        let in_force = settings::in_force(ctx, &home, &[]);
        apply(
            ctx,
            &mut store,
            &node,
            &in_force.resolved.settings,
            &mut OsEntropy,
        );
    });

    assert!(counted(&p, "purge_refused")[0].contains("pending ingest marker"));
    let writer = store.ingested_runs().unwrap()[0].writer;
    assert!(store.log(writer).unwrap()[0].content.is_some());
    assert!(Provenance::parse("unparsed").is_some());
}

#[test]
fn a_key_is_a_policy_only_when_it_names_a_room_a_class_and_a_window() {
    let days = Value::Integer(30);
    let id = RoomId::derive(&[1; 32], &[2; 16]).to_string();

    assert!(policy("retention_policy.*.web", &days).is_some());
    assert!(policy(&format!("retention_policy.{id}.web"), &days).is_some());
    for (name, value) in [
        ("recall.max_k", Value::Integer(30)),
        ("retention_policy.*.web", Value::Bool(true)),
        ("retention_policy.*.nonsense", Value::Integer(30)),
        ("retention_policy.not-a-room.web", Value::Integer(30)),
        ("retention_policy.*.web", Value::Integer(-1)),
    ] {
        assert!(policy(name, &value).is_none(), "{name}");
    }
}

#[test]
fn a_purge_whose_logs_were_held_open_is_audited_as_residue_too() {
    let p = Process::new();
    let range = administration::retention::Aged {
        writer: record::id::WriterId::derive(&[1; 32]),
        first: record::id::Seq::FIRST,
        last: record::id::Seq::FIRST,
        policy: "retention_policy.*.web".to_owned(),
    };
    let done = administration::purge::Done {
        act: None,
        tombstone: record::id::Address::new(range.writer, range.first),
        erased: 1,
        finished: store::Finished::default(),
    };
    let applied = Applied {
        purged: vec![(range, done)],
        refused: Vec::new(),
    };

    p.with_context(&DEV, |ctx| audit(ctx, &applied));

    assert_eq!(counted(&p, "retention_purged").len(), 1);
    assert_eq!(counted(&p, "purge_unfinished").len(), 1);
}

#[test]
fn a_store_that_would_not_open_is_audited_as_a_retention_failure() {
    let p = Process::new();
    let none = configuration::resolve::Settings::defaults();

    p.with_context(&DEV, |ctx| {
        apply_opened(
            ctx,
            Err("the store is gone".to_owned()),
            &none,
            &mut OsEntropy,
        );
    });

    assert_eq!(counted(&p, "retention_failed").len(), 1);
}
