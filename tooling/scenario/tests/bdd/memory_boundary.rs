//! The steps of SRS §5.9, the boundary to durable memory (MEM), that
//! phase 11 of plan 2609292012 carries. `@MEM-01`: every hook handler
//! runs over a transcript carrying a canary, the transcript is ingested
//! and `parinn rebuild` derives again; no file outside the store holds
//! the canary, and no memory, summary or cross-room store exists under
//! the ParleyInn home or `HOME`.

use std::fs;
use std::path::{Path, PathBuf};

use cucumber::{then, when};
use serde_json::json;

use crate::World;
use crate::record::{last_run, run_core};
use crate::record_hooks::{prompt_submit, run_hook, session_start, typed_line};
use crate::security_redaction::{append, transcript};

/// The text the canary transcript carries.
const CANARY: &str = "MEMORY-CANARY-9d2f41c7";

/// The harness session the canary transcript belongs to.
const SESSION: &str = "memory-canary";

/// The names of a store of long-term memory, a summary or a
/// cross-room store, which ParleyInn keeps none of (MEM-01).
const MEMORY_NAMES: [&str; 6] = [
    "memory",
    "memories",
    "summary",
    "summaries",
    "cross_room",
    "cross-room",
];

/// The files the store is: the record, its search index and its
/// payloads, in the ParleyInn home.
fn is_store(home: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(home) else {
        return false;
    };
    let first = rel
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .unwrap_or_default();

    first.starts_with("store.db") || first.starts_with("search_index") || first == "payloads"
}

/// Every file under `dir`, with its path.
fn files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            files(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

#[when(
    regex = r#"^every hook runs over the whole transcript and the person runs "parinn rebuild"$"#
)]
fn hooks_then_rebuild(w: &mut World) -> Result<(), String> {
    let typed = typed_line(CANARY, "p-1", "typed", SESSION)?;
    append(w, SESSION, &typed)?;
    append(
        w,
        SESSION,
        &json!({"type": "assistant", "message": {"role": "assistant", "content": [
            {"type": "text", "text": CANARY},
        ]}, "sessionId": SESSION}),
    )?;
    let path = transcript(w, SESSION)?;
    run_hook(
        w,
        "SessionStart",
        &session_start(w, SESSION, "startup")?,
        &[],
    )?;
    run_hook(
        w,
        "UserPromptSubmit",
        &prompt_submit(w, SESSION, "p-1", CANARY)?,
        &[],
    )?;
    let permission = json!({
        "session_id": SESSION,
        "transcript_path": path,
        "cwd": "/w/app",
        "hook_event_name": "PermissionRequest",
        "tool_name": "Bash",
        "tool_input": {"command": format!("echo {CANARY}")},
    });
    run_hook(w, "PermissionRequest", &permission, &[])?;
    run_core(w, &["ingest", "--all"], None)?;
    ok(w, "ingest")?;
    run_core(w, &["rebuild"], None)?;
    ok(w, "rebuild")
}

/// Fails unless the last command exited 0.
fn ok(w: &mut World, command: &str) -> Result<(), String> {
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "parinn {command} exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    ))
}

#[then(regex = r"^no file outside the ParleyInn home's store contains the canary text$")]
fn canary_only_in_the_store(w: &mut World) -> Result<(), String> {
    let source = transcript(w, SESSION)?;
    let mut all = Vec::new();
    files(&w.home, &mut all)?;
    // The control: the record does hold the canary, so a scan that finds
    // none elsewhere is not one that reads nothing.
    let mut in_store = false;
    for path in all.iter().filter(|p| is_store(&w.parinn_home, p)) {
        let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        in_store |= bytes.windows(CANARY.len()).any(|b| b == CANARY.as_bytes());
    }
    if !in_store {
        return Err("the store holds no canary: the run recorded nothing".to_owned());
    }
    for path in all {
        if path == source || is_store(&w.parinn_home, &path) {
            continue;
        }
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if bytes
            .windows(CANARY.len())
            .any(|window| window == CANARY.as_bytes())
        {
            return Err(format!("{} holds the canary", path.display()));
        }
    }
    Ok(())
}

#[then(
    regex = r"^no memory, summary or cross-room store under the ParleyInn home or HOME contains the canary text$"
)]
fn no_memory_store(w: &mut World) -> Result<(), String> {
    let mut all = Vec::new();
    files(&w.home, &mut all)?;
    let named = |path: &PathBuf| {
        path.strip_prefix(&w.home)
            .unwrap_or(path)
            .components()
            // The harness's own configuration is not ParleyInn's.
            .skip_while(|c| c.as_os_str() == ".claude")
            .any(|c| {
                let name = c.as_os_str().to_string_lossy().to_lowercase();
                MEMORY_NAMES.iter().any(|m| name.contains(m))
            })
    };
    if let Some(found) = all
        .iter()
        .filter(|p| !p.starts_with(w.home.join(".claude")))
        .find(|p| named(p))
    {
        return Err(format!("{} is a memory store", found.display()));
    }
    // The harness keeps memory of its own under HOME/.claude; none of it
    // holds the canary.
    let source = transcript(w, SESSION)?;
    for path in all
        .iter()
        .filter(|p| p.starts_with(w.home.join(".claude")) && **p != source)
    {
        let lower = path.to_string_lossy().to_lowercase();
        if !MEMORY_NAMES.iter().any(|m| lower.contains(m)) {
            continue;
        }
        let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        if bytes
            .windows(CANARY.len())
            .any(|window| window == CANARY.as_bytes())
        {
            return Err(format!("{} holds the canary", path.display()));
        }
    }

    Ok(())
}
