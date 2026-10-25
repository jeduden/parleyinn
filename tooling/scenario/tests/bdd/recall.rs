//! The steps of RCL-01, RCL-03, RCL-04 and RCL-07 that phase 13 of plan
//! 2609292012 carries: the recall tools `parinn mcp` serves, their
//! envelope, the 8,000-token cap with its continuation cursor and the
//! recall event each call leaves in the record.
//!
//! The agent's call is a request to `parinn mcp` on its standard input,
//! as the harness sends it, with the scope `room`: no run is bound to a
//! server started this way, so only `room` is served (RCL-05, whose
//! steps bind a run). The runs the scenarios name are fixtures built
//! here: a transcript a step registered and left empty is given its
//! fixture's lines, then ingested, before the first call.
//!
//! `recall_calls` is no audit counter, which would fail `parinn doctor`
//! on every recall: it is the number of recall events the record
//! holds, the count `stat_list` derives.

use std::collections::BTreeSet;

use ::record::event::EventKind;
use ::record::id::Address;
use cucumber::{then, when};
use envelope::tokens;
use serde_json::{Value, json};
use store::Store;

use crate::World;
use crate::live::{Harness, wait_until};
use crate::record::{home, last_run, run_core};
use crate::record_search::{Reply, arguments_of, call_tool, last_reply};
use crate::record_transcript::{ingest, recorded_call};
use crate::security_redaction::{append, transcripts};

/// What the recall steps track.
#[derive(Debug, Default)]
pub struct RecallSteps {
    /// The transcripts already given their lines and ingested.
    recorded: BTreeSet<String>,
    /// The tools the last listing offered.
    tools: Vec<Value>,
    /// The six tools' calls, by tool.
    calls: Vec<(String, Reply)>,
    /// How many recall events the record held before the last call.
    before: usize,
    /// The pages of an `event_expand` followed by its cursor.
    pages: Vec<Value>,
    /// What `mcp_server_get` answered, without a session binding and
    /// with one held.
    states: Vec<String>,
}

pub fn recall(w: &mut World) -> &mut RecallSteps {
    w.section::<RecallSteps>()
}

/// A `uuid` for fixture line `n`.
fn uuid(n: u64) -> String {
    format!("00000000-0000-4000-8000-{n:012x}")
}

/// A Bash call and its result, numbered `n`, the result's output
/// `output`, each line with a `uuid` of its own.
fn pair(n: u64, output: &str) -> Result<Vec<Value>, String> {
    let (mut call, mut result) = recorded_call("Bash")?;
    let id = format!("toolu_fixture{n}");
    call["message"]["content"][0]["id"] = json!(id);
    call["uuid"] = json!(uuid(2 * n));
    result["message"]["content"][0]["tool_use_id"] = json!(id);
    result["message"]["content"][0]["content"] = json!(output);
    result["uuid"] = json!(uuid(2 * n + 1));
    if result["toolUseResult"].is_object() {
        result["toolUseResult"]["stdout"] = json!(output);
    }

    Ok(vec![call, result])
}

/// The lines the fixture run `name` holds.
fn fixture(name: &str) -> Result<Vec<Value>, String> {
    let mut lines = Vec::new();
    match name {
        "short-run" => {
            lines.push(crate::record_search::user_line(
                "please deploy the staging build",
                1,
            )?);
            lines.extend(pair(1, "deploy finished: staging is live")?);
            lines.extend(pair(2, "tests passed")?);
        }
        "mixed-provenance" => {
            for n in 1..=12 {
                lines.extend(pair(n, &format!("deploy step {n}: ok"))?);
            }
        }
        "large-payloads" => {
            for n in 1..=200 {
                // Some results pass the payload threshold and fit a page,
                // some pass the cap and are cut, the rest stay inline.
                let size = match n % 20 {
                    1 => 8_000,
                    11 => 40_000,
                    _ => 2_000,
                };
                let output = format!("payload {n} {}", "lorem ipsum ".repeat(size / 12));
                lines.extend(pair(n, &output)?);
            }
        }
        other => return Err(format!("no fixture is named {other:?}")),
    }

    Ok(lines)
}

/// Gives each registered transcript still empty its fixture's lines and
/// ingests it, once; one a step gave lines is left to that step.
pub fn record_runs(w: &mut World) -> Result<(), String> {
    for (name, path) in transcripts(w) {
        if recall(w).recorded.contains(&name) {
            continue;
        }
        let empty = std::fs::metadata(&path).map_err(|e| e.to_string())?.len() == 0;
        // A transcript a step gave lines is the step's to ingest.
        if empty {
            for line in fixture(&name)? {
                append(w, &name, &line)?;
            }
            ingest(w, &path)?;
        }
        recall(w).recorded.insert(name);
    }

    Ok(())
}

/// The recall events the record holds, with their addresses.
pub fn recall_events(w: &World) -> Result<Vec<(Address, Value)>, String> {
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for writer in store.writers().map_err(|e| e.to_string())? {
        for stored in store.log(writer).map_err(|e| e.to_string())? {
            if stored.link.header.kind == EventKind::Recall {
                out.push((
                    stored.link.header.address,
                    stored.content.unwrap_or_default(),
                ));
            }
        }
    }

    Ok(out)
}

/// The arguments a step text names for `tool`: `range from "A" to "B"`
/// is two addresses, the rest key and value pairs.
fn tool_arguments(w: &mut World, text: &str) -> Result<Value, String> {
    if let Some(ends) = text.strip_prefix("range from ") {
        let (from, to) = ends
            .split_once(" to ")
            .ok_or_else(|| format!("{text:?} names no end of the range"))?;
        let end = |quoted: &str| -> Result<Value, String> {
            serde_json::from_str(quoted).map_err(|e| format!("{quoted:?}: {e}"))
        };
        return Ok(json!({"range": {"from": end(from)?, "to": end(to)?}}));
    }

    arguments_of(w, text)
}

#[when(regex = r#"^the agent calls the MCP tool "(event_expand|event_get)" with (.+)$"#)]
fn calls_tool(w: &mut World, tool: String, args: String) -> Result<(), String> {
    if crate::recall_scope::serves_a_run(w) {
        return crate::recall_scope::call(w, &tool, &args);
    }
    record_runs(w)?;
    if args == "that address" {
        return crate::recall_address::call_that_address(w, &tool);
    }
    let arguments = tool_arguments(w, &args)?;
    recall(w).before = recall_events(w)?.len();
    call_tool(w, &tool, arguments).map(drop)
}

/// The envelope the last recall call answered.
fn envelope_of(w: &mut World) -> Result<Value, String> {
    let answer = last_reply(w)?;
    if answer.is_error {
        return Err(format!("the recall call was refused: {}", answer.text));
    }

    serde_json::from_str(&answer.text).map_err(|e| format!("the answer is no JSON: {e}"))
}

/// Fails unless `envelope` has the shape of §9.3.
fn wrapped(envelope: &Value) -> Result<(), String> {
    let mut names: Vec<&str> = envelope
        .as_object()
        .ok_or("the answer is no object")?
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    let closed = [
        "items",
        "next_cursor",
        "parinn_envelope",
        "tombstones",
        "truncated",
        "warning",
    ];
    // A structural envelope adds its marked counts.
    let names: Vec<&str> = names.into_iter().filter(|n| *n != "structural").collect();
    if names != closed {
        return Err(format!("the envelope's fields are {names:?}"));
    }
    if envelope["parinn_envelope"] != 1 || envelope["warning"] != envelope::WARNING {
        return Err("the envelope carries no version 1 and fixed warning".to_owned());
    }

    Ok(())
}

#[then(regex = r"^the recall result is wrapped in the envelope$")]
fn wrapped_in_envelope(w: &mut World) -> Result<(), String> {
    let envelope = envelope_of(w)?;
    wrapped(&envelope)
}

#[then(
    regex = r#"^the envelope carries "parinn_envelope" 1 and the fixed envelope warning in its field "warning"$"#
)]
fn carries_version_and_warning(w: &mut World) -> Result<(), String> {
    let envelope = envelope_of(w)?;
    if envelope["parinn_envelope"] == 1 && envelope["warning"] == envelope::WARNING {
        return Ok(());
    }
    Err(format!(
        "the envelope carries {} and {}",
        envelope["parinn_envelope"], envelope["warning"]
    ))
}

// RCL-01.

/// A request line.
fn request(id: u64, method: &str, params: &Value) -> String {
    format!(
        "{}\n",
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
    )
}

/// The result of request `id` among what `parinn mcp` answered.
fn answer(w: &mut World, id: u64) -> Result<Value, String> {
    let out = last_run(w)?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|r| r["id"] == id)
        .map(|r| r["result"].clone())
        .ok_or_else(|| format!("parinn mcp answered no request {id}"))
}

#[when(regex = r#"^the agent lists the tools of the MCP server started by "parinn mcp"$"#)]
fn lists_tools(w: &mut World) -> Result<(), String> {
    record_runs(w)?;
    let input = format!(
        "{}{}",
        request(1, "initialize", &json!({"protocolVersion": "2025-06-18"})),
        request(2, "tools/list", &json!({})),
    );
    run_core(w, &["mcp"], Some(input.as_bytes()))?;
    let tools = answer(w, 2)?["tools"]
        .as_array()
        .cloned()
        .ok_or("tools/list answered no tools")?;
    recall(w).tools = tools;
    Ok(())
}

/// The rows of SRS §9.2's tool table: each tool's name and the names
/// its parameters cell lists outside parentheses.
fn specified() -> Result<Vec<(String, Vec<String>)>, String> {
    let path = testkit::repo_root().join("docs/srs/09-interfaces.md");
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    for line in text.lines().filter(|l| l.starts_with("| `")) {
        let cells = cells_of(line);
        let name = cells.first().map(|c| c.trim().trim_matches('`').to_owned());
        if let (Some(name), Some(params)) = (name, cells.get(2)) {
            rows.push((name, parameters_of(params)));
        }
    }

    Ok(rows)
}

/// The cells of a Markdown table row, an escaped pipe kept in its cell.
fn cells_of(line: &str) -> Vec<String> {
    let mut cells = vec![String::new()];
    let mut escaped = false;
    for c in line.trim().trim_start_matches('|').chars() {
        match (c, escaped) {
            ('|', false) => cells.push(String::new()),
            (c, _) => {
                if let Some(cell) = cells.last_mut() {
                    cell.push(c);
                }
                escaped = c == '\\' && !escaped;
            }
        }
    }
    cells.pop();
    cells
}

/// The backticked names a parameters cell lists outside parentheses.
fn parameters_of(cell: &str) -> Vec<String> {
    let (mut depth, mut inside, mut name) = (0usize, false, String::new());
    let mut out = Vec::new();
    for c in cell.chars() {
        match c {
            '`' if inside => {
                let plain = name.chars().all(|c| c.is_ascii_lowercase() || c == '_');
                if depth == 0 && plain && !name.is_empty() {
                    out.push(name.clone());
                }
                inside = false;
                name.clear();
            }
            '`' => inside = true,
            c if inside => name.push(c),
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    out
}

#[then(regex = r"^the tool list includes (.+), among the other tools SRS section 9\.2 specifies$")]
fn tool_list_includes(w: &mut World, listed: String) -> Result<(), String> {
    let wanted: Vec<&str> = listed.split('"').skip(1).step_by(2).collect();
    let offered: Vec<String> = recall(w)
        .tools
        .iter()
        .filter_map(|t| t["name"].as_str().map(str::to_owned))
        .collect();
    for name in &wanted {
        if !offered.iter().any(|o| o == name) {
            return Err(format!("the tool list lacks {name}: {offered:?}"));
        }
    }
    let rows = specified()?;
    for name in &offered {
        if !rows.iter().any(|(n, _)| n == name) {
            return Err(format!("SRS section 9.2 specifies no tool {name}"));
        }
    }
    if wanted.len() != 6 {
        return Err(format!(
            "wanting six recall tools, the step names {wanted:?}"
        ));
    }
    Ok(())
}

#[then(regex = r"^each tool declares the parameters specified in SRS section 9\.2$")]
fn tools_declare_parameters(w: &mut World) -> Result<(), String> {
    let rows = specified()?;
    for tool in recall(w).tools.clone() {
        let name = tool["name"].as_str().unwrap_or_default();
        let (_, params) = rows
            .iter()
            .find(|(n, _)| n == name)
            .ok_or_else(|| format!("SRS section 9.2 specifies no tool {name}"))?;
        let declared: Vec<&str> = tool["inputSchema"]["properties"]
            .as_object()
            .map(|p| p.keys().map(String::as_str).collect())
            .unwrap_or_default();
        for param in params {
            if !declared.contains(&param.as_str()) {
                return Err(format!(
                    "{name} does not declare {param}, which section 9.2 specifies: {declared:?}"
                ));
            }
        }
    }
    Ok(())
}

/// The calls of RCL-01's six tools.
fn six_calls() -> [(&'static str, Value); 6] {
    [
        ("event_search", json!({"query": "deploy"})),
        ("event_expand", json!({"range": "A1:1-2"})),
        ("event_get", json!({"address": "A1:1"})),
        ("landmark_list", json!({})),
        ("pin_list", json!({})),
        ("stat_list", json!({})),
    ]
}

#[when(regex = r"^the agent calls each of those six tools$")]
fn calls_six_tools(w: &mut World) -> Result<(), String> {
    record_runs(w)?;
    recall(w).before = recall_events(w)?.len();
    recall(w).calls.clear();
    for (tool, arguments) in six_calls() {
        let reply = call_tool(w, tool, arguments)?;
        recall(w).calls.push((tool.to_owned(), reply));
    }
    Ok(())
}

#[then(
    regex = r#"^each returns its content in the envelope, "stat_list" its counts in an envelope marked structural$"#
)]
fn each_in_envelope(w: &mut World) -> Result<(), String> {
    let calls = recall(w).calls.clone();
    if calls.len() != 6 {
        return Err(format!("wanting six calls, the steps made {}", calls.len()));
    }
    for (tool, reply) in calls {
        if reply.is_error {
            return Err(format!("{tool} was refused: {}", reply.text));
        }
        let envelope: Value =
            serde_json::from_str(&reply.text).map_err(|e| format!("{tool}: {e}"))?;
        wrapped(&envelope).map_err(|e| format!("{tool}: {e}"))?;
        let counts = &envelope["structural"];
        match (tool.as_str(), counts.as_object()) {
            ("stat_list", Some(counts)) => {
                let names: BTreeSet<&str> = counts.keys().map(String::as_str).collect();
                let want = BTreeSet::from(["events", "runs", "compactions", "recalls"]);
                if names != want || !counts.values().all(Value::is_u64) {
                    return Err(format!("stat_list counts {counts:?}"));
                }
                if envelope["items"] != json!([]) {
                    return Err("stat_list returned items".to_owned());
                }
            }
            ("stat_list", None) => {
                return Err("stat_list's envelope is not marked structural".to_owned());
            }
            (_, Some(_)) => return Err(format!("{tool}'s envelope is marked structural")),
            (_, None) => {}
        }
    }
    Ok(())
}

#[then(regex = r"^each call is recorded as a recall event$")]
fn each_recorded(w: &mut World) -> Result<(), String> {
    let events = recall_events(w)?;
    let gained = events.len().saturating_sub(recall(w).before);
    let tools: Vec<String> = events
        .iter()
        .skip(recall(w).before)
        .filter_map(|(_, c)| c["tool"].as_str().map(str::to_owned))
        .collect();
    let mut want: Vec<String> = six_calls().iter().map(|(t, _)| (*t).to_owned()).collect();
    want.sort();
    let mut got = tools;
    got.sort();
    if gained == 6 && got == want {
        return Ok(());
    }
    Err(format!(
        "the record gained {gained} recall events, of {got:?}"
    ))
}

// RCL-03.

/// Fetches every page of the `event_expand` the last step started, by
/// its cursor, keeping each answer.
fn follow_cursor(w: &mut World) -> Result<(), String> {
    let mut page = envelope_of(w)?;
    recall(w).pages = vec![page.clone()];
    while let Some(cursor) = page["next_cursor"].as_str().map(str::to_owned) {
        call_tool(w, "event_expand", json!({"cursor": cursor}))?;
        page = envelope_of(w)?;
        recall(w).pages.push(page.clone());
        if recall(w).pages.len() > 400 {
            return Err("the cursor never ends".to_owned());
        }
    }
    Ok(())
}

#[then(
    regex = r"^the items carry the exact post-redaction content with payload references resolved$"
)]
fn exact_content(w: &mut World) -> Result<(), String> {
    let envelope = envelope_of(w)?;
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let mut resolved = 0;
    let items = envelope["items"].as_array().cloned().unwrap_or_default();
    if items.is_empty() {
        return Err("the envelope holds no item".to_owned());
    }
    for item in &items {
        let cut = item["flags"]
            .as_array()
            .is_some_and(|f| f.iter().any(|f| f == "cut"));
        let address = Address::parse(item["address"].as_str().unwrap_or_default())
            .map_err(|e| e.to_string())?
            .0;
        let stored = store.event(address).map_err(|e| e.to_string())?;
        let kept = stored
            .as_ref()
            .and_then(|s| s.content.as_ref())
            .and_then(|c| c["text"].as_str())
            .ok_or_else(|| format!("{address} holds no text"))?;
        let shown = item["content"].as_str().unwrap_or_default();
        if stored.as_ref().is_some_and(|s| s.payload.is_some()) {
            resolved += 1;
        }
        let exact = if cut {
            kept.starts_with(shown)
        } else {
            kept == shown
        };
        if !exact {
            return Err(format!(
                "{address} shows other content than the record holds"
            ));
        }
    }
    if resolved == 0 {
        let seen: Vec<String> = items
            .iter()
            .map(|i| {
                format!(
                    "{}:{}",
                    i["address"]
                        .as_str()
                        .unwrap_or_default()
                        .rsplit('/')
                        .next()
                        .unwrap_or_default(),
                    i["content"].as_str().map_or(0, str::len)
                )
            })
            .collect();
        return Err(format!("no item of the range was a payload: {seen:?}"));
    }
    Ok(())
}

#[then(
    regex = r#"^the envelope is at most 8,000 estimated model tokens with "truncated" true and a "next_cursor"$"#
)]
fn at_most_the_cap(w: &mut World) -> Result<(), String> {
    let answer = last_reply(w)?;
    let envelope = envelope_of(w)?;
    let spent = tokens::estimate(&answer.text);
    if spent > 8_000 {
        return Err(format!("the envelope is {spent} estimated model tokens"));
    }
    if envelope["truncated"] != true || !envelope["next_cursor"].is_string() {
        return Err("the envelope is neither truncated nor continued by a cursor".to_owned());
    }
    Ok(())
}

#[then(
    regex = r#"^calling "event_expand" with that cursor returns the following events without gap or overlap$"#
)]
fn cursor_continues(w: &mut World) -> Result<(), String> {
    follow_cursor(w)?;
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let pages = recall(w).pages.clone();
    for (n, page) in pages.iter().enumerate() {
        let spent = tokens::estimate(&page.to_string());
        if n > 0 && spent > 8_100 {
            return Err(format!("page {n} is {spent} estimated model tokens"));
        }
    }
    // Each event in order, a cut one made whole again from its pieces.
    let mut events: Vec<(String, String)> = Vec::new();
    for item in pages
        .iter()
        .flat_map(|p| p["items"].as_array().cloned().unwrap_or_default())
    {
        let address = item["address"].as_str().unwrap_or_default().to_owned();
        let content = item["content"].as_str().unwrap_or_default();
        let continued = item["flags"]
            .as_array()
            .is_some_and(|f| f.iter().any(|f| f == "continued"));
        match events.last_mut() {
            Some((last, text)) if continued && *last == address => text.push_str(content),
            _ => events.push((address, content.to_owned())),
        }
    }
    let first = Address::parse(&events[0].0).map_err(|e| e.to_string())?.0;
    let mut want = 0;
    for (n, (address, text)) in events.iter().enumerate() {
        let parsed = Address::parse(address).map_err(|e| e.to_string())?.0;
        want = first.seq.get() + n as u64;
        if parsed.writer != first.writer || parsed.seq.get() != want {
            return Err(format!(
                "event {n} is {address}, wanting seq {want}: a gap or an overlap"
            ));
        }
        let kept = store.event(parsed).map_err(|e| e.to_string())?;
        let kept = kept
            .as_ref()
            .and_then(|s| s.content.as_ref())
            .and_then(|c| c["text"].as_str());
        if kept != Some(text.as_str()) {
            return Err(format!(
                "{address} came back other than the record holds it"
            ));
        }
    }
    if events.len() < 400 || want != first.seq.get() + events.len() as u64 - 1 {
        return Err(format!(
            "the pages cover {} events, wanting 400",
            events.len()
        ));
    }
    Ok(())
}

// RCL-07.

#[then(
    regex = r#"^the record gains one recall event with provenance "assistant" carrying the query$"#
)]
fn gains_one_recall_event(w: &mut World) -> Result<(), String> {
    let before = recall(w).before;
    let events = recall_events(w)?;
    if events.len() != before + 1 {
        return Err(format!(
            "the record held {before} recall events and now holds {}",
            events.len()
        ));
    }
    let (address, content) = events.last().ok_or("no recall event")?.clone();
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let stored = store
        .event(address)
        .map_err(|e| e.to_string())?
        .ok_or("event gone")?;
    if stored.link.header.provenance.name() != "assistant" {
        return Err(format!(
            "its provenance is {}",
            stored.link.header.provenance.name()
        ));
    }
    if content["tool"] != "event_search" || content["arguments"]["query"] != "deploy" {
        return Err(format!("the recall event carries {content}"));
    }
    Ok(())
}

#[then(regex = r"^that event lists the addresses of the returned events$")]
fn lists_addresses(w: &mut World) -> Result<(), String> {
    let (_, content) = recall_events(w)?.last().ok_or("no recall event")?.clone();
    let envelope = envelope_of(w)?;
    let returned: Vec<&str> = envelope["items"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|i| i["address"].as_str())
        .collect();
    let listed: Vec<&str> = content["returned"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    if returned.is_empty() || returned != listed {
        return Err(format!(
            "the call returned {returned:?}, the event lists {listed:?}"
        ));
    }
    Ok(())
}

// RCL-01: `mcp_server_get`.

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
    harness.stop()?;
    recall(w).states.push(text);
    Ok(())
}

#[then(
    regex = r#"^it returns "session_binding" as "held" or "not_held", "search_index" as "readable" or "unreadable", "searches_timed_out" as a count and "version", and no recalled content or envelope, whether or not a run is found for the call$"#
)]
fn server_state_is_closed(w: &mut World) -> Result<(), String> {
    let states = recall(w).states.clone();
    let [none, held] = states.as_slice() else {
        return Err(format!(
            "wanting two answers, the step made {}",
            states.len()
        ));
    };
    for (text, binding) in [(none, "not_held"), (held, "held")] {
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
