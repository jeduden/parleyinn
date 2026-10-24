//! The steps of SRS §5.1c, principal acts (OWN). `@OWN-11`, `@OWN-12`
//! and `@OWN-22` stay `@pending`: they land in part in M1, their other
//! steps needing pins, rooms, presence proofs and recorded sandbox
//! state. The steps M1 reaches are bound here: a widening act at the
//! CLI's terminal accepted or refused under OWN-22, and the counters'
//! acknowledgement recorded as a neutral act.
//!
//! Before PRV-10 ships, the principal surface is the CLI at a terminal
//! and the principal is this node's own: the scenarios' "alice".

use cucumber::{given, then, when};
use serde_json::Value;

use crate::World;
use crate::administration::{agent_runs, configure};
use crate::record::{home, last_run, run_core, run_core_at_terminal};

/// The content of every principal act the scenario's store holds.
fn recorded(w: &World) -> Result<Vec<Value>, String> {
    let home = home(w)?;
    let store = store::Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = administration::node::Node::find(&home, &store).map_err(|e| e.to_string())?;
    let acts = administration::act::acts(&store, &node).map_err(|e| e.to_string())?;
    Ok(acts.into_iter().map(|a| a.content).collect())
}

#[given(regex = r#"^a principal surface "authenticated"$"#)]
fn authenticated_surface(_w: &mut World) {
    // The CLI at a terminal, which each act step starts the core at.
}

#[given(regex = r#"^OWN-22 "(permits|does not permit)" widening acts$"#)]
fn own_22(w: &mut World, state: String) -> Result<(), String> {
    if state == "does not permit" {
        // A run with no recorded sandbox state leaves every residual
        // risk open, and no risk acceptance names them.
        agent_runs(w)?;
    }
    Ok(())
}

#[given(regex = r"^a run with no recorded sandbox state$")]
fn unsandboxed_run(w: &mut World) -> Result<(), String> {
    agent_runs(w)
}

#[when(regex = r#"^"alice" writes a "widening" principal act there$"#)]
fn writes_widening_there(w: &mut World) -> Result<(), String> {
    writes_widening(w)
}

#[when(regex = r#"^"alice" writes a widening principal act$"#)]
fn writes_widening(w: &mut World) -> Result<(), String> {
    configure(w, "recall.max_k = 10\n")?;
    run_core_at_terminal(w, &["configuration", "accept"])
}

#[then(regex = r#"^the act is "(accepted|refused)"$"#)]
fn act_is(w: &mut World, outcome: String) -> Result<(), String> {
    let recorded = recorded(w)?
        .iter()
        .any(|a| a["act"] == "configuration_accept" && a["class"] == "widening");
    let out = last_run(w)?;
    let accepted = out.status.success();
    if accepted == (outcome == "accepted") && recorded == accepted {
        return Ok(());
    }
    Err(format!(
        "the act exited {} and was {}recorded, wanting it {outcome}: {}",
        out.status,
        if recorded { "" } else { "not " },
        String::from_utf8_lossy(&out.stdout).trim()
    ))
}

#[then(regex = r"^the run counts as unsandboxed and is still recorded$")]
fn counts_as_unsandboxed(w: &mut World) -> Result<(), String> {
    run_core(w, &["risk-acceptance", "show", "--json"], None)?;
    let out = last_run(w)?;
    let shown: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let open = shown["open"].as_array().map_or(0, Vec::len);
    let runs = shown["runs"].as_array().map_or(0, Vec::len);
    if open == 8 && runs == 1 {
        return Ok(());
    }
    Err(format!(
        "risk-acceptance show gave {shown}, wanting one run leaving all eight open"
    ))
}

#[then(
    regex = r"^the acknowledgement is recorded as a neutral principal act with nothing more asked$"
)]
fn acknowledgement_recorded(w: &mut World) -> Result<(), String> {
    let out = last_run(w)?;
    if !out.status.success() {
        return Err(format!(
            "parinn counter ack exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stdout).trim()
        ));
    }
    let acts = recorded(w)?;
    if acts.iter().any(|a| {
        a["act"] == "counter_ack" && a["class"] == "neutral" && a["principal_surface"] == "cli"
    }) {
        return Ok(());
    }
    Err(format!("no neutral counter acknowledgement among {acts:?}"))
}

#[given(regex = r#"^an agent's run started on "alice"'s node$"#)]
fn run_on_alices_node(w: &mut World) -> Result<(), String> {
    agent_runs(w)
}

#[then(
    regex = r#"^the run's one principal is "alice", the principal of the node that records it$"#
)]
fn the_nodes_principal_is_the_runs(w: &mut World) -> Result<(), String> {
    // Before PRV-10 no principal key exists: a run's principal is the
    // node's, whose one device key certified every seat that records the
    // run (OWN-01, PRV-11). No seat of another key holds an event.
    // This narrowing is temporary: once principal keys exist (PRV-10), the
    // step names the run's principal by its key, not by the one device key
    // that certified every seat.
    run_core(w, &["verify", "--json"], None)?;
    let out = last_run(w)?;
    let report: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let seats = report["seats"]
        .as_array()
        .ok_or("verify reported no seats")?;
    let keys: std::collections::BTreeSet<_> = seats
        .iter()
        .map(|s| s["certificate"]["certifying_key"].to_string())
        .collect();
    if seats.len() < 2 || keys.len() != 1 {
        return Err(format!(
            "the run's seats are not all certified by the node's one device key: {keys:?}"
        ));
    }
    let rooms: std::collections::BTreeSet<_> =
        seats.iter().map(|s| s["room"].to_string()).collect();
    if rooms.len() != 1 {
        return Err(format!(
            "the run's seats lie in more than one room: {rooms:?}"
        ));
    }
    Ok(())
}
