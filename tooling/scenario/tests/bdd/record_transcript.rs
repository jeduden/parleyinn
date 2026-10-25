//! The steps of REC-05 and REC-26 that phase 7 of plan 2609292012
//! carries: the transcript parser keeps every line, a line it cannot
//! parse as a redacted `unparsed` event and a line that drifts from its
//! recorded shape whole as untrusted `harness_text`, each counted and
//! audited. It also holds what the provenance steps share: Claude Code
//! 2.1.296 lines taken from the recordings, an ingest of a transcript,
//! and the recall of one event through `event_get`.
//!
//! `@REC-26` stays pending: its harness session's MCP server ingests
//! live (REC-13, phase 10). Until then its drift steps ingest with
//! `parinn ingest`, its typed lines matching a prompt the hook handlers
//! got (phase 9).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ::record::event::{EventKind, Provenance};
use ::record::id::{Address, Seq, WriterId};
use audit::log::Log;
use cucumber::{given, then, when};
use harness_contract::{recording, shapes};
use serde_json::{Value, json};
use store::{Store, Stored};

use crate::World;
use crate::record::{home, last_run, run_core};
use crate::security_redaction::{append, transcript};

/// An Anthropic API key in its documented shape, which the rule set
/// redacts.
pub const API_KEY: &str = "sk-ant-api03-Abcdefghijklmnopqrstuvwxyz0123456789";
/// An AWS access key, as AWS documents an example one.
pub const AWS_ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
/// The recorded Claude Code version the shapes are generated from.
pub const VERSION: &str = "2.1.296";

/// What the transcript steps track: the counters before the scenario's
/// first ingest, the raw lines a step appended, and the event the last
/// recorded item became.
#[derive(Debug, Default)]
pub struct TranscriptSteps {
    baseline: Option<BTreeMap<String, u64>>,
    raw: Vec<String>,
    /// The writer and seq of the event the last step's item became.
    item: Option<(String, u64)>,
}

fn steps(w: &mut World) -> &mut TranscriptSteps {
    w.section::<TranscriptSteps>()
}

/// Every line the recordings of [`VERSION`] hold, by transcript.
fn recorded_transcripts() -> Result<Vec<Vec<Value>>, String> {
    let version = testkit::repo_root()
        .join("testdata/claude-code")
        .join(VERSION);
    shapes::transcripts(&version)?
        .iter()
        .map(|path| recording::lines(path))
        .collect()
}

/// The first recorded line `pick` accepts.
pub fn recorded(what: &str, pick: impl Fn(&Value) -> bool) -> Result<Value, String> {
    recorded_transcripts()?
        .into_iter()
        .flatten()
        .find(|line| pick(line))
        .ok_or_else(|| format!("no recorded line is {what}"))
}

/// A recorded assistant line calling `tool`, and the line holding its
/// result, from one transcript.
pub fn recorded_call(tool: &str) -> Result<(Value, Value), String> {
    for lines in recorded_transcripts()? {
        for call in &lines {
            let Some(id) = tool_use_id(call, tool) else {
                continue;
            };
            let result = lines.iter().find(|l| {
                l["message"]["content"][0]["tool_use_id"].as_str() == Some(id.as_str())
                    && l["message"]["content"].as_array().map(Vec::len) == Some(1)
            });
            if let Some(result) = result {
                return Ok((call.clone(), result.clone()));
            }
        }
    }
    Err(format!(
        "no recording holds a call of {tool} with its result"
    ))
}

/// The id of the call of `tool` the assistant line `line` makes.
fn tool_use_id(line: &Value, tool: &str) -> Option<String> {
    let block = &line["message"]["content"][0];
    (line["type"] == "assistant"
        && block["type"] == "tool_use"
        && block["name"].as_str().is_some_and(|n| n.starts_with(tool)))
    .then(|| block["id"].as_str().map(str::to_owned))
    .flatten()
}

/// The counts of every counter the audit log holds now.
fn counts(w: &World) -> Result<BTreeMap<String, u64>, String> {
    let audit = home(w)?.audit();
    let mut out = BTreeMap::new();
    if !audit.exists() {
        return Ok(out);
    }
    for entry in Log::new(audit).entries().map_err(|e| e.to_string())? {
        if let Some(counter) = entry.counter {
            *out.entry(counter).or_insert(0) += 1;
        }
    }
    Ok(out)
}

/// Keeps the counters as they stand before the scenario's first
/// ingest, once.
pub fn remember_counters(w: &mut World) -> Result<(), String> {
    if steps(w).baseline.is_none() {
        let now = counts(w)?;
        steps(w).baseline = Some(now);
    }
    Ok(())
}

/// How far `counter` rose since [`remember_counters`].
fn risen(w: &mut World, counter: &str) -> Result<u64, String> {
    let now = counts(w)?.get(counter).copied().unwrap_or(0);
    let before = steps(w)
        .baseline
        .as_ref()
        .and_then(|b| b.get(counter).copied())
        .unwrap_or(0);
    Ok(now - before)
}

/// Every transcript line's event the store holds, with its writer.
pub fn transcript_events(w: &World) -> Result<Vec<(WriterId, Stored)>, String> {
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for writer in store.writers().map_err(|e| e.to_string())? {
        for stored in store.log(writer).map_err(|e| e.to_string())? {
            if matches!(
                stored.link.header.kind,
                EventKind::TranscriptLine | EventKind::UnparsedLine
            ) {
                out.push((writer, stored));
            }
        }
    }
    Ok(out)
}

/// The text an event's content holds.
fn text(stored: &Stored) -> &str {
    stored
        .content
        .as_ref()
        .and_then(|c| c["text"].as_str())
        .unwrap_or_default()
}

/// Ingests the transcript at `path` with `parinn ingest --path`, and
/// keeps its last line's event as the scenario's item.
pub fn ingest(w: &mut World, path: &Path) -> Result<(), String> {
    remember_counters(w)?;
    let shown = path.display().to_string();
    run_core(w, &["ingest", "--path", &shown, "--json"], None)?;
    let out = last_run(w)?;
    if !out.status.success() {
        return Err(format!(
            "parinn ingest exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let report: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let writer = report["writer"].as_str().unwrap_or_default().to_owned();
    let events = report["events"].as_u64().unwrap_or_default();
    steps(w).item = Some((writer, events));
    Ok(())
}

/// Keeps the event at (`writer`, `seq`) as the scenario's item.
pub fn set_item(w: &mut World, writer: String, seq: u64) {
    steps(w).item = Some((writer, seq));
}

/// Moves the scenario's item back by `back` lines from its transcript's
/// last line.
pub fn item_back(w: &mut World, back: u64) -> Result<(), String> {
    let (writer, seq) = steps(w)
        .item
        .clone()
        .ok_or("no step ingested a transcript")?;
    steps(w).item = Some((writer, seq - back));
    Ok(())
}

/// The scenario's item as the store holds it.
pub fn item(w: &mut World) -> Result<Stored, String> {
    let (writer, seq) = steps(w)
        .item
        .clone()
        .ok_or("no step ingested a transcript")?;
    let writer = WriterId::parse(&writer).map_err(|e| e.to_string())?;
    let seq = Seq::new(seq).ok_or("no event at seq 0")?;
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    store
        .event(Address::new(writer, seq))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "the item's event is not in the store".to_owned())
}

/// The scenario's item as `event_get` recalls it, inside the envelope.
pub fn recalled_item(w: &mut World) -> Result<Value, String> {
    let (writer, seq) = steps(w)
        .item
        .clone()
        .ok_or("no step ingested a transcript")?;
    let requests = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
            "name": "event_get",
            "arguments": {"address": format!("{writer}:{seq}"), "scope": "room"},
        }}),
    ];
    let input: String = requests.iter().map(|r| format!("{r}\n")).collect();
    run_core(w, &["mcp"], Some(input.as_bytes()))?;
    let out = last_run(w)?;
    let replies: Vec<Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    let text = replies
        .iter()
        .find(|r| r["id"] == 2)
        .and_then(|r| r["result"]["content"][0]["text"].as_str())
        .ok_or_else(|| format!("event_get answered no envelope: {replies:?}"))?;
    let envelope: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    Ok(envelope["items"][0].clone())
}

/// Checks the scenario's item is stored with `provenance` and recalled
/// with it at `trust`.
pub fn item_is(w: &mut World, provenance: &str, trust: &str) -> Result<(), String> {
    let stored = item(w)?.link.header.provenance.name();
    let recalled = recalled_item(w)?;
    if stored == provenance && recalled["provenance"] == provenance && recalled["trust"] == trust {
        return Ok(());
    }
    Err(format!(
        "the event is stored as {stored} and recalled as {} at {}, wanting {provenance} at {trust}",
        recalled["provenance"], recalled["trust"]
    ))
}

/// Appends `text`, a raw line, to the transcript of the run `name`.
fn append_raw(w: &mut World, name: &str, text: &str) -> Result<(), String> {
    let path = transcript(w, name)?;
    let mut body = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    body.push_str(text);
    body.push('\n');
    fs::write(&path, body).map_err(|e| e.to_string())?;
    steps(w).raw.push(text.to_owned());
    Ok(())
}

#[given(
    regex = r#"^"([^"]+)" contains a malformed JSON line and a line of unknown type "([^"]+)", each containing an API key$"#
)]
fn malformed_and_unknown(w: &mut World, name: String, kind: String) -> Result<(), String> {
    remember_counters(w)?;
    append_raw(
        w,
        &name,
        &format!("{{\"type\": \"user\", \"note\": \"{API_KEY}\""),
    )?;
    let unknown = json!({"type": kind, "version": VERSION, "note": API_KEY});
    append_raw(w, &name, &unknown.to_string())
}

#[then(
    regex = r#"^both lines are stored as events with provenance "unparsed" carrying the raw line with the API key redacted$"#
)]
fn both_unparsed(w: &mut World) -> Result<(), String> {
    let events = transcript_events(w)?;
    let raw = steps(w).raw.clone();
    for line in raw {
        let want = line.replace(API_KEY, "[REDACTED:anthropic_api_key]");
        let found = events.iter().any(|(_, e)| {
            e.link.header.provenance == Provenance::Unparsed
                && e.link.header.kind == EventKind::UnparsedLine
                && text(e) == want
        });
        if !found {
            return Err(format!("no unparsed event carries {want:?}"));
        }
    }
    Ok(())
}

#[then(regex = r#"^no line of "([^"]+)" is missing from the record$"#)]
fn no_line_missing(w: &mut World, name: String) -> Result<(), String> {
    let path = transcript(w, &name)?;
    let lines = fs::read_to_string(&path)
        .map_err(|e| e.to_string())?
        .lines()
        .count() as u64;
    let mut numbers: Vec<u64> = transcript_events(w)?
        .iter()
        .filter_map(|(_, e)| e.content.as_ref()?["line"].as_u64())
        .collect();
    numbers.sort_unstable();
    if numbers == (1..=lines).collect::<Vec<_>>() {
        return Ok(());
    }
    Err(format!(
        "{name} holds {lines} lines; the record holds lines {numbers:?}"
    ))
}

#[then(regex = r#"^the counter "([a-z_]+)" increases by (\d+)$"#)]
fn counter_increases(w: &mut World, counter: String, by: u64) -> Result<(), String> {
    let rose = risen(w, &counter)?;
    if rose == by {
        return Ok(());
    }
    Err(format!("{counter} rose by {rose}, not {by}"))
}

#[given(regex = r"^the shapes recorded for the supported Claude Code versions$")]
fn recorded_shapes(_w: &mut World) -> Result<(), String> {
    let supported = transcript::shape::Shapes::supported();
    let versions: Vec<&str> = supported.versions().map(|r| r.version.as_str()).collect();
    if versions.contains(&VERSION) {
        return Ok(());
    }
    Err(format!(
        "the supported versions are {versions:?}, not {VERSION}"
    ))
}

/// The typed `user` line of the drift `drift` names, carrying an AWS
/// access key in its prompt, after the hook handlers of its harness
/// session recorded its process ancestry and its prompt commitment, so
/// only the drift keeps it from `user`.
fn typed_drift(w: &mut World, drift: &str) -> Result<Value, String> {
    use crate::record_hooks::{prompt_submit, run_hook, session_start, typed_line};
    let prompt = format!("deploy with {AWS_ACCESS_KEY}");
    let start = session_start(w, "main-run", "startup")?;
    run_hook(w, "SessionStart", &start, &[])?;
    let submit = prompt_submit(w, "main-run", "p-drift", &prompt)?;
    run_hook(w, "UserPromptSubmit", &submit, &[])?;
    let mut line = typed_line(&prompt, "p-drift", "typed", "main-run")?;
    if drift.contains("\"version\"") {
        line["version"] = json!("9.9.9");
    } else {
        line["futureKey"] = json!(true);
    }
    Ok(line)
}

/// A line of the drift `drift` names, carrying an AWS access key.
fn drift_line(drift: &str) -> Result<Value, String> {
    match drift {
        "the harness appends a line of a type no supported version records" => Ok(json!({
            "type": "future_kind",
            "version": VERSION,
            "note": AWS_ACCESS_KEY,
        })),
        _ => {
            let mut line = recorded("an assistant text line", |l| {
                l["type"] == "assistant" && l["message"]["content"][0]["type"] == "text"
            })?;
            if let Some(fields) = line.as_object_mut() {
                fields.remove("message");
                fields.insert("requestId".to_owned(), json!(AWS_ACCESS_KEY));
            }
            Ok(line)
        }
    }
}

#[when(
    regex = r#"^(the harness appends a line of a type no supported version records|the harness appends an "assistant" line without the "message" key|the harness appends a "user" line typed at the input, matching a prompt, whose "version" is no supported version|the harness appends a "user" line typed at the input, matching a prompt, with a key no recorded shape has|the hook "SessionStart" runs with a hook input that lacks "source"), carrying an AWS access key$"#
)]
fn harness_drifts(w: &mut World, drift: String) -> Result<(), String> {
    remember_counters(w)?;
    let line = if drift.starts_with("the hook") {
        // The hook input drifts; the transcript goes on with a clean
        // line, which ingest records.
        let mut input = crate::record_hooks::session_start(w, "main-run", "startup")?;
        if let Some(fields) = input.as_object_mut() {
            fields.remove("source");
        }
        input["cwd"] = json!(format!("/w/{AWS_ACCESS_KEY}"));
        run_core(
            w,
            &["hook", "SessionStart"],
            Some(input.to_string().as_bytes()),
        )?;
        crate::record_hooks::note_answer(w)?;
        recorded("a mode line", |l| l["type"] == "mode")?
    } else if drift.contains("matching a prompt") {
        typed_drift(w, &drift)?
    } else {
        drift_line(&drift)?
    };
    append_raw(w, "main-run", &line.to_string())?;
    append(
        w,
        "main-run",
        &recorded("a mode line", |l| l["type"] == "mode")?,
    )?;
    let path = transcript(w, "main-run")?;
    ingest(w, &path)?;
    // The drifting line is the one before the transcript's last.
    item_back(w, 1)
}

#[then(
    regex = r#"^the line is stored as an "unparsed" event, counted as format drift too, and the counter "events_unparsed" increases by 1$"#
)]
fn stored_unparsed_and_drift(w: &mut World) -> Result<(), String> {
    let stored = item(w)?;
    if stored.link.header.provenance != Provenance::Unparsed {
        return Err(format!(
            "the line is stored as {}",
            stored.link.header.provenance.name()
        ));
    }
    match (risen(w, "format_drift")?, risen(w, "events_unparsed")?) {
        (1, 1) => Ok(()),
        (drift, unparsed) => Err(format!(
            "format_drift rose by {drift} and events_unparsed by {unparsed}"
        )),
    }
}

#[then(regex = r#"^the line is stored whole as an untrusted "harness_text" event, never "user"$"#)]
fn stored_whole_never_user(w: &mut World) -> Result<(), String> {
    // A typed line matching a prompt in a harness session whose process
    // ancestry is observed would be `user` but for its drift.
    stored_whole_harness_text(w)
}

#[then(regex = r#"^the line is stored whole as an untrusted "harness_text" event$"#)]
fn stored_whole_harness_text(w: &mut World) -> Result<(), String> {
    let stored = item(w)?;
    let raw = steps(w).raw.last().cloned().unwrap_or_default();
    let want = raw.replace(AWS_ACCESS_KEY, "[REDACTED:aws_access_key]");
    if text(&stored) != want {
        return Err(format!(
            "the event holds {:?}, not the whole line",
            text(&stored)
        ));
    }
    item_is(w, "harness_text", "untrusted")
}

#[then(regex = r"^the stored text contains no AWS access key$")]
fn no_aws_key_stored(w: &mut World) -> Result<(), String> {
    let store = fs::read(home(w)?.store()).map_err(|e| e.to_string())?;
    let found = store
        .windows(AWS_ACCESS_KEY.len())
        .any(|win| win == AWS_ACCESS_KEY.as_bytes());
    if found
        || transcript_events(w)?
            .iter()
            .any(|(_, e)| text(e).contains(AWS_ACCESS_KEY))
    {
        return Err("the store holds the AWS access key".to_owned());
    }
    Ok(())
}

#[then(
    regex = r#"^the counter "format_drift" increases by 1 and the audit log names the harness version and what differed$"#
)]
fn drift_counted_and_named(w: &mut World) -> Result<(), String> {
    let rose = risen(w, "format_drift")?;
    let entries = Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    let named = entries
        .iter()
        .rev()
        .find(|e| e.counter.as_deref() == Some("format_drift"));
    let detail = named.map(|e| e.detail.clone()).unwrap_or_default();
    // The harness version the drifting line names, or for a hook input,
    // which names none, the newest supported version it was checked
    // against.
    let named_version = [VERSION, "9.9.9"]
        .iter()
        .any(|v| detail.contains(&format!("claude-code:{v}:")));
    let differed = [
        "unrecorded-type",
        "missing:message",
        "unsupported-version",
        "beyond:futureKey",
        "missing:source",
    ]
    .iter()
    .any(|d| detail.ends_with(d));
    if rose == 1 && named_version && differed {
        return Ok(());
    }
    Err(format!(
        "format_drift rose by {rose}; the audit log says {detail:?}"
    ))
}

#[then(regex = r#"^"parinn status" shows the format drift$"#)]
fn status_shows_drift(w: &mut World) -> Result<(), String> {
    run_core(w, &["status"], None)?;
    let shown = String::from_utf8_lossy(&last_run(w)?.stdout).into_owned();
    if shown.contains("format_drift 1") {
        return Ok(());
    }
    Err(format!("parinn status shows no format drift:\n{shown}"))
}

#[then(regex = r#"^ingest goes on with the next line of "([^"]+)"$"#)]
fn ingest_goes_on(w: &mut World, _name: String) -> Result<(), String> {
    let (writer, seq) = steps(w)
        .item
        .clone()
        .ok_or("no step ingested a transcript")?;
    steps(w).item = Some((writer, seq + 1));
    let next = item(w)?;
    if next.link.header.kind == EventKind::TranscriptLine
        && next.link.header.provenance == Provenance::HarnessMeta
    {
        return Ok(());
    }
    Err(format!(
        "the next line is stored as {} {}",
        next.link.header.kind.as_str(),
        next.link.header.provenance.name()
    ))
}

#[then(
    regex = r#"^regenerating the shapes from "testdata/claude-code" reproduces the recorded ones byte for byte, and a drift case fails when they differ$"#
)]
fn shapes_regenerate(_w: &mut World) -> Result<(), String> {
    let root: PathBuf = testkit::repo_root();
    let problems = shapes::regenerate(
        &root.join("testdata/claude-code"),
        &root.join(shapes::SHAPES_DIR),
        false,
    )?;
    if !problems.is_empty() {
        return Err(problems.join("\n"));
    }
    let guarded = drift::cases().iter().any(|c| {
        c.guards == "REC-26"
            && c.check
                .args
                .iter()
                .any(|a| a == "recorded_shapes_regenerate_byte_for_byte")
    });
    if guarded {
        return Ok(());
    }
    Err("no drift case changes the recorded shapes".to_owned())
}
