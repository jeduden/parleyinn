use super::*;
use crate::settings::tests::ctx_of;
use crate::tests::{DEV, Process, audited};
use administration::home::Home;
use mcp::Note;
use record::id::WriterId;

fn entries(p: &Process, notes: &[Note]) -> Vec<(String, Option<String>, String, String)> {
    audit_notes(&ctx_of(p), notes);

    audited(p)
        .into_iter()
        .map(|e| (e.kind, e.counter, e.operation, e.detail))
        .collect()
}

#[test]
fn a_call_cut_at_its_cap_is_counted_as_truncated() {
    let p = Process::new();

    let got = entries(&p, &[Note::Truncated("event_expand")]);

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].0, "truncated");
    assert_eq!(got[0].1.as_deref(), Some("recall_truncated"));
    assert_eq!(got[0].2, "event_expand");
}

#[test]
fn a_recall_that_extends_past_the_run_is_logged_without_a_counter() {
    let p = Process::new();

    let got = entries(
        &p,
        &[
            Note::Extended {
                tool: "event_search",
                scope: "room",
            },
            Note::Extended {
                tool: "event_get",
                scope: "rooms",
            },
        ],
    );

    assert_eq!(got.len(), 2);
    assert!(
        got.iter()
            .all(|e| e.0 == "recall_extended" && e.1.is_none())
    );
    assert!(got[0].3.ends_with("to a room the run has a seat in"));
    assert!(got[1].3.ends_with("to every room the run has a seat in"));
    assert_eq!(got[0].2, "event_search");
}

#[test]
fn an_unrecorded_recall_and_a_redacted_argument_are_counted() {
    let p = Process::new();

    let got = entries(
        &p,
        &[
            Note::NotRecorded("stat_list"),
            Note::Redacted("aws_access_key".to_owned()),
        ],
    );

    assert_eq!(got[0].0, "dropped");
    assert_eq!(got[0].1.as_deref(), Some("recall_not_recorded"));
    assert_eq!(got[1].0, "redacted");
    assert_eq!(got[1].1.as_deref(), Some("redacted"));
    assert!(got[1].3.ends_with(" aws_access_key"), "{}", got[1].3);
}

#[test]
fn the_recorder_redacts_by_the_patterns_it_was_given_and_records_in_the_device_writer() {
    let writer = WriterId::derive(&[4; 32]);
    let (redactor, _) = Redactor::new(&["ACME-[0-9]{8}".to_owned()]).unwrap();

    let recorder = recorder(writer, Some(redactor));

    assert_eq!(recorder.writer, writer);
    let (text, rules) = (recorder.redact.unwrap())("token ACME-12345678 here");
    assert_eq!(text, "token [REDACTED:extra_pattern_1] here");
    assert_eq!(rules, ["extra_pattern_1"]);
    assert!(super::recorder(writer, None).redact.is_none());
}

#[test]
fn no_redactor_is_made_while_a_settings_layer_fails() {
    let p = Process::new();
    p.configure("recal.max_k = 10\n");
    let ctx = ctx_of(&p);
    let home = ctx.home().unwrap();
    let in_force = crate::settings::in_force(&ctx, &home, &[]);
    let mut err = Vec::new();

    let none = redactor(&ctx, &in_force, &mut err);

    assert!(none.is_none());
    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    assert!(counters.contains(&Some("ingest_refused_settings_invalid".to_owned())));
}

#[test]
fn a_redactor_is_made_from_the_patterns_in_force() {
    let p = Process::new();
    p.configure("[redaction]\nextra_patterns = [\"ACME-[0-9]{8}\"]\n");
    let ctx = ctx_of(&p);
    let home = ctx.home().unwrap();
    let in_force = crate::settings::in_force(&ctx, &home, &[]);
    let mut err = Vec::new();

    let made = redactor(&ctx, &in_force, &mut err).unwrap();

    assert_eq!(made.redact_text("ACME-12345678").rules, ["extra_pattern_1"]);
    assert!(err.is_empty());
}

#[test]
fn an_mcp_server_records_each_recall_it_serves_and_logs_one_that_extends_past_the_run() {
    let mut p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\nb\n")], &DEV);
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let store = store::Store::open(&home.store()).unwrap();
    let writer = store
        .seats()
        .unwrap()
        .into_iter()
        .find(|s| !administration::origin::is_device_seat(s))
        .unwrap()
        .writer;
    drop(store);
    let call = |id: u64, tool: &str, arguments: serde_json::Value| {
        serde_json::json!({"jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": {"name": tool, "arguments": arguments}})
        .to_string()
    };
    p.lines = vec![
        call(
            1,
            "event_expand",
            serde_json::json!({"range": format!("{writer}:1-2"), "scope": "room"}),
        ),
        call(2, "stat_list", serde_json::json!({})),
    ];

    let (code, out, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (crate::EXIT_OK, ""));
    let replies: Vec<serde_json::Value> = out
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(replies[0]["result"]["isError"], false, "{out}");
    let stats: serde_json::Value =
        serde_json::from_str(replies[1]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(stats["structural"]["recalls"], 1, "{stats}");
    let kinds: Vec<String> = audited(&p).into_iter().map(|e| e.kind).collect();
    assert_eq!(kinds.iter().filter(|k| *k == "recall_extended").count(), 1);
    let store = store::Store::open(&home.store()).unwrap();
    let device = administration::node::Node::find(&home, &store)
        .unwrap()
        .device_writer();
    let recalls = store
        .log(device)
        .unwrap()
        .into_iter()
        .filter(|s| s.link.header.kind == record::event::EventKind::Recall)
        .count();
    assert_eq!(recalls, 2);
    // The record the recalls and the labels grew still verifies.
    let (verified, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!((verified, err.as_str()), (crate::EXIT_OK, ""));
}
