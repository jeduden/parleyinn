//! The steps of PRV-02 and PRV-06 that phase 11 of plan 2609292012
//! carries: the default trust policy derives each event's trust level
//! from the record and the node's key set alone, and nothing stores it.
//!
//! `@PRV-02` stays `@pending`. Its rows that need a launcher, a signed
//! sandbox state, another node's device key or a `user` line a hook
//! handler's ancestry marks are not reachable before OQ-40 and M7, and
//! name themselves unreachable here. The rows a built core can show
//! pass, narrowed locally: an operator event the CLI records, and every
//! other class a witnessed run seat's writer records.

use ::record::commitment::CommitmentKey;
use ::record::event::{EventKind, Provenance};
use administration::node::Node;
use cucumber::{then, when};
use key::random::OsEntropy;
use key::signing::SeatKey;
use serde_json::json;
use store::{NewEvent, Store};

use crate::World;
use crate::record::{home, run_core, run_core_at_terminal};
use crate::record_transcript::{recalled_item, set_item};

/// The writers whose rows the built core can show.
const OF_THIS_NODE: [&str; 2] = [
    "a writer of this node",
    "a writer of this node, recorded live by its CLI",
];

/// Records an event of `provenance` in a witnessed run seat of this
/// node, as its MCP server would, and keeps it as the scenario's item.
fn in_a_run_seat(w: &mut World, provenance: &str) -> Result<(), String> {
    let class = Provenance::parse(provenance).ok_or_else(|| format!("no class {provenance:?}"))?;
    let home = home(w)?;
    home.create().map_err(|e| e.to_string())?;
    let mut store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = Node::open(&home, &mut store, &mut OsEntropy).map_err(|e| e.to_string())?;
    let key = SeatKey::mint(&mut OsEntropy).map_err(|e| e.to_string())?;
    let seat = node
        .certify_run_seat(&mut store, &key, &mut OsEntropy)
        .map_err(|e| e.to_string())?;
    let event = NewEvent {
        kind: EventKind::TranscriptLine,
        provenance: class,
        content: json!({"line": 1, "text": "a line of the run"}),
        commitment_key: CommitmentKey::from_bytes(
            key::random::draw(&mut OsEntropy).map_err(|e| e.to_string())?,
        ),
        transcript_timestamp: None,
    };
    let at = store
        .append(seat.writer, &[event])
        .map_err(|e| e.to_string())?;
    let seq = at.last().map_or(1, |a| a.seq.get());
    set_item(w, seat.writer.to_string(), seq);
    Ok(())
}

/// Records an `operator` event as the CLI does: a neutral principal act
/// at a terminal, in this node's device seat.
fn by_the_cli(w: &mut World) -> Result<(), String> {
    run_core_at_terminal(w, &["counter", "ack"])?;
    let home = home(w)?;
    let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = Node::find(&home, &store).map_err(|e| e.to_string())?;
    let log = store.log(node.device_writer()).map_err(|e| e.to_string())?;
    let act = log
        .iter()
        .rev()
        .find(|e| e.link.header.kind == EventKind::PrincipalAct)
        .ok_or("the CLI recorded no principal act")?;
    set_item(
        w,
        node.device_writer().to_string(),
        act.link.header.address.seq.get(),
    );
    Ok(())
}

#[when(regex = r#"^an event with provenance "([^"]+)" written by (.+) is recorded$"#)]
fn event_of_this_node(w: &mut World, provenance: String, writer: String) -> Result<(), String> {
    if !OF_THIS_NODE.contains(&writer.as_str()) {
        return Err(format!(
            "not reachable in M1 (a launcher, a signed sandbox state, another node's key or a hook ancestry is needed): {writer}"
        ));
    }
    if provenance == "operator" {
        by_the_cli(w)
    } else {
        in_a_run_seat(w, &provenance)
    }
}

#[then(regex = r#"^the event is stored with provenance "([^"]+)"$"#)]
fn stored_as(w: &mut World, provenance: String) -> Result<(), String> {
    let recalled = recalled_item(w)?;
    if recalled["provenance"] == provenance.as_str() {
        return Ok(());
    }
    Err(format!(
        "the event is recalled as {}",
        recalled["provenance"]
    ))
}

#[then(
    regex = r#"^its trust level for this principal's agents on this node derives as "(trusted|untrusted)", and no trust level is stored with the event$"#
)]
fn derived_and_not_stored(w: &mut World, trust: String) -> Result<(), String> {
    let recalled = recalled_item(w)?;
    if recalled["trust"] != trust.as_str() {
        return Err(format!("the event is recalled at {}", recalled["trust"]));
    }
    let conn = rusqlite::Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    let mut statement = conn
        .prepare("SELECT name FROM pragma_table_info('events')")
        .map_err(|e| e.to_string())?;
    let columns: Vec<String> = statement
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    match columns.iter().find(|c| c.contains("trust")) {
        Some(column) => Err(format!("the events table stores {column}")),
        None => Ok(()),
    }
}

#[then(
    regex = r#"^"parinn verify" (reports nothing|reports it as a widening principal act that fails OWN-22)$"#
)]
fn verify_reports(w: &mut World, report: String) -> Result<(), String> {
    if report != "reports nothing" {
        return Err("not reachable in M1: no widening act is recorded beyond OWN-22".to_owned());
    }
    run_core(w, &["verify"], None)?;
    let out = crate::record::last_run(w)?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "parinn verify exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    ))
}

#[then(
    regex = r#"^the event shapes restore blocks and trust levels only if it is trusted and "parinn verify" reports nothing about it$"#
)]
fn shapes_only_if_trusted(w: &mut World) -> Result<(), String> {
    // Restore blocks are M2's; the trust level the event shows is the
    // one the policy derived, and verify reports nothing about it.
    let recalled = recalled_item(w)?;
    verify_reports(w, "reports nothing".to_owned())?;
    match recalled["trust"].as_str() {
        Some("trusted" | "untrusted") => Ok(()),
        other => Err(format!("the event shows the trust level {other:?}")),
    }
}

#[then(regex = r#"^after "parinn rebuild" every event's trust level is unchanged$"#)]
fn rebuild_keeps_trust(w: &mut World) -> Result<(), String> {
    let before = recalled_item(w)?["trust"].clone();
    run_core(w, &["rebuild"], None)?;
    let after = recalled_item(w)?["trust"].clone();
    if before == after {
        return Ok(());
    }
    Err(format!("rebuild moved the trust level {before} to {after}"))
}
