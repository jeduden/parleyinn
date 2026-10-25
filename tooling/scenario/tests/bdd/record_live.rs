//! The steps of REC-13 and REC-06 that phase 10 of plan 2609292012
//! carries: the harness session's MCP server, started by a stand-in
//! harness process ([`crate::live`]), takes only its own harness
//! process's session binding, ingests its transcripts as they grow into
//! its runs' run seats, and leaves an ingest marker when it stops short,
//! from which the next MCP server or `parinn ingest` resumes; with `parinn
//! ingest` reading the same transcript at once, every line is recorded
//! once and each writer's `seq` stays gap-free.
//!
//! `@REC-06` stays pending: its purged range needs `parinn purge
//! --range` (phase 14), so its first step fails naming it.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use ::record::event::EventKind;
use audit::log::Log;
use cucumber::{given, then, when};
use key::certificate::SeatCertificate;
use serde_json::{Value, json};

use crate::World;
use crate::live::{
    Harness, append_lines, clean_line, lines_read, placed_lines, source_key, store, wait_until,
};
use crate::record::{home, last_run, run_core};
use crate::record_hooks::session_start;

/// What the live steps track.
#[derive(Debug, Default)]
pub struct LiveSteps {
    /// Each transcript a step named, by name.
    pub transcripts: BTreeMap<String, PathBuf>,
    /// The harness whose MCP server serves the run, while it runs.
    pub harness: Option<Harness>,
    /// The line of "main-run" the session binding is recorded at.
    binding_line: u64,
    /// How the last stop went: by the MCP server or killed.
    killed: bool,
    /// How many lines each transcript held when the server stopped.
    held_at_stop: BTreeMap<String, u64>,
    /// How many paths were refused before a step swapped one.
    refused_before: usize,
}

pub fn live(w: &mut World) -> &mut LiveSteps {
    w.section::<LiveSteps>()
}

/// The transcript a step named `name`.
pub fn named(w: &mut World, name: &str) -> Result<PathBuf, String> {
    live(w)
        .transcripts
        .get(name)
        .cloned()
        .ok_or_else(|| format!("no step made the transcript {name:?}"))
}

/// The directory the scenario's transcripts lie in, under the default
/// transcript root.
fn projects(w: &World) -> Result<PathBuf, String> {
    let dir = w.home.join(".claude").join("projects").join("-w-app");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Makes the main run's transcript `name` holding `lines` clean lines,
/// and, when given, its subagent's transcript `subagent`.
pub fn make_run(
    w: &mut World,
    name: &str,
    lines: u64,
    subagent: Option<&str>,
) -> Result<(), String> {
    let main = projects(w)?.join(format!("{name}.jsonl"));
    fs::write(&main, (1..=lines).map(clean_line).collect::<String>()).map_err(|e| e.to_string())?;
    live(w).transcripts.insert(name.to_owned(), main);
    if let Some(subagent) = subagent {
        let dir = projects(w)?.join(name).join("subagents");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let agent = subagent.trim_end_matches("-agent");
        let path = dir.join(format!("agent-{agent}.jsonl"));
        fs::write(&path, "").map_err(|e| e.to_string())?;
        live(w).transcripts.insert(subagent.to_owned(), path);
    }
    Ok(())
}

/// The harness serving the run.
pub fn harness(w: &mut World) -> Result<&mut Harness, String> {
    live(w)
        .harness
        .as_mut()
        .ok_or_else(|| "no MCP server is running".to_owned())
}

/// Starts a harness whose MCP server serves the main run `name` from
/// now: it runs the `SessionStart` hook with `source`, then waits until
/// the MCP server took the session binding.
pub fn serve(
    w: &mut World,
    name: &str,
    source: &str,
    extra: &[(&str, &str)],
) -> Result<(), String> {
    let before = taken_bindings(w, name)?;
    let mut harness = Harness::start(w, true, extra)?;
    harness.hook("SessionStart", &session_start(w, name, source)?)?;
    live(w).harness = Some(harness);
    wait_until("the MCP server to take its session binding", || {
        Ok(taken_bindings(w, name)? > before)
    })
}

/// How many session bindings of `session` an MCP server took.
pub fn taken_bindings(w: &World, session: &str) -> Result<usize, String> {
    if !home(w)?.store().exists() {
        return Ok(0);
    }
    Ok(store(w)?
        .hook_observations()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|h| h.observation.harness_session == session && h.taken)
        .count())
}

/// How many audit entries feed `counter`, by `operation` when given.
fn audited(
    w: &World,
    counter: &str,
    operation: Option<&str>,
) -> Result<Vec<audit::entry::Entry>, String> {
    let entries = Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some(counter))
        .filter(|e| operation.is_none_or(|o| e.operation == o))
        .collect())
}

/// Whether the writer `writer`'s seat is a run seat of this node and
/// no ingest seat: one whose events are witnessed.
fn witnessed(w: &World, writer: ::record::id::WriterId) -> Result<bool, String> {
    let seat = store(w)?.seat(writer).map_err(|e| e.to_string())?;
    Ok(seat
        .and_then(|s| SeatCertificate::from_value(&s.certificate).ok())
        .is_some_and(|c| !c.ingest_seat && c.seat_kind.as_str() == "run"))
}

#[given(
    regex = r#"^an agent run with a Claude Code transcript "([^"]+)" and a subagent transcript "([^"]+)"$"#
)]
fn run_with_subagent(w: &mut World, main: String, subagent: String) -> Result<(), String> {
    make_run(w, &main, 7, Some(&subagent))
}

#[given(
    regex = r#"^the "SessionStart" hook handler of the run's harness process recorded the session binding at line (\d+) of "([^"]+)"$"#
)]
fn binding_at(w: &mut World, line: u64, _name: String) -> Result<(), String> {
    // The hook runs once the MCP server has started, at this line.
    live(w).binding_line = line;
    Ok(())
}

#[given(regex = r#"^a neighbouring transcript "([^"]+)" in the same transcript directory$"#)]
fn neighbour(w: &mut World, name: String) -> Result<(), String> {
    make_run(w, &name, 3, None)
}

#[given(
    regex = r#"^a session binding another harness process's "SessionStart" hook handler recorded in the home for "([^"]+)"$"#
)]
fn other_binding(w: &mut World, name: String) -> Result<(), String> {
    let mut other = Harness::start(w, false, &[])?;
    other.hook("SessionStart", &session_start(w, &name, "startup")?)?;
    other.stop()
}

#[when(
    regex = r#"^the harness session's MCP server starts before any session binding exists, and the harness appends (\d+) lines to "([^"]+)"$"#
)]
fn server_starts_unbound(w: &mut World, count: u64, name: String) -> Result<(), String> {
    // The run stops at line 120, as the rows' stops ask.
    let harness = Harness::start(w, true, &[("PARINN_TEST_INGEST_LIMIT", "120")])?;
    live(w).harness = Some(harness);
    append_lines(&named(w, &name)?, count)?;
    // Several of its reads pass.
    std::thread::sleep(std::time::Duration::from_millis(600));
    Ok(())
}

#[then(regex = r"^none of those lines is ingested as witnessed$")]
fn none_witnessed(w: &mut World) -> Result<(), String> {
    let main = named(w, "main-run")?;
    match placed_lines(w, &main)?.len() {
        0 => Ok(()),
        n => Err(format!(
            "{n} lines were ingested before any session binding"
        )),
    }
}

#[when(
    regex = r#"^the MCP server takes the session binding of its own harness process, ignoring the one for "([^"]+)"$"#
)]
fn takes_own_binding(w: &mut World, _other: String) -> Result<(), String> {
    let main = named(w, "main-run")?;
    let held = fs::read_to_string(&main)
        .map_err(|e| e.to_string())?
        .lines()
        .count() as u64;
    let line = live(w).binding_line;
    if held != line {
        return Err(format!("main-run holds {held} lines, not {line}"));
    }
    let start = session_start(w, "main-run", "startup")?;
    harness(w)?.hook("SessionStart", &start)?;
    wait_until("the MCP server to take its session binding", || {
        Ok(taken_bindings(w, "main-run")? > 0)
    })
}

#[when(regex = r#"^the harness appends (\d+) lines to "([^"]+)" and (\d+) to "([^"]+)"$"#)]
fn appends_both(
    w: &mut World,
    a: u64,
    first: String,
    b: u64,
    second: String,
) -> Result<(), String> {
    append_lines(&named(w, &first)?, a)?;
    append_lines(&named(w, &second)?, b)
}

#[then(
    regex = r#"^the events of the first (\d+) lines of "([^"]+)" are in the record before the MCP server stops$"#
)]
fn first_lines_recorded(w: &mut World, count: u64, name: String) -> Result<(), String> {
    let path = named(w, &name)?;
    wait_until(&format!("{count} lines of {name} in the record"), || {
        Ok(lines_read(w, &path)? >= count)
    })?;
    let explore = named(w, "explore-agent")?;
    let held = fs::read_to_string(&explore)
        .map_err(|e| e.to_string())?
        .lines()
        .count() as u64;
    wait_until("explore-agent in the record", || {
        Ok(lines_read(w, &explore)? >= held)
    })
}

#[then(regex = r#"^nothing of "([^"]+)" is ingested as witnessed$"#)]
fn nothing_of(w: &mut World, name: String) -> Result<(), String> {
    let path = named(w, &name)?;
    match placed_lines(w, &path)?.len() {
        0 => Ok(()),
        n => Err(format!("{n} lines of {name} were ingested")),
    }
}

/// Notes how many lines each transcript holds as the server stops.
fn note_held(w: &mut World) -> Result<(), String> {
    let names: Vec<(String, PathBuf)> = live(w).transcripts.clone().into_iter().collect();
    for (name, path) in names {
        let held = fs::read_to_string(&path)
            .map_err(|e| e.to_string())?
            .lines()
            .count() as u64;
        live(w).held_at_stop.insert(name, held);
    }
    Ok(())
}

#[when(regex = r#"^the MCP server stops after ingesting (\d+) lines of "([^"]+)"$"#)]
fn server_stops(w: &mut World, _count: u64, _name: String) -> Result<(), String> {
    note_held(w)?;
    let harness = live(w).harness.take().ok_or("no MCP server is running")?;
    live(w).killed = false;
    harness.stop()
}

#[when(regex = r#"^the MCP server is killed after ingesting (\d+) lines of "([^"]+)"$"#)]
fn server_killed(w: &mut World, _count: u64, _name: String) -> Result<(), String> {
    note_held(w)?;
    let harness = live(w).harness.take().ok_or("no MCP server is running")?;
    live(w).killed = true;
    harness.kill()
}

#[then(regex = r"^the events ingested are a prefix of each transcript, in its order$")]
fn prefix_in_order(w: &mut World) -> Result<(), String> {
    for name in ["main-run", "explore-agent"] {
        let path = named(w, name)?;
        let lines = placed_lines(w, &path)?;
        let prefix: Vec<u64> = (1..=lines.len() as u64).collect();
        if lines != prefix {
            return Err(format!("{name}'s events hold the lines {lines:?}"));
        }
    }
    Ok(())
}

#[then(regex = r#"^the event the "SessionStart" hook input made sits at line (\d+) of "([^"]+)"$"#)]
fn observation_at(w: &mut World, line: u64, name: String) -> Result<(), String> {
    let path = named(w, &name)?;
    let store = store(w)?;
    let source = store
        .source(&source_key(&path))
        .map_err(|e| e.to_string())?
        .ok_or("main-run was never ingested")?;
    let placed = store.placed(source.id).map_err(|e| e.to_string())?;
    let writer = placed
        .first()
        .ok_or("no line of main-run is placed")?
        .address
        .writer;
    if !witnessed(w, writer)? {
        return Err("main-run's lines are not in a run seat's writer".to_owned());
    }
    let log = store.log(writer).map_err(|e| e.to_string())?;
    let at = log
        .iter()
        .position(|e| e.link.header.kind == EventKind::HookObservation)
        .ok_or("main-run's run seat holds no hook observation")?;
    let line_of = |i: usize| -> Option<u64> { log.get(i)?.content.as_ref()?["line"].as_u64() };
    match (at.checked_sub(1).and_then(line_of), line_of(at + 1)) {
        (Some(before), Some(after)) if before == line && after == line + 1 => Ok(()),
        other => Err(format!(
            "the hook observation sits between lines {other:?}, not after line {line}"
        )),
    }
}

#[then(regex = r"^an ingest marker records the remainder of each transcript ingest did not reach$")]
fn marker_records_remainder(w: &mut World) -> Result<(), String> {
    let main = named(w, "main-run")?;
    let store = store(w)?;
    let source = store
        .source(&source_key(&main))
        .map_err(|e| e.to_string())?
        .ok_or("main-run was never ingested")?;
    let size = fs::metadata(&main).map_err(|e| e.to_string())?.len();
    let held = live(w).held_at_stop.get("main-run").copied().unwrap_or(0);
    if source.position.lines < held && source.position.offset < size {
        return Ok(());
    }
    Err(format!(
        "the ingest position stands at line {} of {held}",
        source.position.lines
    ))
}

#[then(
    regex = r#"^(the next MCP server for "main-run"|"parinn ingest --all") resumes from the ingest marker, and the record then holds every line once$"#
)]
fn resumes(w: &mut World, who: String) -> Result<(), String> {
    let main = named(w, "main-run")?;
    let held = live(w).held_at_stop.get("main-run").copied().unwrap_or(0);
    if who.starts_with("the next MCP server") {
        serve(w, "main-run", "resume", &[])?;
        wait_until("the next MCP server to read main-run", || {
            Ok(lines_read(w, &main)? >= held)
        })?;
        let harness = live(w).harness.take().ok_or("no MCP server is running")?;
        harness.stop()?;
    } else {
        run_core(w, &["ingest", "--all"], None)?;
        let out = last_run(w)?;
        if !out.status.success() {
            return Err(format!(
                "parinn ingest --all exited {}: {}",
                out.status,
                String::from_utf8_lossy(&out.stderr)
            ));
        }
    }
    let mut lines = placed_lines(w, &main)?;
    lines.sort_unstable();
    let once: Vec<u64> = (1..=held).collect();
    if lines == once {
        return Ok(());
    }
    Err(format!("the record holds main-run's lines {lines:?}"))
}

#[then(
    regex = r#"^(the MCP server|"parinn ingest --all", finding the marker,) audits the stop under a named counter that is not a failure counter, shown by "parinn status"$"#
)]
fn stop_audited(w: &mut World, auditor: String) -> Result<(), String> {
    let operation = if auditor == "the MCP server" {
        "mcp"
    } else {
        "ingest"
    };
    let entries = audited(w, "ingest_remainder_left", Some(operation))?;
    let all = audited(w, "ingest_remainder_left", None)?;
    if entries.len() != 1 || all.len() != 1 || entries[0].kind == "failed" {
        return Err(format!(
            "the stop was audited {} times by {operation} and {} in all, kind {:?}",
            entries.len(),
            all.len(),
            entries.first().map(|e| e.kind.clone())
        ));
    }
    run_core(w, &["status"], None)?;
    let shown = String::from_utf8_lossy(&last_run(w)?.stdout).into_owned();
    if shown.contains("ingest_remainder_left") {
        return Ok(());
    }
    Err(format!("parinn status does not show the stop:\n{shown}"))
}

#[when(
    regex = r#"^after its session binding is checked, the subagent transcript "([^"]+)" is replaced by a symlink to "~/\.ssh/id_ed25519"$"#
)]
fn swapped_for_link(w: &mut World, name: String) -> Result<(), String> {
    let path = named(w, &name)?;
    serve(w, "main-run", "resume", &[])?;
    // A read passes, so the subagent's transcript is checked.
    std::thread::sleep(std::time::Duration::from_millis(400));
    let ssh = w.home.join(".ssh");
    fs::create_dir_all(&ssh).map_err(|e| e.to_string())?;
    let key = ssh.join("id_ed25519");
    fs::write(&key, format!("{}\n", PRIVATE_KEY)).map_err(|e| e.to_string())?;
    live(w).refused_before = audited(w, "path_outside_allowed_roots", None)?.len();
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    std::os::unix::fs::symlink(&key, &path).map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(600));
    let harness = live(w).harness.take().ok_or("no MCP server is running")?;
    harness.stop()
}

/// The private key the swapped link points at.
const PRIVATE_KEY: &str = "-----BEGIN OPENSSH PRIVATE KEY----- b3BlbnNzaC1rZXktdjE";

#[then(regex = r"^the MCP server refuses to read it, audited, and opened no socket throughout$")]
fn refuses_link(w: &mut World) -> Result<(), String> {
    let before = live(w).refused_before;
    let after = audited(w, "path_outside_allowed_roots", Some("mcp"))?.len();
    if after <= before {
        return Err("no refusal of the swapped link was audited".to_owned());
    }
    let store = store(w)?;
    for writer in store.writers().map_err(|e| e.to_string())? {
        for event in store.log(writer).map_err(|e| e.to_string())? {
            let text = event.content.map(|c| c.to_string()).unwrap_or_default();
            if text.contains("OPENSSH PRIVATE KEY") {
                return Err(format!("writer {writer} recorded the private key"));
            }
        }
    }
    // Every harness and MCP server the scenario started stopped with its
    // trace checked: none opened a socket (ENG-12).
    Ok(())
}

#[given(
    regex = r#"^an agent run with a Claude Code transcript "([^"]+)" whose writer has appended seq 1 to 20 and purged seq 5-9$"#
)]
fn appended_and_purged(w: &mut World, name: String) -> Result<(), String> {
    make_run(w, &name, 20, None)?;
    run_core(w, &["ingest", "--all"], None)?;
    Err(
        "the record holds seq 1 to 20; purging seq 5-9 awaits parinn purge --range, phase 14 of \
         plan 2609292012"
            .to_owned(),
    )
}

#[given(regex = r"^an append transaction of that writer that rolled back$")]
fn rolled_back(w: &mut World) -> Result<(), String> {
    let main = named(w, "main-run")?;
    let mut store = store(w)?;
    let source = store
        .source(&source_key(&main))
        .map_err(|e| e.to_string())?
        .ok_or("main-run was never ingested")?;
    let writer = store
        .placed(source.id)
        .map_err(|e| e.to_string())?
        .first()
        .ok_or("no line of main-run is placed")?
        .address
        .writer;
    let bad = store::NewEvent {
        kind: EventKind::TranscriptLine,
        provenance: ::record::event::Provenance::HarnessText,
        // No canonical encoding: the transaction rolls back.
        content: json!({"x": 0.5}),
        commitment_key: ::record::commitment::CommitmentKey::from_bytes([1; 32]),
        transcript_timestamp: None,
    };
    let before = store.log(writer).map_err(|e| e.to_string())?.len();
    if store.append(writer, &[bad]).is_ok() {
        return Err("the append did not roll back".to_owned());
    }
    let after = store.log(writer).map_err(|e| e.to_string())?.len();
    if before != after {
        return Err("the rolled-back append left an event".to_owned());
    }
    live(w).binding_line = before as u64;
    Ok(())
}

#[when(
    regex = r#"^the person runs "parinn ingest --all" while the run's MCP server ingests "([^"]+)" concurrently$"#
)]
fn ingest_beside_server(w: &mut World, name: String) -> Result<(), String> {
    let path = named(w, &name)?;
    serve(w, &name, "resume", &[])?;
    for _ in 0..10 {
        append_lines(&path, 10)?;
        run_core(w, &["ingest", "--all"], None)?;
        if !last_run(w)?.status.success() {
            return Err(format!(
                "parinn ingest --all failed: {}",
                String::from_utf8_lossy(&last_run(w)?.stderr)
            ));
        }
    }
    let held = fs::read_to_string(&path)
        .map_err(|e| e.to_string())?
        .lines()
        .count() as u64;
    wait_until("every line in the record", || {
        Ok(lines_read(w, &path)? >= held)
    })?;
    let harness = live(w).harness.take().ok_or("no MCP server is running")?;
    harness.stop()
}

#[then(regex = r"^every event carries the address \(writer, seq\) of the writer that appended it$")]
fn addressed_by_writer(w: &mut World) -> Result<(), String> {
    let store = store(w)?;
    for writer in store.writers().map_err(|e| e.to_string())? {
        for event in store.log(writer).map_err(|e| e.to_string())? {
            if event.link.header.address.writer != writer {
                return Err(format!("an event of {writer} names another writer"));
            }
        }
    }
    Ok(())
}

#[then(
    regex = r"^that writer's new events continue from seq (\d+) with no gap and no repeat, and no seq is reused$"
)]
fn continue_from(w: &mut World, from: u64) -> Result<(), String> {
    let store = store(w)?;
    for writer in store.writers().map_err(|e| e.to_string())? {
        let seqs: Vec<u64> = store
            .log(writer)
            .map_err(|e| e.to_string())?
            .iter()
            .map(|e| e.link.header.address.seq.get())
            .collect();
        let gap_free: Vec<u64> = (1..=seqs.len() as u64).collect();
        if seqs != gap_free {
            return Err(format!("writer {writer} holds the seqs {seqs:?}"));
        }
    }
    let main = named(w, "main-run")?;
    let source = store
        .source(&source_key(&main))
        .map_err(|e| e.to_string())?
        .ok_or("main-run was never ingested")?;
    let placed = store.placed(source.id).map_err(|e| e.to_string())?;
    let first = placed.first().ok_or("nothing placed")?.address.writer;
    let next = placed
        .iter()
        .filter(|p| p.address.writer == first)
        .map(|p| p.address.seq.get())
        .find(|seq| *seq > live(w).binding_line);
    // The MCP server may have recorded every new line first; what the
    // writer gained, if anything, follows its last seq.
    match next {
        None => Ok(()),
        Some(seq) if seq == from => Ok(()),
        other => Err(format!("the writer's new events begin at seq {other:?}")),
    }
}

#[then(
    regex = r"^each seq was assigned by its writer inside the append transaction that wrote its event$"
)]
fn assigned_in_transaction(w: &mut World) -> Result<(), String> {
    run_core(w, &["verify"], None)?;
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "parinn verify exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout)
    ))
}

#[then(regex = r"^a local index position is never shown or accepted as an address$")]
fn no_local_index(w: &mut World) -> Result<(), String> {
    let mut harness = Harness::start(w, true, &[])?;
    let mut refusals = Vec::new();
    for address in ["1", "parinn:1", "#21"] {
        let reply = harness.request(
            "tools/call",
            json!({"name": "event_get", "arguments": {"address": address, "scope": "room"}}),
        )?;
        refusals.push(reply["result"]["content"][0]["text"].clone());
    }
    harness.stop()?;
    let refused = refusals.iter().all(|t| {
        t.as_str()
            .is_some_and(|t| t.starts_with("the address is neither"))
    });
    if refused {
        return Ok(());
    }
    Err(format!("a local position was accepted: {refusals:?}"))
}

/// The hook observation event of `session` in a run seat's writer, once
/// its MCP server appended it.
pub fn observation_in_run_seat(w: &World, session: &str) -> Result<bool, String> {
    let store = store(w)?;
    for writer in store.writers().map_err(|e| e.to_string())? {
        if !witnessed(w, writer)? {
            continue;
        }
        let found = store
            .log(writer)
            .map_err(|e| e.to_string())?
            .iter()
            .any(|e| {
                e.link.header.kind == EventKind::HookObservation
                    && e.content
                        .as_ref()
                        .is_some_and(|c| c["harness_session"] == session)
            });
        if found {
            return Ok(true);
        }
    }
    Ok(false)
}

/// A parsed reply's text, for the steps that read one.
pub fn reply_text(reply: &Value) -> String {
    reply["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}
