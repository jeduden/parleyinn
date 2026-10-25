//! The scripted agent: it asks the recall tools what a model would, as
//! an MCP client does. For a question it calls `event_search` with the
//! question's words in its own run, then `event_get` on each hit, and
//! answers from what came back inside the envelope. No model is in the
//! loop: the agent reads the recalled events whole, so what it measures
//! is whether recall puts the exact value in front of the model.

use serde_json::{Value, json};

use crate::report::PAGE;

/// What the agent talks to: one JSON-RPC request in, one reply out, as
/// the MCP server's `handle` does.
pub trait Tools {
    /// The reply to `request`, none for a notification.
    fn handle(&mut self, request: &str) -> Option<String>;
}

impl Tools for mcp::Server {
    fn handle(&mut self, request: &str) -> Option<String> {
        mcp::Server::handle(self, request)
    }
}

/// The envelope one tool call answered with.
///
/// # Errors
///
/// Fails when the call gets no reply, an error, or no envelope.
pub fn call(tools: &mut dyn Tools, name: &str, arguments: &Value) -> Result<Value, String> {
    let request = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": name, "arguments": arguments}});
    let reply = tools
        .handle(&request.to_string())
        .ok_or_else(|| format!("{name} got no reply"))?;
    let reply: Value = serde_json::from_str(&reply).map_err(|e| format!("{name}: {e}"))?;
    let result = &reply["result"];
    let text = result["content"][0]["text"].as_str().unwrap_or_default();
    if result["isError"] != false {
        let why = if text.is_empty() {
            reply["error"].to_string()
        } else {
            text.to_owned()
        };
        return Err(format!("{name} refused: {why}"));
    }

    serde_json::from_str(text).map_err(|e| format!("{name} answered no envelope: {e}"))
}

/// The contents of the events the agent read for `question`, asked in
/// the run whose writer is `run`: one page of [`PAGE`] hits, each read
/// whole through `event_get`, in the order the search ranked them.
///
/// # Errors
///
/// Fails as [`call`] does.
pub fn recall(tools: &mut dyn Tools, run: &str, question: &str) -> Result<Vec<String>, String> {
    let found = call(
        tools,
        "event_search",
        &json!({"query": question, "scope": "room", "run": run, "k": PAGE}),
    )?;
    let mut contents = Vec::new();
    for hit in found["items"].as_array().into_iter().flatten() {
        let Some(address) = hit["address"].as_str() else {
            continue;
        };
        let got = call(
            tools,
            "event_get",
            &json!({"address": address, "scope": "room"}),
        )?;
        for item in got["items"].as_array().into_iter().flatten() {
            contents.push(item["content"].as_str().unwrap_or_default().to_owned());
        }
    }

    Ok(contents)
}

/// Whether `content` holds `value` exactly, as a whole: neither
/// neighbour of the match is a letter or digit, so `12` is not found in
/// `123`.
#[must_use]
pub fn holds(content: &str, value: &str) -> bool {
    let word = |c: Option<char>| c.is_some_and(char::is_alphanumeric);
    content.match_indices(value).any(|(at, _)| {
        !word(content[..at].chars().next_back())
            && !word(content[at + value.len()..].chars().next())
    })
}

#[cfg(test)]
mod tests;
