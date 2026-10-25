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
fn lines(n: usize, width: usize) -> Vec<String> {
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
fn recalls(f: &Fixture) -> Vec<Value> {
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
struct Fixed(String);

impl tool_call::Runs for Fixed {
    fn serving(&self) -> bool {
        true
    }

    fn find(&self, _id: &str, _wait: Duration) -> (Vec<String>, Duration) {
        (vec![self.0.clone()], Duration::ZERO)
    }
}

/// Runs that find no run for any tool-use id.
struct NoRun;

impl tool_call::Runs for NoRun {
    fn serving(&self) -> bool {
        true
    }

    fn find(&self, _id: &str, _wait: Duration) -> (Vec<String>, Duration) {
        (Vec::new(), Duration::ZERO)
    }
}

/// The name of the run the fixture's ingest recorded.
fn run_name(f: &Fixture) -> String {
    let store = Store::open(&f.home.store()).unwrap();
    store.ingested_runs().unwrap()[0].run_name()
}

/// A call under a tool-use id.
fn bound(server: &mut Server, name: &str, arguments: Value) -> Value {
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

fn contents(envelope: &Value) -> Vec<String> {
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

#[test]
fn stat_list_counts_within_the_scope_in_an_envelope_marked_structural() {
    let f = fixture_of(&lines(4, 5));
    let mut server = recording(&f);
    tool(&mut server, "landmark_list", json!({"scope": "room"}));

    let result = tool(&mut server, "stat_list", json!({}));

    let envelope = envelope_of(&result);
    assert_eq!(envelope["items"], json!([]));
    let stats = &envelope["structural"];
    assert_eq!(stats["recalls"], 1);
    assert_eq!(stats["compactions"], 0);
    assert_eq!(stats["runs"], 1);
    assert!(stats["events"].as_u64().unwrap() >= 4);
    assert_eq!(stats.as_object().unwrap().len(), 4);
    assert_eq!(recalls(&f).len(), 2);
}

#[test]
fn landmark_list_and_pin_list_answer_an_empty_envelope_and_check_their_arguments() {
    let f = fixture_of(&lines(2, 5));
    let mut server = f.server();
    let room = f.ingested.room.to_string();

    let landmarks = tool(
        &mut server,
        "landmark_list",
        json!({"scope": "room", "room": room, "tier": "t1"}),
    );
    let pins = tool(&mut server, "pin_list", json!({"room": room}));

    assert_eq!(envelope_of(&landmarks)["items"], json!([]));
    assert_eq!(envelope_of(&pins)["items"], json!([]));
    for (name, arguments, want) in [
        (
            "landmark_list",
            json!({"scope": "room", "tier": "a b"}),
            "the tier is not a short word",
        ),
        (
            "landmark_list",
            json!({"scope": "room", "tier": 3}),
            "the tier is not a short word",
        ),
        ("landmark_list", json!({}), OUTSIDE_RUN),
        (
            "pin_list",
            json!({"room": "x"}),
            "the room is not a room id",
        ),
        ("pin_list", json!({"room": 3}), "the room is not a room id"),
    ] {
        assert_eq!(
            refusal(&tool(&mut server, name, arguments.clone())),
            want,
            "{name} {arguments}"
        );
    }
}

#[test]
fn a_bound_server_serves_the_run_by_default_and_a_wider_scope_only_when_asked() {
    let f = fixture_of(&lines(3, 5));
    let w = f.ingested.writer;
    let mut server = recording(&f).with_runs(Box::new(Fixed(run_name(&f))));

    let own = bound(
        &mut server,
        "event_get",
        json!({"address": format!("{w}:1")}),
    );
    let search = bound(&mut server, "event_search", json!({"query": "line"}));
    let expanded = bound(
        &mut server,
        "event_expand",
        json!({"range": format!("{w}:1-3")}),
    );

    assert_eq!(envelope_of(&own)["items"][0]["run"], "current");
    assert_eq!(envelope_of(&search)["items"].as_array().unwrap().len(), 3);
    assert_eq!(contents(&envelope_of(&expanded)).len(), 3);
    assert!(
        server.take_notes().is_empty(),
        "the run is not an extension"
    );
    let recalled = recalls(&f);
    assert_eq!(recalled[0]["run"], run_name(&f));
    assert_eq!(recalled[0]["scope"], "run");
}

#[test]
fn another_run_s_event_lies_outside_the_scope_until_it_is_extended() {
    let f = fixture_of(&lines(3, 5));
    let w = f.ingested.writer;
    let mut server = recording(&f).with_runs(Box::new(Fixed("someone-else".to_owned())));
    let address = format!("{w}:1");

    let default = bound(&mut server, "event_get", json!({"address": address}));
    let search = bound(&mut server, "event_search", json!({"query": "line"}));
    let stats = bound(&mut server, "stat_list", json!({}));
    let room = bound(
        &mut server,
        "event_get",
        json!({"address": address, "scope": "room"}),
    );
    let rooms = bound(
        &mut server,
        "event_search",
        json!({"query": "line", "scope": "rooms"}),
    );

    assert_eq!(refusal(&default), expand::OUTSIDE_SCOPE);
    assert_eq!(envelope_of(&search)["items"], json!([]));
    assert_eq!(envelope_of(&stats)["structural"]["events"], 0);
    assert_eq!(envelope_of(&room)["items"][0]["run"], "other");
    assert_eq!(envelope_of(&rooms)["items"].as_array().unwrap().len(), 3);
    assert_eq!(
        server.take_notes(),
        [
            Note::ScopeRefused("event_get"),
            Note::Extended {
                tool: "event_get",
                scope: "room"
            },
            Note::Extended {
                tool: "event_search",
                scope: "rooms"
            },
        ]
    );
}

#[test]
fn a_room_the_run_has_no_seat_in_is_refused_and_a_foreign_one_never_reached() {
    let f = fixture_of(&lines(2, 5));
    let w = f.ingested.writer;
    let mut server = f.server().with_runs(Box::new(Fixed(run_name(&f))));
    let elsewhere = RoomId::derive(&[5; 32], &[6; 16]).to_string();

    let result = bound(
        &mut server,
        "event_get",
        json!({"address": format!("{w}:1"), "scope": "room", "room": elsewhere}),
    );
    let search = bound(
        &mut server,
        "event_search",
        json!({"query": "line", "scope": "room", "room": elsewhere}),
    );

    assert_eq!(refusal(&result), "the run has no seat in that room");
    assert_eq!(refusal(&search), "the run has no seat in that room");
    assert_eq!(
        server.take_notes(),
        [
            Note::ScopeRefused("event_get"),
            Note::ScopeRefused("event_search")
        ]
    );
}

#[test]
fn a_search_scoped_to_a_run_with_no_writer_returns_nothing_not_everything() {
    let f = fixture_of(&lines(2, 5));
    let mut server = f
        .server()
        .with_runs(Box::new(Fixed("unknown-run".to_owned())));

    let result = bound(&mut server, "event_search", json!({"query": "line"}));
    let narrowed = bound(
        &mut server,
        "event_search",
        json!({"query": "line", "run": f.ingested.writer.to_string()}),
    );

    assert_eq!(envelope_of(&result)["items"], json!([]));
    assert_eq!(envelope_of(&narrowed)["items"], json!([]));
}

#[test]
fn an_unreadable_record_refuses_in_fixed_text() {
    let f = fixture_of(&lines(2, 5));
    let mut server = f.server().with_runs(Box::new(Fixed(run_name(&f))));
    f.sql("ALTER TABLE ingested_runs RENAME TO moved");

    let result = bound(
        &mut server,
        "event_get",
        json!({"address": format!("{}:1", f.ingested.writer)}),
    );

    assert_eq!(refusal(&result), NO_INDEX);
}

#[test]
fn a_label_that_cannot_be_assigned_is_a_noted_failure_not_a_missing_label() {
    let f = fixture_of(&lines(3, 5));
    let mut server = f.server().with_labeller(device(&f));
    f.sql(
        "CREATE TRIGGER no_label BEFORE INSERT ON events WHEN NEW.kind = 'writer_label' \
         BEGIN SELECT RAISE(ABORT, 'no'); END",
    );

    let result = tool(
        &mut server,
        "event_expand",
        json!({"range": "A1:1-2", "scope": "room"}),
    );

    assert_eq!(
        refusal(&result),
        "the writer labels cannot be assigned: run parinn verify"
    );
    let notes = server.take_notes();
    let failed: Vec<_> = notes
        .iter()
        .filter_map(|n| match n {
            Note::LabelsFailed { tool, cause } => Some((*tool, cause.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(failed.len(), 1, "{notes:?}");
    assert_eq!(failed[0].0, "event_expand");
    assert!(!failed[0].1.is_empty());
}

#[test]
fn a_server_that_only_labels_assigns_labels_and_records_no_recall() {
    let f = fixture_of(&lines(3, 5));
    let mut server = f.server().with_labeller(device(&f));

    let result = tool(
        &mut server,
        "event_expand",
        json!({"range": "A1:1-2", "scope": "room"}),
    );

    assert_eq!(contents(&envelope_of(&result)).len(), 2);
    assert!(recalls(&f).is_empty());
    assert!(
        !server
            .take_notes()
            .contains(&Note::NotRecorded("event_expand"))
    );
}

#[test]
fn event_get_takes_a_cursor_for_the_rest_of_one_event_and_checks_it_names_the_same_writer() {
    let whole = format!("start {}", "ab ".repeat(30_000));
    let f = fixture_of(&[whole.clone(), "second".to_owned()]);
    let mut server = f.server();
    let w = f.ingested.writer;
    let mut joined = String::new();
    let mut arguments = json!({"address": format!("{w}:1"), "scope": "room"});

    loop {
        let envelope = envelope_of(&tool(&mut server, "event_get", arguments));
        joined.push_str(&contents(&envelope)[0]);
        assert_eq!(
            contents(&envelope).len(),
            1,
            "an event_get returns one event"
        );
        match envelope["next_cursor"].as_str() {
            Some(cursor) => arguments = json!({"cursor": cursor, "scope": "room"}),
            None => break,
        }
    }

    assert_eq!(joined, whole);
    let other = WriterId::derive(&[9; 32]);
    let mismatched = tool(
        &mut server,
        "event_get",
        json!({"cursor": cursor::Cursor { writer: w, seq: Seq::FIRST, offset: 0, to: Seq::FIRST }.encode(),
               "address": format!("{other}:1"), "scope": "room"}),
    );
    assert_eq!(
        refusal(&mismatched),
        "the cursor belongs to another writer than the range"
    );
    let range = tool(
        &mut server,
        "event_get",
        json!({"address": format!("{w}:1-2"), "scope": "room"}),
    );
    assert_eq!(refusal(&range), "event_get takes one address, not a range");
}

#[test]
fn a_cursor_may_come_with_the_range_it_continues() {
    let f = fixture_of(&lines(100, 400));
    let mut server = f.server();
    let w = f.ingested.writer;
    let range = format!("{w}:1-100");
    let first = envelope_of(&tool(
        &mut server,
        "event_expand",
        json!({"range": range, "scope": "room"}),
    ));

    let next = tool(
        &mut server,
        "event_expand",
        json!({"range": range, "cursor": first["next_cursor"], "scope": "room"}),
    );

    assert!(!contents(&envelope_of(&next)).is_empty());
}

#[test]
fn pin_list_and_stat_list_are_served_in_the_scope_they_name() {
    let f = fixture_of(&lines(3, 5));
    let mut server = f
        .server()
        .with_runs(Box::new(Fixed("someone-else".to_owned())));
    let elsewhere = RoomId::derive(&[5; 32], &[6; 16]).to_string();

    let run = bound(&mut server, "stat_list", json!({}));
    let rooms = bound(&mut server, "stat_list", json!({"scope": "rooms"}));
    let pins = bound(
        &mut server,
        "pin_list",
        json!({"scope": "room", "room": f.ingested.room.to_string()}),
    );
    let room_alone = bound(
        &mut server,
        "pin_list",
        json!({"room": f.ingested.room.to_string()}),
    );
    let foreign = bound(&mut server, "pin_list", json!({"room": elsewhere}));
    let foreign_stats = bound(
        &mut server,
        "stat_list",
        json!({"scope": "room", "room": elsewhere}),
    );
    let bad = bound(&mut server, "pin_list", json!({"room": "x"}));

    assert_eq!(envelope_of(&run)["structural"]["events"], 0);
    assert!(envelope_of(&rooms)["structural"]["events"].as_u64() >= Some(3));
    assert_eq!(envelope_of(&pins)["items"], json!([]));
    assert_eq!(envelope_of(&room_alone)["items"], json!([]));
    assert_eq!(refusal(&foreign), "the run has no seat in that room");
    assert_eq!(refusal(&foreign_stats), "the run has no seat in that room");
    assert_eq!(refusal(&bad), "the room is not a room id");
    assert_eq!(
        server.take_notes(),
        [
            Note::Extended {
                tool: "stat_list",
                scope: "rooms"
            },
            Note::Extended {
                tool: "pin_list",
                scope: "room"
            },
            Note::Extended {
                tool: "pin_list",
                scope: "room"
            },
            Note::ScopeRefused("pin_list"),
            Note::ScopeRefused("stat_list"),
            Note::ScopeRefused("pin_list"),
        ]
    );
}

#[test]
fn a_server_serving_no_run_serves_pin_list_and_stat_list_in_the_personal_room() {
    let f = fixture_of(&lines(2, 5));
    let mut server = f.server();
    let elsewhere = RoomId::derive(&[5; 32], &[6; 16]).to_string();

    let stats = tool(&mut server, "stat_list", json!({}));
    let pins = tool(&mut server, "pin_list", json!({}));
    let foreign = tool(&mut server, "pin_list", json!({"room": elsewhere}));
    let run = tool(&mut server, "stat_list", json!({"scope": "run"}));

    assert!(envelope_of(&stats)["structural"]["events"].as_u64() >= Some(2));
    assert_eq!(envelope_of(&pins)["items"], json!([]));
    assert!(refusal(&foreign).contains("only the personal room"));
    assert_eq!(refusal(&run), OUTSIDE_RUN);
    // The scope the server chose is no extension; the two refusals are.
    assert_eq!(
        server.take_notes(),
        [
            Note::ScopeRefused("pin_list"),
            Note::ScopeRefused("stat_list")
        ]
    );
}

#[test]
fn mcp_server_get_tells_its_state_under_a_tool_use_id_too() {
    let f = fixture_of(&lines(1, 5));
    let mut held = f.server().with_runs(Box::new(Fixed("a-run".to_owned())));
    let mut gone = f.server().with_runs(Box::new(NoRun));

    let with_run = bound(&mut held, "mcp_server_get", json!({}));
    let without = bound(&mut gone, "mcp_server_get", json!({}));

    for result in [&with_run, &without] {
        assert_eq!(result["isError"], false, "{result}");
        let state: Value = serde_json::from_str(text_of(result)).unwrap();
        assert_eq!(state["session_binding"], "held");
    }
    // The state is told whether or not the call's run was found, and
    // finding it is not what the answer rests on.
    assert!(held.take_found().is_empty() && gone.take_found().is_empty());
}

#[test]
fn a_missing_event_before_later_ones_is_told_not_taken_for_the_end_of_the_log() {
    let f = fixture_of(&lines(4, 5));
    let mut server = recording(&f);
    let w = f.ingested.writer;
    f.sql(&format!(
        "DELETE FROM events WHERE writer = '{w}' AND seq = 3"
    ));

    let gap = tool(
        &mut server,
        "event_expand",
        json!({"range": format!("{w}:1-4"), "scope": "room"}),
    );
    let whole = fixture_of(&lines(2, 5));
    let mut whole_server = whole.server();
    let end = tool(
        &mut whole_server,
        "event_expand",
        json!({"range": format!("{}:2-9", whole.ingested.writer), "scope": "room"}),
    );

    assert!(refusal(&gap).starts_with(&format!(
        "the record has no event at {w}:3 though later ones exist"
    )));
    assert_eq!(contents(&envelope_of(&end)).len(), 1);
    assert_eq!(
        server.take_notes(),
        [
            Note::Extended {
                tool: "event_expand",
                scope: "room"
            },
            Note::RecordGap("event_expand")
        ]
    );
}
