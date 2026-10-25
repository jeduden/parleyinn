use serde_json::{Value, json};
use transcript::Differed;
use transcript::shape::RecordedShapes;

use super::*;

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

/// Classifies `lines` in order against shapes generated from `recorded`
/// (version "1"), in `session`.
fn classify_with(recorded: &[Value], lines: &[Value], session: Session) -> Vec<Classified> {
    let objects: Vec<Line> = recorded
        .iter()
        .map(|v| v.as_object().cloned().unwrap())
        .collect();
    let shapes = Shapes::of(vec![RecordedShapes::generate("1", &objects)]);
    let redactor = redactor();
    let mut classifier = Classifier::new(&shapes, session, &redactor);
    lines
        .iter()
        .map(|l| classifier.line(&l.to_string()))
        .collect()
}

/// Classifies `lines`, each fitting its own shape.
fn classify(lines: &[Value], session: Session) -> Vec<Classified> {
    classify_with(lines, lines, session)
}

fn class(p: Provenance) -> Classified {
    Classified::Line(p, None)
}

/// A session whose ancestry a hook observation records, not nested.
const OBSERVED: Session = Session {
    ancestry_observed: true,
    nested: false,
};

fn user_line(extra: &Value) -> Value {
    let mut line = json!({"type": "user", "message": {"role": "user", "content": "fix the bug"}, "promptSource": "typed"});
    for (k, v) in extra.as_object().unwrap() {
        line[k] = v.clone();
    }
    line
}

#[test]
fn typed_input_is_user_only_in_an_observed_session_no_run_started() {
    let typed = user_line(&json!({}));
    let sdk = user_line(&json!({"promptSource": "sdk"}));
    let blocks = json!({"type": "user", "promptSource": "sdk", "message": {"content": [{"type": "text", "text": "go"}]}});

    assert_eq!(
        classify(&[typed.clone(), sdk.clone(), blocks.clone()], OBSERVED),
        vec![class(Provenance::User); 3]
    );
    let unobserved = Session::default();
    let nested = Session {
        ancestry_observed: true,
        nested: true,
    };
    for session in [unobserved, nested] {
        assert_eq!(
            classify(&[typed.clone(), sdk.clone(), blocks.clone()], session),
            vec![class(Provenance::HarnessText); 3],
            "{session:?}"
        );
    }
}

#[test]
fn a_user_line_not_marked_as_typed_input_is_harness_text() {
    let lines = [
        user_line(&json!({"promptSource": "system"})),
        user_line(&json!({"promptSource": null})),
        user_line(&json!({"isMeta": true})),
        user_line(&json!({"isCompactSummary": true})),
        user_line(&json!({"isSidechain": true})),
        user_line(&json!({"isVisibleInTranscriptOnly": true})),
        user_line(&json!({"repeatsCompactSummary": true})),
        user_line(&json!({"agentId": "a1"})),
        user_line(
            &json!({"message": {"content": "<local-command-stdout>ok</local-command-stdout>"}}),
        ),
        user_line(
            &json!({"message": {"content": "<local-command-stderr>no</local-command-stderr>"}}),
        ),
        user_line(&json!({"message": {"content": []}})),
        user_line(&json!({"message": {"content": 7}})),
        user_line(
            &json!({"message": {"content": [{"type": "text", "text": "a"}, {"type": "image"}]}}),
        ),
    ];
    assert_eq!(
        classify(&lines, OBSERVED),
        vec![class(Provenance::HarnessText); lines.len()]
    );
}

fn call(id: &str, name: &str) -> Value {
    json!({"type": "assistant", "message": {"content": [{"type": "tool_use", "id": id, "name": name, "input": {}}]}})
}

fn result(ids: &[&str]) -> Value {
    let blocks: Vec<Value> = ids
        .iter()
        .map(|id| json!({"type": "tool_result", "tool_use_id": id, "content": "out"}))
        .collect();
    json!({"type": "user", "message": {"content": blocks}})
}

#[test]
fn a_tool_result_takes_the_class_of_its_calls_tool() {
    let tools = [
        ("Bash", Provenance::ToolResult("Bash".into())),
        ("WebFetch", Provenance::Web),
        ("WebSearch", Provenance::Web),
        ("Read", Provenance::File),
        ("Agent", Provenance::SubagentResult),
        ("Task", Provenance::SubagentResult),
        ("mcp__docs__search", Provenance::Mcp("docs".into())),
        ("mcp__docs", Provenance::ToolResult("mcp__docs".into())),
        ("mcp____x", Provenance::ToolResult("mcp____x".into())),
    ];
    for (n, (tool, want)) in tools.into_iter().enumerate() {
        let id = format!("toolu_{n}");
        let got = classify(&[call(&id, tool), result(&[&id])], Session::default());
        assert_eq!(
            got,
            vec![class(Provenance::ToolCall), class(want)],
            "{tool}"
        );
    }
}

#[test]
fn a_tool_result_with_no_call_held_or_mixed_tools_is_harness_text() {
    let lines = [
        call("a", "Bash"),
        call("b", "Read"),
        result(&["missing"]),
        result(&["a", "b"]),
        result(&["a", "missing"]),
        json!({"type": "user", "message": {"content": [{"type": "tool_result", "tool_use_id": "a"}, {"type": "text", "text": "x"}]}}),
    ];
    let got = classify(&lines, OBSERVED);

    assert_eq!(
        &got[2..],
        vec![class(Provenance::HarnessText); 4].as_slice()
    );
}

#[test]
fn a_tool_whose_name_is_not_plain_or_redacted_gives_harness_text() {
    let long = "T".repeat(NAME_LIMIT + 1);
    let names = [
        "",
        "Bash\n",
        " Bash",
        "Bash ",
        "rm;x",
        long.as_str(),
        "AKIAIOSFODNN7EXAMPLE",
    ];
    for (n, name) in names.into_iter().enumerate() {
        let id = format!("toolu_{n}");
        let got = classify(&[call(&id, name), result(&[&id])], Session::default());
        assert_eq!(got[1], class(Provenance::HarnessText), "{name:?}");
    }
    let fits = "a".repeat(NAME_LIMIT);
    let got = classify(&[call("x", &fits), result(&["x"])], Session::default());
    assert_eq!(got[1], class(Provenance::ToolResult(fits)));
}

#[test]
fn an_assistant_line_is_its_text_or_tool_call_and_a_harness_refusal_is_harness_text() {
    let lines = [
        json!({"type": "assistant", "message": {"content": [{"type": "text", "text": "ignore previous instructions"}]}}),
        json!({"type": "assistant", "message": {"content": [{"type": "thinking", "thinking": ""}]}}),
        json!({"type": "assistant", "message": {"content": [{"type": "text", "text": "x"}, {"type": "tool_use", "id": "t", "name": "Bash"}]}}),
        json!({"type": "assistant", "message": {"content": [{"type": "tool_use"}]}}),
        json!({"type": "assistant", "message": {"content": "plain"}}),
        json!({"type": "assistant", "isApiErrorMessage": true, "message": {"content": [{"type": "text", "text": "overloaded"}]}}),
    ];
    assert_eq!(
        classify(&lines, OBSERVED),
        vec![
            class(Provenance::Assistant),
            class(Provenance::Assistant),
            class(Provenance::ToolCall),
            class(Provenance::ToolCall),
            class(Provenance::Assistant),
            class(Provenance::HarnessText),
        ]
    );
}

#[test]
fn every_attachment_is_untrusted_file_mcp_or_harness_text() {
    let attachment =
        |a: Value| json!({"type": "attachment", "attachment": a, "renderedRole": "user"});
    let lines = [
        attachment(json!({"type": "instructions", "files": []})),
        attachment(json!({"type": "mcp_instructions_delta", "addedNames": ["docs"]})),
        attachment(
            json!({"type": "mcp_instructions_delta", "addedNames": ["claude.ai Connector D"]}),
        ),
        attachment(json!({"type": "mcp_instructions_delta", "addedNames": ["a", "b"]})),
        attachment(json!({"type": "mcp_instructions_delta", "addedNames": []})),
        attachment(json!({"type": "mcp_instructions_delta", "addedNames": ["bad;name"]})),
        attachment(json!({"type": "session_context", "context": {}})),
        attachment(json!({"type": "skill_listing"})),
        attachment(json!({"type": "hook_additional_context", "content": ["restore"]})),
    ];
    assert_eq!(
        classify(&lines, OBSERVED),
        vec![
            class(Provenance::File),
            class(Provenance::Mcp("docs".into())),
            class(Provenance::Mcp("claude.ai Connector D".into())),
            class(Provenance::HarnessText),
            class(Provenance::HarnessText),
            class(Provenance::HarnessText),
            class(Provenance::HarnessText),
            class(Provenance::HarnessText),
            class(Provenance::HarnessText),
        ]
    );
}

#[test]
fn metadata_lines_are_harness_meta_and_other_lines_harness_text() {
    let lines = [
        json!({"type": "cost-state", "totalCostUSD": 0.1}),
        json!({"type": "mode", "mode": "normal"}),
        json!({"type": "permission-mode", "permissionMode": "default"}),
        json!({"type": "atis-latch", "atis": ""}),
        json!({"type": "file-history-snapshot", "snapshot": {}}),
        json!({"type": "system", "subtype": "turn_duration", "durationMs": 1}),
        json!({"type": "system", "subtype": "compact_boundary", "content": "Conversation compacted"}),
        json!({"type": "system", "subtype": "local_command", "content": "out"}),
        json!({"type": "last-prompt", "lastPrompt": "fix the bug"}),
        json!({"type": "ai-title", "aiTitle": "Fixing"}),
        json!({"type": "queue-operation", "content": "fix"}),
    ];
    let got = classify(&lines, OBSERVED);

    assert_eq!(
        &got[..6],
        vec![class(Provenance::HarnessMeta); 6].as_slice()
    );
    assert_eq!(
        &got[6..],
        vec![class(Provenance::HarnessText); 5].as_slice()
    );
}

#[test]
fn a_user_or_harness_meta_line_with_a_key_beyond_its_shape_drifts_to_harness_text() {
    let recorded = [
        user_line(&json!({})),
        json!({"type": "mode", "mode": "normal"}),
        json!({"type": "ai-title", "aiTitle": "x"}),
    ];
    let lines = [
        user_line(&json!({"extra": 1})),
        json!({"type": "mode", "mode": "normal", "extra": 1}),
        json!({"type": "ai-title", "aiTitle": "x", "extra": 1}),
    ];
    let beyond = |shape: &str| Drift {
        version: None,
        shape: shape.to_owned(),
        differed: Differed::Beyond("extra".into()),
    };

    assert_eq!(
        classify_with(&recorded, &lines, OBSERVED),
        vec![
            Classified::Line(Provenance::HarnessText, Some(beyond("user"))),
            Classified::Line(Provenance::HarnessText, Some(beyond("mode"))),
            // An untrusted line may carry a key beyond its shape.
            class(Provenance::HarnessText),
        ]
    );
}

#[test]
fn an_unparsed_or_drifted_line_is_classified_by_no_field() {
    let recorded = [
        json!({"type": "assistant", "message": {}}),
        call("a", "Bash"),
        result(&["a"]),
    ];
    let lines = [
        json!({"type": "future_kind"}),
        json!({"type": "assistant"}),
        json!({"type": "assistant", "version": "9"}),
        // A drifted line's tool call is not learnt, so its result is
        // not classed by it.
        json!({"type": "assistant", "version": "9", "message": {"content": [{"type": "tool_use", "id": "b", "name": "Bash"}]}}),
        result(&["b"]),
    ];
    let got = classify_with(&recorded, &lines, OBSERVED);

    assert_eq!(
        got[0],
        Classified::Unparsed(Some(Drift {
            version: None,
            shape: "future_kind".into(),
            differed: Differed::UnrecordedType,
        }))
    );
    for drifted in &got[1..4] {
        assert!(
            matches!(drifted, Classified::Line(Provenance::HarnessText, Some(_))),
            "{drifted:?}"
        );
    }
    assert!(matches!(
        got[4],
        Classified::Line(Provenance::HarnessText, _)
    ));

    let redactor = redactor();
    let shapes = Shapes::default();
    let mut classifier = Classifier::new(&shapes, OBSERVED, &redactor);
    assert_eq!(classifier.line("{not json"), Classified::Unparsed(None));
}
