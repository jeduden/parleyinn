//! The steps of REC-09, REC-12, REC-19 and ADM-05 that phase 6 of plan
//! 2609292012 carries: content above the payload threshold goes to the
//! payload store under a name that confirms no guess, transcript lines
//! are appended in transcript order with their transcript timestamps
//! kept as metadata, `parinn verify` and `parinn status` report each
//! writer chain ended without a closed segment, and `parinn status`
//! exits 3 on a store of a newer schema version, leaving it unchanged.
//!
//! `@REC-12` stays pending: its last step runs `parinn event expand`,
//! the principal's recall (phase 13). `@REC-19`'s other steps need the
//! live MCP server (phase 10) and seals (M7); `@ADM-05`'s `parinn
//! migrate` and its backup are M3's. Those steps are unbound.

use std::fs;
use std::path::{Path, PathBuf};

use ::record::commitment::commit;
use ::record::digest::Digest;
use ::record::id::WriterId;
use cucumber::{given, then};
use key::KeyError;
use key::random::Entropy;
use serde_json::{Value, json};
use store::payload::{self, Disk, OsDisk};
use store::{NewEvent, Store, Stored};
use testkit::TempDir;

use crate::World;
use crate::record::{first_start, home, last_run, run_core};
use crate::record_transcript::{recorded_call, transcript_events};
use crate::security_redaction::{append, transcript};

/// What the store steps track: the marker each sized tool result's
/// output repeats, by its size in KiB.
#[derive(Debug, Default)]
pub struct StoreSteps {
    markers: Vec<(u64, String)>,
    /// The bytes of `store.db` before the step that must leave it be.
    before: Option<Vec<u8>>,
    /// The writer a step left with an open segment.
    open: Option<WriterId>,
}

fn steps(w: &mut World) -> &mut StoreSteps {
    w.section::<StoreSteps>()
}

/// The text of a tool result `kib` KiB long, made of `marker`.
fn output(marker: &str, kib: u64) -> String {
    let mut text = String::new();
    while (text.len() as u64) < kib << 10 {
        text.push_str(marker);
        text.push(' ');
    }
    text.truncate((kib << 10) as usize);
    text
}

#[given(
    regex = r#"^"([^"]+)" contains a Bash tool result of (\d+) KiB and one of (\d+) KiB, against the default 8 KiB threshold$"#
)]
fn sized_results(w: &mut World, name: String, big: u64, small: u64) -> Result<(), String> {
    let (call, result) = recorded_call("Bash")?;
    for kib in [big, small] {
        let marker = format!("payload-{kib}-kib");
        let mut line = result.clone();
        line["message"]["content"][0]["content"] = json!(output(&marker, kib));
        if let Some(r) = line.get_mut("toolUseResult") {
            *r = json!({"stdout": "", "stderr": "", "interrupted": false});
        }
        append(w, &name, &call)?;
        append(w, &name, &line)?;
        steps(w).markers.push((kib, marker));
    }
    Ok(())
}

/// The event holding the tool result of `kib` KiB, with its writer.
fn sized_event(w: &mut World, kib: u64) -> Result<(WriterId, Stored), String> {
    let marker = steps(w)
        .markers
        .iter()
        .find(|(k, _)| *k == kib)
        .map(|(_, m)| m.clone())
        .ok_or_else(|| format!("no step added a tool result of {kib} KiB"))?;
    transcript_events(w)?
        .into_iter()
        .find(|(_, e)| {
            e.content
                .as_ref()
                .and_then(|c| c["text"].as_str())
                .is_some_and(|t| t.contains(&marker))
        })
        .ok_or_else(|| format!("no event holds the tool result of {kib} KiB"))
}

/// The unkeyed digests a reader off this node could compute from a
/// guess at `content`.
fn guesses(content: &Value) -> Result<Vec<String>, String> {
    let canonical = ::record::canonical::encode(content).map_err(|e| e.to_string())?;
    let text = content["text"].as_str().unwrap_or_default();
    Ok([canonical.as_bytes(), text.as_bytes()]
        .iter()
        .map(|b| Digest::of(&[b]).to_hex())
        .collect())
}

#[then(
    regex = r"^the (\d+) KiB content is stored in the payload store under a name from which no one off this node can confirm a guess at the content, and its event carries a preview of at most 512 bytes, the payload reference and the commitment$"
)]
fn stored_as_payload(w: &mut World, kib: u64) -> Result<(), String> {
    let (_, event) = sized_event(w, kib)?;
    let reference = event
        .payload
        .clone()
        .ok_or("the event keeps its content inline")?;
    let content = event.content.clone().ok_or("the event holds no content")?;
    let key = event.commitment_key.clone().ok_or("no commitment key")?;
    let file = home(w)?.root().join("payloads").join(&reference.name);
    let held = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    let canonical = ::record::canonical::encode(&content).map_err(|e| e.to_string())?;
    let checks = [
        (
            held == canonical,
            "the payload is the content's canonical encoding",
        ),
        (
            reference.name == payload::name(&key),
            "the name is keyed by the event's commitment key",
        ),
        (
            !guesses(&content)?.contains(&reference.name),
            "the name is no digest of the content",
        ),
        (
            reference.preview.len() <= 512,
            "the preview is at most 512 bytes",
        ),
        (
            reference.size == canonical.len() as u64,
            "the reference names its size",
        ),
        (
            commit(&key, &content).is_ok_and(|c| c == event.link.header.commitment),
            "the commitment covers the whole content",
        ),
    ];
    match checks.iter().find(|(ok, _)| !ok) {
        None => Ok(()),
        Some((_, what)) => Err(format!("not so: {what}")),
    }
}

#[then(regex = r"^the (\d+) KiB content is stored inline in its event$")]
fn stored_inline(w: &mut World, kib: u64) -> Result<(), String> {
    let (_, event) = sized_event(w, kib)?;
    if event.payload.is_none() && event.content.is_some() {
        return Ok(());
    }
    Err(format!("the {kib} KiB content went to the payload store"))
}

#[then(
    regex = r"^nothing in an export lets a reader off this node confirm a guess at the content from a payload name$"
)]
fn no_guess_from_a_name(w: &mut World) -> Result<(), String> {
    // M1 has no export; what one would carry of a payload is its name.
    // Ingesting the same content again must give it another name, and
    // no name may be a digest a guess at the content gives. A source
    // read again records nothing new (REC-03), so the same lines are
    // ingested as another transcript, in a directory of their own.
    let (_, first) = sized_event(w, 20)?;
    let source = transcript(w, "main-run")?;
    let dir = w.home.join(".claude").join("projects").join("-w-copy");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let copy = dir.join("main-run.jsonl");
    std::fs::copy(&source, &copy).map_err(|e| e.to_string())?;
    let path = copy.display().to_string();
    run_core(w, &["ingest", "--path", &path], None)?;
    let named: Vec<(String, Value)> = transcript_events(w)?
        .into_iter()
        .filter_map(|(_, e)| Some((e.payload?.name, e.content?)))
        .filter(|(_, c)| Some(c) == first.content.as_ref())
        .collect();
    if named.len() != 2 || named[0].0 == named[1].0 {
        return Err(format!(
            "the same content ingested twice holds the names {:?}",
            named.iter().map(|(n, _)| n).collect::<Vec<_>>()
        ));
    }
    for (name, content) in &named {
        if guesses(content)?.contains(name) {
            return Err(format!(
                "the payload name {name} is a digest of the content"
            ));
        }
    }
    Ok(())
}

/// A disk that records each payload operation, then makes it.
#[derive(Clone, Default)]
struct Recording(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

impl Recording {
    fn note(&self, what: String) {
        if let Ok(mut ops) = self.0.lock() {
            ops.push(what);
        }
    }
}

/// A path's last component.
fn last(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

impl Disk for Recording {
    fn write_synced(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        self.note(format!("write and fsync {}", last(path)));
        OsDisk.write_synced(path, bytes)
    }
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        self.note(format!("rename {} to {}", last(from), last(to)));
        OsDisk.rename(from, to)
    }
    fn sync_dir(&self, dir: &Path) -> std::io::Result<()> {
        self.note(format!("fsync {}", last(dir)));
        OsDisk.sync_dir(dir)
    }
    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        OsDisk.read(path)
    }
}

#[then(regex = r"^each payload was written to a temporary file, fsynced, then renamed into place$")]
fn written_atomically(w: &mut World) -> Result<(), String> {
    let dir = home(w)?.root().join("payloads");
    let left: Vec<String> = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .filter(|n| n.starts_with('.'))
        .collect();
    if !left.is_empty() {
        return Err(format!("the payload store keeps temporary files {left:?}"));
    }
    // The steps the store takes for the same content, recorded through
    // the disk it writes with, in a scratch store.
    let (_, event) = sized_event(w, 20)?;
    let scratch = TempDir::new().map_err(|e| e.to_string())?;
    let ops = Recording::default();
    let mut store = Store::open(&scratch.path().join("store.db"))
        .map_err(|e| e.to_string())?
        .with_disk(Box::new(ops.clone()));
    let key = event.commitment_key.clone().ok_or("no commitment key")?;
    let name = payload::name(&key);
    let new = NewEvent {
        kind: event.link.header.kind,
        provenance: event.link.header.provenance.clone(),
        content: event.content.clone().ok_or("no content")?,
        commitment_key: key,
        transcript_timestamp: None,
    };
    store
        .append(WriterId::derive(&[1; 32]), &[new])
        .map_err(|e| e.to_string())?;
    let made = ops.0.lock().map_err(|e| e.to_string())?.clone();
    let want = [
        format!("write and fsync .tmp-{name}"),
        format!("rename .tmp-{name} to {name}"),
        "fsync payloads".to_owned(),
    ];
    if made == want {
        return Ok(());
    }
    Err(format!("the store wrote its payload by {made:?}"))
}

/// A transcript line of the run `n`, with `timestamp` when given.
fn timed_line(n: u64, timestamp: Option<&str>) -> Value {
    let mut line = json!({
        "type": "user",
        "message": {"role": "user", "content": format!("line {n}")},
        "uuid": format!("u{n}"),
    });
    if let Some(t) = timestamp {
        line["timestamp"] = json!(t);
    }
    line
}

/// The transcript timestamp the scenario gives line `n`: a minute apart,
/// line 11 before line 10.
fn timestamp_of(n: u64) -> Option<String> {
    let minute = match n {
        12 => return None,
        11 => 9,
        10 => 11,
        n => n,
    };
    Some(format!("2026-10-17T12:{minute:02}:00.000Z"))
}

#[given(regex = r#"^the timestamp of line 11 of "([^"]+)" is earlier than that of line 10$"#)]
fn out_of_order(w: &mut World, name: String) -> Result<(), String> {
    for n in 1..=11 {
        append(w, &name, &timed_line(n, timestamp_of(n).as_deref()))?;
    }
    Ok(())
}

#[given(regex = r#"^line 12 of "([^"]+)" carries no timestamp$"#)]
fn without_timestamp(w: &mut World, name: String) -> Result<(), String> {
    append(w, &name, &timed_line(12, None))
}

/// The events of the scenario's one transcript, in seq order, each with
/// the line number its content names.
fn lines(w: &World) -> Result<Vec<(u64, Stored)>, String> {
    let mut events = transcript_events(w)?;
    events.sort_by_key(|(_, e)| e.link.header.address.seq);
    events
        .into_iter()
        .map(|(_, e)| {
            let n = e
                .content
                .as_ref()
                .and_then(|c| c["line"].as_u64())
                .ok_or("an event names no line")?;
            Ok((n, e))
        })
        .collect()
}

#[then(regex = r#"^the events of "([^"]+)" have seq in the order of their transcript lines$"#)]
fn seq_in_line_order(w: &mut World, _name: String) -> Result<(), String> {
    let numbers: Vec<u64> = lines(w)?.iter().map(|(n, _)| *n).collect();
    if numbers == (1..=12).collect::<Vec<_>>() {
        return Ok(());
    }
    Err(format!("by seq, the events hold lines {numbers:?}"))
}

#[then(regex = r"^each event whose line carries a timestamp keeps it as metadata$")]
fn timestamps_kept(w: &mut World) -> Result<(), String> {
    for (n, event) in lines(w)? {
        if let Some(want) = timestamp_of(n)
            && event.transcript_timestamp.as_deref() != Some(want.as_str())
        {
            return Err(format!(
                "line {n} keeps {:?}, not {want}",
                event.transcript_timestamp
            ));
        }
    }
    Ok(())
}

#[then(regex = r"^the event of line (\d+) carries no transcript timestamp$")]
fn no_timestamp(w: &mut World, line: u64) -> Result<(), String> {
    let found = lines(w)?.into_iter().find(|(n, _)| *n == line);
    match found {
        Some((_, e)) if e.transcript_timestamp.is_none() => Ok(()),
        Some((_, e)) => Err(format!("line {line} carries {:?}", e.transcript_timestamp)),
        None => Err(format!("no event holds line {line}")),
    }
}

/// A seeded random source: the same seed gives the same keys, the same
/// commitment keys and so the same writer id and chain.
struct Seeded(u64);

impl Entropy for Seeded {
    fn fill(&mut self, buf: &mut [u8]) -> Result<(), KeyError> {
        for byte in buf {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            *byte = (self.0 >> 33) as u8;
        }
        Ok(())
    }
}

/// The writer id and chain head an ingest of `transcript` gives in a
/// fresh home under `dir`, its random source seeded with `seed`.
fn seeded_ingest(dir: &Path, transcript: &Path, seed: u64) -> Result<(WriterId, Digest), String> {
    let home = administration::home::Home::locate(Some(dir.join(".parinn").as_os_str()), None)
        .map_err(|e| e.to_string())?;
    let redactor = redaction::Redactor::new(&[]).map_err(|e| e.to_string())?.0;
    let ingested = ingest::ingest_path(&home, transcript, &redactor, &mut Seeded(seed))
        .map_err(|e| e.to_string())?;
    let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let head = store
        .log(ingested.writer)
        .map_err(|e| e.to_string())?
        .last()
        .map(|s| s.link.hash)
        .ok_or("the ingest recorded nothing")?;
    Ok((ingested.writer, head))
}

#[then(
    regex = r#"^a second fresh home that ingests "([^"]+)" under the same writer id derives the same chain head hash$"#
)]
fn same_chain_head(w: &mut World, name: String) -> Result<(), String> {
    let path: PathBuf = transcript(w, &name)?;
    let (a, b) = (
        TempDir::new().map_err(|e| e.to_string())?,
        TempDir::new().map_err(|e| e.to_string())?,
    );
    let first = seeded_ingest(a.path(), &path, 7)?;
    let second = seeded_ingest(b.path(), &path, 7)?;
    if first == second {
        return Ok(());
    }
    Err(format!(
        "two fresh homes ingesting under writer ids {} and {} derive heads {} and {}",
        first.0, second.0, first.1, second.1
    ))
}

#[given(regex = r"^another writer whose chain ended without a closed segment$")]
fn ended_open(w: &mut World) -> Result<(), String> {
    let dir = w.home.join(".claude/projects/open");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("open.jsonl");
    fs::write(&path, "{\"type\":\"user\"}\n").map_err(|e| e.to_string())?;
    run_core(
        w,
        &["ingest", "--path", &path.display().to_string(), "--json"],
        None,
    )?;
    let report: Value = serde_json::from_slice(&last_run(w)?.stdout).map_err(|e| e.to_string())?;
    let writer = report["writer"]
        .as_str()
        .and_then(|t| WriterId::parse(t).ok())
        .ok_or("ingest named no writer")?;
    // The writer appends once more and stops before it closes its
    // segment, as a killed one would.
    let mut store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let new = NewEvent {
        kind: ::record::event::EventKind::UnparsedLine,
        provenance: ::record::event::Provenance::Unparsed,
        content: json!({"line": 2, "text": "cut short"}),
        commitment_key: ::record::commitment::CommitmentKey::from_bytes([9; 32]),
        transcript_timestamp: None,
    };
    store
        .append_open(writer, &[new])
        .map_err(|e| e.to_string())?;
    steps(w).open = Some(writer);
    Ok(())
}

#[then(
    regex = r#"^"parinn verify" and "parinn status" each report the other writer's chain as ended without a closed segment$"#
)]
fn reported_open(w: &mut World) -> Result<(), String> {
    let writer = steps(w)
        .open
        .ok_or("no step left a writer's segment open")?
        .to_string();
    for (args, pointer) in [
        (["verify", "--json"], "/open_segments"),
        (["status", "--json"], "/segments/open"),
    ] {
        run_core(w, &args, None)?;
        let out = last_run(w)?;
        let report: Value =
            serde_json::from_slice(&out.stdout).map_err(|e| format!("parinn {}: {e}", args[0]))?;
        let named = report
            .pointer(pointer)
            .and_then(Value::as_array)
            .is_some_and(|open| open.iter().any(|s| s["writer"] == writer.as_str()));
        if !out.status.success() || !named {
            return Err(format!("parinn {} reports {report}", args[0]));
        }
    }
    Ok(())
}

#[given(regex = r"^derived artifacts whose schema version is newer than this node supports$")]
fn newer_schema(w: &mut World) -> Result<(), String> {
    let home = first_start(w)?;
    let path = home.store();
    let conn = rusqlite::Connection::open(&path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "PRAGMA journal_mode = DELETE; UPDATE meta SET value = '2' WHERE key = 'schema_version'",
    )
    .map_err(|e| e.to_string())?;
    drop(conn);
    steps(w).before = Some(fs::read(&path).map_err(|e| e.to_string())?);
    Ok(())
}

#[given(regex = r"^a segment whose format version is newer than this node supports$")]
fn newer_segment(w: &mut World) -> Result<(), String> {
    let home = home(w)?;
    if !home.store().exists() {
        first_start(w)?;
    }
    let conn = rusqlite::Connection::open(home.store()).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE segments SET format_version = 2 WHERE first_seq = 1",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[then(regex = r"^the derived artifacts are unchanged$")]
fn unchanged(w: &mut World) -> Result<(), String> {
    let path = home(w)?.store();
    let before = steps(w)
        .before
        .clone()
        .ok_or("no step kept the store as it stood")?;
    if fs::read(&path).map_err(|e| e.to_string())? == before {
        return Ok(());
    }
    Err("the store changed".to_owned())
}
