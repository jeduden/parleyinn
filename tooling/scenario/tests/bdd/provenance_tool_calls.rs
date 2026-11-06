//! The steps of PRV-14 that phase 10 of plan 2609292012 carries: the
//! harness session's MCP server, under a stand-in harness process
//! ([`crate::live`]), finds the run that made each tool call only by the
//! transcript line carrying the call's tool-use id, waiting for its
//! ingest at most 2 s, and refuses, counts and logs every call whose run
//! it cannot find.
//!
//! `@PRV-14`'s calls name `event_search`, which phase 12 serves; until
//! it lands, a call found under its run is answered "no such tool", so
//! the scenario stays pending and its rows pass narrowed to `event_get`.

use std::fs;
use std::io::Write;
use std::time::Duration;

use cucumber::{given, then, when};
use serde_json::{Value, json};

use crate::World;
use crate::live::{lines_read, wait_until};
use crate::record::home;
use crate::record_hooks::counted;
use crate::record_live::{harness, live, make_run, named, reply_text, serve};
use crate::record_transcript::recorded_call;

/// The main run's transcript, and its subagent's.
const MAIN: &str = "main-run";
const SUBAGENT: &str = "explore-agent";

/// The subagent's run, as the MCP server names it.
fn subagent_run() -> String {
    format!("{MAIN}/explore")
}

/// What the tool-call steps track: the tool the runs call, and each
/// call's tool-use id and reply.
#[derive(Debug, Default)]
pub struct ToolCallSteps {
    tool: String,
    calls: Vec<(Option<String>, Value)>,
    /// How many each refusal counter held before the case.
    before: Vec<(String, usize)>,
}

fn steps(w: &mut World) -> &mut ToolCallSteps {
    w.section::<ToolCallSteps>()
}

/// Appends to the transcript `name` a recorded line calling a tool with
/// the tool-use id `id`, in Claude Code 2.1.296's shape.
fn append_call(w: &mut World, name: &str, id: &str) -> Result<(), String> {
    let (mut call, _) = recorded_call("Bash")?;
    call["message"]["content"][0]["id"] = json!(id);
    let path = named(w, name)?;
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    writeln!(file, "{call}").map_err(|e| e.to_string())
}

/// Waits until ingest read every line of the transcript `name`.
fn read_all(w: &mut World, name: &str) -> Result<(), String> {
    let path = named(w, name)?;
    let held = fs::read_to_string(&path)
        .map_err(|e| e.to_string())?
        .lines()
        .count() as u64;
    wait_until(&format!("ingest to read {name}"), || {
        Ok(lines_read(w, &path)? >= held)
    })
}

/// The arguments a call of `tool` takes, `extra` among them.
fn arguments(w: &mut World, tool: &str, extra: &Value) -> Result<Value, String> {
    let mut args = if tool == "event_get" {
        let writer = crate::live::store(w)?
            .writers()
            .map_err(|e| e.to_string())?
            .into_iter()
            .next()
            .ok_or("the store holds no writer")?;
        json!({"address": format!("{writer}:1"), "scope": "room"})
    } else {
        json!({"query": "alpha", "scope": "room"})
    };
    if let (Some(args), Some(extra)) = (args.as_object_mut(), extra.as_object()) {
        args.extend(extra.clone());
    }
    Ok(args)
}

/// Sends a call of the scenario's tool carrying the tool-use id `id`,
/// when given, without waiting for its reply; returns the request's id.
fn send_call(w: &mut World, id: Option<&str>, extra: &Value) -> Result<u64, String> {
    let tool = steps(w).tool.clone();
    let mut params = json!({"name": tool, "arguments": arguments(w, &tool, extra)?});
    if let Some(id) = id {
        params["_meta"] = json!({"claudecode/toolUseId": id, "progressToken": 2});
    }
    harness(w)?.send("tools/call", params)
}

/// Sends a call and records its reply.
fn call(w: &mut World, id: Option<&str>, extra: &Value) -> Result<(), String> {
    let request = send_call(w, id, extra)?;
    let reply = harness(w)?.reply(request)?;
    steps(w).calls.push((id.map(str::to_owned), reply));
    Ok(())
}

#[given(
    regex = r#"^a harness session whose main run and one subagent's run each call "([^"]+)", served by its harness session's MCP server$"#
)]
fn harness_session(w: &mut World, tool: String) -> Result<(), String> {
    steps(w).tool = tool;
    make_run(w, MAIN, 3, Some(SUBAGENT))?;
    serve(w, MAIN, "startup", &[])?;
    for counter in REFUSALS {
        let n = counted(w, counter)?;
        steps(w).before.push((counter.to_owned(), n));
    }
    Ok(())
}

/// The counters a refused call feeds.
const REFUSALS: [&str; 3] = [
    "tool_call_run_not_found",
    "tool_call_without_tool_use_id",
    "tool_call_run_ambiguous",
];

#[when(
    regex = r"^ingest has read the line carrying the call's tool-use id before the call arrives$"
)]
fn read_before(w: &mut World) -> Result<(), String> {
    append_call(w, MAIN, "toolu_read_before")?;
    read_all(w, MAIN)?;
    call(w, Some("toolu_read_before"), &json!({}))
}

#[when(
    regex = r"^the line carrying the call's tool-use id reaches the transcript 0\.5 s after the call$"
)]
fn line_after(w: &mut World) -> Result<(), String> {
    let request = send_call(w, Some("toolu_late"), &json!({}))?;
    std::thread::sleep(Duration::from_millis(500));
    append_call(w, MAIN, "toolu_late")?;
    let reply = harness(w)?.reply(request)?;
    steps(w).calls.push((Some("toolu_late".to_owned()), reply));
    Ok(())
}

#[when(regex = r"^no line carries the call's tool-use id within 2 s$")]
fn no_line(w: &mut World) -> Result<(), String> {
    call(w, Some("toolu_never"), &json!({}))
}

#[when(regex = r"^the call carries no tool-use id$")]
fn no_id(w: &mut World) -> Result<(), String> {
    call(w, None, &json!({}))
}

#[when(regex = r"^lines of both runs carry the call's tool-use id$")]
fn both_runs(w: &mut World) -> Result<(), String> {
    append_call(w, MAIN, "toolu_both")?;
    append_call(w, SUBAGENT, "toolu_both")?;
    read_all(w, MAIN)?;
    read_all(w, SUBAGENT)?;
    call(w, Some("toolu_both"), &json!({}))
}

#[when(regex = r"^the call's arguments name the other run$")]
fn arguments_name_other(w: &mut World) -> Result<(), String> {
    append_call(w, MAIN, "toolu_args")?;
    read_all(w, MAIN)?;
    call(w, Some("toolu_args"), &json!({"run": subagent_run()}))
}

#[when(regex = r"^the subagent runs in the background$")]
fn background(w: &mut World) -> Result<(), String> {
    append_call(w, SUBAGENT, "toolu_background")?;
    call(w, Some("toolu_background"), &json!({}))
}

#[when(regex = r"^both runs call at the same moment$")]
fn same_moment(w: &mut World) -> Result<(), String> {
    append_call(w, MAIN, "toolu_main_now")?;
    append_call(w, SUBAGENT, "toolu_subagent_now")?;
    let first = send_call(w, Some("toolu_main_now"), &json!({}))?;
    let second = send_call(w, Some("toolu_subagent_now"), &json!({}))?;
    let a = harness(w)?.reply(first)?;
    let b = harness(w)?.reply(second)?;
    steps(w).calls.push((Some("toolu_main_now".to_owned()), a));
    steps(w)
        .calls
        .push((Some("toolu_subagent_now".to_owned()), b));
    Ok(())
}

#[when(regex = r"^the MCP server restarts after the line was written and before the call$")]
fn restarts(w: &mut World) -> Result<(), String> {
    append_call(w, MAIN, "toolu_restart")?;
    read_all(w, MAIN)?;
    let old = live(w).harness.take().ok_or("no MCP server is running")?;
    old.stop()?;
    serve(w, MAIN, "resume", &[])?;
    call(w, Some("toolu_restart"), &json!({}))
}

/// The run each logged call of the tool-use id `id` names, how long it
/// waited, and the cause of its refusal, read from the structured log.
fn logged(w: &World, id: &str) -> Result<Vec<Value>, String> {
    let file = home(w)?.root().join("logs").join("parinn.jsonl");
    let text = fs::read_to_string(&file).map_err(|e| format!("read {}: {e}", file.display()))?;
    Ok(text
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .map(|l| l["fields"].clone())
        .filter(|f| f["tool_use_id"] == id && f.get("waited_ms").is_some())
        .collect())
}

/// Fails unless the call `index` was served, under `run` per the log.
fn served_under(w: &mut World, index: usize, run: &str) -> Result<Value, String> {
    let (id, reply) = steps(w)
        .calls
        .get(index)
        .cloned()
        .ok_or("no step made the call")?;
    if reply["result"]["isError"] != false {
        return Err(format!("the call was not served: {}", reply_text(&reply)));
    }
    let id = id.ok_or("the call carries no tool-use id")?;
    let log = logged(w, &id)?;
    match log.as_slice() {
        [one] if one["run"] == run => Ok(one.clone()),
        other => Err(format!("the log of {id} holds {other:?}, not run {run}")),
    }
}

#[then(regex = r"^the call is served under the run that line belongs to, without waiting$")]
fn served_at_once(w: &mut World) -> Result<(), String> {
    let line = served_under(w, 0, MAIN)?;
    match line["waited_ms"].as_u64() {
        Some(ms) if ms < 100 => Ok(()),
        other => Err(format!("the call waited {other:?} ms")),
    }
}

#[then(
    regex = r"^the call waits until ingest reads the line, then is served under the run that line belongs to$"
)]
fn served_after_wait(w: &mut World) -> Result<(), String> {
    let line = served_under(w, 0, MAIN)?;
    match line["waited_ms"].as_u64() {
        Some(ms) if (400..2000).contains(&ms) => Ok(()),
        other => Err(format!("the call waited {other:?} ms")),
    }
}

#[then(
    regex = r#"^the call is refused with an explicit error, and the counter "([^"]+)" increases by 1$"#
)]
fn refused(w: &mut World, counter: String) -> Result<(), String> {
    let (_, reply) = steps(w)
        .calls
        .first()
        .cloned()
        .ok_or("no step made the call")?;
    if reply["result"]["isError"] != true || !reply_text(&reply).contains("tool-use id") {
        return Err(format!("the call was not refused: {reply}"));
    }
    let before = steps(w)
        .before
        .iter()
        .find(|(c, _)| *c == counter)
        .map_or(0, |(_, n)| *n);
    match counted(w, &counter)? - before {
        1 => Ok(()),
        n => Err(format!("{counter} increased by {n}")),
    }
}

#[then(
    regex = r"^the call is served under the run its line belongs to, not the run its arguments name$"
)]
fn served_not_argued(w: &mut World) -> Result<(), String> {
    served_under(w, 0, MAIN).map(drop)
}

#[then(regex = r"^the call is served under the subagent's run$")]
fn served_subagent(w: &mut World) -> Result<(), String> {
    served_under(w, 0, &subagent_run()).map(drop)
}

#[then(regex = r"^each call is served under its own run$")]
fn each_own(w: &mut World) -> Result<(), String> {
    served_under(w, 0, MAIN)?;
    served_under(w, 1, &subagent_run()).map(drop)
}

#[then(
    regex = r"^ingest resumes from its ingest marker and the call is served under the run that line belongs to$"
)]
fn served_after_restart(w: &mut World) -> Result<(), String> {
    let main = named(w, MAIN)?;
    let mut lines = crate::live::placed_lines(w, &main)?;
    lines.sort_unstable();
    let once: Vec<u64> = (1..=lines.len() as u64).collect();
    if lines != once {
        return Err(format!("the record holds main-run's lines {lines:?}"));
    }
    served_under(w, 0, MAIN).map(drop)
}

#[then(regex = r"^the redacted log \(OPS-05\) names the call's tool-use id and (.+)$")]
fn log_names(w: &mut World, what: String) -> Result<(), String> {
    let calls = steps(w).calls.clone();
    let harness = live(w).harness.take();
    if let Some(h) = harness {
        h.stop()?;
    }
    for (id, _) in &calls {
        let lines = match id {
            Some(id) => logged(w, id)?,
            None => logged(w, "")?,
        };
        let [line] = lines.as_slice() else {
            return Err(format!(
                "the log holds {} lines of the call {id:?}",
                lines.len()
            ));
        };
        let names = match what.as_str() {
            "that run and no wait" => {
                line["run"].is_string() && line["waited_ms"].as_u64() < Some(100)
            }
            "the cause and how long the call waited" => {
                line["cause"].is_string() && line["waited_ms"].as_u64() >= Some(2000)
            }
            "the cause" => line["cause"].is_string(),
            _ => line["run"].is_string() && line["waited_ms"].is_u64(),
        };
        if !names {
            return Err(format!(
                "the log of the call {id:?} names {line}, not {what}"
            ));
        }
    }
    Ok(())
}
