//! Spike S7, in-house side: `event_search` as a stub tool on a minimal
//! MCP server over stdio, std threads and serde_json only.
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

type Out = Arc<Mutex<std::io::Stdout>>;
type Inflight = Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>;

fn send(out: &Out, msg: &Value) {
    if let Ok(mut o) = out.lock() {
        let _ = writeln!(o, "{msg}");
        let _ = o.flush();
    }
}

fn reply(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn error(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn tools() -> Value {
    json!({"tools": [{
        "name": "event_search",
        "description": "Search the record (stub)",
        "inputSchema": {
            "type": "object",
            "properties": {"query": {"type": "string"}, "k": {"type": "integer"}},
            "required": ["query"],
        },
    }]})
}

/// The stub search: "slow" stands in for a long query and polls its
/// cancel flag against a deadline (ENG-03).
fn search(query: &str, cancelled: &AtomicBool) -> Option<Value> {
    if query == "slow" {
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if cancelled.load(Ordering::Relaxed) {
                return None;
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
    let envelope = json!({"trust": "untrusted", "hits": [{"address": "w1:1", "query": query}]});
    Some(json!({
        "content": [{"type": "text", "text": envelope.to_string()}],
        "structuredContent": envelope,
        "isError": false,
    }))
}

fn call(id: Value, params: &Value, out: Out, inflight: Inflight) {
    if params["name"] != "event_search" {
        send(&out, &error(&id, -32602, "unknown tool"));
        return;
    }
    let Some(query) = params["arguments"]["query"].as_str().map(str::to_owned) else {
        send(&out, &error(&id, -32602, "query is required"));
        return;
    };
    let flag = Arc::new(AtomicBool::new(false));
    if let Ok(mut m) = inflight.lock() {
        m.insert(id.to_string(), flag.clone());
    }
    thread::spawn(move || {
        let result = search(&query, &flag);
        if let Ok(mut m) = inflight.lock() {
            m.remove(&id.to_string());
        }
        // A cancelled request gets no response (MCP cancellation).
        if let Some(r) = result.filter(|_| !flag.load(Ordering::Relaxed)) {
            send(&out, &reply(&id, r));
        }
    });
}

fn handle(line: &str, out: &Out, inflight: &Inflight) {
    let Ok(msg) = serde_json::from_str::<Value>(line) else {
        send(out, &error(&Value::Null, -32700, "parse error"));
        return;
    };
    let id = msg.get("id").cloned();
    let method = msg["method"].as_str().unwrap_or_default();
    match (method, id) {
        ("initialize", Some(id)) => {
            let asked = msg["params"]["protocolVersion"].as_str().unwrap_or_default();
            let version = VERSIONS.iter().find(|v| **v == asked).unwrap_or(&VERSIONS[0]);
            send(out, &reply(&id, json!({
                "protocolVersion": version,
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "parinn", "version": "dev"},
            })));
        }
        ("ping", Some(id)) => send(out, &reply(&id, json!({}))),
        ("tools/list", Some(id)) => send(out, &reply(&id, tools())),
        ("tools/call", Some(id)) => call(id, &msg["params"], out.clone(), inflight.clone()),
        ("notifications/cancelled", None) => {
            let key = msg["params"]["requestId"].to_string();
            if let Some(flag) = inflight.lock().ok().and_then(|m| m.get(&key).cloned()) {
                flag.store(true, Ordering::Relaxed);
            }
        }
        (_, None) => {}
        (_, Some(id)) => send(out, &error(&id, -32601, "method not found")),
    }
}

fn main() {
    let out: Out = Arc::new(Mutex::new(std::io::stdout()));
    let inflight: Inflight = Arc::default();
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if !line.trim().is_empty() {
            handle(&line, &out, &inflight);
        }
    }
}
