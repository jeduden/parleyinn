//! The steps of RCL-05 that phase 13 of plan 2609292012 carries: a
//! recall defaults to the calling agent's current run, through every
//! writer of its seats, and widening it to its rooms is explicit and
//! logged.
//!
//! The run is bound: a stand-in harness ([`crate::live`]) runs its MCP
//! server, which finds the run of each call by the tool-use id of a
//! line the run's transcript carries (PRV-14). No M1 command joins a
//! room, so the rooms the scenario names are written into the record as
//! a join would leave them: seats, the runs' ingest rows and events,
//! and the writer labels the device seat assigned.

use std::fs;
use std::io::Write;

use ::record::commitment::CommitmentKey;
use ::record::event::{EventKind, Provenance};
use ::record::id::{RoomId, SeatId, WriterId};
use audit::log::Log;
use cucumber::{given, then};
use serde_json::{Value, json};
use store::{IngestedRun, NewEvent, Store, StoredSeat};

use crate::World;
use crate::live::{lines_read, store, wait_until};
use crate::record::home;
use crate::record_live::{harness, make_run, named, serve};
use crate::record_search::{Reply, set_reply, user_line};
use crate::record_transcript::recorded_call;

/// The run the scenario's harness serves.
const RUN: &str = "main-run";

/// What the scope steps track.
#[derive(Debug, Default)]
pub struct ScopeSteps {
    /// Whether a bound run serves the calls.
    bound: bool,
    /// The writers of each world the step built.
    world: Option<World5>,
    /// How many calls got a tool-use id.
    calls: u64,
    /// The writers of the hits every call returned.
    seen: Vec<WriterId>,
    /// How many extended-recall entries the audit log held before.
    extended_before: usize,
}

/// The writers of RCL-05's world, by who they belong to.
#[derive(Debug, Clone)]
struct World5 {
    /// The run's writers: its personal-room seat and its seat in `L1`.
    own: [WriterId; 2],
    /// The writers of the other runs in `L1`, on two worktrees.
    others: [WriterId; 2],
    /// The writer of a run in `L2`.
    elsewhere: WriterId,
    l1: RoomId,
    l2: RoomId,
}

fn steps(w: &mut World) -> &mut ScopeSteps {
    w.section::<ScopeSteps>()
}

/// Whether a bound run serves the calls the steps make.
pub fn serves_a_run(w: &mut World) -> bool {
    steps(w).bound
}

/// A seat in `room` for `writer`, with `count` events saying `deploy`,
/// and, when `run` names one, the run ingest recorded in it: its harness
/// session, and the worktree its transcript lies under, a run's
/// transcript path being where the record keeps its worktree.
fn seat_with_events(
    store: &mut Store,
    writer: WriterId,
    room: RoomId,
    run: Option<(&str, &str)>,
    count: u64,
) -> Result<(), String> {
    store
        .add_seat(&StoredSeat {
            writer,
            seat: SeatId::derive(&room, writer.digest().as_bytes()),
            room,
            certificate: json!({}),
        })
        .map_err(|e| e.to_string())?;
    if let Some((run, worktree)) = run {
        store
            .add_run(&IngestedRun {
                writer,
                path: format!("{worktree}/{run}.jsonl"),
                harness_session: run.to_owned(),
                agent_id: None,
            })
            .map_err(|e| e.to_string())?;
    }
    let events: Vec<NewEvent> = (1..=count)
        .map(|n| NewEvent {
            kind: EventKind::TranscriptLine,
            provenance: Provenance::User,
            content: json!({"line": n, "text": format!("deploy step {n} of {writer}")}),
            commitment_key: CommitmentKey::from_bytes([n as u8; 32]),
            transcript_timestamp: None,
        })
        .collect();
    store.append(writer, &events).map_err(|e| e.to_string())?;
    Ok(())
}

/// Appends to the transcript `name` the lines `lines`, and waits until
/// ingest read them.
fn append_and_read(w: &mut World, name: &str, lines: &[Value]) -> Result<(), String> {
    let path = named(w, name)?;
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    for line in lines {
        writeln!(file, "{line}").map_err(|e| e.to_string())?;
    }
    drop(file);
    let held = fs::read_to_string(&path)
        .map_err(|e| e.to_string())?
        .lines()
        .count() as u64;
    wait_until(&format!("ingest to read {name}"), || {
        Ok(lines_read(w, &path)? >= held)
    })
}

#[given(
    regex = r#"^a run with seats in its personal room and in room "L1", whose writers this node holds beside those of other runs in "L1" on two worktrees and of room "L2" where the run has no seat$"#
)]
fn the_world(w: &mut World) -> Result<(), String> {
    make_run(w, RUN, 1, None)?;
    serve(w, RUN, "startup", &[])?;
    let lines = [
        user_line("deploy the app", 1)?,
        user_line("deploy again", 2)?,
    ];
    append_and_read(w, RUN, &lines)?;
    let mut store = store(w)?;
    let own = store
        .ingested_runs()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|r| r.path.ends_with("main-run.jsonl"))
        .ok_or("ingest recorded no run for main-run")?;
    let (l1, l2) = (
        RoomId::derive(&[11; 32], &[1; 16]),
        RoomId::derive(&[12; 32], &[1; 16]),
    );
    let writer = |n: u8| WriterId::derive(&[n; 32]);
    let world = World5 {
        own: [own.writer, writer(21)],
        others: [writer(22), writer(23)],
        elsewhere: writer(24),
        l1,
        l2,
    };
    // The other runs in L1 lie on two worktrees of their own.
    let own_run = (own.harness_session.as_str(), "/w/app");
    seat_with_events(&mut store, world.own[1], l1, Some(own_run), 3)?;
    seat_with_events(
        &mut store,
        world.others[0],
        l1,
        Some(("other-a", "/w/app-a")),
        6,
    )?;
    seat_with_events(
        &mut store,
        world.others[1],
        l1,
        Some(("other-b", "/w/app-b")),
        3,
    )?;
    seat_with_events(
        &mut store,
        world.elsewhere,
        l2,
        Some(("other-c", "/w/app")),
        3,
    )?;
    // The labels the device seat assigned: A1 is the run's own writer,
    // A2 an other run's in L1.
    let node = administration::node::Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let labels: Vec<NewEvent> = [
        world.own[0],
        world.others[0],
        world.others[1],
        world.own[1],
        world.elsewhere,
    ]
    .iter()
    .enumerate()
    .map(|(n, writer)| NewEvent {
        kind: EventKind::WriterLabel,
        provenance: Provenance::Structural,
        content: json!({"writer": writer.to_string()}),
        commitment_key: CommitmentKey::from_bytes([100 + n as u8; 32]),
        transcript_timestamp: None,
    })
    .collect();
    store
        .append(node.device_writer(), &labels)
        .map_err(|e| e.to_string())?;
    let steps = steps(w);
    steps.world = Some(world);
    steps.bound = true;
    Ok(())
}

fn world(w: &mut World) -> Result<World5, String> {
    steps(w)
        .world
        .clone()
        .ok_or_else(|| "no step built the rooms".to_owned())
}

/// The arguments `text` names for a bound call: the clauses the
/// scenario adds to say what it leaves out are dropped.
fn bound_arguments(w: &mut World, text: &str) -> Result<Value, String> {
    let text = text
        .strip_suffix(", an event of another run in \"L1\", and no scope")
        .or_else(|| text.strip_suffix(" and no scope"))
        .unwrap_or(text);

    crate::record_search::arguments_of(w, text)
}

/// Calls `tool` with `arguments` as the run does: under the tool-use id
/// of a line its transcript carries.
fn call_bound(w: &mut World, tool: &str, arguments: Value) -> Result<Reply, String> {
    steps(w).calls += 1;
    let id = format!("toolu_scope{}", steps(w).calls);
    let (mut call, _) = recorded_call("Bash")?;
    call["message"]["content"][0]["id"] = json!(id);
    call["uuid"] = json!(format!(
        "00000000-0000-4000-8000-{:012x}",
        9000 + steps(w).calls
    ));
    append_and_read(w, RUN, &[call])?;
    let params = json!({
        "name": tool,
        "arguments": arguments,
        "_meta": {"claudecode/toolUseId": id},
    });
    let reply = harness(w)?.request("tools/call", params)?;
    let answer = Reply {
        is_error: reply["result"]["isError"] == true,
        text: reply["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
    };
    if !answer.is_error {
        let envelope: Value = serde_json::from_str(&answer.text).map_err(|e| e.to_string())?;
        for item in envelope["items"].as_array().into_iter().flatten() {
            let writer = item["writer"].as_str().unwrap_or_default();
            steps(w)
                .seen
                .push(WriterId::parse(writer).map_err(|e| e.to_string())?);
        }
    }
    set_reply(w, answer.clone());
    Ok(answer)
}

/// A bound call whose scope the step text leaves out.
pub fn call(w: &mut World, tool: &str, args: &str) -> Result<(), String> {
    let arguments = bound_arguments(w, args)?;
    call_bound(w, tool, arguments).map(drop)
}

/// The writers of the hits of `answer`, each once.
fn writers_of(answer: &Reply) -> Result<Vec<WriterId>, String> {
    if answer.is_error {
        return Err(format!("the call was refused: {}", answer.text));
    }
    let envelope: Value = serde_json::from_str(&answer.text).map_err(|e| e.to_string())?;
    let mut writers: Vec<WriterId> = Vec::new();
    for item in envelope["items"].as_array().into_iter().flatten() {
        let writer = WriterId::parse(item["writer"].as_str().unwrap_or_default())
            .map_err(|e| e.to_string())?;
        if !writers.contains(&writer) {
            writers.push(writer);
        }
    }
    writers.sort();
    Ok(writers)
}

fn sorted(mut writers: Vec<WriterId>) -> Vec<WriterId> {
    writers.sort();
    writers
}

#[then(regex = r"^every hit belongs to the calling run, across the writers of both its seats$")]
fn hits_are_the_runs(w: &mut World) -> Result<(), String> {
    let world = world(w)?;
    let answer = crate::record_search::last_reply(w)?;
    let got = writers_of(&answer)?;
    if got == sorted(world.own.to_vec()) {
        return Ok(());
    }
    Err(format!(
        "the hits come from {got:?}, wanting {:?}",
        world.own
    ))
}

/// Audit entries that note a recall extended past the run.
fn extended(w: &World) -> Result<usize, String> {
    let audit = home(w)?.audit();
    if !audit.exists() {
        return Ok(0);
    }
    Ok(Log::new(audit)
        .entries()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|e| e.kind == "recall_extended")
        .count())
}

#[then(
    regex = r#"^with scope "room" and room "L1" the hits come from every writer of "L1", and with scope "rooms" from every room the run has a seat in, and an audit entry logs each call whose recall scope extends past the run$"#
)]
fn wider_scopes(w: &mut World) -> Result<(), String> {
    let world = world(w)?;
    let before = extended(w)?;
    let in_l1 = vec![world.own[1], world.others[0], world.others[1]];
    let room = call_bound(
        w,
        "event_search",
        json!({"query": "deploy", "scope": "room", "room": world.l1.to_string()}),
    )?;
    let got = writers_of(&room)?;
    if got != sorted(in_l1.clone()) {
        return Err(format!(
            "scope room and room L1 returned {got:?}, wanting {in_l1:?}"
        ));
    }
    let rooms = call_bound(
        w,
        "event_search",
        json!({"query": "deploy", "scope": "rooms"}),
    )?;
    let got = writers_of(&rooms)?;
    let every = sorted([world.own[0], world.own[1], world.others[0], world.others[1]].to_vec());
    if got != every {
        return Err(format!("scope rooms returned {got:?}, wanting {every:?}"));
    }
    let logged = extended(w)? - before;
    if logged != 2 {
        return Err(format!(
            "the audit log holds {logged} extended-recall entries, wanting 2"
        ));
    }
    Ok(())
}

#[then(regex = r#"^no scope returns any hit from "L2"$"#)]
fn nothing_from_l2(w: &mut World) -> Result<(), String> {
    let world = world(w)?;
    if steps(w).seen.contains(&world.elsewhere) {
        return Err("a hit came from room L2, where the run has no seat".to_owned());
    }
    // Not even the explicit scopes reach it.
    let l2 = call_bound(
        w,
        "event_search",
        json!({"query": "deploy", "scope": "room", "room": world.l2.to_string()}),
    )?;
    if !l2.is_error {
        return Err("a call naming room L2 was served".to_owned());
    }
    Ok(())
}

#[then(
    regex = r"^the event is not returned, and the error says the address lies outside the current scope$"
)]
fn outside_the_scope(w: &mut World) -> Result<(), String> {
    let answer = crate::record_search::last_reply(w)?;
    if answer.is_error && answer.text.contains("lies outside the current scope") {
        return Ok(());
    }
    Err(format!("the call answered: {}", answer.text))
}

#[then(
    regex = r#"^with scope "room" and room "L1" the event is returned, and an audit entry logs the extended recall scope$"#
)]
fn event_with_the_room(w: &mut World) -> Result<(), String> {
    let world = world(w)?;
    let before = extended(w)?;
    let answer = call_bound(
        w,
        "event_get",
        json!({"address": "A2:5", "scope": "room", "room": world.l1.to_string()}),
    )?;
    let got = writers_of(&answer)?;
    if got != vec![world.others[0]] {
        return Err(format!(
            "the event came from {got:?}, wanting {:?}",
            world.others[0]
        ));
    }
    if extended(w)? != before + 1 {
        return Err("the audit log holds no entry for the extended recall scope".to_owned());
    }
    steps(w).extended_before = before;
    Ok(())
}
