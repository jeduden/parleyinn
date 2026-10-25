//! The steps of PRV-04 and PRV-08 that phase 9 of plan 2609292012
//! carries: the hook handlers record the process ancestry, the attended
//! mark and a prompt commitment, and ingest classes and matches a typed
//! line from them. Every run leaves R3 and R8 open until a signed
//! sandbox state exists (OQ-40), so no `user` event is trusted yet.
//!
//! `@PRV-04` stays pending: its last row's trusted `user` event needs
//! trust (phase 11) and a signed sandbox state (OQ-40). `@PRV-08` stays
//! pending for its first row, the same.

use std::fs;

use cucumber::{given, then, when};

use crate::World;
use crate::administration::configure;
use crate::record::{home, last_run, run_core_at_terminal, set_env};
use crate::record_hooks::{
    SESSION, attended_mark, counted, prompt_submit, run_hook, run_nested_hook, session_path,
    session_start, typed_line,
};
use crate::record_transcript::ingest;

/// The prompt a user turn's line carries, and its prompt id.
const PROMPT: &str = "Reply with just the word A1.";
const PROMPT_ID: &str = "7761911f-d0d3-455b-a491-45a2837bbf50";

/// Opts into deployment mode `interactive` in the person's own
/// configuration, accepted at a terminal (PRV-04, ADM-04).
fn interactive(w: &mut World) -> Result<(), String> {
    configure(w, "[node]\ndeployment_mode = \"interactive\"\n")?;
    run_core_at_terminal(w, &["configuration", "accept"])?;
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "parinn configuration accept exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout).trim()
    ))
}

#[given(
    regex = r#"^the person's configuration sets (no deployment mode, and the repository's "\.parinn\.toml" sets "node\.deployment_mode" to "interactive"|"node\.deployment_mode" to "interactive"|"node\.deployment_mode" to "interactive", and the MCP server's environment has "CLAUDE_CODE_SESSION_ATTENDED" set to "1")$"#
)]
fn deployment_configured(w: &mut World, how: String) -> Result<(), String> {
    if how.starts_with("no deployment mode") {
        let git = w.home.join(".git");
        fs::create_dir_all(&git).map_err(|e| e.to_string())?;
        let path = w.home.join(".parinn.toml");
        return fs::write(&path, "[node]\ndeployment_mode = \"interactive\"\n")
            .map_err(|e| format!("{}: {e}", path.display()));
    }
    interactive(w)?;
    if how.ends_with("set to \"1\"") {
        // Every command after this step starts with the mark, as the
        // MCP server inherits it; each hook handler gets its own.
        set_env(w, "CLAUDE_CODE_SESSION_ATTENDED", "1");
    }
    Ok(())
}

/// The run of `SESSION`: its `SessionStart` hook handler, then its
/// `UserPromptSubmit` hook handler for `prompt`, attended or not, then
/// its transcript holding `line`, ingested.
fn user_turn(
    w: &mut World,
    prompt: &str,
    attended: bool,
    line: &serde_json::Value,
) -> Result<(), String> {
    let start = session_start(w, SESSION, "startup")?;
    run_hook(w, "SessionStart", &start, &[attended_mark(attended)])?;
    let submit = prompt_submit(w, SESSION, PROMPT_ID, prompt)?;
    run_hook(w, "UserPromptSubmit", &submit, &[attended_mark(attended)])?;
    let path = session_path(w, SESSION)?;
    fs::write(&path, format!("{line}\n")).map_err(|e| e.to_string())?;
    ingest(w, &path)
}

#[when(
    regex = r#"^this node's capture records the "user" event of a user turn, matched to the prompt its "UserPromptSubmit" hook handler got (with|without) the attended mark$"#
)]
fn capture_records_user_turn(w: &mut World, mark: String) -> Result<(), String> {
    let line = typed_line(PROMPT, PROMPT_ID, "typed", SESSION)?;
    user_turn(w, PROMPT, mark == "with", &line)
}

#[then(
    regex = r#"^the counter "harness_session_not_attended" increases by 1 and an audit entry names it$"#
)]
fn not_attended_counted(w: &mut World) -> Result<(), String> {
    match counted(w, "harness_session_not_attended")? {
        1 => Ok(()),
        n => Err(format!(
            "harness_session_not_attended was counted {n} times"
        )),
    }
}

#[then(regex = r"^no audit entry names the deployment mode$")]
fn no_entry_names_the_mode(w: &mut World) -> Result<(), String> {
    let entries = audit::log::Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    let named: Vec<String> = entries
        .into_iter()
        .filter(|e| {
            e.detail.contains("deployment_mode")
                || e.counter.as_deref() == Some("harness_session_not_attended")
        })
        .map(|e| e.detail)
        .collect();
    if named.is_empty() {
        return Ok(());
    }
    Err(format!("audit entries name the deployment mode: {named:?}"))
}

#[when(
    regex = r#"^(a "user" line carrying a prompt typed at the harness's input, matched, while a run on the node leaves R8 open|a "user" line typed at the input whose text matches no prompt a "UserPromptSubmit" hook handler of its harness session got|a "user" line whose promptSource is "sdk", matched to its prompt commitment, in deployment mode "interactive"|a "user" line of a harness session a run's Bash tool call started) is recorded$"#
)]
fn observed_item_recorded(w: &mut World, item: String) -> Result<(), String> {
    if item.contains("Bash tool call started") {
        let start = session_start(w, SESSION, "startup")?;
        run_nested_hook(w, "SessionStart", &start)?;
        let path = session_path(w, SESSION)?;
        let line = typed_line(PROMPT, PROMPT_ID, "typed", SESSION)?;
        fs::write(&path, format!("{line}\n")).map_err(|e| e.to_string())?;
        return ingest(w, &path);
    }
    let (prompt, source) = if item.contains("matches no prompt") {
        ("A prompt the line does not carry.", "typed")
    } else if item.contains("\"sdk\"") {
        (PROMPT, "sdk")
    } else {
        (PROMPT, "typed")
    };
    // No run on the node has a signed sandbox state: every run leaves R3
    // and R8 open (OWN-22, OQ-40).
    let line = typed_line(PROMPT, PROMPT_ID, source, SESSION)?;
    user_turn(w, prompt, false, &line)
}
