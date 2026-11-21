//! The recall tools through the server (RCL-01, RCL-03, RCL-05,
//! RCL-07, RCL-08): the range an event_expand reads, the cap and its
//! cursor, the scopes, the recall event each call leaves and the
//! labels a range is named by.

use super::*;
use envelope::tokens;
use record::id::{Address, Seq};
use record::range::Label;

/// A fixture whose transcript holds the lines `lines`, each an
/// unparsed line, so its text is the line itself.
pub(super) fn fixture_of(lines: &[String]) -> Fixture {
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap();
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, format!("{}\n", lines.join("\n"))).unwrap();
    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    Fixture {
        _dir: dir,
        home,
        ingested,
    }
}

/// `n` lines of `width` characters each, no two alike.
pub(super) fn lines(n: usize, width: usize) -> Vec<String> {
    (1..=n)
        .map(|i| format!("line {i} {}", "w".repeat(width)))
        .collect()
}

pub(super) fn tool(server: &mut Server, name: &str, arguments: Value) -> Value {
    call(
        server,
        json!({"jsonrpc": "2.0", "id": 9, "method": "tools/call", "params": {"name": name, "arguments": arguments}}),
    )["result"]
        .clone()
}

/// The writer of the device seat of the fixture's node.
pub(super) fn device(f: &Fixture) -> WriterId {
    let store = Store::open(&f.home.store()).unwrap();
    Node::find(&f.home, &store).unwrap().device_writer()
}

/// A server that records each recall without redacting anything.
pub(super) fn recording(f: &Fixture) -> Server {
    f.server().with_recall_writer(RecallWriter {
        writer: device(f),
        redact: Some(Box::new(|t| (t.to_owned(), Vec::new(), false))),
    })
}

/// The recall events the device seat recorded.
pub(super) fn recalls(f: &Fixture) -> Vec<Value> {
    let store = Store::open(&f.home.store()).unwrap();
    store
        .log(device(f))
        .unwrap()
        .into_iter()
        .filter(|s| s.link.header.kind == EventKind::Recall)
        .map(|s| s.content.unwrap())
        .collect()
}

/// Runs that find the run of any tool-use id as `name`.
pub(super) struct Fixed(pub(super) String);

impl tool_call::Runs for Fixed {
    fn serving(&self) -> bool {
        true
    }

    fn find(&self, _id: &str, _wait: Duration) -> (Vec<String>, Duration) {
        (vec![self.0.clone()], Duration::ZERO)
    }
}

/// Runs that find no run for any tool-use id.
pub(super) struct NoRun;

impl tool_call::Runs for NoRun {
    fn serving(&self) -> bool {
        true
    }

    fn find(&self, _id: &str, _wait: Duration) -> (Vec<String>, Duration) {
        (Vec::new(), Duration::ZERO)
    }
}

/// The name of the run the fixture's ingest recorded.
pub(super) fn run_name(f: &Fixture) -> String {
    let store = Store::open(&f.home.store()).unwrap();
    store.ingested_runs().unwrap()[0].run_name()
}

/// A call under a tool-use id.
pub(super) fn bound(server: &mut Server, name: &str, arguments: Value) -> Value {
    call(
        server,
        json!({"jsonrpc": "2.0", "id": 9, "method": "tools/call", "params": {
            "name": name, "arguments": arguments, "_meta": {"claudecode/toolUseId": "toolu_1"},
        }}),
    )["result"]
        .clone()
}

pub(super) fn text_of(result: &Value) -> &str {
    result["content"][0]["text"].as_str().unwrap()
}

pub(super) fn contents(envelope: &Value) -> Vec<String> {
    envelope["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["content"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn event_expand_returns_a_range_in_order_whole() {
    let f = fixture_of(&lines(5, 10));
    let mut server = f.server();
    let w = f.ingested.writer;

    let result = tool(
        &mut server,
        "event_expand",
        json!({"range": format!("{w}:2-4"), "scope": "room"}),
    );

    let envelope = envelope_of(&result);
    assert_eq!(
        contents(&envelope),
        [
            format!("line 2 {}", "w".repeat(10)),
            format!("line 3 {}", "w".repeat(10)),
            format!("line 4 {}", "w".repeat(10)),
        ]
    );
    assert_eq!(envelope["truncated"], false);
    assert_eq!(envelope["next_cursor"], Value::Null);
    assert_eq!(envelope["items"][0]["run"], "other");
    assert_eq!(
        envelope["items"][0]["address"],
        Address::new(w, Seq::new(2).unwrap()).full(&f.ingested.room)
    );
}

#[test]
fn a_range_may_be_two_addresses_of_one_writer_and_ends_where_the_writer_does() {
    let f = fixture_of(&lines(3, 5));
    let mut server = f.server();
    let w = f.ingested.writer;

    let two = tool(
        &mut server,
        "event_expand",
        json!({"range": {"from": format!("{w}:1"), "to": format!("{w}:2")}, "scope": "room"}),
    );
    let past = tool(
        &mut server,
        "event_expand",
        json!({"range": format!("{w}:2-99"), "scope": "room"}),
    );

    assert_eq!(contents(&envelope_of(&two)).len(), 2);
    assert_eq!(contents(&envelope_of(&past)).len(), 2);
}

#[test]
fn event_expand_refuses_what_it_cannot_read_in_fixed_text() {
    let f = fixture_of(&lines(3, 5));
    let mut server = f.server();
    let w = f.ingested.writer;
    let other = WriterId::derive(&[9; 32]);
    let cursor = |plain: &str| -> String { plain.bytes().map(|b| format!("{b:02x}")).collect() };

    let cases = [
        (
            json!({"scope": "room"}),
            "event_expand needs a range or a cursor",
        ),
        (json!({"range": 3, "scope": "room"}), expand::BAD_RANGE),
        (json!({"range": "nope", "scope": "room"}), expand::BAD_RANGE),
        (
            json!({"range": {"from": format!("{w}:1")}, "scope": "room"}),
            expand::BAD_RANGE,
        ),
        (
            json!({"range": {"from": format!("{w}:2"), "to": format!("{w}:1")}, "scope": "room"}),
            expand::BAD_RANGE,
        ),
        (
            json!({"range": {"from": format!("{w}:1-2"), "to": format!("{w}:3")}, "scope": "room"}),
            expand::BAD_RANGE,
        ),
        (
            json!({"range": {"from": format!("{w}:1"), "to": format!("{other}:3")}, "scope": "room"}),
            expand::BAD_RANGE,
        ),
        (
            json!({"range": format!("{w}:9-10"), "scope": "room"}),
            &format!("no event at {w}:9"),
        ),
        (
            json!({"range": format!("{other}:1"), "scope": "room"}),
            "the event's writer has no seat",
        ),
        (
            json!({"range": "A99:1", "scope": "room"}),
            "no writer has that label",
        ),
        (json!({"range": format!("{w}:1")}), OUTSIDE_RUN),
        (
            json!({"range": format!("{w}:1"), "scope": "world"}),
            scope::BAD_SCOPE,
        ),
        (json!({"cursor": "nope"}), cursor::BAD_CURSOR),
        (
            json!({"cursor": cursor(&format!("{other}:1:0:2")), "range": format!("{w}:1"), "scope": "room"}),
            "the cursor belongs to another writer than the range",
        ),
    ];
    for (arguments, want) in cases {
        assert_eq!(
            refusal(&tool(&mut server, "event_expand", arguments.clone())),
            want,
            "{arguments}"
        );
    }
}

#[test]
fn a_call_stops_at_the_cap_and_its_cursor_goes_on_without_gap_or_overlap() {
    let f = fixture_of(&lines(100, 400));
    let mut server = f.server();
    let w = f.ingested.writer;
    let mut seen: Vec<String> = Vec::new();
    let mut arguments = json!({"range": format!("{w}:1-100"), "scope": "room"});
    let mut calls = 0;

    loop {
        let result = tool(&mut server, "event_expand", arguments);
        calls += 1;
        assert!(
            tokens::estimate(text_of(&result)) <= expand::CAP,
            "call {calls}"
        );
        let envelope = envelope_of(&result);
        seen.extend(contents(&envelope));
        match envelope["next_cursor"].as_str() {
            Some(cursor) => {
                assert_eq!(envelope["truncated"], true);
                arguments = json!({"cursor": cursor, "scope": "room"});
            }
            None => break,
        }
    }

    assert!(calls > 3, "{calls} calls");
    let want = lines(100, 400);
    let first = seen.iter().zip(&want).position(|(a, b)| a != b);
    assert_eq!((seen.len(), first), (100, None));
    let cut = server
        .take_notes()
        .into_iter()
        .filter(|n| matches!(n, Note::Truncated(_)))
        .count();
    assert_eq!(cut, calls - 1);
}

#[test]
fn an_event_past_the_cap_is_cut_and_its_cursor_continues_its_content() {
    let whole = format!("start {}", "ab ".repeat(30_000));
    let f = fixture_of(&[whole.clone(), "after".to_owned()]);
    let mut server = f.server();
    let w = f.ingested.writer;
    let mut joined = String::new();
    let mut flags: Vec<Value> = Vec::new();
    let mut arguments = json!({"range": format!("{w}:1-2"), "scope": "room"});

    loop {
        let result = tool(&mut server, "event_expand", arguments);
        let envelope = envelope_of(&result);
        assert!(tokens::estimate(text_of(&result)) <= expand::CAP);
        for item in envelope["items"].as_array().unwrap() {
            if item["address"].as_str().unwrap().ends_with("/1") {
                joined.push_str(item["content"].as_str().unwrap());
                assert_eq!(item["flags"], json!([]), "a cut is no injection mark");
                flags.push(item["continuation"].clone());
            }
        }
        match envelope["next_cursor"].as_str() {
            Some(cursor) => arguments = json!({"cursor": cursor, "scope": "room"}),
            None => {
                assert_eq!(contents(&envelope).last().unwrap(), "after");
                break;
            }
        }
    }

    assert_eq!(joined, whole);
    assert_eq!(flags[0], json!(["cut"]));
    assert_eq!(flags[1], json!(["continued", "cut"]));
    assert_eq!(flags.last().unwrap(), &json!(["continued"]));
}

#[test]
fn event_get_cuts_an_event_past_the_cap_and_event_expand_continues_it() {
    let whole = format!("start {}", "ab ".repeat(30_000));
    let f = fixture_of(std::slice::from_ref(&whole));
    let mut server = f.server();
    let w = f.ingested.writer;

    let first = envelope_of(&tool(
        &mut server,
        "event_get",
        json!({"address": format!("{w}:1"), "scope": "room"}),
    ));
    let rest = envelope_of(&tool(
        &mut server,
        "event_expand",
        json!({"cursor": first["next_cursor"], "scope": "room"}),
    ));

    let head = contents(&first).remove(0);
    assert!(whole.starts_with(&head) && head.len() < whole.len());
    assert_eq!(first["truncated"], true);
    let tail = contents(&rest).remove(0);
    assert!(whole[head.len()..].starts_with(&tail));
}

#[test]
fn a_range_is_named_by_a_label_a_node_assigns_once_and_never_twice() {
    let f = fixture_of(&lines(3, 5));
    let mut server = recording(&f);
    let by_label = |range: &str| json!({"range": range, "scope": "room"});

    let first = tool(&mut server, "event_expand", by_label("A1:1-2"));
    let again = tool(&mut server, "event_expand", by_label("A1·2"));
    let dashed = tool(&mut server, "event_expand", by_label("A1·2–3"));
    let other = tool(
        &mut server,
        "event_get",
        json!({"address": "A1:3", "scope": "room"}),
    );

    assert_eq!(contents(&envelope_of(&first)).len(), 2);
    assert_eq!(contents(&envelope_of(&again)).len(), 1);
    assert_eq!(contents(&envelope_of(&dashed)).len(), 2);
    assert_eq!(
        contents(&envelope_of(&other)),
        [format!("line 3 {}", "w".repeat(5))]
    );
    let store = Store::open(&f.home.store()).unwrap();
    let labels = store.writer_labels(device(&f)).unwrap();
    assert_eq!(labels[&f.ingested.writer], Label::new(1).unwrap());
    let unique: std::collections::BTreeSet<_> = labels.values().collect();
    assert_eq!(unique.len(), labels.len());
    let assignments = store
        .log(device(&f))
        .unwrap()
        .into_iter()
        .filter(|s| s.link.header.kind == EventKind::WriterLabel)
        .count();
    assert_eq!(
        assignments,
        labels.len(),
        "the first use assigned them all, once"
    );
}

#[test]
fn a_label_names_no_writer_while_the_server_cannot_assign_one() {
    let f = fixture_of(&lines(2, 5));
    let mut server = f.server();

    let result = tool(
        &mut server,
        "event_get",
        json!({"address": "A1:1", "scope": "room"}),
    );

    assert_eq!(refusal(&result), "no writer has that label");
}

#[test]
fn every_call_that_returns_an_envelope_leaves_a_recall_event() {
    let f = fixture_of(&lines(3, 5));
    let mut server = f.server().with_recall_writer(RecallWriter {
        writer: device(&f),
        redact: Some(Box::new(|t| {
            let rules = t
                .matches("secret")
                .map(|_| "extra_pattern_1".to_owned())
                .collect();
            (
                t.replace("secret", "[REDACTED]"),
                rules,
                t.contains("secret"),
            )
        })),
    });
    let w = f.ingested.writer;

    tool(
        &mut server,
        "event_search",
        json!({"query": "line", "scope": "room", "phrase": "secret"}),
    );
    tool(
        &mut server,
        "event_get",
        json!({"address": format!("{w}:1"), "scope": "room"}),
    );
    tool(
        &mut server,
        "event_expand",
        json!({"range": format!("{w}:1-2"), "scope": "room"}),
    );
    tool(&mut server, "landmark_list", json!({"scope": "room"}));
    tool(&mut server, "pin_list", json!({}));
    tool(&mut server, "stat_list", json!({}));
    tool(&mut server, "mcp_server_get", json!({}));
    tool(&mut server, "event_get", json!({"address": "nope"}));

    let recalled = recalls(&f);
    let names: Vec<&str> = recalled
        .iter()
        .map(|r| r["tool"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "event_search",
            "event_get",
            "event_expand",
            "landmark_list",
            "pin_list",
            "stat_list"
        ]
    );
    assert_eq!(recalled[0]["arguments"]["phrase"], "[REDACTED]");
    assert_eq!(recalled[0]["scope"], "room");
    assert_eq!(recalled[0]["run"], Value::Null);
    let returned = |n: usize| recalled[n]["returned"].as_array().unwrap().len();
    assert_eq!(
        (returned(1), returned(2), returned(3), returned(4)),
        (1, 2, 0, 0)
    );
    assert_eq!(
        recalled[1]["returned"][0],
        Address::new(w, Seq::FIRST).full(&f.ingested.room)
    );
    assert_eq!(recalled[5]["scope"], "room");
    assert_eq!(recalled[4]["scope"], "room");
    assert_eq!(recalled[2]["truncated"], false);
    let notes = server.take_notes();
    let extended = notes
        .iter()
        .filter(|n| matches!(n, Note::Extended { .. }))
        .count();
    assert_eq!(extended, 4, "four calls extended past the run");
    assert!(notes.contains(&Note::Redacted("extra_pattern_1".to_owned())));
    assert!(notes.contains(&Note::RedactionLimited));
}

#[test]
fn a_recall_event_is_assistant_provenance_and_no_search_finds_it() {
    let f = fixture_of(&lines(2, 5));
    let mut server = recording(&f);
    tool(
        &mut server,
        "event_search",
        json!({"query": "needlepossum", "scope": "room"}),
    );

    let found = tool(
        &mut server,
        "event_search",
        json!({"query": "needlepossum", "scope": "room"}),
    );

    assert_eq!(envelope_of(&found)["items"], json!([]));
    let store = Store::open(&f.home.store()).unwrap();
    let log = store.log(device(&f)).unwrap();
    let recall = log
        .iter()
        .find(|s| s.link.header.kind == EventKind::Recall)
        .unwrap();
    assert_eq!(recall.link.header.provenance, Provenance::Assistant);
}

#[test]
fn a_call_whose_recall_event_cannot_be_written_is_served_and_noted() {
    let f = fixture_of(&lines(2, 5));
    let w = f.ingested.writer;
    let mut unredacted = f.server().with_recall_writer(RecallWriter {
        writer: device(&f),
        redact: None,
    });
    let mut unwritable = recording(&f);
    f.sql(
        "CREATE TRIGGER no_recall BEFORE INSERT ON events WHEN NEW.kind = 'recall' \
         BEGIN SELECT RAISE(ABORT, 'no'); END",
    );

    let a = tool(
        &mut unredacted,
        "event_get",
        json!({"address": format!("{w}:1"), "scope": "room"}),
    );
    let b = tool(
        &mut unwritable,
        "event_get",
        json!({"address": format!("{w}:1"), "scope": "room"}),
    );

    envelope_of(&a);
    envelope_of(&b);
    assert!(
        unredacted
            .take_notes()
            .contains(&Note::NotRecorded("event_get"))
    );
    assert!(
        unwritable
            .take_notes()
            .contains(&Note::NotRecorded("event_get"))
    );
    assert!(recalls(&f).is_empty());
}

#[test]
fn a_server_without_a_recorder_writes_nothing() {
    let f = fixture_of(&lines(2, 5));
    let mut server = f.server();
    let w = f.ingested.writer;

    envelope_of(&tool(
        &mut server,
        "event_get",
        json!({"address": format!("{w}:1"), "scope": "room"}),
    ));

    assert!(recalls(&f).is_empty());
    assert!(
        !server
            .take_notes()
            .contains(&Note::NotRecorded("event_get"))
    );
}
