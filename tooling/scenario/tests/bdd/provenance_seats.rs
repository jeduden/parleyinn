//! The steps of SRS §5.6, provenance (PRV), that phase 5 of plan
//! 2609292012 carries. `@PRV-11` is bound: the node's device key
//! certifies each seat key with a seat certificate naming its seat kind
//! and whether it is an ingest seat, and `parinn verify --json` reports
//! each seat's kind from that certificate alone. The first steps of
//! PRV-12 are bound too: the genesis log's `minted`.
//!
//! A run seat's key lives only in its harness session's MCP server
//! (SEC-10), which phase 10 of plan 2609292012 builds; until then the
//! step that starts a run's run seat certifies one through the product
//! crates, as that server will, its key held in the step's memory.

use ::record::canonical;
use ::record::commitment::CommitmentKey;
use ::record::event::{EventKind, Provenance};
use ::record::id::{RoomId, WriterId};
use administration::node::Node;
use cucumber::{given, then, when};
use key::certificate::SeatCertificate;
use key::random::OsEntropy;
use key::signing::{PublicKey, SeatKey};
use serde_json::{Value, json};
use store::{NewEvent, Store};

use crate::World;
use crate::record::{first_start, home, last_run};

/// What the provenance steps track: the seats a scenario started.
#[derive(Debug, Default)]
pub struct ProvenanceSteps {
    device_key: Option<PublicKey>,
    run_key: Option<PublicKey>,
    room: Option<RoomId>,
    run_writer: Option<WriterId>,
    device_writer: Option<WriterId>,
}

fn prv(w: &mut World) -> &mut ProvenanceSteps {
    w.section::<ProvenanceSteps>()
}

#[given(regex = r"^a node whose device key no principal key has certified$")]
fn uncertified_node(w: &mut World) -> Result<(), String> {
    // No principal key exists before PRV-10 ships, so none certified it:
    // the node's first start is all there is.
    let home = first_start(w)?;
    let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = Node::find(&home, &store).map_err(|e| e.to_string())?;
    prv(w).device_key = Some(node.device().public());
    Ok(())
}

#[when(
    regex = r"^a run's run seat starts in a room and the node's device seat starts in the same room$"
)]
fn seats_start(w: &mut World) -> Result<(), String> {
    let home = home(w)?;
    let mut store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    // The device seat started in the personal room at the node's first
    // start; the run's run seat starts there now, its key in memory.
    let node = Node::open(&home, &mut store, &mut OsEntropy).map_err(|e| e.to_string())?;
    let key = SeatKey::mint(&mut OsEntropy).map_err(|e| e.to_string())?;
    let seat = node
        .certify_run_seat(&mut store, &key, &mut OsEntropy)
        .map_err(|e| e.to_string())?;
    // An event of the run seat claiming another seat kind, which no
    // report may read a seat's kind from.
    let claim = NewEvent {
        kind: EventKind::UnparsedLine,
        provenance: Provenance::Unparsed,
        content: json!({"line": 1, "text": "{\"seat_kind\":\"device\",\"ingest_seat\":true}"}),
        commitment_key: CommitmentKey::from_bytes(
            key::random::draw(&mut OsEntropy).map_err(|e| e.to_string())?,
        ),
        transcript_timestamp: None,
    };
    store
        .append(seat.writer, &[claim])
        .map_err(|e| e.to_string())?;
    let steps = prv(w);
    steps.run_key = Some(key.public());
    steps.room = Some(node.personal_room());
    steps.run_writer = Some(seat.writer);
    steps.device_writer = Some(node.device_writer());
    Ok(())
}

/// The seats the last `parinn verify --json` reported.
fn reported(w: &mut World) -> Result<Vec<Value>, String> {
    let out = last_run(w)?;
    let v: Value = serde_json::from_slice(&out.stdout).map_err(|e| {
        format!(
            "parinn verify --json printed no JSON ({e}): {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })?;
    v["seats"]
        .as_array()
        .cloned()
        .ok_or_else(|| format!("parinn verify --json reported no seats: {v}"))
}

/// The seat of `writer` the last `parinn verify --json` reported.
fn reported_seat(w: &mut World, writer: Option<WriterId>) -> Result<Value, String> {
    let writer = writer.ok_or("no step started that seat")?.to_string();
    reported(w)?
        .into_iter()
        .find(|s| s["writer"] == writer.as_str())
        .ok_or_else(|| format!("parinn verify --json reported no seat of writer {writer}"))
}

/// Checks that `seat` carries a seat certificate the node's device key
/// made over `key`, scoped to `room`, naming `kind`, and countersigned.
fn certified(
    seat: &Value,
    device: PublicKey,
    key: Option<PublicKey>,
    room: Option<RoomId>,
    kind: &str,
) -> Result<(), String> {
    let cert = SeatCertificate::from_value(&seat["certificate"]).map_err(|e| e.to_string())?;
    cert.check().map_err(|e| e.to_string())?;
    let room = room.ok_or("no step started a seat")?;
    if cert.certifying_key != device || cert.room != room || cert.seat_kind.as_str() != kind {
        return Err(format!(
            "the seat certificate is not the device key's for a {kind} seat in {room}: {seat}"
        ));
    }
    if key.is_some_and(|k| k != cert.seat_key) {
        return Err(format!(
            "the seat certificate names another seat key: {seat}"
        ));
    }
    if seat["seat_kind"] != kind || seat["room"] != room.to_string().as_str() {
        return Err(format!("the report names another kind or room: {seat}"));
    }
    Ok(())
}

#[then(
    regex = r#"^the device key has certified the run seat's key with a seat certificate scoped to that room and naming the seat kind "run"$"#
)]
fn run_seat_certified(w: &mut World) -> Result<(), String> {
    let (device, key, room, writer) = {
        let s = prv(w);
        (
            s.device_key.ok_or("no device key")?,
            s.run_key,
            s.room,
            s.run_writer,
        )
    };
    let seat = reported_seat(w, writer)?;
    certified(&seat, device, key, room, "run")
}

#[then(
    regex = r#"^the device key has certified the device seat's key with a seat certificate scoped to that room and naming the seat kind "device"$"#
)]
fn device_seat_certified(w: &mut World) -> Result<(), String> {
    let (device, room, writer) = {
        let s = prv(w);
        (
            s.device_key.ok_or("no device key")?,
            s.room,
            s.device_writer,
        )
    };
    let seat = reported_seat(w, writer)?;
    certified(&seat, device, None, room, "device")
}

#[then(
    regex = r"^each seat certificate names whether its seat is an ingest seat, and neither the run seat's nor the device seat's names it one$"
)]
fn no_ingest_seat(w: &mut World) -> Result<(), String> {
    let writers = {
        let s = prv(w);
        [s.run_writer, s.device_writer]
    };
    for writer in writers {
        let seat = reported_seat(w, writer)?;
        if seat["certificate"]["ingest_seat"] != false || seat["ingest_seat"] != false {
            return Err(format!(
                "the seat's certificate does not name it no ingest seat: {seat}"
            ));
        }
    }
    for seat in reported(w)? {
        if !seat["certificate"]["ingest_seat"].is_boolean() {
            return Err(format!("a seat certificate names no ingest flag: {seat}"));
        }
    }
    Ok(())
}

#[then(
    regex = r"^the output reports, for each seat, its seat certificate and the seat kind it names, taken only from that seat certificate, never from a field in its events$"
)]
fn kind_from_certificate(w: &mut World) -> Result<(), String> {
    for seat in reported(w)? {
        let cert = SeatCertificate::from_value(&seat["certificate"]).map_err(|e| e.to_string())?;
        let canonical = canonical::encode(&seat["certificate"]).map_err(|e| e.to_string())?;
        if seat["seat_kind"] != cert.seat_kind.as_str()
            || seat["ingest_seat"] != cert.ingest_seat
            || seat["seat"] != cert.seat_id().to_string().as_str()
            || canonical != canonical::encode(&cert.to_value()).map_err(|e| e.to_string())?
        {
            return Err(format!(
                "a seat is reported otherwise than its certificate names it: {seat}"
            ));
        }
    }
    // The run seat's own event claims the device kind and an ingest
    // seat; the report keeps to the certificate.
    let writer = prv(w).run_writer;
    let run = reported_seat(w, writer)?;
    if run["seat_kind"] != "run" || run["ingest_seat"] != false {
        return Err(format!("the run seat is reported from its events: {run}"));
    }
    Ok(())
}

#[then(regex = r"^a recorded I2 security review of this requirement exists before it ships$")]
fn i2_review(_w: &mut World) -> Result<(), String> {
    // ENG-29: an accepted ADR records the named security reviewer's
    // approval; ADR-2610112004 carries PRV-11's.
    let dir = testkit::repo_root().join("docs/adr");
    let adrs = adr::load(&dir).map_err(|e| e.to_string())?;
    let review = adrs.iter().find(|a| {
        let text: String = a.sections.values().cloned().collect::<Vec<_>>().join("\n");
        a.status == adr::ACCEPTED
            && text.contains("PRV-11's I2")
            && text.contains("security review")
            && text.contains("approved this record as security")
    });
    review
        .map(|_| ())
        .ok_or_else(|| "no accepted ADR records PRV-11's I2 security review".to_owned())
}

#[given(regex = r"^an isolated ParleyInn home with no authenticator$")]
fn home_without_authenticator(w: &mut World) -> Result<(), String> {
    // No authenticator can be enrolled before OWN-11's ship; the home is
    // as fresh as any isolated one.
    if w.parinn_home.starts_with(&w.home) && !w.parinn_home.exists() {
        Ok(())
    } else {
        Err(format!(
            "the ParleyInn home {} is not a fresh one inside the scenario's home",
            w.parinn_home.display()
        ))
    }
}

#[when(regex = r"^ParleyInn starts for the first time$")]
fn starts_first(w: &mut World) -> Result<(), String> {
    crate::record_keys::ingest_one_line(w, "first")
}

#[then(
    regex = r#"^a "minted" key event, the first event of the device seat that created the personal room, starts the genesis log, committing to the digest of its recovery key's public key$"#
)]
fn genesis_minted(w: &mut World) -> Result<(), String> {
    let home = home(w)?;
    let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = Node::find(&home, &store).map_err(|e| e.to_string())?;
    let log = store.log(node.device_writer()).map_err(|e| e.to_string())?;
    let minted = log
        .first()
        .and_then(|s| s.content.as_ref())
        .and_then(|c| key::key_event::KeyEvent::from_value(c).ok())
        .ok_or("the device seat begins with no key event")?;
    let created = log.get(1).is_some_and(|s| {
        s.link.header.kind == EventKind::CreateRoom
            && s.content
                .as_ref()
                .is_some_and(|c| c["room"] == node.personal_room().to_string().as_str())
    });
    let recovery = key::signing::RecoveryKey::fetch(
        home.key_store().as_ref(),
        &key::key_store::Entry::recovery(),
    )
    .map_err(|e| e.to_string())?;
    if minted.check_minted(&node.device().public()).is_err()
        || !created
        || minted.recovery != recovery.public().digest()
    {
        return Err("the genesis device seat does not begin with the device key's minted, committing to its recovery key".to_owned());
    }
    Ok(())
}
