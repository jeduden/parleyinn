//! The steps of RCL-08 that phase 13 of plan 2609292012 carries: every
//! address form ParleyInn shows, and the ASCII input form, resolves
//! wherever an address is taken, to the same events, on the MCP tools
//! and on `parinn event expand`.
//!
//! `@RCL-08` stays pending: its clause on the tombstone or quarantine
//! marker of a purged or quarantined event needs `parinn purge --range`
//! and `parinn quarantine add` (phase 14), so that step fails naming
//! them. Its other clauses pass for every row.

use ::record::id::{Address, RoomId, Seq, WriterId};
use ::record::range::{self, Label};
use cucumber::{given, then};
use serde_json::{Value, json};
use store::Store;

use crate::World;
use crate::record::{home, last_run, run_core_at_terminal};
use crate::record_search::{Reply, call_tool, last_reply};

/// What the address steps track.
#[derive(Debug, Default)]
pub struct AddressSteps {
    /// The address as shown, and the events it names.
    shown: Option<Shown>,
}

#[derive(Debug, Clone)]
struct Shown {
    text: String,
    writer: WriterId,
    from: u64,
    to: u64,
    room: RoomId,
}

fn steps(w: &mut World) -> &mut AddressSteps {
    w.section::<AddressSteps>()
}

/// The events the shown address names: events 2 and 3 of the run.
const FROM: u64 = 2;
const TO: u64 = 3;

#[given(regex = r"^an event address shown to a person or an agent in (.+) form$")]
fn an_address_in_form(w: &mut World, form: String) -> Result<(), String> {
    crate::recall::record_runs(w)?;
    // The first address asked for by label assigns the labels.
    run_core_at_terminal(w, &["event", "expand", "A1:1"])?;
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let node = administration::node::Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let labels = store
        .writer_labels(node.device_writer())
        .map_err(|e| e.to_string())?;
    let (writer, label) = labels
        .iter()
        .find(|(_, l)| Label::new(1) == Some(**l))
        .map(|(w, l)| (*w, *l))
        .ok_or("no writer holds the label A1")?;
    let room = store
        .seat(writer)
        .map_err(|e| e.to_string())?
        .ok_or("the writer has no seat")?
        .room;
    let seq = |n: u64| Seq::new(n).ok_or("a seq starts at 1");
    let (from, to) = (seq(FROM)?, seq(TO)?);
    let (text, last) = match form.as_str() {
        "short" => (range::short(label, from), FROM),
        "range" => (range::range(label, from, to), TO),
        "full" => (Address::new(writer, from).full(&room), FROM),
        r#"ASCII input, such as "A1:7""# => (format!("{label}:{FROM}"), FROM),
        r#"ASCII input range, such as "A1:3-9""# => (format!("{label}:{FROM}-{TO}"), TO),
        other => return Err(format!("no step knows the form {other:?}")),
    };
    steps(w).shown = Some(Shown {
        text,
        writer,
        from: FROM,
        to: last,
        room,
    });
    Ok(())
}

/// Calls `tool` with the address shown, as the form asks: an address for
/// `event_get`, a range for `event_expand`.
pub fn call_that_address(w: &mut World, tool: &str) -> Result<(), String> {
    let text = steps(w)
        .shown
        .clone()
        .ok_or("no step showed an address")?
        .text;
    let arguments = if tool == "event_get" {
        json!({"address": text})
    } else {
        json!({"range": text})
    };
    call_tool(w, tool, arguments).map(drop)
}

fn shown(w: &mut World) -> Result<Shown, String> {
    steps(w)
        .shown
        .clone()
        .ok_or_else(|| "no step showed an address".to_owned())
}

/// The items of the last recall call.
fn items(answer: &Reply) -> Result<Vec<Value>, String> {
    if answer.is_error {
        return Err(format!("the call was refused: {}", answer.text));
    }
    let envelope: Value = serde_json::from_str(&answer.text).map_err(|e| e.to_string())?;

    Ok(envelope["items"].as_array().cloned().unwrap_or_default())
}

#[then(regex = r"^the recall result contains exactly the events the address names$")]
fn exactly_the_events(w: &mut World) -> Result<(), String> {
    let shown = shown(w)?;
    let answer = last_reply(w)?;
    let got: Vec<String> = items(&answer)?
        .iter()
        .filter_map(|i| i["address"].as_str().map(str::to_owned))
        .collect();
    let want: Vec<String> = (shown.from..=shown.to)
        .filter_map(Seq::new)
        .map(|seq| Address::new(shown.writer, seq).full(&shown.room))
        .collect();
    if got == want {
        return Ok(());
    }
    Err(format!("the result holds {got:?}, wanting {want:?}"))
}

#[then(
    regex = r"^an address of a purged or quarantined event resolves to its tombstone or quarantine marker$"
)]
fn purged_resolves(_w: &mut World) -> Result<(), String> {
    Err("needs parinn purge --range and parinn quarantine add (phase 14)".to_owned())
}

#[then(
    regex = r#"^"parinn event expand" given the same address in the same form returns the same events$"#
)]
fn the_command_agrees(w: &mut World) -> Result<(), String> {
    let shown = shown(w)?;
    let answer = last_reply(w)?;
    let contents: Vec<String> = items(&answer)?
        .iter()
        .filter_map(|i| i["content"].as_str().map(str::to_owned))
        .collect();
    run_core_at_terminal(w, &["event", "expand", &shown.text])?;
    let out = last_run(w)?;
    if !out.status.success() {
        return Err(format!(
            "parinn event expand exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let listed = text
        .lines()
        .filter(|l| l.starts_with("parinn:room/"))
        .count();
    if listed != contents.len() {
        return Err(format!(
            "the command lists {listed} events, the tool returned {}",
            contents.len()
        ));
    }
    let mut at = 0;
    for content in &contents {
        let safe = content.replace(['\u{1b}', '\r'], "");
        match text[at..].find(&safe) {
            Some(found) => at += found + safe.len(),
            None => return Err(format!("the command does not show {content:?}, in order")),
        }
    }
    Ok(())
}

#[then(
    regex = r"^no two writers this node holds share a writer label, and each writer-label assignment is a structural event in the personal-room writer of the device seat that assigned it$"
)]
fn labels_are_unique_structural_events(w: &mut World) -> Result<(), String> {
    use ::record::event::{EventKind, Provenance};
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let node = administration::node::Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let device = node.device_writer();
    let labels = store.writer_labels(device).map_err(|e| e.to_string())?;
    let mut shown: Vec<String> = labels.values().map(ToString::to_string).collect();
    shown.sort();
    shown.dedup();
    if shown.len() != labels.len() || labels.is_empty() {
        return Err(format!("the labels are not unique: {labels:?}"));
    }
    let seat = store
        .seat(device)
        .map_err(|e| e.to_string())?
        .ok_or("no device seat")?;
    if seat.room != node.personal_room() {
        return Err("the device seat's writer lies outside the personal room".to_owned());
    }
    let assignments: Vec<_> = store
        .log(device)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|s| s.link.header.kind == EventKind::WriterLabel)
        .collect();
    if assignments.is_empty()
        || assignments
            .iter()
            .any(|s| s.link.header.provenance != Provenance::Structural)
    {
        return Err("a writer-label assignment is no structural event".to_owned());
    }
    for writer in store.writers().map_err(|e| e.to_string())? {
        let has_events = !store.log(writer).map_err(|e| e.to_string())?.is_empty();
        if has_events && writer != device && !labels.contains_key(&writer) {
            return Err(format!("the writer {writer} holds events and no label"));
        }
    }
    Ok(())
}
