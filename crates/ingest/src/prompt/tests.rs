use record::commitment::CommitmentKey;
use serde_json::json;

use super::*;

fn committed(text: &str, used: bool) -> PromptCommitment {
    let key = CommitmentKey::from_bytes([3; 32]);
    PromptCommitment {
        harness_session: "s".into(),
        prompt_id: "p".into(),
        commitment: commit(&key, &json!(text)).unwrap(),
        key,
        used,
    }
}

fn typed(text: &str) -> PromptLine {
    PromptLine {
        prompt_id: "p".into(),
        typed: true,
        text: Some(text.into()),
    }
}

fn plain(text: &str) -> String {
    text.to_owned()
}

#[test]
fn a_prompt_line_carries_a_prompt_id_and_a_prompt_source() {
    let line = json!({"promptId": "p", "promptSource": "typed", "message": {"content": "hi"}});
    assert_eq!(PromptLine::of(&line), Some(typed("hi")));
    let blocks = json!({"promptId": "p", "promptSource": "sdk",
        "message": {"content": [{"type": "text", "text": "a"}, {"type": "text", "text": "b"}]}});
    assert_eq!(
        PromptLine::of(&blocks),
        Some(PromptLine {
            prompt_id: "p".into(),
            typed: false,
            text: Some("a\nb".into())
        })
    );
    let result = json!({"promptId": "p", "promptSource": "typed",
        "message": {"content": [{"type": "tool_result", "content": "x"}]}});
    assert_eq!(PromptLine::of(&result).unwrap().text, None);
    let empty = json!({"promptId": "p", "promptSource": "typed", "message": {"content": []}});
    assert_eq!(PromptLine::of(&empty).unwrap().text, None);
    assert_eq!(
        PromptLine::of(&json!({"promptId": "p", "message": {"content": "hi"}})),
        None,
        "a tool result's line carries no promptSource"
    );
    assert_eq!(PromptLine::of(&json!({"promptSource": "typed"})), None);
}

#[test]
fn the_first_typed_user_line_whose_redacted_text_matches_is_matched() {
    let c = committed("hi [REDACTED:aws]", false);
    let redact = |t: &str| t.replace("AKIA", "[REDACTED:aws]");

    assert_eq!(
        outcome(Some(&c), false, true, &typed("hi AKIA"), &redact),
        Some(Outcome::Matched)
    );
}

#[test]
fn a_line_that_uses_a_commitment_up_without_matching_is_unmatched() {
    let c = committed("hi", false);
    let sdk = PromptLine {
        typed: false,
        ..typed("hi")
    };
    let no_text = PromptLine {
        text: None,
        ..typed("hi")
    };
    for (user, line) in [
        (true, typed("other")),
        (false, typed("hi")),
        (true, sdk),
        (true, no_text),
    ] {
        assert_eq!(
            outcome(Some(&c), false, user, &line, &plain),
            Some(Outcome::Unmatched),
            "{line:?}"
        );
    }
}

#[test]
fn a_commitment_used_up_already_is_a_conflict_and_none_is_nothing() {
    assert_eq!(
        outcome(
            Some(&committed("hi", true)),
            false,
            true,
            &typed("hi"),
            &plain
        ),
        Some(Outcome::Conflict)
    );
    assert_eq!(
        outcome(
            Some(&committed("hi", false)),
            true,
            true,
            &typed("hi"),
            &plain
        ),
        Some(Outcome::Conflict)
    );
    assert_eq!(outcome(None, false, true, &typed("hi"), &plain), None);
}

#[test]
fn each_outcome_has_its_name() {
    assert_eq!(Outcome::Matched.as_str(), "matched");
    assert_eq!(Outcome::Unmatched.as_str(), "unmatched");
    assert_eq!(Outcome::Conflict.as_str(), "conflict");
}
