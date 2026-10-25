use serde_json::{Value, json};
use transcript::Differed;

use super::*;

/// A `SessionStart` hook input as Claude Code 2.1.296 recorded it.
fn session_start() -> Value {
    json!({
        "session_id": "205beaae-e4b8-4798-a1aa-cdf50c807136",
        "transcript_path": "/home/u/.claude/projects/-w-my-app/205beaae.jsonl",
        "cwd": "/w/my_app",
        "hook_event_name": "SessionStart",
        "source": "startup",
    })
}

/// A `UserPromptSubmit` hook input as Claude Code 2.1.296 recorded it.
fn user_prompt_submit() -> Value {
    json!({
        "session_id": "686aa14f-b535-45bc-9ed0-5400ea84bec7",
        "transcript_path": "/home/u/.claude/projects/-w-my-app/686aa14f.jsonl",
        "cwd": "/w/my_app",
        "prompt_id": "b4d185a7-444a-48c7-b7b3-4f5da463e538",
        "permission_mode": "default",
        "hook_event_name": "UserPromptSubmit",
        "prompt": "Say forked.",
    })
}

#[test]
fn a_recorded_hook_input_decodes_into_the_fields_a_handler_uses_without_drift() {
    let decoded = decode(Hook::SessionStart, &session_start().to_string()).unwrap();
    assert_eq!(
        decoded,
        Decoded {
            input: HookInput {
                session_id: Some("205beaae-e4b8-4798-a1aa-cdf50c807136".into()),
                transcript_path: Some("/home/u/.claude/projects/-w-my-app/205beaae.jsonl".into()),
                cwd: Some("/w/my_app".into()),
                source: Some("startup".into()),
                prompt_id: None,
                prompt: None,
            },
            drift: None,
        }
    );
    let decoded = decode(Hook::UserPromptSubmit, &user_prompt_submit().to_string()).unwrap();
    assert_eq!(
        (
            decoded.input.prompt_id.as_deref(),
            decoded.input.prompt.as_deref()
        ),
        (
            Some("b4d185a7-444a-48c7-b7b3-4f5da463e538"),
            Some("Say forked.")
        )
    );
    assert_eq!(decoded.drift, None);
}

#[test]
fn a_field_of_the_wrong_type_or_an_id_that_is_not_plain_fails_the_schema() {
    for (field, value) in [
        ("session_id", json!(42)),
        ("session_id", json!("")),
        ("session_id", json!("a b")),
        ("session_id", json!("x".repeat(ID_LIMIT + 1))),
        ("transcript_path", json!(["/a"])),
        ("cwd", json!(null)),
        ("source", json!(true)),
        ("prompt_id", json!("../etc")),
        ("prompt", json!({"text": "hi"})),
        ("hook_event_name", json!("UserPromptSubmit")),
        ("hook_event_name", json!(1)),
    ] {
        let mut input = session_start();
        input[field] = value.clone();
        assert_eq!(
            decode(Hook::SessionStart, &input.to_string()),
            Err(Failure::Invalid),
            "{field} = {value}"
        );
    }
}

#[test]
fn a_missing_or_extra_key_is_format_drift_while_the_rest_decodes() {
    let mut input = session_start();
    input.as_object_mut().unwrap().remove("source");
    let decoded = decode(Hook::SessionStart, &input.to_string()).unwrap();
    assert_eq!(decoded.input.source, None);
    let drift = decoded.drift.unwrap();
    assert_eq!(
        (
            drift.version.as_deref(),
            drift.shape.as_str(),
            drift.differed
        ),
        (
            Some("2.1.296"),
            "SessionStart",
            Differed::Missing("source".into())
        )
    );

    let mut input = user_prompt_submit();
    input["extra"] = json!(1);
    let drift = decode(Hook::UserPromptSubmit, &input.to_string())
        .unwrap()
        .drift
        .unwrap();
    assert_eq!(drift.differed, Differed::Beyond("extra".into()));
}

#[test]
fn a_hook_no_supported_version_records_is_checked_by_the_schema_alone() {
    let input = json!({"session_id": "s", "anything": 1});

    let decoded = decode(Hook::PermissionRequest, &input.to_string()).unwrap();

    assert_eq!(decoded.drift, None);
    assert_eq!(decoded.input.session_id.as_deref(), Some("s"));
}

#[test]
fn a_hook_input_that_is_no_json_object_is_malformed() {
    for input in ["", "{\"session_id\": ", "[1]", "\"text\"", "{} {}"] {
        assert_eq!(
            decode(Hook::UserPromptSubmit, input),
            Err(Failure::Malformed),
            "{input:?}"
        );
    }
}

/// A JSON object nested `depth` levels deep.
fn nested(depth: usize) -> String {
    format!("{}{}", "{\"a\":".repeat(depth - 1) + "{", "}".repeat(depth))
}

#[test]
fn a_hook_input_past_1_mib_is_refused_before_it_is_parsed() {
    let fits = format!("{{\"p\":\"{}\"}}", "x".repeat(HOOK_INPUT_LIMIT - 8));
    assert_eq!(fits.len(), HOOK_INPUT_LIMIT);
    assert!(decode(Hook::PermissionRequest, &fits).is_ok());
    let over = format!("{fits} ");
    assert_eq!(
        decode(Hook::PermissionRequest, &over),
        Err(Failure::TooLarge)
    );
}

#[test]
fn a_hook_input_nested_past_depth_64_is_refused_before_it_is_parsed() {
    assert!(decode(Hook::PermissionRequest, &nested(64)).is_ok());
    assert_eq!(
        decode(Hook::PermissionRequest, &nested(65)),
        Err(Failure::TooDeep)
    );
    // Deep enough that the parser's own limit would refuse it as malformed.
    assert_eq!(
        decode(Hook::PermissionRequest, &nested(200)),
        Err(Failure::TooDeep)
    );
}

#[test]
fn depth_counts_brackets_outside_strings_only() {
    assert_eq!(depth(""), 0);
    assert_eq!(depth("{}"), 1);
    assert_eq!(depth(r#"{"a":[1,{"b":[]}]}"#), 4);
    assert_eq!(depth(r#"{"a":"[[[[{{"}"#), 1);
    assert_eq!(depth(r#"{"a":"\"[[["}"#), 1);
    assert_eq!(depth(r#"{"a":"\\"}"#), 1);
    assert_eq!(depth("}}}{"), 1);
}

#[test]
fn a_plain_id_is_short_and_of_plain_characters() {
    assert!(plain_id("205beaae-e4b8-4798-a1aa-cdf50c807136"));
    assert!(plain_id("a.b_c-D9"));
    assert!(!plain_id(""));
    assert!(!plain_id("a/b"));
    assert!(!plain_id(&"x".repeat(ID_LIMIT + 1)));
    assert!(plain_id(&"x".repeat(ID_LIMIT)));
}
