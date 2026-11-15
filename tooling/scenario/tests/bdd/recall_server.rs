//! The steps of RCL-01 for `mcp_server_get`: the server's state, told
//! with a session binding held and with none, and no recall event left.

use cucumber::{then, when};
use serde_json::{Value, json};

use crate::World;
use crate::live::{Harness, wait_until};
use crate::recall::{recall, recall_events, record_runs};
use crate::record_search::call_tool;

#[when(regex = r#"^the agent calls "mcp_server_get", with a session binding held and with none$"#)]
fn calls_server_get(w: &mut World) -> Result<(), String> {
    record_runs(w)?;
    recall(w).before = recall_events(w)?.len();
    recall(w).states.clear();
    // With none: a server no harness session bound.
    let none = call_tool(w, "mcp_server_get", json!({}))?;
    recall(w).states.push(none.text);
    // With one held: a stand-in harness runs the `SessionStart` hook and
    // starts its MCP server, which takes the binding. The call names no
    // tool-use id, so no run is found for it.
    let transcript = w.home.join("bound-session.jsonl");
    let input = json!({
        "session_id": "bound-session",
        "transcript_path": transcript,
        "cwd": w.home,
        "hook_event_name": "SessionStart",
        "source": "startup",
    });
    let mut harness = Harness::start(w, false, &[])?;
    harness.hook("SessionStart", &input)?;
    harness.start_mcp()?;
    let params = json!({"name": "mcp_server_get", "arguments": {}});
    let mut text = String::new();
    wait_until("the MCP server to hold its session binding", || {
        let reply = harness.request("tools/call", params.clone())?;
        text = reply["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        Ok(text.contains("\"held\""))
    })?;
    recall(w).states.push(text);
    // And under a tool-use id no transcript line carries: no run is
    // found for the call, and the state is told all the same.
    let named = json!({
        "name": "mcp_server_get",
        "arguments": {},
        "_meta": {"claudecode/toolUseId": "toolu_no_such_call"},
    });
    let reply = harness.request("tools/call", named)?;
    let text = reply["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    harness.stop()?;
    recall(w).states.push(text);
    Ok(())
}

#[then(
    regex = r#"^it returns "session_binding" as "held" or "not_held", "search_index" as "readable" or "unreadable", "searches_timed_out" as a count and "version", and no recalled content or envelope, whether or not a run is found for the call$"#
)]
fn server_state_is_closed(w: &mut World) -> Result<(), String> {
    let states = recall(w).states.clone();
    let [none, held, named] = states.as_slice() else {
        return Err(format!(
            "wanting three answers, the step made {}",
            states.len()
        ));
    };
    for (text, binding) in [(none, "not_held"), (held, "held"), (named, "held")] {
        let state: Value = serde_json::from_str(text).map_err(|e| format!("{text:?}: {e}"))?;
        let object = state.as_object().ok_or("the answer is no object")?;
        let mut names: Vec<&str> = object.keys().map(String::as_str).collect();
        names.sort_unstable();
        if names
            != [
                "search_index",
                "searches_timed_out",
                "session_binding",
                "version",
            ]
        {
            return Err(format!("the answer carries {names:?}"));
        }
        if state["session_binding"] != binding {
            return Err(format!(
                "the binding reads {}, wanting {binding}",
                state["session_binding"]
            ));
        }
        if !matches!(
            state["search_index"].as_str(),
            Some("readable" | "unreadable")
        ) || !state["searches_timed_out"].is_u64()
            || !state["version"].is_string()
        {
            return Err(format!(
                "the answer's values are not closed words and counts: {state}"
            ));
        }
        if text.contains("parinn_envelope") || text.contains("warning") {
            return Err("the answer holds an envelope".to_owned());
        }
    }
    Ok(())
}

#[then(regex = r"^the call is recorded as no recall event$")]
fn recorded_as_none(w: &mut World) -> Result<(), String> {
    let now = recall_events(w)?.len();
    if now == recall(w).before {
        return Ok(());
    }
    Err(format!(
        "the record gained {} recall events",
        now - recall(w).before
    ))
}
