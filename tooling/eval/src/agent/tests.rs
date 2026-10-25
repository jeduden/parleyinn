use std::collections::VecDeque;

use serde_json::{Value, json};

use super::*;

/// Answers each request with the next reply, and keeps the requests.
#[derive(Default)]
struct Scripted {
    replies: VecDeque<Option<String>>,
    asked: Vec<Value>,
}

impl Tools for Scripted {
    fn handle(&mut self, request: &str) -> Option<String> {
        self.asked.push(serde_json::from_str(request).unwrap());
        self.replies.pop_front().flatten()
    }
}

/// A tool call's reply holding `envelope`.
fn answer(envelope: &Value) -> Option<String> {
    Some(
        json!({"jsonrpc": "2.0", "id": 1, "result": {
            "isError": false, "content": [{"type": "text", "text": envelope.to_string()}],
        }})
        .to_string(),
    )
}

fn scripted(replies: Vec<Option<String>>) -> Scripted {
    Scripted {
        replies: replies.into(),
        asked: Vec::new(),
    }
}

#[test]
fn the_agent_searches_its_own_run_then_reads_each_hit_whole_in_rank_order() {
    let mut tools = scripted(vec![
        answer(
            &json!({"items": [{"address": "w:2"}, {"preview": "no address"}, {"address": "w:1"}]}),
        ),
        answer(&json!({"items": [{"content": "second"}]})),
        answer(&json!({"items": [{"content": "first"}]})),
    ]);

    let read = recall(&mut tools, "w", "ledger debug port").unwrap();

    assert_eq!(read, ["second", "first"]);
    assert_eq!(tools.asked[0]["params"]["name"], "event_search");
    assert_eq!(
        tools.asked[0]["params"]["arguments"],
        json!({"query": "ledger debug port", "scope": "room", "run": "w", "k": PAGE})
    );
    assert_eq!(tools.asked[1]["params"]["name"], "event_get");
    assert_eq!(
        tools.asked[1]["params"]["arguments"],
        json!({"address": "w:2", "scope": "room"})
    );
}

#[test]
fn a_call_with_no_reply_an_error_or_no_envelope_fails_naming_the_tool() {
    let mut silent = scripted(vec![None]);
    assert_eq!(
        call(&mut silent, "event_get", &json!({})),
        Err("event_get got no reply".to_owned())
    );
    let mut garbled = scripted(vec![Some("not json".to_owned())]);
    assert!(
        call(&mut garbled, "event_get", &json!({}))
            .unwrap_err()
            .starts_with("event_get: ")
    );
    let refusal = json!({"result": {"isError": true, "content": [{"text": "no such run"}]}});
    let mut refused = scripted(vec![Some(refusal.to_string())]);
    assert_eq!(
        call(&mut refused, "event_search", &json!({})),
        Err("event_search refused: no such run".to_owned())
    );
    let rpc = json!({"error": {"code": -32601}});
    let mut unknown = scripted(vec![Some(rpc.to_string())]);
    assert_eq!(
        call(&mut unknown, "event_x", &json!({})),
        Err("event_x refused: {\"code\":-32601}".to_owned())
    );
    let plain = json!({"result": {"isError": false, "content": [{"text": "plain"}]}});
    let mut bare = scripted(vec![Some(plain.to_string())]);
    assert!(
        call(&mut bare, "event_get", &json!({}))
            .unwrap_err()
            .starts_with("event_get answered no envelope: ")
    );
}

#[test]
fn a_failed_read_of_a_hit_fails_the_recall() {
    let mut tools = scripted(vec![answer(&json!({"items": [{"address": "w:1"}]})), None]);
    assert_eq!(
        recall(&mut tools, "w", "q"),
        Err("event_get got no reply".to_owned())
    );
}

#[test]
fn a_value_is_held_only_as_a_whole() {
    assert!(holds("port 48211 open", "48211"));
    assert!(holds("48211", "48211"));
    assert!(holds("\"LED-12345\"", "LED-12345"));
    assert!(!holds("port 482110", "48211"));
    assert!(!holds("LED-123456", "LED-12345"));
    assert!(!holds("x48211", "48211"));
    assert!(holds("x48211 and 48211.", "48211"), "a later match counts");
    assert!(!holds("nothing", "48211"));
}
