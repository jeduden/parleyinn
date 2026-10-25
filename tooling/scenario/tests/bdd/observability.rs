//! The steps of SRS §5.10, observability (OPS). `@OPS-02` and
//! `@OPS-03` are bound: the audit log's chain across a rotation, and
//! `parinn doctor` failing while a counter rose since the counters were
//! last acknowledged. A hook a step runs is the built core's `parinn
//! hook`, confined, with the hook input on its stdin.

use std::fs;
use std::io::Write as _;
use std::path::PathBuf;

use ::record::canonical;
use ::record::digest::Digest;
use audit::entry::{Counter, Entry, Kind, Occurrence};
use audit::log::{Log, ROTATE_AT};
use cucumber::{given, then, when};
use serde_json::Value;

use crate::World;
use crate::record::{first_start, home, last_run, run_core, run_core_at_terminal};

/// A recorded Claude Code 2.1.296 `SessionStart` hook input.
const SESSION_START: &str = "testdata/claude-code/2.1.296/hooks/clear/01-SessionStart.json";

/// A hook input cut off inside its first member: no JSON object.
const MALFORMED: &str = "{\"session_id\": ";

/// What a filled audit file's entries report.
const FILLER: Occurrence = Occurrence::Degraded {
    kind: Kind::Rejected,
    counter: Counter::HookRejected,
    operation: "hook UserPromptSubmit",
    detail: "not a JSON object",
};

#[given(regex = r"^an audit log that has reached its rotation size$")]
fn audit_log_at_rotation_size(w: &mut World) -> Result<(), String> {
    let home = first_start(w)?;
    let log = Log::new(home.audit());
    let mut last = log.append(&FILLER, 0).map_err(|e| e.to_string())?;
    let path = log.file(1);
    let mut text = String::new();
    let mut size = fs::metadata(&path).map_err(|e| e.to_string())?.len();
    while size < ROTATE_AT {
        let e = Entry::next(&FILLER, last.number + 1, 0, last.hash).map_err(|e| e.to_string())?;
        let line = format!("{}\n", e.line().map_err(|e| e.to_string())?);
        size += line.len() as u64;
        text.push_str(&line);
        last = e;
    }
    fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .and_then(|mut f| f.write_all(text.as_bytes()))
        .map_err(|e| e.to_string())?;
    if log.file(2).exists() {
        return Err("the audit log has rotated already".to_owned());
    }
    Ok(())
}

/// The audit file at `name`, relative to the home.
fn audit_file(w: &World, name: &str) -> Result<PathBuf, String> {
    Ok(home(w)?.root().join(name))
}

/// Each line of the file at `path`.
fn lines(path: &PathBuf) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(text.lines().map(str::to_owned).collect())
}

#[then(regex = r#"^the file "(audit/[^"]+)" is created with mode (0[0-7]{3})$"#)]
fn created_with_mode(w: &mut World, name: String, mode: String) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let path = audit_file(w, &name)?;
    let meta = fs::metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let found = meta.permissions().mode() & 0o777;
    if format!("{found:04o}") == mode {
        return Ok(());
    }
    Err(format!("{name} has mode {found:04o}, not {mode}"))
}

#[then(
    regex = r#"^its first entry's previous hash equals the hash of the last entry in "(audit/[^"]+)"$"#
)]
fn first_chains_to_last(w: &mut World, before: String) -> Result<(), String> {
    let next = audit_file(w, "audit/audit-000002.jsonl")?;
    let first = lines(&next)?
        .into_iter()
        .next()
        .ok_or("the new audit file holds no entry")?;
    let last = lines(&audit_file(w, &before)?)?
        .pop()
        .ok_or("the full audit file holds no entry")?;
    let (first, last) = (
        Entry::parse(&first).map_err(str::to_owned)?,
        Entry::parse(&last).map_err(str::to_owned)?,
    );
    if first.prev_hash == last.hash && first.number == last.number + 1 {
        return Ok(());
    }
    Err(format!(
        "entry {} does not chain to entry {}, the last in {before}",
        first.number, last.number
    ))
}

#[then(
    regex = r#"^every entry is RFC 8785 canonical JSON whose SHA-256 chain verifies with "parinn verify"$"#
)]
fn every_entry_canonical_and_chained(w: &mut World) -> Result<(), String> {
    let log = Log::new(home(w)?.audit());
    let mut prev = Digest::ZERO;
    let mut count = 0_u64;
    for n in 1.. {
        let path = log.file(n);
        if !path.exists() {
            break;
        }
        for line in lines(&path)? {
            let mut value: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
            if canonical::encode(&value).map_err(|e| e.to_string())? != line {
                return Err(format!(
                    "an entry of {} is not canonical JSON",
                    path.display()
                ));
            }
            let hash = value
                .as_object_mut()
                .and_then(|o| o.remove("hash"))
                .and_then(|h| h.as_str().map(str::to_owned))
                .ok_or("an entry has no hash")?;
            if value["prev_hash"].as_str() != Some(prev.to_hex().as_str()) {
                return Err(format!(
                    "entry {} does not chain to the one before",
                    value["entry"]
                ));
            }
            let body = canonical::encode(&value).map_err(|e| e.to_string())?;
            let want = Digest::of(&[body.as_bytes()]);
            if want.to_hex() != hash {
                return Err(format!(
                    "entry {}'s hash is not SHA-256 over the rest",
                    value["entry"]
                ));
            }
            prev = want;
            count += 1;
        }
    }
    run_core(w, &["verify", "--json"], None)?;
    let out = last_run(w)?;
    if !out.status.success() {
        return Err(format!(
            "parinn verify exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let report: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    if report["audit"]["entries"] == serde_json::json!(count) && count > 1 {
        return Ok(());
    }
    Err(format!(
        "parinn verify checked {} audit entries, not the {count} the log holds",
        report["audit"]["entries"]
    ))
}

#[given(
    regex = r#"^the person has acknowledged the counters at a terminal with "parinn counter ack", a neutral principal act$"#
)]
fn acknowledged(w: &mut World) -> Result<(), String> {
    run_core_at_terminal(w, &["counter", "ack"])?;
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "parinn counter ack exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout).trim()
    ))
}

#[when(regex = r#"^the hook "([A-Za-z]+)" runs with (a malformed JSON object|a valid input)$"#)]
fn hook_runs(w: &mut World, hook: String, input: String) -> Result<(), String> {
    let input = if input == "a valid input" {
        let path = testkit::repo_root().join(SESSION_START);
        fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?
    } else {
        MALFORMED.as_bytes().to_vec()
    };
    run_core(w, &["hook", &hook], Some(&input))
}
