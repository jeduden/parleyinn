//! The steps of REC-04 that need live MCP servers: a harness fork's
//! lines a server ingests are witnessed, so they are never repeat
//! references; they are held up to 30 s, on an injected clock, only for
//! a transcript another server holds a session binding for, and go at
//! once into their own run's run seat otherwise.
//!
//! Homes E, F, G and I are homes of their own ([`crate::record_homes`]).
//! A server's clock is a file in its home a step writes the seconds of,
//! so a step moves it past 30 s without waiting.

use std::fs;
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::PathBuf;

use ::record::event::EventKind;
use cucumber::{given, then, when};

use crate::World;
use crate::live::{source_key, store, wait_until};
use crate::record::run_core;
use crate::record_homes::{
    REPEATED, enter, homes, ingest, placed, recorded_events, session, stop_servers, write_fork,
};
use crate::record_hooks::{counted, session_path};
use crate::record_live::{live, serve, witnessed};

/// The lines the fork records in all: its repeated lines and its two
/// own.
const FORK_LINES: usize = REPEATED as usize + 2;

/// The lines the parent records in all.
const PARENT_LINES: usize = REPEATED as usize + 5;

/// The file in the home a development build's live ingest reads the
/// time from.
const CLOCK: &str = "test-clock";

/// The clock file of the current home.
fn clock_path(w: &World) -> PathBuf {
    w.parinn_home.join(CLOCK)
}

/// Sets the current home's injected clock to `seconds`.
fn set_clock(w: &World, seconds: u64) -> Result<(), String> {
    // The home the core makes, for its own user alone.
    let mut dir = fs::DirBuilder::new();
    dir.recursive(true).mode(0o700);
    dir.create(&w.parinn_home).map_err(|e| e.to_string())?;
    let mut file = fs::OpenOptions::new();
    file.write(true).create(true).truncate(true).mode(0o600);
    file.open(clock_path(w))
        .and_then(|mut f| f.write_all(seconds.to_string().as_bytes()))
        .map_err(|e| e.to_string())
}

/// Now, in seconds since the Unix epoch, the injected clock's start.
fn wall_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Starts a harness whose MCP server serves the transcript `name` from
/// now, ingesting no line of it when `idle`, and keeps it running until
/// a step enters another home.
fn start_server(w: &mut World, name: &str, source: &str, idle: bool) -> Result<(), String> {
    let limit: &[(&str, &str)] = if idle {
        &[("PARINN_TEST_INGEST_LIMIT", "0")]
    } else {
        &[]
    };
    serve(w, session(name)?, source, limit)?;
    if let Some(server) = live(w).harness.take() {
        homes(w).servers.push(server);
    }
    Ok(())
}

/// Waits until the record holds `lines` events of the transcript `name`.
fn wait_for_lines(w: &World, name: &str, lines: usize) -> Result<(), String> {
    let path = session_path(w, session(name)?)?;
    wait_until(&format!("{lines} events of {name}"), || {
        Ok(recorded_events(w, &path)? >= lines)
    })
}

/// How many entries the audit log holds under `counter` once at least
/// `wanted` are there: the server audits what it ingested just after
/// its events land.
fn counted_by(w: &World, counter: &str, wanted: usize) -> Result<usize, String> {
    wait_until(&format!("{wanted} audit entries of {counter}"), || {
        Ok(counted(w, counter)? >= wanted)
    })?;
    counted(w, counter)
}

/// Fails unless the first [`REPEATED`] lines of `name` are events of
/// their own in a run seat, none a repeat reference.
fn own_events(w: &World, name: &str) -> Result<(), String> {
    let events = placed(w, name)?;
    for (i, (writer, kind, content)) in events.iter().take(REPEATED as usize).enumerate() {
        if *kind != EventKind::TranscriptLine || !witnessed(w, *writer)? {
            return Err(format!("{name}'s line {} is {kind:?} {content}", i + 1));
        }
    }
    if events.len() < REPEATED as usize {
        return Err(format!("{name} holds {} events", events.len()));
    }
    Ok(())
}

/// Fails unless the fork's repeated lines sit in a writer other than the
/// parent's, or the parent has none.
fn apart_from_parent(w: &World) -> Result<(), String> {
    let fork = placed(w, "forked-run")?;
    let parent = placed(w, "parent-run")?;
    let writer = fork.first().map(|(writer, _, _)| *writer);
    if writer.is_none() || writer == parent.first().map(|(w, _, _)| *w) {
        return Err("the fork's lines are not in a writer of their own".to_owned());
    }
    Ok(())
}

#[given(
    regex = r#"^a fresh isolated ParleyInn home "([^"]+)" where "parinn ingest" recorded "parent-run"$"#
)]
fn home_with_ingested_parent(w: &mut World, home: String) -> Result<(), String> {
    enter(w, &home)?;
    write_fork(w)?;
    ingest(w, "parent-run")
}

#[when(regex = r#"^the MCP server of "forked-run" ingests it live$"#)]
fn fork_ingested_live(w: &mut World) -> Result<(), String> {
    start_server(w, "forked-run", "fork", false)?;
    wait_for_lines(w, "forked-run", FORK_LINES)
}

#[then(
    regex = r#"^the 40 repeated lines are events of their own in the writer of the run of "forked-run", never repeat references to the ingested events, each audited$"#
)]
fn own_and_audited(w: &mut World) -> Result<(), String> {
    own_events(w, "forked-run")?;
    apart_from_parent(w)?;
    match counted_by(w, "witnessed_repeat_recorded", 40)? {
        40 => Ok(()),
        n => Err(format!("{n} witnessed repeats audited, not 40")),
    }
}

#[given(
    regex = r#"^a fresh isolated ParleyInn home "([^"]+)" on an injected clock, where no MCP server holds a session binding for "parent-run"$"#
)]
fn home_unbound(w: &mut World, home: String) -> Result<(), String> {
    enter(w, &home)?;
    set_clock(w, wall_seconds())?;
    write_fork(w)
}

#[when(regex = r#"^the MCP server of "forked-run" ingests its repeated lines live$"#)]
fn fork_ingests_unbound(w: &mut World) -> Result<(), String> {
    start_server(w, "forked-run", "fork", false)?;
    wait_for_lines(w, "forked-run", FORK_LINES)
}

#[then(
    regex = r#"^they go at once into the writer of the run of "forked-run", audited, with no hold$"#
)]
fn at_once(w: &mut World) -> Result<(), String> {
    own_events(w, "forked-run")?;
    let held = counted(w, "repeat_held")?;
    let audited = counted_by(w, "witnessed_repeat_recorded", 40)?;
    if (held, audited) == (0, 40) {
        return Ok(());
    }
    Err(format!(
        "{held} holds and {audited} audited lines, not none and 40"
    ))
}

#[given(
    regex = r#"^a fresh isolated ParleyInn home "([^"]+)" on an injected clock, where an MCP server holds the session binding for "parent-run" but has not ingested the repeated lines$"#
)]
fn home_bound(w: &mut World, home: String) -> Result<(), String> {
    enter(w, &home)?;
    set_clock(w, wall_seconds())?;
    write_fork(w)?;
    start_server(w, "parent-run", "startup", true)?;
    // The server names its transcript in its start marker once it
    // holds the binding.
    let parent = session("parent-run")?;
    let markers = w.parinn_home.join("mcp");
    wait_until("the server's start marker", || {
        Ok(fs::read_dir(&markers)
            .map(|l| {
                l.flatten()
                    .filter_map(|e| fs::read_to_string(e.path()).ok())
                    .any(|text| text.contains(parent))
            })
            .unwrap_or(false))
    })?;
    if recorded_events(w, &session_path(w, parent)?)? != 0 {
        return Err("the server ingested lines of parent-run".to_owned());
    }
    Ok(())
}

#[when(regex = r#"^the MCP server of "forked-run" ingests them live$"#)]
fn fork_ingests_held(w: &mut World) -> Result<(), String> {
    start_server(w, "forked-run", "fork", false)?;
    wait_until("the hold to be counted", || {
        Ok(counted(w, "repeat_held")? == 1)
    })
}

#[then(regex = r#"^the hold is counted and shown by "parinn status"$"#)]
fn hold_shown(w: &mut World) -> Result<(), String> {
    run_core(w, &["status"], None)?;
    let out = crate::record::last_run(w)?;
    let shown = String::from_utf8_lossy(&out.stdout).into_owned();
    if !shown.contains("repeat_held") {
        return Err(format!("parinn status does not show the hold:\n{shown}"));
    }
    let path = session_path(w, session("forked-run")?)?;
    let held = recorded_events(w, &path)?;
    let source = store(w)?
        .source(&source_key(&path))
        .map_err(|e| e.to_string())?;
    let since = source.and_then(|s| s.held_since);
    if held != 0 || since.is_none() {
        return Err(format!(
            "{held} events of forked-run recorded, hold since {since:?}"
        ));
    }
    Ok(())
}

#[then(
    regex = r#"^once the clock passes 30 s the lines are released into the writer of the run of "forked-run" and audited$"#
)]
fn released_after_30_s(w: &mut World) -> Result<(), String> {
    let now = fs::read_to_string(clock_path(w))
        .map_err(|e| e.to_string())?
        .trim()
        .parse::<u64>()
        .map_err(|e| e.to_string())?;
    set_clock(w, now + 31)?;
    wait_for_lines(w, "forked-run", FORK_LINES)?;
    own_events(w, "forked-run")?;
    apart_from_parent(w)?;
    let released = counted_by(w, "repeat_released", REPEATED as usize)?;
    if released < REPEATED as usize {
        return Err(format!("{released} releases audited, not {REPEATED}"));
    }
    Ok(())
}

#[given(
    regex = r#"^a fresh isolated ParleyInn home "([^"]+)" where two witnessed transcripts that repeat each other's lines were recorded by different recorders$"#
)]
fn home_two_recorders(w: &mut World, home: String) -> Result<(), String> {
    enter(w, &home)?;
    write_fork(w)
}

#[when(regex = r"^both are ingested$")]
fn both_ingested(w: &mut World) -> Result<(), String> {
    start_server(w, "parent-run", "startup", false)?;
    wait_for_lines(w, "parent-run", PARENT_LINES)?;
    start_server(w, "forked-run", "startup", false)?;
    wait_for_lines(w, "forked-run", FORK_LINES)?;
    stop_servers(w)
}

#[then(
    regex = r"^the repeated lines are events of their own in each run's writer, never repeat references, each audited$"
)]
fn each_own(w: &mut World) -> Result<(), String> {
    own_events(w, "parent-run")?;
    own_events(w, "forked-run")?;
    apart_from_parent(w)?;
    match counted_by(w, "witnessed_repeat_recorded", 40)? {
        40 => Ok(()),
        n => Err(format!("{n} witnessed repeats audited, not 40")),
    }
}
