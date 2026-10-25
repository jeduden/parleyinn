use std::time::Duration;

use administration::home::Home;
use administration::node::Node;
use record::event::EventKind;
use record::id::{Address, Seq, WriterId};
use serde_json::Value;
use store::{Position, Store};

use super::*;
use crate::tests::{DEV, Process, audited};
use crate::{EXIT_FAILURE, EXIT_OK, EXIT_USAGE};

/// Four lines the parser leaves unread, so each is an unparsed line.
const LINES: &str = "one\ntwo\nthree\nfour\n";

fn open(p: &Process) -> (Home, Store) {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let store = Store::open(&home.store()).unwrap();

    (home, store)
}

/// A process whose node ingested `LINES` and accepted the open risks,
/// as a person must before a widening act; the run's writer too.
fn with_a_run() -> (Process, WriterId) {
    let p = Process::new();
    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript(LINES)], &DEV);
    assert_eq!(code, EXIT_OK, "{err}");
    let (code, _, err) = p.invoke(&["risk-acceptance", "record"], &DEV);
    assert_eq!(code, EXIT_OK, "{err}");
    let writer = writer_of(&p);

    (p, writer)
}

fn writer_of(p: &Process) -> WriterId {
    let (_, store) = open(p);

    store.ingested_runs().unwrap()[0].writer
}

fn counted(p: &Process, counter: &str) -> Vec<String> {
    audited(p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some(counter))
        .map(|e| e.detail)
        .collect()
}

#[test]
fn the_command_line_wants_a_range_and_knows_only_a_reason_and_json() {
    for (args, why) in [
        (
            &["purge"][..],
            "parinn purge: want --range, such as A1:30-40\n",
        ),
        (
            &["purge", "--range"],
            "parinn purge: --range wants a value\n",
        ),
        (
            &["purge", "--range", "A1:1", "--reason"],
            "parinn purge: --reason wants a value\n",
        ),
        (
            &["purge", "--run", "r-1"],
            "parinn purge: unknown flag \"--run\"; M1 purges an address range alone: --range\n",
        ),
        (&["purge", "A1:1"], "parinn purge: unexpected \"A1:1\"\n"),
    ] {
        assert_eq!(
            Process::new().invoke(args, &DEV),
            (EXIT_USAGE, String::new(), why.into()),
            "{args:?}"
        );
    }
}

#[test]
fn a_purge_without_a_terminal_is_refused_before_anything_and_counted() {
    let mut p = Process::new();
    p.terminal = false;

    let (code, out, err) = p.invoke(&["purge", "--range", "A1:1-2"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(
        err.contains("refused: a principal act needs a terminal"),
        "{err}"
    );
    assert_eq!(counted(&p, "principal_act_refused").len(), 1);
    assert!(!p.parinn_home().join("store.db").exists());
}

#[test]
fn a_widening_purge_needs_the_risk_acceptance_a_run_leaves_open() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(LINES)], &DEV);
    let writer = writer_of(&p);

    let (code, _, err) = p.invoke(&["purge", "--range", &format!("{writer}:2-3")], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("residual risks are open"), "{err}");
    let (_, store) = open(&p);
    let line = store
        .event(Address::new(writer, Seq::new(2).unwrap()))
        .unwrap()
        .unwrap();
    assert!(line.content.is_some(), "nothing was erased");
    assert_eq!(counted(&p, "principal_act_refused").len(), 1);
}

#[test]
fn a_purge_erases_the_range_records_its_act_and_tombstone_and_the_record_still_verifies() {
    let (p, writer) = with_a_run();

    let (code, out, err) = p.invoke(
        &[
            "purge",
            "--range",
            &format!("{writer}:2-3"),
            "--reason",
            "a secret",
        ],
        &DEV,
    );

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(
        out.starts_with(&format!("purged 2 events of {writer}:2-3; tombstone ")),
        "{out}"
    );
    let (_, store) = open(&p);
    let log = store.log(writer).unwrap();
    let erased: Vec<bool> = log.iter().map(|s| s.content.is_none()).collect();
    assert_eq!(erased, [false, true, true, false]);
    let acts = crate::tests::recorded(&p);
    let purge = acts.iter().find(|a| a["act"] == "purge").unwrap();
    assert_eq!(purge["class"], "widening");
    assert_eq!(purge["ranges"][0], format!("{writer}:2-3"));
    assert_eq!(purge["reason"], "a secret");
    let entry = audited(&p).into_iter().find(|e| e.kind == "purge").unwrap();
    assert_eq!(entry.counter, None);
    assert!(
        entry
            .detail
            .starts_with(&format!("purge of {writer}:2-3 (purged) by principal act "))
    );
    assert!(!entry.detail.contains("secret"));
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
    let (_, shown, _) = p.invoke(&["event", "expand", &format!("{writer}:1-4")], &DEV);
    assert!(
        shown.contains(&format!("{writer}")) || shown.contains("one"),
        "{shown}"
    );
    assert!(!shown.contains("two") && !shown.contains("three"));
}

#[test]
fn a_range_written_by_label_is_purged_and_json_names_what_it_did() {
    let (p, writer) = with_a_run();

    let (code, out, err) = p.invoke(&["purge", "--range", "A1·4–4", "--json"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let got: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(got["purged"], 1);
    assert_eq!(got["range"], format!("{writer}:4-4"));
    assert!(got["act"].as_str().unwrap().contains(':'));
    assert!(got["tombstone"].as_str().unwrap().contains(':'));
}

#[test]
fn a_range_that_names_no_writer_or_a_foreign_room_is_refused_saying_why() {
    let (p, writer) = with_a_run();
    let other_room = record::id::RoomId::derive(&[3; 32], &[3; 16]);

    for (range, why) in [
        ("A99:1-2".to_owned(), "no writer has that label"),
        ("nonsense".to_owned(), "not an address or range"),
        (
            format!("parinn:room/{other_room}/w/{writer}/1-2"),
            "the writer has no seat in the room the address names",
        ),
    ] {
        let (code, _, err) = p.invoke(&["purge", "--range", &range], &DEV);

        assert_eq!(code, EXIT_FAILURE, "{range}");
        assert!(err.contains(why), "{range}: {err}");
    }
}

#[test]
fn a_range_holding_an_event_a_purge_keeps_is_refused_audited_and_counted_once_a_kind() {
    let (p, _) = with_a_run();
    let (home, store) = open(&p);
    let device = Node::find(&home, &store).unwrap().device_writer();
    drop(store);

    let (code, out, err) = p.invoke(&["purge", "--range", &format!("{device}:1-3")], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(err.starts_with("parinn purge: refused:\n  "), "{err}");
    assert!(err.contains("event, which a purge keeps"), "{err}");
    let kinds: Vec<String> = counted(&p, "purge_refused");
    assert!(
        kinds.len() >= 2 && kinds.iter().all(|d| d.contains("a kind a purge keeps")),
        "{kinds:?}"
    );
    let keys: Vec<String> = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("purge_refused"))
        .map(|e| e.detail)
        .collect();
    let distinct: std::collections::BTreeSet<_> = keys.iter().collect();
    assert_eq!(distinct.len(), keys.len(), "one entry for each kind");
    assert!(keys.iter().any(|d| d.ends_with("key_event")), "{keys:?}");
}

#[test]
fn a_range_with_nothing_left_to_erase_is_refused() {
    let (p, writer) = with_a_run();
    let range = format!("{writer}:1-1");
    assert_eq!(p.invoke(&["purge", "--range", &range], &DEV).0, EXIT_OK);

    let (code, _, err) = p.invoke(&["purge", "--range", &range], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("the writer holds no event in the range"),
        "{err}"
    );
    assert!(counted(&p, "purge_refused")[0].contains("holds no event"));
}

#[test]
fn a_run_with_a_pending_ingest_marker_is_refused_naming_its_writer() {
    let (p, writer) = with_a_run();
    let (_, mut store) = open(&p);
    let source = store.sources().unwrap().remove(0);
    let behind = Position {
        seen: source.position.offset + 100,
        ..source.position
    };
    store.advance(source.id, &behind).unwrap();
    drop(store);

    let (code, _, err) = p.invoke(&["purge", "--range", &format!("{writer}:1-2")], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains(&format!("writer {writer} has a pending ingest marker")),
        "{err}"
    );
    assert!(counted(&p, "purge_refused")[0].contains("pending ingest marker"));
}

#[test]
fn the_causes_of_a_refusal_are_named_up_to_a_limit_and_the_rest_counted() {
    let kept = |n: u64| Refusal::Kept {
        address: Address::new(WriterId::derive(&[1; 32]), Seq::new(n).unwrap()),
        kind: EventKind::KeyEvent,
    };
    let all: Vec<Refusal> = (1..=5).map(kept).collect();

    let text = describe(&all, 3);

    assert_eq!(text.lines().count(), 4);
    assert!(text.ends_with("  and 2 more"), "{text}");
    assert_eq!(describe(&all[..1], 3).lines().count(), 1);
}

#[test]
fn a_residue_the_purge_could_not_clear_is_audited_counted_and_said() {
    let p = Process::new();
    let mut err = Vec::new();

    let code = p.with_context(&DEV, |ctx| residue(ctx, &mut err));

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        String::from_utf8(err)
            .unwrap()
            .contains("pending, not finished: residue remains")
    );
    assert_eq!(counted(&p, "purge_unfinished").len(), 1);
}

#[test]
fn a_purge_that_failed_before_erasing_reports_its_cause_and_no_residue() {
    let (p, _) = with_a_run();
    let mut err = Vec::new();

    let code = p.with_context(&DEV, |ctx| {
        unfinished(ctx, false, "the store is gone", &mut err)
    });

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        String::from_utf8(err).unwrap(),
        "parinn purge: the store is gone\n"
    );
    assert!(counted(&p, "purge_unfinished").is_empty());
}

#[test]
fn a_purge_whose_erasure_stands_but_whose_residue_remains_says_so() {
    let p = Process::new();
    let long = format!("{}\n", "x".repeat(9_000));
    p.invoke(&["ingest", "--path", &p.transcript(&long)], &DEV);
    p.invoke(&["risk-acceptance", "record"], &DEV);
    let writer = writer_of(&p);
    let (home, store) = open(&p);
    let name = store.log(writer).unwrap()[0].payload.clone().unwrap().name;
    drop(store);
    // A payload file that cannot be removed: a directory in its place.
    let path = home.store().parent().unwrap().join("payloads").join(&name);
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir_all(path.join("inside")).unwrap();
    crate::tests::set_mode(&path.join("inside"), 0o700);
    crate::tests::set_mode(&path, 0o700);

    let (code, out, err) = p.invoke(&["purge", "--range", &format!("{writer}:1-1")], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(
        err.contains("pending, not finished: residue remains"),
        "{err}"
    );
    assert!(
        err.trim_end().lines().count() >= 2,
        "the cause follows: {err}"
    );
    assert_eq!(counted(&p, "purge_unfinished").len(), 1);
    let (_, store) = open(&p);
    assert!(
        store.log(writer).unwrap()[0].content.is_none(),
        "the erasure stands"
    );
    assert!(store.purge_pending().unwrap());
}

#[test]
fn a_purge_the_vacuum_could_not_follow_is_counted_and_fails_though_the_erasure_stands() {
    let (p, writer) = with_a_run();
    let (home, _) = open(&p);
    let reader = rusqlite::Connection::open(home.store()).unwrap();
    reader.execute_batch("BEGIN").unwrap();
    reader
        .query_row("SELECT count(*) FROM events", [], |r| r.get::<_, i64>(0))
        .unwrap();
    let held = Duration::from_secs(5);
    assert!(held.as_secs() > 0);

    let (code, out, err) = p.invoke(&["purge", "--range", &format!("{writer}:2-2")], &DEV);

    reader.execute_batch("COMMIT").unwrap();
    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(
        err.contains("pending, not finished: residue remains"),
        "{err}"
    );
    assert!(!err.contains("erased"), "{err}");
    assert_eq!(counted(&p, "purge_unfinished").len(), 1);
    assert!(
        audited(&p).iter().any(|e| e.kind == "purge"),
        "the purge itself was audited"
    );
}

/// A process whose purge of a payload line stands but could not clear
/// its payload file; the file's path.
fn with_residue() -> (Process, std::path::PathBuf) {
    let p = Process::new();
    let long = format!("{}\n", "x".repeat(9_000));
    p.invoke(&["ingest", "--path", &p.transcript(&long)], &DEV);
    p.invoke(&["risk-acceptance", "record"], &DEV);
    let writer = writer_of(&p);
    let (home, store) = open(&p);
    let name = store.log(writer).unwrap()[0].payload.clone().unwrap().name;
    drop(store);
    let path = home.store().parent().unwrap().join("payloads").join(&name);
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir_all(path.join("inside")).unwrap();
    crate::tests::set_mode(&path.join("inside"), 0o700);
    crate::tests::set_mode(&path, 0o700);
    assert_eq!(
        p.invoke(&["purge", "--range", &format!("{writer}:1-1")], &DEV)
            .0,
        EXIT_FAILURE
    );

    (p, path)
}

#[test]
fn the_next_rebuild_or_ingest_finishes_what_a_purge_left_and_a_stuck_step_is_counted_again() {
    let (p, path) = with_residue();
    let before = counted(&p, "purge_unfinished").len();

    let (code, _, err) = p.invoke(&["rebuild"], &DEV);

    assert_eq!(
        (code, err.as_str()),
        (EXIT_OK, ""),
        "a stuck step stops no command"
    );
    assert_eq!(counted(&p, "purge_unfinished").len(), before + 1);
    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("more\n")], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert_eq!(counted(&p, "purge_unfinished").len(), before + 2);

    std::fs::remove_dir_all(&path).unwrap();
    let (code, _, _) = p.invoke(&["rebuild"], &DEV);

    assert_eq!(code, EXIT_OK);
    assert_eq!(
        counted(&p, "purge_unfinished").len(),
        before + 2,
        "finished, nothing more counted"
    );
    let (_, store) = open(&p);
    assert!(!store.purge_pending().unwrap());
}

#[test]
fn finishing_left_residue_when_a_step_failed_or_the_logs_were_held_open() {
    let done = |nothing, vacuumed| {
        Ok(store::Finished {
            nothing,
            vacuumed,
            ..store::Finished::default()
        })
    };

    assert!(!left_over(&done(true, false)), "nothing to finish");
    assert!(!left_over(&done(false, true)), "finished");
    assert!(left_over(&done(false, false)), "the logs were held open");
    assert!(left_over(&Err(store::StoreError::NoIndex)));
}

#[test]
fn failures_say_what_failed_and_a_failure_to_audit_is_only_logged() {
    let Failure::Plain(why) = unnoted("the disk is full") else {
        panic!("a plain failure");
    };
    assert_eq!(
        why,
        "the purge is recorded, but the audit log took no note of it: the disk is full"
    );
    crate::logged("a step", Ok::<(), String>(()));
    crate::logged("a step", Err::<(), _>("it failed"));
}

#[test]
fn a_home_of_a_newer_store_is_refused_by_the_purge_before_anything_else() {
    let (p, writer) = with_a_run();
    let (home, store) = open(&p);
    drop(store);
    let conn = rusqlite::Connection::open(home.store()).unwrap();
    conn.execute(
        "UPDATE meta SET value = '9' WHERE key = 'schema_version'",
        [],
    )
    .unwrap();
    drop(conn);

    let (code, _, err) = p.invoke(&["purge", "--range", &format!("{writer}:1-1")], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("newer than this parinn's"), "{err}");
}

#[test]
fn a_full_address_of_the_writer_s_own_room_is_purged_and_each_kind_kept_is_audited_once() {
    let (p, writer) = with_a_run();
    let (_, store) = open(&p);
    let room = store.seat(writer).unwrap().unwrap().room;
    drop(store);

    let full = format!("parinn:room/{room}/w/{writer}/1-1");
    let (code, out, err) = p.invoke(&["purge", "--range", &full], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""), "{out}");
    let kept = |n| Refusal::Kept {
        address: Address::new(writer, Seq::new(n).unwrap()),
        kind: EventKind::KeyEvent,
    };
    p.with_context(&DEV, |ctx| {
        audit_refusals(ctx, &[kept(1), kept(2), kept(3)])
    });
    assert_eq!(counted(&p, "purge_refused").len(), 1);
}
