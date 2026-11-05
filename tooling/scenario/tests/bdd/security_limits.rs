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
use serde_json::Value;

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
            ingest(w, &path)
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

/// What the audit entry of a line over the limit says (SEC-05).
const LINE_TOO_LARGE_ENTRY: &str = "transcript line over the 4 MiB limit stored unparsed and redacted whole without being held in memory; its place is recorded (SEC-05)";

#[then(
    regex = r#"^the line is stored unparsed and redacted whole without being held in memory, an audit entry of kind redacted records "transcript line over the 4 MiB limit stored unparsed and redacted whole without being held in memory; its place is recorded \(SEC-05\)", the counter `transcript_line_too_large` counts it, its place is recorded, and every later line of the source is recorded$"#
)]
fn line_stored_unparsed(w: &mut World) -> Result<(), String> {
    let entries = Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    let counted = entries
        .iter()
        .filter(|e| {
            e.kind == "redacted"
                && e.counter.as_deref() == Some("transcript_line_too_large")
                && e.detail.starts_with(LINE_TOO_LARGE_ENTRY)
        })
        .count();
    if counted == 0 {
        return Err(format!(
            "no redacted entry counts the line among {}",
            entries.len()
        ));
    }
    // Nothing of the line's bytes is in the home: it was never held, and
    // its event holds a whole-line redaction marker.
    if let Some(found) = found_in_home(w, &[vec![b'x'; 1000]])? {
        return Err(format!("the line's bytes are stored: {found}"));
    }
    let at = w.section::<Limits>().refused_at.ok_or("no refused line")?;
    let conn = Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    let placed: i64 = conn
        .query_row(
            "SELECT count(*) FROM transcript_purged_copies WHERE byte_offset = ?1 AND byte_length = ?2",
            [at as i64, (LINE_LIMIT + 2) as i64],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if placed != 1 {
        return Err(format!("no place is recorded for the line at offset {at}"));
    }
    // Every line but the refused one is recorded, the one after it too.
    let lines = w.section::<Limits>().lines as i64;
    let (recorded, position): (i64, i64) = conn
        .query_row(
            "SELECT lines, byte_offset FROM transcript_sources",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    let stored: i64 = conn
        .query_row("SELECT count(*) FROM transcript_places", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if recorded != lines || stored != lines {
        return Err(format!(
            "{recorded} lines are ingested and {stored} places hold an event, of {lines} lines"
        ));
    }
    // The line's event is an unparsed line holding the redaction marker,
    // recoverable by its address where the line stood.
    let (kind, content): (String, String) = conn
        .query_row(
            "SELECT e.kind, e.content FROM transcript_places p JOIN events e ON e.rowid = p.event WHERE p.byte_offset = ?1",
            [at as i64],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    if kind != "unparsed_line" || !content.contains("[REDACTED:transcript_line_too_large]") {
        return Err(format!(
            "the line's event is {kind} holding no redaction marker"
        ));
    }
    if position <= at as i64 + LINE_LIMIT as i64 {
        return Err(format!("the source stays at offset {position}"));
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
    // One report a window, each resumed at the ingest marker.
    let out = last_run(w)?;
    let reports: Vec<Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    if reports.len() < 2 {
        return Err(format!("{} reads of a source over 64 MiB", reports.len()));
    }
    let events: u64 = reports
        .iter()
        .map(|r| r["events"].as_u64().unwrap_or(0))
        .sum();
    if events != lines {
        return Err(format!("the reads recorded {events} of {lines} lines"));
    }
    for report in &reports {
        let held = report["read_window_high_water"]
            .as_u64()
            .ok_or("the ingest report names no read-window high-water mark")?;
        if held == 0 || held > (64 * MIB) as u64 {
            return Err(format!("ingest held {held} bytes at once"));
        }
    }
    Ok(())
}
