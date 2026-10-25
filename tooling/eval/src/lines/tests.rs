use serde_json::{Value, json};

use super::*;

fn fixture() -> String {
    std::fs::read_to_string(testkit::repo_root().join(FIXTURE)).unwrap()
}

fn templates() -> Templates {
    Templates::from_transcript(&fixture()).unwrap()
}

fn parsed(lines: &[String]) -> Vec<Value> {
    lines
        .iter()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn every_shape_a_run_needs_is_found_in_the_recorded_transcript() {
    let t = templates();
    assert_eq!(shape(&t.prompt), Some("prompt"));
    assert_eq!(shape(&t.tool_use), Some("tool_use"));
    assert_eq!(shape(&t.tool_result), Some("tool_result"));
    assert_eq!(shape(&t.reply), Some("reply"));
    assert_eq!(shape(&t.boundary), Some("boundary"));
    assert_eq!(shape(&t.summary), Some("summary"));
}

#[test]
fn lines_of_no_run_shape_are_passed_over() {
    assert_eq!(
        shape(&json!({"type": "user", "isMeta": true, "message": {"content": "x"}})),
        None
    );
    assert_eq!(shape(&json!({"type": "attachment"})), None);
    assert_eq!(
        shape(&json!({"type": "user", "message": {"content": [{"type": "image"}]}})),
        None
    );
    assert_eq!(
        shape(&json!({"type": "user", "message": {"content": []}})),
        None
    );
    assert_eq!(shape(&json!({"subtype": "compact_boundary"})), None);
}

#[test]
fn a_transcript_short_of_a_shape_or_holding_no_json_is_refused() {
    let no_boundary: String = fixture()
        .lines()
        .filter(|l| !l.contains("compact_boundary"))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_eq!(
        Templates::from_transcript(&no_boundary),
        Err("the fixture has no boundary line".to_owned())
    );
    let err = Templates::from_transcript("{}\nnot json\n").unwrap_err();
    assert!(err.starts_with("line 2: "), "{err}");
}

/// The lines of one turn: a prompt, a command and its output, a reply.
fn one_turn() -> Vec<Value> {
    let t = templates();
    let mut s = Session::new(&t, "0123abcd-0000-4000-a000-000000000000");
    s.prompt("check the ledger");
    s.command("ls", "a\nb");
    s.reply("done");
    assert_eq!(s.id(), "0123abcd-0000-4000-a000-000000000000");
    let (lines, boundary) = s.into_lines();
    assert_eq!(boundary, 0);
    parsed(&lines)
}

#[test]
fn a_turn_s_lines_link_in_order_and_carry_the_run_s_ids() {
    let v = one_turn();
    assert_eq!(v.len(), 4);
    assert_eq!(v[0]["parentUuid"], Value::Null);
    assert!(v.windows(2).all(|p| p[1]["parentUuid"] == p[0]["uuid"]));
    let uuids: std::collections::BTreeSet<&str> =
        v.iter().map(|l| l["uuid"].as_str().unwrap()).collect();
    assert_eq!(uuids.len(), 4, "every uuid is new");
    assert!(
        v.iter()
            .all(|l| l["sessionId"] == "0123abcd-0000-4000-a000-000000000000")
    );
    assert!(
        uuids
            .iter()
            .all(|u| u.starts_with("0123abcd-0000-4000-a000-")),
        "{uuids:?}"
    );
    let times: Vec<&str> = v.iter().map(|l| l["timestamp"].as_str().unwrap()).collect();
    assert_eq!(times[0], "2026-10-11T09:00:07.000Z");
    assert!(times.windows(2).all(|p| p[0] < p[1]), "{times:?}");
}

#[test]
fn a_turn_s_command_and_reply_carry_their_text_and_tie_the_result_to_its_call() {
    let v = one_turn();
    assert_eq!(v[0]["message"]["content"], "check the ledger");
    assert_eq!(
        v[2]["promptId"], v[0]["promptId"],
        "the result answers the prompt"
    );
    let tool = &v[1]["message"]["content"][0];
    assert_eq!(tool["input"]["command"], "ls");
    let result = &v[2]["message"]["content"][0];
    assert_eq!(result["tool_use_id"], tool["id"]);
    assert_eq!(result["content"], "a\nb");
    assert_eq!(v[2]["toolUseResult"]["stdout"], "a\nb");
    assert_eq!(v[2]["sourceToolAssistantUUID"], v[1]["uuid"]);
    let id = tool["id"].as_str().unwrap();
    assert_eq!(v[1]["wireToolInputs"][id]["command"], "ls");
    assert_eq!(v[3]["message"]["content"][0]["text"], "done");
    assert_ne!(v[1]["message"]["id"], v[3]["message"]["id"]);
}

#[test]
fn a_compaction_starts_a_new_chain_with_its_summary_and_marks_where_the_context_starts() {
    let t = templates();
    let mut s = Session::new(&t, "0123abcd-0000-4000-a000-000000000000");
    s.prompt("first");
    s.compact("we looked at the ledger");
    s.prompt("second");
    let (lines, boundary) = s.into_lines();
    let v = parsed(&lines);
    assert_eq!(boundary, 1);
    assert_eq!(v[1]["subtype"], "compact_boundary");
    assert_eq!(v[1]["parentUuid"], Value::Null);
    assert_eq!(v[1]["logicalParentUuid"], v[0]["uuid"]);
    assert_eq!(
        v[1]["compactMetadata"]["preservedSegment"]["tailUuid"],
        v[0]["uuid"]
    );
    assert_eq!(v[2]["isCompactSummary"], true);
    assert_eq!(v[2]["parentUuid"], v[1]["uuid"]);
    let summary = v[2]["message"]["content"].as_str().unwrap();
    assert!(
        summary.ends_with("Summary:\nwe looked at the ledger"),
        "{summary}"
    );
    assert_eq!(v[3]["parentUuid"], v[2]["uuid"]);
}
