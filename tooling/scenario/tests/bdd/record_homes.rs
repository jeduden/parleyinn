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
//! `@REC-04` stays pending: homes C and D, E, F, G and I, and its purged
//! lines, are not bound yet (phase 14 brings purge).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use ::record::event::EventKind;
use ::record::id::WriterId;
use cucumber::{given, then, when};
use serde_json::{Value, json};

use crate::World;
use crate::live::{source_key, store};
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
const REPEATED: u64 = 40;

/// What the home steps track: each home's directory, by name, and the
/// scenario's own home to return to.
#[derive(Debug, Default)]
pub struct HomeSteps {
    homes: BTreeMap<String, PathBuf>,
    own: Option<PathBuf>,
}

fn homes(w: &mut World) -> &mut HomeSteps {
    w.section::<HomeSteps>()
}

/// The harness session of the transcript `name`.
fn session(name: &str) -> Result<&'static str, String> {
    SESSIONS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, s)| *s)
        .ok_or_else(|| format!("no harness session for {name:?}"))
}

/// Points the scenario's home at home `name`, making it.
fn enter(w: &mut World, name: &str) -> Result<(), String> {
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
/// and adds two of its own, and records the fork observation.
fn fork_in_home(w: &mut World) -> Result<(), String> {
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
    fs::write(session_path(w, fork)?, fork_lines).map_err(|e| e.to_string())?;
    let input = session_start(w, fork, "fork")?;
    run_hook(w, "SessionStart", &input, &[])
}

/// Ingests the transcript `name` in the current home.
fn ingest(w: &mut World, name: &str) -> Result<(), String> {
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
fn placed(w: &World, name: &str) -> Result<Vec<(WriterId, EventKind, Value)>, String> {
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
