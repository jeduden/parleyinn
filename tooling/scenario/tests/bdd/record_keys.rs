//! The steps of REC-24 that phase 5 of plan 2609292012 carries: a home
//! whose node identity changed mints a new device key and a new device
//! seat in the same personal room before its next append, audits and
//! counts the change, and appends nothing under an old key.
//!
//! The scenario stays pending. Its append is the `SessionStart` hook
//! handler's (phase 9), and its quarantine (phase 14), origins
//! (phase 11) and confirm (M3, PRV-12) come later; those steps are
//! unbound. Every command a step starts reads its node identity from the
//! scenario's own `machine-id` file, through the development build's
//! seam, never the machine's.

use std::fs;
use std::path::PathBuf;

use administration::home::Home;
use administration::node::Node;
use administration::node_identity::IdentitySource;
use administration::role;
use cucumber::{given, then};
use key::certificate::{SeatCertificate, SeatKind};
use key::key_event::{KeyEvent, NodeIdentityChange};
use key::signing::PublicKey;
use record::event::EventKind;
use record::id::{Address, RoomId, Seq, WriterId};
use serde_json::Value;
use store::{Store, Stored};

use crate::World;
use crate::administration::configure;
use crate::record::{last_run, run_core, run_core_at_terminal, set_env};

/// The file the scenario's node identity is read from, in its home.
const IDENTITY: &str = "machine-id";

/// What the node identity steps track: the node before the change.
#[derive(Debug, Default)]
pub struct KeySteps {
    old_device: Option<PublicKey>,
    old_device_writer: Option<WriterId>,
    room: Option<RoomId>,
    /// Each writer the record held just before the change, with its
    /// length.
    lengths: Vec<(WriterId, usize)>,
}

fn keys(w: &mut World) -> &mut KeySteps {
    w.section::<KeySteps>()
}

/// The scenario's home, its node identity read as the core reads it.
fn home(w: &World) -> Result<Home, String> {
    Ok(crate::record::home(w)?.with_identity(IdentitySource::Files(vec![w.home.join(IDENTITY)])))
}

/// The node the scenario's home holds now, and its store.
fn node(w: &World) -> Result<(Node, Store), String> {
    let home = home(w)?;
    let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = Node::find(&home, &store).map_err(|e| e.to_string())?;
    Ok((node, store))
}

/// Starts the core on a transcript of one line, as an agent's run
/// leaves it under the default transcript root, and fails when it does.
pub fn ingest_one_line(w: &mut World, text: &str) -> Result<(), String> {
    let dir = w.home.join(".claude/projects/keys");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{text}.jsonl"));
    fs::write(
        &path,
        format!("{{\"type\":\"user\",\"text\":\"{text}\"}}\n"),
    )
    .map_err(|e| e.to_string())?;
    let path = path.display().to_string();
    run_core(w, &["ingest", "--path", &path], None)?;
    succeeded(w).map(drop)
}

/// The last command's stdout, when it exited 0.
fn succeeded(w: &mut World) -> Result<String, String> {
    let out = last_run(w)?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(format!(
            "the command exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Each device seat's writer and certificate.
fn device_seats(store: &Store) -> Result<Vec<(WriterId, SeatCertificate)>, String> {
    let mut out = Vec::new();
    for seat in store.seats().map_err(|e| e.to_string())? {
        let cert = SeatCertificate::from_value(&seat.certificate).map_err(|e| e.to_string())?;
        if cert.seat_kind == SeatKind::Device {
            out.push((seat.writer, cert));
        }
    }
    Ok(out)
}

#[given(
    regex = r"^a device key and a seat key bound to the node identity read, from outside the home, on the node the home was created on$"
)]
fn bound_keys(w: &mut World) -> Result<(), String> {
    let identity: PathBuf = w.home.join(IDENTITY);
    fs::write(&identity, "the-first-machine\n").map_err(|e| e.to_string())?;
    let identity = identity.display().to_string();
    set_env(w, "PARINN_TEST_NODE_IDENTITY", &identity);
    ingest_one_line(w, "before")?;
    let (node, _) = node(w)?;
    let steps = keys(w);
    steps.old_device = Some(node.device().public());
    steps.old_device_writer = Some(node.device_writer());
    steps.room = Some(node.personal_room());
    Ok(())
}

/// Keeps the length of each writer's log as the change finds it.
fn before_the_change(w: &mut World) -> Result<(), String> {
    let (_, store) = node(w)?;
    let mut lengths = Vec::new();
    for writer in store.writers().map_err(|e| e.to_string())? {
        let length = store.log(writer).map_err(|e| e.to_string())?.len();
        lengths.push((writer, length));
    }
    keys(w).lengths = lengths;
    Ok(())
}

#[given(
    regex = r#"^a configuration setting "node\.deployment_mode" to "interactive", accepted by "parinn configuration accept"$"#
)]
fn interactive_accepted(w: &mut World) -> Result<(), String> {
    configure(w, "[node]\ndeployment_mode = \"interactive\"\n")?;
    // The run the home recorded leaves OWN-22's residual risks open, so
    // the person accepts them first, as a widening act needs.
    run_core_at_terminal(w, &["risk-acceptance", "record"])?;
    succeeded(w)?;
    run_core_at_terminal(w, &["configuration", "accept"])?;
    succeeded(w)?;
    run_core(w, &["status", "--json"], None)?;
    let status: Value = serde_json::from_str(&succeeded(w)?).map_err(|e| e.to_string())?;
    if status["deployment_mode"] == "interactive" {
        Ok(())
    } else {
        Err(format!("the acceptance did not take effect: {status}"))
    }
}

#[given(
    regex = r"^(the home is part of a cloned runner image on another machine|the home is a copied volume mounted on another machine|the home is recovered from a snapshot on another machine)$"
)]
fn on_another_machine(w: &mut World, _how: String) -> Result<(), String> {
    before_the_change(w)?;
    fs::write(w.home.join(IDENTITY), "another-machine\n").map_err(|e| e.to_string())
}

#[given(regex = r"^the node identity value is unavailable on every read$")]
fn identity_unavailable(w: &mut World) -> Result<(), String> {
    before_the_change(w)?;
    fs::remove_file(w.home.join(IDENTITY)).map_err(|e| e.to_string())
}

/// The device seat the change started, with its log.
fn new_device_seat(w: &mut World) -> Result<(WriterId, SeatCertificate, Vec<Stored>), String> {
    let old = keys(w)
        .old_device_writer
        .ok_or("no step recorded the old node")?;
    let (_, store) = node(w)?;
    let seats = device_seats(&store)?;
    let fresh: Vec<_> = seats
        .into_iter()
        .filter(|(writer, _)| *writer != old)
        .collect();
    let [(writer, cert)] = <[_; 1]>::try_from(fresh)
        .map_err(|f| format!("want one new device seat, found {}", f.len()))?;
    let log = store.log(writer).map_err(|e| e.to_string())?;
    Ok((writer, cert, log))
}

#[then(
    regex = r"^ParleyInn mints a new device key and a new seat key before that append, the new seat key starting a new seat and writer that names the old seat$"
)]
fn new_keys(w: &mut World) -> Result<(), String> {
    let (old_key, old_writer) = {
        let s = keys(w);
        (s.old_device, s.old_device_writer)
    };
    let (node, _) = node(w)?;
    if Some(node.device().public()) == old_key {
        return Err("the device key is the old one".to_owned());
    }
    let (writer, cert, log) = new_device_seat(w)?;
    let change = log
        .first()
        .and_then(|s| s.content.as_ref())
        .and_then(|c| NodeIdentityChange::from_value(c).ok())
        .ok_or("the new device seat does not begin with a node identity change event")?;
    if Some(change.chain_head.address.writer) != old_writer {
        return Err("the new device seat names no old seat".to_owned());
    }
    if cert.certifying_key != node.device().public() || writer != node.device_writer() {
        return Err("the new device key did not certify the new seat".to_owned());
    }
    Ok(())
}

#[then(
    regex = r"^the new seat inherits no add or role of the old seat, and, outside the personal room, joins a room only as any seat does, while a personal-room seat, a device seat's included, is a member from its first event$"
)]
fn inherits_nothing(w: &mut World) -> Result<(), String> {
    let room = keys(w).room.ok_or("no step recorded the old node")?;
    let (node, _) = node(w)?;
    let (_, cert, log) = new_device_seat(w)?;
    // Its first event is its own, in the personal room; its standing
    // follows from its own certificate and the new key alone.
    let own = [node.device().public()];
    let standing = role::seat_role(&cert, &own);
    let first = log.first().map(|s| s.link.header.kind);
    if cert.room != room || standing.is_some() || first != Some(EventKind::NodeIdentityChange) {
        return Err(format!(
            "the new device seat is not the owner's seat of the personal room from its first event: {cert:?}"
        ));
    }
    Ok(())
}

#[then(regex = r"^the personal room keeps its room id$")]
fn same_room(w: &mut World) -> Result<(), String> {
    let room = keys(w).room;
    let (node, _) = node(w)?;
    if Some(node.personal_room()) == room {
        Ok(())
    } else {
        Err(format!("the personal room is now {}", node.personal_room()))
    }
}

#[then(
    regex = r#"^that configuration acceptance takes no effect on the node: the deployment mode stays "automation" until the person accepts the configuration there$"#
)]
fn acceptance_left_behind(w: &mut World) -> Result<(), String> {
    run_core(w, &["status", "--json"], None)?;
    let status: Value = serde_json::from_str(&succeeded(w)?).map_err(|e| e.to_string())?;
    let awaiting = status["awaiting_acceptance"]
        .as_array()
        .is_some_and(|a| a.iter().any(|k| k == "node.deployment_mode"));
    if status["deployment_mode"] == "automation" && awaiting {
        Ok(())
    } else {
        Err(format!("the old acceptance still counts: {status}"))
    }
}

#[then(regex = r"^an audit entry records the node identity change$")]
fn audited(w: &mut World) -> Result<(), String> {
    run_core(w, &["status", "--json"], None)?;
    let status: Value = serde_json::from_str(&succeeded(w)?).map_err(|e| e.to_string())?;
    if status["counters"]["node_identity_changed"]["total"] == 1 {
        Ok(())
    } else {
        Err(format!("no node identity change is counted: {status}"))
    }
}

#[then(
    regex = r"^a structural node identity change event in the new device seat in the personal room carries only the fingerprints of the replaced and new device key or token key, followed by the new key's inception, which starts a new key event log$"
)]
fn change_event(w: &mut World) -> Result<(), String> {
    let old = keys(w).old_device.ok_or("no step recorded the old node")?;
    let (node, _) = node(w)?;
    let (_, _, log) = new_device_seat(w)?;
    let content = log
        .first()
        .and_then(|s| s.content.clone())
        .ok_or("the new device seat holds no event")?;
    // PRV-12 adds the chain head of the key event log it leaves to the
    // fingerprints REC-24 names; nothing else.
    let fields: Vec<&String> = content
        .as_object()
        .map(|o| o.keys().collect())
        .unwrap_or_default();
    let change = NodeIdentityChange::from_value(&content).map_err(|e| e.to_string())?;
    let inception = log
        .get(1)
        .and_then(|s| s.content.as_ref())
        .and_then(|c| KeyEvent::from_value(c).ok());
    let starts = inception
        .is_some_and(|e| e.check_minted(&node.device().public()).is_ok() && e.follows.is_none());
    if fields != ["chain_head", "key", "replaces"]
        || change.replaces != old.fingerprint()
        || change.key != node.device().public().fingerprint()
        || !starts
    {
        return Err(format!(
            "the node identity change event is not as REC-24 names it: {content}"
        ));
    }
    Ok(())
}

#[then(regex = r"^no event is appended under the old device key or the old seat key$")]
fn nothing_under_old_keys(w: &mut World) -> Result<(), String> {
    let lengths = keys(w).lengths.clone();
    let (_, store) = node(w)?;
    for (writer, length) in lengths {
        let now = store.log(writer).map_err(|e| e.to_string())?.len();
        if now != length {
            return Err(format!(
                "writer {writer} grew from {length} to {now} events"
            ));
        }
    }
    Ok(())
}

#[then(
    regex = r"^an unavailable node identity value read again within the hook budgets and found unchanged counts as no change$"
)]
fn unchanged_is_no_change(w: &mut World) -> Result<(), String> {
    let before = {
        let (_, store) = node(w)?;
        device_seats(&store)?.len()
    };
    ingest_one_line(w, "again")?;
    let (node, store) = node(w)?;
    let after = device_seats(&store)?.len();
    let first = store
        .event(Address::new(node.device_writer(), Seq::FIRST))
        .map_err(|e| e.to_string())?;
    if before == after && first.is_some() {
        Ok(())
    } else {
        Err(format!(
            "a start with the identity unchanged found a change: {before} then {after} device seats"
        ))
    }
}
