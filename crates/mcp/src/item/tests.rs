use super::*;
use serde_json::json;

#[test]
fn an_event_shows_its_text_when_it_has_one_and_its_json_otherwise() {
    assert_eq!(content_text(&json!({"line": 1, "text": "hello"})), "hello");
    assert_eq!(content_text(&json!({"text": 3})), r#"{"text":3}"#);
    assert_eq!(content_text(&json!({"room": "r"})), r#"{"room":"r"}"#);
    assert_eq!(content_text(&json!("plain")), r#""plain""#);
}
