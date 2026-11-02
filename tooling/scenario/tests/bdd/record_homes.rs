//! The steps of REC-04 that need several homes, which phase 10 of plan
//! 2609292012 carries: homes A and B ingest a harness fork's
//! transcripts in either order, with the `SessionStart` fork
//! observation both recorded, and record each repeated line once, in
//! the writer of its owning run; home H releases lines held for a
//! transcript that is deleted.
//!
//! Each home is a home of its own in the scenario's directory, with
//! its own `HOME` and transcript roots; a step works in one by pointing
//! the scenario's home at it.
//!
//! Homes C and D record no fork observation, so the run that first
//! recorded each repeated line owns it; the planted transcript repeats a
//! `uuid` with other text; the late fork repeats lines the person
//! purged. Homes E, F, G and I need live MCP servers
//! ([`crate::record_servers`]).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use ::record::event::EventKind;
use ::record::id::WriterId;
use audit::log::Log;
use cucumber::{given, then, when};
use serde_json::{Value, json};

use crate::World;
use crate::live::{Harness, source_key, store};
use crate::record::{last_run, run_core};
use crate::record_hooks::{counted, run_hook, session_path, session_start};
use crate::record_transcript::recorded;

/// The harness session of each transcript the home steps name: a fork's
/// sorts before its parent's.
const SESSIONS: [(&str, &str); 2] = [
    ("parent-run", "5e55a0de-0000-4000-8000-0000000000bb"),
    ("forked-run", "5e55a0de-0000-4000-8000-0000000000aa"),
];

/// How many lines the fork repeats.
pub const REPEATED: u64 = 40;

/// What the home steps track: each home's directory, by name, and the
/// scenario's own home to return to.
#[derive(Debug, Default)]
pub struct HomeSteps {
    homes: BTreeMap<String, PathBuf>,
    own: Option<PathBuf>,
    /// The harnesses whose MCP servers run, stopped when a step enters
    /// another home.
    pub servers: Vec<Harness>,
    /// `repeat_conflicts` before the planted transcript was ingested.
    conflicts: usize,
    /// The events the person purged.
    purged: Vec<::record::id::Address>,
}

pub fn homes(w: &mut World) -> &mut HomeSteps {
    w.section::<HomeSteps>()
}

/// The harness session of the transcript `name`.
pub fn session(name: &str) -> Result<&'static str, String> {
    SESSIONS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, s)| *s)
        .ok_or_else(|| format!("no harness session for {name:?}"))
}

/// Points the scenario's home at home `name`, making it.
pub fn enter(w: &mut World, name: &str) -> Result<(), String> {
    stop_servers(w)?;
    if homes(w).own.is_none() {
        let own = w.home.clone();
        homes(w).own = Some(own);
    }
    let base = homes(w).own.clone().ok_or("no home of the scenario")?;
    let dir = base.join("homes").join(name);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    homes(w).homes.insert(name.to_owned(), dir.clone());
    w.parinn_home = dir.join(".parinn");
    w.home = dir;
    Ok(())
}

/// Stops each MCP server the steps started.
pub fn stop_servers(w: &mut World) -> Result<(), String> {
    for server in std::mem::take(&mut homes(w).servers) {
        server.stop()?;
    }
    Ok(())
}

/// A typed line in Claude Code 2.1.296's recorded shape, numbered `n`,
/// with the `uuid` `u-<n>`, of harness session `session`.
fn numbered(n: u64, session: &str) -> Result<String, String> {
    let mut line = recorded("a typed line", |l| l["promptSource"] == "typed")?;
    line["uuid"] = json!(format!("u-{n}"));
    line["message"]["content"] = json!(format!("line {n}"));
    line["sessionId"] = json!(session);
    line["promptId"] = json!(format!("p-{n}"));
    Ok(format!("{line}\n"))
}

/// Writes, in the current home, the parent's transcript and its harness
/// fork's, which repeats its first [`REPEATED`] lines with their uuids
/// and adds two of its own.
pub fn write_fork(w: &mut World) -> Result<(), String> {
    let (parent, fork) = (session("parent-run")?, session("forked-run")?);
    let parent_lines: String = (1..=REPEATED + 5)
        .map(|n| numbered(n, parent))
        .collect::<Result<_, _>>()?;
    fs::write(session_path(w, parent)?, parent_lines).map_err(|e| e.to_string())?;
    let mut fork_lines: String = (1..=REPEATED)
        .map(|n| numbered(n, fork))
        .collect::<Result<_, _>>()?;
    for n in 1001..=1002 {
        fork_lines.push_str(&numbered(n, fork)?);
    }
    fs::write(session_path(w, fork)?, fork_lines).map_err(|e| e.to_string())
}

/// [`write_fork`], and the fork observation the home records.
fn fork_in_home(w: &mut World) -> Result<(), String> {
    write_fork(w)?;
    let input = session_start(w, session("forked-run")?, "fork")?;
    run_hook(w, "SessionStart", &input, &[])
}

/// Ingests the transcript `name` in the current home.
pub fn ingest(w: &mut World, name: &str) -> Result<(), String> {
    let path = session_path(w, session(name)?)?.display().to_string();
    run_core(w, &["ingest", "--path", &path], None)?;
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "parinn ingest exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    ))
}

#[given(regex = r#"^two fresh isolated ParleyInn homes "([^"]+)" and "([^"]+)"$"#)]
fn two_homes(w: &mut World, a: String, b: String) -> Result<(), String> {
    enter(w, &a)?;
    enter(w, &b)
}

#[given(
    regex = r#"^a harness fork of "parent-run" with the transcript "forked-run", whose harness session id sorts before that of "parent-run", which repeats 40 lines of "parent-run" with their uuids, and whose "SessionStart" hook observation with source "fork" both homes recorded$"#
)]
fn fork_in_both(w: &mut World) -> Result<(), String> {
    let names: Vec<String> = homes(w).homes.keys().cloned().collect();
    for name in names {
        enter(w, &name)?;
        fork_in_home(w)?;
    }
    Ok(())
}

#[when(regex = r#"^home "([^"]+)" ingests "([^"]+)", then "([^"]+)"$"#)]
fn home_ingests(w: &mut World, home: String, first: String, then: String) -> Result<(), String> {
    enter(w, &home)?;
    ingest(w, &first)?;
    ingest(w, &then)
}

/// The events of the transcript `name` in the current home, in line
/// order, each with its writer.
pub fn placed(w: &World, name: &str) -> Result<Vec<(WriterId, EventKind, Value)>, String> {
    let path = session_path(w, session(name)?)?;
    let store = store(w)?;
    let mut out = Vec::new();
    for generation in store
        .generations(&source_key(&path))
        .map_err(|e| e.to_string())?
    {
        for p in store.placed(generation.id).map_err(|e| e.to_string())? {
            let event = store
                .event(p.address)
                .map_err(|e| e.to_string())?
                .ok_or("a placed event is missing")?;
            out.push((
                p.address.writer,
                p.kind,
                event.content.unwrap_or(Value::Null),
            ));
        }
    }
    Ok(out)
}

/// Fails unless, in the current home, each repeated line is one event
/// of the parent's run and the fork's run refers to it.
fn recorded_once(w: &World) -> Result<(), String> {
    let parent = placed(w, "parent-run")?;
    let fork = placed(w, "forked-run")?;
    let parent_writer = parent.first().ok_or("parent-run holds no event")?.0;
    for (i, (_, kind, content)) in fork.iter().take(REPEATED as usize).enumerate() {
        let owner = &parent[i];
        let refers = *kind == EventKind::RepeatReference
            && owner.0 == parent_writer
            && content["repeats"].as_str().is_some_and(|a| {
                a.ends_with(&format!(":{}", i + 1)) || a.contains(&owner.0.to_string())
            });
        if !refers {
            return Err(format!("forked-run's line {} is {kind:?} {content}", i + 1));
        }
    }
    Ok(())
}

#[then(
    regex = r#"^in both homes each of the 40 repeated lines is one event in the writer of the run of "parent-run", and the run of "forked-run" records a repeat reference to its address, never a copy$"#
)]
fn once_in_both(w: &mut World) -> Result<(), String> {
    let names: Vec<String> = homes(w).homes.keys().cloned().collect();
    for name in names {
        enter(w, &name)?;
        recorded_once(w).map_err(|e| format!("home {name}: {e}"))?;
    }
    Ok(())
}

#[then(
    regex = r#"^in home "([^"]+)" the repeated lines of "forked-run" were held at an ingest marker until "parent-run" was ingested$"#
)]
fn held_until_parent(w: &mut World, home: String) -> Result<(), String> {
    enter(w, &home)?;
    match counted(w, "repeat_held")? {
        1 => Ok(()),
        n => Err(format!("home {home} held the fork's lines {n} times")),
    }
}

/// The record of the current home's two transcripts, each writer named
/// by the transcript whose run it records.
fn membership(w: &World) -> Result<Value, String> {
    let mut names = BTreeMap::new();
    let mut out = BTreeMap::new();
    for (name, _) in SESSIONS {
        let events = placed(w, name)?;
        if let Some((writer, _, _)) = events.first() {
            names.insert(writer.to_string(), name);
        }
        out.insert(name, events);
    }
    let rename = |text: String| {
        names
            .iter()
            .fold(text, |t, (writer, name)| t.replace(writer.as_str(), name))
    };
    let shown: BTreeMap<&str, Vec<Value>> = out
        .into_iter()
        .map(|(name, events)| {
            let events = events
                .into_iter()
                .map(|(writer, kind, content)| {
                    json!({
                        "writer": rename(writer.to_string()),
                        "kind": kind.as_str(),
                        "content": rename(content.to_string()),
                    })
                })
                .collect();
            (name, events)
        })
        .collect();
    Ok(json!(shown))
}

#[then(
    regex = r"^the run membership of the two homes is identical, and each derived artifact is identical in content apart from each home's own writer ids$"
)]
fn homes_alike(w: &mut World) -> Result<(), String> {
    let names: Vec<String> = homes(w).homes.keys().cloned().collect();
    let mut seen = Vec::new();
    for name in &names {
        enter(w, name)?;
        seen.push(membership(w)?);
    }
    match seen.as_slice() {
        [a, b] if a == b => Ok(()),
        [a, b] => Err(format!("the homes differ:\n{a}\n{b}")),
        _ => Err(format!("{} homes, not two", seen.len())),
    }
}

#[given(
    regex = r#"^a fresh isolated ParleyInn home "([^"]+)" where the lines of "forked-run" are held for "parent-run"$"#
)]
fn home_holding(w: &mut World, home: String) -> Result<(), String> {
    enter(w, &home)?;
    fork_in_home(w)?;
    ingest(w, "forked-run")?;
    match counted(w, "repeat_held")? {
        1 => Ok(()),
        n => Err(format!("the fork's lines were held {n} times")),
    }
}

#[when(regex = r#"^the transcript of "([^"]+)" is deleted$"#)]
fn transcript_deleted(w: &mut World, name: String) -> Result<(), String> {
    let path = session_path(w, session(&name)?)?;
    fs::remove_file(path).map_err(|e| e.to_string())
}

#[then(
    regex = r#"^the held lines are released into the writer of the run of "forked-run" and audited \(REC-08\)$"#
)]
fn released_when_gone(w: &mut World) -> Result<(), String> {
    run_core(w, &["ingest", "--all"], None)?;
    let fork = placed(w, "forked-run")?;
    let own = fork
        .iter()
        .take(REPEATED as usize)
        .all(|(_, kind, _)| *kind == EventKind::TranscriptLine);
    let released = counted(w, "repeat_released")?;
    if own && fork.len() as u64 == REPEATED + 2 && released >= REPEATED as usize {
        return Ok(());
    }
    Err(format!(
        "forked-run holds {} events, own lines {own}, {released} releases audited",
        fork.len()
    ))
}

#[given(
    regex = r#"^two fresh isolated ParleyInn homes "([^"]+)" and "([^"]+)" with no "SessionStart" hook observation for either transcript$"#
)]
fn two_homes_unobserved(w: &mut World, a: String, b: String) -> Result<(), String> {
    for name in [a, b] {
        enter(w, &name)?;
        write_fork(w)?;
    }
    Ok(())
}

/// The writer of the run `name` records in the current home.
pub fn writer_of(w: &World, name: &str) -> Result<WriterId, String> {
    placed(w, name)?
        .first()
        .map(|(writer, _, _)| *writer)
        .ok_or_else(|| format!("{name} holds no event"))
}

/// Fails unless, in the current home, each repeated line is one event
/// of the run `owner` and the run `other` records a repeat reference to
/// its address.
fn owned_by(w: &World, owner: &str, other: &str) -> Result<(), String> {
    let held = placed(w, owner)?;
    let owner_writer = writer_of(w, owner)?;
    for (i, (_, kind, _)) in held.iter().take(REPEATED as usize).enumerate() {
        if *kind != EventKind::TranscriptLine {
            return Err(format!("{owner}'s line {} is {kind:?}", i + 1));
        }
    }
    for (i, (_, kind, content)) in placed(w, other)?.iter().take(REPEATED as usize).enumerate() {
        let names = content["repeats"]
            .as_str()
            .is_some_and(|a| a.contains(&owner_writer.to_string()));
        if *kind != EventKind::RepeatReference || !names {
            return Err(format!("{other}'s line {} is {kind:?} {content}", i + 1));
        }
    }
    Ok(())
}

/// Fails unless a structural event of the current home's device seat
/// names `owner`'s writer as the repeat owner, and one audit entry says
/// so.
fn choice_recorded(w: &World, owner: &str) -> Result<(), String> {
    let store = store(w)?;
    let device = administration::node::Node::find(&crate::record::home(w)?, &store)
        .map_err(|e| e.to_string())?
        .device_writer();
    let wanted = writer_of(w, owner)?.to_string();
    let chosen = store
        .log(device)
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|e| e.link.header.kind == EventKind::RepeatOwner)
        .filter(|e| {
            e.content
                .as_ref()
                .is_some_and(|c| c["owner"] == wanted.as_str())
        })
        .count();
    match (chosen, counted(w, "repeat_owner_chosen")?) {
        (1, 1) => Ok(()),
        (events, audits) => Err(format!(
            "{events} repeat owner events and {audits} audit entries name {owner}"
        )),
    }
}

#[then(
    regex = r#"^in each home each of the 40 repeated lines is one event in the writer of the run that first recorded it there, "parent-run"'s in "([^"]+)" and "forked-run"'s in "([^"]+)", the other run recording a repeat reference, and a structural event records and audits that choice$"#
)]
fn first_recorder_owns(w: &mut World, c: String, d: String) -> Result<(), String> {
    for (home, owner, other) in [
        (c, "parent-run", "forked-run"),
        (d, "forked-run", "parent-run"),
    ] {
        enter(w, &home)?;
        owned_by(w, owner, other).map_err(|e| format!("home {home}: {e}"))?;
        choice_recorded(w, owner).map_err(|e| format!("home {home}: {e}"))?;
    }
    Ok(())
}

/// Whether a step entered a home of its own: the planted transcript and
/// its conflict are then this module's, in that home, not the
/// scenario's first.
pub fn in_homes(w: &mut World) -> bool {
    homes(w).own.is_some()
}

/// The planted transcript, in the transcript directory of the current
/// home's "parent-run": one line repeating a `uuid` of it with other
/// text.
pub fn plant(w: &mut World) -> Result<(), String> {
    let planted = session_path(w, "5e55a0de-0000-4000-8000-0000000000dd")?;
    let line = numbered(3, "5e55a0de-0000-4000-8000-0000000000dd")?
        .replace("line 3", "text another line never held");
    fs::write(planted, line).map_err(|e| e.to_string())?;
    let before = counted(w, "repeat_conflicts")?;
    homes(w).conflicts = before;
    Ok(())
}

/// That line of the planted transcript is its own event, the counter
/// `repeat_conflicts` went up by 1 and an audit entry records it.
pub fn planted_conflicts(w: &mut World) -> Result<(), String> {
    let path = session_path(w, "5e55a0de-0000-4000-8000-0000000000dd")?;
    let events = recorded_events(w, &path)?;
    if events != 1 {
        return Err(format!("planted-run holds {events} events, not its line"));
    }
    let before = homes(w).conflicts;
    let after = counted(w, "repeat_conflicts")?;
    if after != before + 1 {
        return Err(format!("repeat_conflicts went from {before} to {after}"));
    }
    let entries = Log::new(crate::record::home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    let said = entries.iter().any(|e| {
        e.counter.as_deref() == Some("repeat_conflicts")
            && e.detail.contains("repeated uuid with other content")
    });
    if said {
        return Ok(());
    }
    Err("no audit entry records the conflict".to_owned())
}

/// How many events the record holds of the transcript at `path`.
pub fn recorded_events(w: &World, path: &std::path::Path) -> Result<usize, String> {
    let store = store(w)?;
    let mut count = 0;
    for generation in store
        .generations(&source_key(path))
        .map_err(|e| e.to_string())?
    {
        count += store
            .placed(generation.id)
            .map_err(|e| e.to_string())?
            .len();
    }
    Ok(count)
}

/// The address of the event of each of the first [`REPEATED`] lines of
/// the transcript `name`, in the current home.
fn line_events(w: &World, name: &str) -> Result<Vec<::record::id::Address>, String> {
    let path = session_path(w, session(name)?)?;
    let store = store(w)?;
    let mut out = Vec::new();
    for generation in store
        .generations(&source_key(&path))
        .map_err(|e| e.to_string())?
    {
        for p in store.placed(generation.id).map_err(|e| e.to_string())? {
            if p.place.line <= REPEATED {
                out.push(p.address);
            }
        }
    }
    Ok(out)
}

#[given(
    regex = r#"^the person purged the events of the 40 repeated lines from the run of "parent-run"$"#
)]
fn purged_repeated(w: &mut World) -> Result<(), String> {
    let events = line_events(w, "parent-run")?;
    let seqs = events.iter().map(|a| a.seq.get());
    let (first, last) = (seqs.clone().min(), seqs.max());
    let (Some(first), Some(last)) = (first, last) else {
        return Err("parent-run holds no event of those lines".to_owned());
    };
    let range = format!("{}:{first}-{last}", events[0].writer);
    homes(w).purged = events;
    crate::erasure::person_runs(w, &format!("purge --range {range}"))?;
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "parinn purge exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout)
    ))
}

#[when(
    regex = r#"^a harness fork of "parent-run" with the transcript "late-fork", repeating those 40 lines, is ingested$"#
)]
fn late_fork_ingested(w: &mut World) -> Result<(), String> {
    let fork = LATE_FORK;
    let mut lines: String = (1..=REPEATED)
        .map(|n| numbered(n, fork))
        .collect::<Result<_, _>>()?;
    lines.push_str(&numbered(2001, fork)?);
    let path = session_path(w, fork)?;
    fs::write(&path, lines).map_err(|e| e.to_string())?;
    let shown = path.display().to_string();
    run_core(w, &["ingest", "--path", &shown], None)?;
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "parinn ingest exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    ))
}

/// The harness session of the late fork.
const LATE_FORK: &str = "5e55a0de-0000-4000-8000-0000000000cc";

#[then(
    regex = r"^none of those lines is recorded again, each refusal is counted and audited, and the record keeps only their uuids beside the tombstones$"
)]
fn refused_and_uuids_kept(w: &mut World) -> Result<(), String> {
    let path = session_path(w, LATE_FORK)?;
    let held = recorded_events(w, &path)?;
    if held != 1 {
        return Err(format!("late-fork holds {held} events, not its own line"));
    }
    let refusals = counted(w, "repeat_of_purged_refused")?;
    if refusals != REPEATED as usize {
        return Err(format!("{refusals} refusals counted, not {REPEATED}"));
    }
    let store = store(w)?;
    let source = session_path(w, session("parent-run")?)?;
    let mut uuids = Vec::new();
    for generation in store
        .generations(&source_key(&source))
        .map_err(|e| e.to_string())?
    {
        for p in store.placed(generation.id).map_err(|e| e.to_string())? {
            if homes(w).purged.contains(&p.address) {
                uuids.push(p.place.uuid.clone().unwrap_or_default());
                let event = store.event(p.address).map_err(|e| e.to_string())?;
                if event.is_none_or(|e| e.content.is_some()) {
                    return Err(format!("{} keeps its content", p.address));
                }
            }
        }
    }
    let wanted: Vec<String> = (1..=REPEATED).map(|n| format!("u-{n}")).collect();
    uuids.sort_by_key(|u| u.trim_start_matches("u-").parse::<u64>().unwrap_or(0));
    if uuids != wanted {
        return Err(format!("the record keeps the uuids {uuids:?}"));
    }
    tombstoned(&store, w)
}

/// Fails unless a tombstone stands in the current home's device seat.
fn tombstoned(store: &store::Store, w: &World) -> Result<(), String> {
    let device = administration::node::Node::find(&crate::record::home(w)?, store)
        .map_err(|e| e.to_string())?
        .device_writer();
    let marks = store
        .log(device)
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|e| e.link.header.kind == EventKind::Tombstone)
        .count();
    if marks > 0 {
        return Ok(());
    }
    Err("the purge left no tombstone".to_owned())
}
