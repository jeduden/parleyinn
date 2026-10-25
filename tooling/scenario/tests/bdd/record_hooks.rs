//! The steps of REC-19 that phase 9 of plan 2609292012 carries, and the
//! hook handler steps the record and provenance scenarios share: a hook
//! handler of the built core runs on a hook input in Claude Code
//! 2.1.296's recorded shape and holds what it observed at the
//! transcript's ingest marker, appending nothing to any writer, until
//! the run's MCP server holds its run-seat key; a session
//! binding no MCP server took within 30 s is counted once.
//!
//! `@REC-19` stays pending: its rooms, sealing and paired phone are M7's
//! and OQ-39's. Its holding step passes: the run's MCP server, started
//! after the hook handler, appends the held observation to its run
//! seat (phase 10); its untaken session binding step passes.

use std::fs;
use std::path::{Path, PathBuf};

use audit::log::Log;
use cucumber::then;
use serde_json::{Value, json};
use store::Store;

use crate::World;
use crate::record::{home, last_run, run_core_with};
use crate::record_transcript::recorded;

/// A harness session id as Claude Code gives one.
pub const SESSION: &str = "5e55a0de-0000-4000-8000-000000000001";

/// The path of harness session `session`'s transcript under the default
/// transcript root, its directory made.
pub fn session_path(w: &World, session: &str) -> Result<PathBuf, String> {
    let dir = w.home.join(".claude").join("projects").join("-w-app");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(format!("{session}.jsonl")))
}

/// A `SessionStart` hook input of `session` with `source`.
pub fn session_start(w: &World, session: &str, source: &str) -> Result<Value, String> {
    Ok(json!({
        "session_id": session,
        "transcript_path": session_path(w, session)?,
        "cwd": "/w/app",
        "hook_event_name": "SessionStart",
        "source": source,
    }))
}

/// A `UserPromptSubmit` hook input of `session` for `prompt`.
pub fn prompt_submit(
    w: &World,
    session: &str,
    prompt_id: &str,
    prompt: &str,
) -> Result<Value, String> {
    Ok(json!({
        "session_id": session,
        "transcript_path": session_path(w, session)?,
        "cwd": "/w/app",
        "prompt_id": prompt_id,
        "permission_mode": "default",
        "hook_event_name": "UserPromptSubmit",
        "prompt": prompt,
    }))
}

/// Runs the hook handler `hook` of the built core on `input`, with
/// `extra` in its environment, failing unless it exits 0 with empty
/// output and nothing on stderr.
pub fn run_hook(
    w: &mut World,
    hook: &str,
    input: &Value,
    extra: &[(&str, &str)],
) -> Result<(), String> {
    run_core_with(
        w,
        &["hook", hook],
        Some(input.to_string().as_bytes()),
        extra,
    )?;
    answered_empty(w)
}

/// Fails unless the last command exited 0 with empty output and nothing
/// on stderr.
pub fn answered_empty(w: &mut World) -> Result<(), String> {
    let out = last_run(w)?;
    if out.status.success() && out.stdout.is_empty() && out.stderr.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the hook exited {} with stdout {:?} and stderr {:?}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    ))
}

/// What the hook steps track: whether the last hook a step ran gave the
/// empty answer and exited 0.
#[derive(Debug, Default)]
pub struct HookSteps {
    answered_empty: Option<bool>,
}

/// Notes whether the last command, a hook handler, gave the empty answer
/// and exited 0, for a later step to judge.
pub fn note_answer(w: &mut World) -> Result<(), String> {
    let empty = answered_empty(w).is_ok();
    w.section::<HookSteps>().answered_empty = Some(empty);
    Ok(())
}

#[then(regex = r"^the hook handler gives the empty answer and exits 0$")]
fn gave_the_empty_answer(w: &mut World) -> Result<(), String> {
    match w.section::<HookSteps>().answered_empty {
        Some(true) => Ok(()),
        Some(false) => Err("the hook handler answered more than nothing, or failed".to_owned()),
        None => Err("no step ran a hook handler".to_owned()),
    }
}

/// The attended mark as a hook handler's environment holds it: `1` in
/// the harness's own terminal interface, `0` elsewhere (ASM-22).
pub fn attended_mark(attended: bool) -> (&'static str, &'static str) {
    (
        "CLAUDE_CODE_SESSION_ATTENDED",
        if attended { "1" } else { "0" },
    )
}

/// A recorded Claude Code 2.1.296 line typed at the harness's input,
/// carrying `text` as prompt `prompt_id` marked `source`, in `session`.
pub fn typed_line(
    text: &str,
    prompt_id: &str,
    source: &str,
    session: &str,
) -> Result<Value, String> {
    let mut line = recorded("a typed line", |l| l["promptSource"] == "typed")?;
    line["message"]["content"] = json!(text);
    line["promptId"] = json!(prompt_id);
    line["promptSource"] = json!(source);
    line["sessionId"] = json!(session);
    Ok(line)
}

/// Every event the store holds, across every writer.
fn events(w: &World) -> Result<usize, String> {
    let path = home(w)?.store();
    if !path.exists() {
        return Ok(0);
    }
    let store = Store::open(&path).map_err(|e| e.to_string())?;
    let mut count = 0;
    for writer in store.writers().map_err(|e| e.to_string())? {
        count += store.log(writer).map_err(|e| e.to_string())?.len();
    }
    Ok(count)
}

/// The hook observations the store holds.
fn held(w: &World) -> Result<Vec<store::Held>, String> {
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    store.hook_observations().map_err(|e| e.to_string())
}

/// How many audit entries feed `counter`.
pub fn counted(w: &World, counter: &str) -> Result<usize, String> {
    let entries = Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    Ok(entries
        .iter()
        .filter(|e| e.counter.as_deref() == Some(counter))
        .count())
}

#[then(
    regex = r#"^a second run, whose "SessionStart" hook handler ran before any MCP server held its run-seat key, appended nothing, and its MCP server appended the hook observation from the ingest marker once it held that key$"#
)]
fn second_run_held(w: &mut World) -> Result<(), String> {
    let session = "5e55a0de-0000-4000-8000-000000000002";
    let path = session_path(w, session)?;
    let mode = recorded("a mode line", |l| l["type"] == "mode")?;
    fs::write(&path, format!("{mode}\n")).map_err(|e| e.to_string())?;
    let before = events(w)?;
    // Its harness runs the hook before it starts its MCP server.
    let mut harness = crate::live::Harness::start(w, false, &[])?;
    let input = session_start(w, session, "startup")?;
    harness.hook("SessionStart", &input)?;
    let appended = events(w)? - before;
    let size = fs::metadata(&path).map_err(|e| e.to_string())?.len();
    let hold = held(w)?
        .into_iter()
        .find(|h| h.observation.harness_session == session && h.observation.offset == Some(size));
    if appended != 0 || hold.is_none() {
        let _ = harness.kill();
        return Err(format!(
            "the hook handler appended {appended} events and held {hold:?}"
        ));
    }
    harness.start_mcp()?;
    let appended = crate::live::wait_until("the MCP server to append the hook observation", || {
        crate::record_live::observation_in_run_seat(w, session)
    });
    harness.stop()?;
    appended?;
    match held(w)?
        .into_iter()
        .find(|h| h.observation.harness_session == session)
    {
        Some(h) if h.appended && h.taken => Ok(()),
        other => Err(format!("the hook observation stands {other:?}")),
    }
}

/// Runs `args` with the clock standing at `at`.
fn run_at(w: &mut World, args: &[&str], input: Option<&[u8]>, at: u64) -> Result<(), String> {
    let at = at.to_string();
    run_core_with(w, args, input, &[("PARINN_TEST_CLOCK", &at)])
}

#[then(
    regex = r#"^a third harness session, for whose "SessionStart" hook observation no MCP server took the session binding within 30 s, is counted and audited as a failure once, by whichever of its next hook handler, "parinn status" and "parinn ingest" finds it first$"#
)]
fn third_session_counted(w: &mut World) -> Result<(), String> {
    let session = "5e55a0de-0000-4000-8000-000000000003";
    let started = 1_900_000_000;
    // Every session binding the scenario's other runs left is counted
    // first, so only the third session's is counted after.
    run_at(w, &["status"], None, started - 1)?;
    let input = session_start(w, session, "startup")?;
    run_at(
        w,
        &["hook", "SessionStart"],
        Some(input.to_string().as_bytes()),
        started,
    )?;
    answered_empty(w)?;
    let before = counted(w, "session_binding_not_taken")?;
    // Within 30 s nothing counts it; after, the first reader does and
    // no later one again.
    run_at(w, &["status"], None, started + 29)?;
    let early = counted(w, "session_binding_not_taken")? - before;
    run_at(w, &["status"], None, started + 31)?;
    let input = prompt_submit(w, session, "p-3", "go on")?;
    run_at(
        w,
        &["hook", "UserPromptSubmit"],
        Some(input.to_string().as_bytes()),
        started + 40,
    )?;
    run_at(w, &["ingest", "--all"], None, started + 41)?;
    let later = counted(w, "session_binding_not_taken")? - before;
    if early == 0 && later == 1 {
        return Ok(());
    }
    Err(format!(
        "counted {early} times within 30 s and {later} times in all"
    ))
}

/// Writes `body` at `path`, executable.
fn executable(path: &Path, body: &str) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::write(path, body).map_err(|e| e.to_string())?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())
}

/// Runs the hook handler `hook` on `input` as a harness started by a
/// run's tool call in another harness would: below two harness
/// processes, each a script named `claude`.
pub fn run_nested_hook(w: &mut World, hook: &str, input: &Value) -> Result<(), String> {
    let fake = w.home.join("fake-harness");
    let (outer, inner) = (fake.join("outer"), fake.join("inner"));
    fs::create_dir_all(&outer).map_err(|e| e.to_string())?;
    fs::create_dir_all(&inner).map_err(|e| e.to_string())?;
    // Neither script ends in the command it runs, so no shell replaces
    // itself with it and both stay in the ancestry.
    let script = |next: &str| format!("#!/bin/sh\n\"{next}\" \"$@\"\nstatus=$?\nexit $status\n");
    let core = testkit::core_executable()?;
    executable(&inner.join("claude"), &script(&core.display().to_string()))?;
    executable(
        &outer.join("claude"),
        &script(&inner.join("claude").display().to_string()),
    )?;
    crate::record::run_program_with(
        w,
        &outer.join("claude"),
        &["hook", hook],
        Some(input.to_string().as_bytes()),
        &[],
    )?;
    answered_empty(w)
}
