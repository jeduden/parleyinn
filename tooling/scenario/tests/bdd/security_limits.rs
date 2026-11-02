//! The steps of SEC-05: a hook input past its limits is refused, and a
//! transcript line is redacted whole and held to its ceilings. The
//! transcript rows write the run's transcript under the scenario's
//! default transcript root, then run `parinn ingest` on it: a line past
//! the payload threshold, one of 4 MiB plus one byte, and a source of
//! more than 64 MiB.

use std::fs;
use std::path::PathBuf;

use audit::log::Log;
use cucumber::{then, when};
use rusqlite::Connection;
use serde_json::{Value, json};

use crate::World;
use crate::record::{home, last_run, run_core};
use crate::security_redaction::{agent_run_with_transcript, found_in_home, transcript};

/// An AWS access key, as AWS documents an example one.
const AWS_ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
/// The line ceiling, and the source ceiling, of SEC-05.
const LINE_LIMIT: usize = 4 << 20;
const MIB: usize = 1 << 20;

/// What the SEC-05 steps keep.
#[derive(Debug, Default)]
pub struct Limits {
    /// The byte offset the refused line starts at, and how many whole
    /// lines of the transcript there are.
    refused_at: Option<u64>,
    lines: usize,
    hook: bool,
}

/// A hook input that is `bytes` long: a JSON object with one string.
fn hook_input_of(bytes: usize) -> Vec<u8> {
    let mut input = b"{\"prompt\":\"".to_vec();
    input.resize(bytes - 2, b'x');
    input.extend_from_slice(b"\"}");
    input
}

/// A transcript line of `bytes` bytes without its line break: a JSON
/// object a transcript reader takes as one line.
fn line_of(bytes: usize, text: &str) -> String {
    let head = format!("{{\"type\":\"atis-latch\",\"sessionId\":\"s\",\"atis\":\"{text}");
    let tail = "\"}";
    let fill = bytes - head.len() - tail.len();

    format!("{head}{}{tail}", "x".repeat(fill))
}

/// The transcript of the run "run-a", its `lines` written.
fn write_transcript(w: &mut World, lines: &[String]) -> Result<PathBuf, String> {
    agent_run_with_transcript(w, "run-a".to_owned())?;
    let path = transcript(w, "run-a")?;
    fs::write(&path, lines.join("\n") + "\n").map_err(|e| e.to_string())?;

    Ok(path)
}

/// `parinn ingest` on the transcript at `path`, as JSON.
fn ingest(w: &mut World, path: &std::path::Path) -> Result<(), String> {
    let shown = path.display().to_string();
    run_core(w, &["ingest", "--path", &shown, "--json"], None)
}

#[when(
    regex = r#"^the hook "([A-Za-z]+)" runs, or "parinn ingest" reads a transcript, with (stdin of 1 MiB plus one byte|a JSON object nested 65 levels deep|a transcript line larger than the payload threshold|a transcript line of 4 MiB plus one byte|a transcript source of more than 64 MiB)$"#
)]
fn runs_with(w: &mut World, hook: String, input: String) -> Result<(), String> {
    let limits = w.section::<Limits>();
    *limits = Limits::default();
    match input.as_str() {
        "stdin of 1 MiB plus one byte" => {
            w.section::<Limits>().hook = true;
            run_core(w, &["hook", &hook], Some(&hook_input_of((1 << 20) + 1)))
        }
        "a JSON object nested 65 levels deep" => {
            w.section::<Limits>().hook = true;
            let nested = format!("{}1{}", "{\"a\":".repeat(65), "}".repeat(65));
            run_core(w, &["hook", &hook], Some(nested.as_bytes()))
        }
        "a transcript line larger than the payload threshold" => {
            let key = format!("export AWS_ACCESS_KEY_ID={AWS_ACCESS_KEY} ");
            let path = write_transcript(w, &[line_of(20_000, &key)])?;
            ingest(w, &path)
        }
        "a transcript line of 4 MiB plus one byte" => {
            let first = line_of(100, "before");
            let over = "x".repeat(LINE_LIMIT + 1);
            let after = line_of(100, "after");
            let path = write_transcript(w, &[first.clone(), over, after])?;
            let limits = w.section::<Limits>();
            limits.refused_at = Some(first.len() as u64 + 1);
            limits.lines = 3;
            ingest(w, &path)
        }
        _ => {
            // 66 lines of 1 MiB: 66 MiB, past the 64 MiB a read holds.
            let lines: Vec<String> = (0..66).map(|_| line_of(MIB, "bulk")).collect();
            let path = write_transcript(w, &lines)?;
            w.section::<Limits>().lines = lines.len();
            for _ in 0..4 {
                ingest(w, &path)?;
            }
            Ok(())
        }
    }
}

#[then(regex = r"^no injection is written, and a hook handler exits 0$")]
fn no_injection_and_exit_0(w: &mut World) -> Result<(), String> {
    let hook = w.section::<Limits>().hook;
    let out = last_run(w)?;
    if hook && !(out.status.success() && out.stdout.is_empty() && out.stderr.is_empty()) {
        return Err(format!(
            "the hook exited {} with stdout {:?} and stderr {:?}",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    if !hook && !out.status.success() {
        return Err(format!(
            "parinn ingest exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

#[then(
    regex = r"^the line is redacted whole, then stored in the payload store, and nothing of it is written unredacted$"
)]
fn redacted_whole_then_stored(w: &mut World) -> Result<(), String> {
    if let Some(found) = found_in_home(w, &[AWS_ACCESS_KEY.as_bytes().to_vec()])? {
        return Err(format!("the key stands unredacted: {found}"));
    }
    let marker = b"[REDACTED:aws_access_key]".to_vec();
    let payloads = home(w)?.root().join("payloads");
    let stored = fs::read_dir(&payloads)
        .map_err(|e| format!("no payload store: {e}"))?
        .filter_map(Result::ok)
        .filter_map(|e| fs::read(e.path()).ok())
        .any(|bytes| bytes.windows(marker.len()).any(|w| w == marker.as_slice()));
    if !stored {
        return Err("the payload store holds no redacted line".to_owned());
    }
    Ok(())
}

/// The transcript source's recorded ingest position, in bytes.
fn position(w: &mut World) -> Result<i64, String> {
    let conn = Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    conn.query_row("SELECT byte_offset FROM transcript_sources", [], |r| {
        r.get(0)
    })
    .map_err(|e| e.to_string())
}

#[then(
    regex = r#"^the line is refused, an audit entry of kind rejected records "transcript line refused over the 4 MiB limit; ingest position stays before the line \(SEC-05, REC-03\)", the counter `transcript_line_too_large` counts it, and the source is left short at that line, resumable$"#
)]
fn line_refused(w: &mut World) -> Result<(), String> {
    let entries = Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    let counted = entries
        .iter()
        .filter(|e| {
            e.kind == "rejected"
                && e.counter.as_deref() == Some("transcript_line_too_large")
                && e.detail.starts_with(
                    "transcript line refused over the 4 MiB limit; ingest position stays before the line (SEC-05, REC-03)",
                )
        })
        .count();
    if counted == 0 {
        return Err(format!(
            "no rejected entry counts the line among {}",
            entries.len()
        ));
    }
    let at = w.section::<Limits>().refused_at.ok_or("no refused line")?;
    if position(w)? != at as i64 {
        return Err(format!("the source is not short at offset {at}"));
    }
    run_core(w, &["status", "--json"], None)?;
    let out = last_run(w)?;
    if !String::from_utf8_lossy(&out.stdout).contains("transcript_line_too_large") {
        return Err("parinn status does not show transcript_line_too_large".to_owned());
    }
    Ok(())
}

#[then(
    regex = r"^ingest holds at most 64 MiB of it in memory at once and ingests every line of it across resumes at its ingest marker \(REC-13\)$"
)]
fn bounded_and_resumed(w: &mut World) -> Result<(), String> {
    let lines = w.section::<Limits>().lines as u64;
    let conn = Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    let recorded: i64 = conn
        .query_row("SELECT lines FROM transcript_sources", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if recorded as u64 != lines {
        return Err(format!("{recorded} of {lines} lines are ingested"));
    }
    let out = last_run(w)?;
    let report: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let held = report["read_window_high_water"].as_u64().ok_or_else(|| {
        json!({"finding": "the ingest report names no read-window high-water mark"}).to_string()
    })?;
    if held > (64 * MIB) as u64 {
        return Err(format!("ingest held {held} bytes at once"));
    }
    Ok(())
}
