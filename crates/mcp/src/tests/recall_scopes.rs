//! The recall tools' scopes through the server (RCL-01, RCL-03, RCL-05,
//! RCL-07, RCL-08): the room and run a call is served in, the lists and
//! counts a scope bounds, the cursor of one event and the labels a
//! server assigns.

use super::recall_calls::*;
use super::*;
use record::id::Seq;

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
