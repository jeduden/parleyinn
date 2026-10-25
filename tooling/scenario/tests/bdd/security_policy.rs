//! The steps of SEC-22 that phase 14 of plan 2609292012 carries: managed
//! policy constrains each boundary and power on its host. A fixed
//! deployment mode stands whatever the person's configuration says; an
//! authenticator required for widening acts refuses them, since presence
//! proofs come later; forbidden risk acceptance refuses the acceptance;
//! and a retention policy has the node purge what its window has passed,
//! leaving a tombstone that names the policy, audited and counted.
//!
//! The retention row ages events by the node's clock, which a
//! development build lets a step set (`PARINN_TEST_CLOCK`): the old
//! notes are ingested at one time, a time mark follows them, and the
//! next ingest, 31 days on, finds them past the window (REC-15).

use std::fs;

use ::record::event::{EventKind, Provenance};
use audit::log::Log;
use cucumber::{given, then, when};
use serde_json::Value;
use store::Store;

use crate::World;
use crate::administration::{agent_runs, configure};
use crate::provenance_trust::managed_policy;
use crate::record::{home, last_run, run_core, run_core_at_terminal, set_env};
use crate::record_transcript::ingest;
use crate::security_redaction::name_transcript;

/// When the old notes are ingested, in seconds since the Unix epoch.
const THEN: u64 = 1_800_000_000;

/// Seconds in a day.
const DAY: u64 = 86_400;

/// What the policy steps track: the personal room, once named.
#[derive(Debug, Default)]
pub struct PolicySteps {
    room: Option<String>,
}

fn steps(w: &mut World) -> &mut PolicySteps {
    w.section::<PolicySteps>()
}

#[given(regex = r"^managed policy sets a fixed deployment mode$")]
fn fixed_mode(w: &mut World) -> Result<(), String> {
    managed_policy(w, "[node]\ndeployment_mode = \"automation\"\n")
}

#[given(regex = r"^managed policy sets an authenticator required for widening acts$")]
fn authenticator_required(w: &mut World) -> Result<(), String> {
    managed_policy(w, "[authenticator]\nrequired = true\n")
}

#[given(regex = r"^managed policy sets risk acceptance forbidden$")]
fn risk_acceptance_forbidden(w: &mut World) -> Result<(), String> {
    managed_policy(w, "[risk_acceptance]\nforbidden = true\n")
}

/// The provenance classes a transcript line can be recorded as, which
/// the retention policy of row 4 names.
const CLASSES: [&str; 9] = [
    "user",
    "assistant",
    "tool_call",
    "tool_result:Bash",
    "web",
    "file",
    "subagent_result",
    "harness_text",
    "unparsed",
];

#[given(regex = r#"^managed policy sets a 30-day retention policy for room "p"$"#)]
fn retention_for_the_personal_room(w: &mut World) -> Result<(), String> {
    // The room's id derives from the node's first key: the first command
    // that starts the node names it.
    run_core(w, &["quarantine", "list"], None)?;
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let node = administration::node::Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let room = node.personal_room().to_string();
    let keys: String = CLASSES.iter().map(|c| format!("{c:?} = 30\n")).collect();
    managed_policy(w, &format!("[retention_policy.\"{room}\"]\n{keys}"))?;
    steps(w).room = Some(room);

    Ok(())
}

#[when(regex = r"^the person changes the deployment mode$")]
fn person_changes_mode(w: &mut World) -> Result<(), String> {
    configure(w, "node.deployment_mode = \"interactive\"\n")?;
    run_core_at_terminal(w, &["configuration", "accept"])
}

#[when(regex = r"^the principal confirms a widening act without the authenticator$")]
fn widening_without_authenticator(w: &mut World) -> Result<(), String> {
    configure(w, "recall.max_k = 10\n")?;
    run_core_at_terminal(w, &["configuration", "accept"])
}

#[when(regex = r"^the principal accepts a residual risk$")]
fn principal_accepts_a_risk(w: &mut World) -> Result<(), String> {
    // A run leaves every residual risk open to accept.
    agent_runs(w)?;
    run_core_at_terminal(w, &["risk-acceptance", "record"])
}

/// Writes the run `name` of `lines` and ingests it with the node's clock
/// at `time`.
fn ingest_at(w: &mut World, name: &str, lines: &[&str], time: u64) -> Result<(), String> {
    let dir = w.home.join(".claude").join("projects").join("-w-app");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{name}.jsonl"));
    fs::write(
        &path,
        lines.iter().map(|l| format!("{l}\n")).collect::<String>(),
    )
    .map_err(|e| e.to_string())?;
    name_transcript(w, name, path.clone());
    set_env(w, "PARINN_TEST_CLOCK", &time.to_string());

    ingest(w, &path)
}

#[when(regex = r#"^an event of "p" ages past 30 days$"#)]
fn an_event_ages(w: &mut World) -> Result<(), String> {
    ingest_at(w, "old-notes", &["an old note", "another old note"], THEN)?;
    // The node's next work, 31 days on, finds them past the window.
    ingest_at(w, "fresh-note", &["a fresh note"], THEN + 31 * DAY)
}

/// Fails unless the last command failed and said `words`.
fn refused_saying(w: &mut World, what: &str, words: &str) -> Result<(), String> {
    let out = last_run(w)?;
    // A command at a terminal has its standard error in its output.
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if out.status.success() || !said.contains(words) {
        return Err(format!(
            "{what} exited {} and said {:?}, wanting a refusal naming {words:?}",
            out.status,
            said.trim()
        ));
    }

    Ok(())
}

/// The names of the principal acts the device seat recorded.
fn acts(w: &World) -> Result<Vec<String>, String> {
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let node = administration::node::Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let acts = administration::act::acts(&store, &node).map_err(|e| e.to_string())?;

    Ok(acts.into_iter().map(|a| a.name).collect())
}

#[then(regex = r"^the change is refused$")]
fn change_refused(w: &mut World) -> Result<(), String> {
    run_core(w, &["status"], None)?;
    let shown = String::from_utf8_lossy(&last_run(w)?.stdout).into_owned();
    if shown.lines().any(|l| l == "deployment mode: automation") {
        return Ok(());
    }

    Err(format!(
        "the deployment mode managed policy fixed does not stand:\n{shown}"
    ))
}

#[then(regex = r"^the act is refused$")]
fn act_refused(w: &mut World) -> Result<(), String> {
    refused_saying(
        w,
        "parinn configuration accept",
        "an authenticator is required",
    )?;
    if acts(w)?.iter().any(|a| a == "configuration_accept") {
        return Err("the widening act was recorded".to_owned());
    }

    Ok(())
}

#[then(regex = r"^the acceptance is refused$")]
fn acceptance_refused(w: &mut World) -> Result<(), String> {
    refused_saying(
        w,
        "parinn risk-acceptance record",
        "forbids risk acceptance",
    )?;
    if acts(w)?.iter().any(|a| a == "risk_acceptance_record") {
        return Err("the risk acceptance was recorded".to_owned());
    }

    Ok(())
}

#[then(
    regex = r"^the node purges it, leaving a tombstone recorded naming the policy, audited and counted$"
)]
fn node_purged_it(w: &mut World) -> Result<(), String> {
    let room = steps(w).room.clone().ok_or("no step named the room")?;
    let policy = format!("retention_policy.{room}.unparsed");
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let runs = store.ingested_runs().map_err(|e| e.to_string())?;
    let held = |name: &str| -> Result<Vec<bool>, String> {
        let run = runs
            .iter()
            .find(|r| r.harness_session == name)
            .ok_or_else(|| format!("{name} was not recorded"))?;
        Ok(store
            .log(run.writer)
            .map_err(|e| e.to_string())?
            .iter()
            .map(|s| s.content.is_some())
            .collect())
    };
    if held("old-notes")?.iter().any(|kept| *kept) {
        return Err("a note past its window keeps its content".to_owned());
    }
    if held("fresh-note")?.iter().any(|kept| !kept) {
        return Err("a note within its window lost its content".to_owned());
    }
    let node = administration::node::Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let tombstone = store
        .log(node.device_writer())
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|s| s.link.header.kind == EventKind::Tombstone)
        .ok_or("no tombstone was recorded")?;
    if tombstone.link.header.provenance != Provenance::Structural {
        return Err("the tombstone is no structural event".to_owned());
    }
    let content: Value = tombstone.content.ok_or("the tombstone holds no content")?;
    if content["reason"] != "retention" || content["policy"] != policy {
        return Err(format!(
            "the tombstone names {content}, wanting the policy {policy}"
        ));
    }
    let entries = Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    let purged = entries
        .iter()
        .find(|e| e.kind == "purge")
        .ok_or("no audit entry notes the purge")?;
    if !purged.detail.contains(&policy) || purged.counter.as_deref() != Some("retention_purged") {
        return Err(format!("the audit entry reads {:?}", purged.detail));
    }
    run_core(w, &["verify"], None)?;
    if !last_run(w)?.status.success() {
        return Err("parinn verify fails after the purge".to_owned());
    }

    Ok(())
}
