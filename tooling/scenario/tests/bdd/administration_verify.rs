//! The steps of ADM-09: `parinn verify` exits 3 and names the failed
//! check for each kind of damage. Each damage is done to the home of an
//! agent's recorded run, by hand, as an attacker or a failing disk
//! would do it: a line the transcript gained, a row's hash, a payload
//! file's byte, an object in the store, the index's ranking or one of
//! its entries.
//!
//! The row that alters a landmark row cannot run in M1: no landmark is
//! recorded before M3 (LMK-01), so there is none to alter. Its step
//! fails with that finding, and the scenario stays `@pending`. The rows
//! of a source an ingest resumes, of a stale ingest marker and of the
//! consumed prefix, and of an index derivation past verify's deadline,
//! have no steps yet: they wait with it. The two rows of a purge a kill
//! stopped after its erasure was recorded run here.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use ::record::digest::Digest;
use ::record::id::{Address, Seq, WriterId};
use cucumber::{given, then};
use rusqlite::Connection;
use serde_json::json;

use crate::World;
use crate::erasure::purge_run_events;
use crate::record::{home, last_run, run_core, set_env};
use crate::record_transcript::{ingest, transcript_events};
use crate::security_redaction::{append, transcript};

/// An all-zero digest, as text.
const ZEROS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// The run recorded, and its store opened for damage.
fn damaged(w: &mut World) -> Result<(Connection, WriterId), String> {
    crate::recall::record_runs(w)?;
    let (writer, _) = transcript_events(w)?
        .into_iter()
        .next()
        .ok_or("the run left no event")?;
    let conn = Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    conn.execute(
        "ATTACH DATABASE ?1 AS idx",
        [home(w)?
            .root()
            .join(::store::search_index::FILE)
            .to_str()
            .ok_or("the index file has no name")?],
    )
    .map_err(|e| e.to_string())?;

    Ok((conn, writer))
}

#[given(regex = r"^the transcript has lines not in the record$")]
fn transcript_gained_lines(w: &mut World) -> Result<(), String> {
    crate::recall::record_runs(w)?;
    let line = json!({"type": "atis-latch", "atis": "a line the record lacks", "sessionId": "s"});
    append(w, "run-a", &line)
}

#[given(regex = r"^an event row's hash has been altered$")]
fn hash_altered(w: &mut World) -> Result<(), String> {
    let (conn, writer) = damaged(w)?;
    conn.execute(
        "UPDATE events SET hash = ?1 WHERE writer = ?2 AND seq = 2",
        [ZEROS, &writer.to_string()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[given(regex = r"^a payload file's bytes have been altered$")]
fn payload_altered(w: &mut World) -> Result<(), String> {
    crate::recall::record_runs(w)?;
    let big = crate::recall::pair(1_000, &"payload ".repeat(4_000))?;
    for line in &big {
        append(w, "run-a", line)?;
    }
    let path = transcript(w, "run-a")?;
    ingest(w, &path)?;
    let dir = home(w)?.root().join("payloads");
    let file = fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.is_file())
        .ok_or("no event left a payload file")?;
    let mut bytes = fs::read(&file).map_err(|e| e.to_string())?;
    bytes[0] ^= 0xff;
    fs::write(&file, bytes).map_err(|e| e.to_string())
}

#[given(regex = r"^a landmark row has been altered$")]
fn landmark_altered(_w: &mut World) -> Result<(), String> {
    Err("no landmark is recorded before M3 (LMK-01): the store holds none to alter".to_owned())
}

#[given(regex = r"^a trust level has been altered in place$")]
fn trust_altered(w: &mut World) -> Result<(), String> {
    crate::recall::record_runs(w)?;
    // The origins the store keeps are derived by a rebuild.
    run_core(w, &["rebuild"], None)?;
    let (conn, writer) = damaged(w)?;
    let changed = conn
        .execute(
            "UPDATE origins SET origin = 'witnessed' WHERE writer = ?1",
            [writer.to_string()],
        )
        .map_err(|e| e.to_string())?;
    if changed == 1 {
        return Ok(());
    }
    Err(format!(
        "the store keeps {changed} origins for the run's writer"
    ))
}

#[given(regex = r"^a schema object ParleyInn does not ship has been added to the store$")]
fn foreign_object(w: &mut World) -> Result<(), String> {
    let (conn, _) = damaged(w)?;
    conn.execute_batch("CREATE TABLE exfiltrated (copy TEXT)")
        .map_err(|e| e.to_string())
}

#[given(regex = r"^the search index's ranking configuration has been altered$")]
fn ranking_altered(w: &mut World) -> Result<(), String> {
    let (conn, _) = damaged(w)?;
    conn.execute(
        "INSERT INTO idx.events_fts (events_fts, rank) VALUES ('rank', 'bm25(0.0)')",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[given(regex = r"^a search index entry names an event whose text lacks the term$")]
fn entry_altered(w: &mut World) -> Result<(), String> {
    let (conn, writer) = damaged(w)?;
    let seq = Seq::new(1).ok_or("seq 1")?;
    let row = ::store::search_index::row_id_of(Address::new(writer, seq))
        .ok_or("the first event has no index row")?;
    conn.execute("DELETE FROM idx.events_fts WHERE rowid = ?1", [row])
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO idx.events_fts (rowid, text) VALUES (?1, 'zebra')",
        [row],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[then(regex = r#"^the output names the failed check "([^"]+)"$"#)]
fn names_the_check(w: &mut World, check: String) -> Result<(), String> {
    let out = last_run(w)?;
    let shown = String::from_utf8_lossy(&out.stderr).into_owned();
    if shown.contains(&format!("parinn verify: {check}: ")) {
        return Ok(());
    }
    Err(format!(
        "parinn verify does not name the check {check:?}: {shown}"
    ))
}

/// The range of the run's event 2, which a purge erases.
fn purged_range(writer: &WriterId) -> String {
    format!("{writer}:2-2")
}

/// A purge of event 2 the person ran, then cut short as a kill would
/// after its erasure was recorded: the posting back in the search
/// index and the finishing steps back in the journal.
fn cut_short_purge(w: &mut World) -> Result<(Connection, WriterId), String> {
    crate::recall::record_runs(w)?;
    purge_run_events(w, "run-a", 2, 2)?;
    let (conn, writer) = damaged(w)?;
    let seq = Seq::new(2).ok_or("seq 2")?;
    let row = ::store::search_index::row_id_of(Address::new(writer, seq))
        .ok_or("the event has no index row")?;
    conn.execute(
        "INSERT INTO idx.events_fts (rowid, text) VALUES (?1, 'residue')",
        [row],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO purge_journal (kind, target) VALUES ('index', ?1), ('vacuum', '')",
        [purged_range(&writer)],
    )
    .map_err(|e| e.to_string())?;
    Ok((conn, writer))
}

#[given(regex = r"^a kill stopped a purge after its erasure was recorded$")]
fn purge_cut_short(w: &mut World) -> Result<(), String> {
    cut_short_purge(w).map(|_| ())
}

#[given(
    regex = r"^a kill stopped a purge after its erasure was recorded, and a search index entry for an event outside its ranges names an event whose text lacks the term$"
)]
fn purge_cut_short_and_entry_altered(w: &mut World) -> Result<(), String> {
    let (conn, writer) = cut_short_purge(w)?;
    let seq = Seq::new(5).ok_or("seq 5")?;
    let row = ::store::search_index::row_id_of(Address::new(writer, seq))
        .ok_or("the event has no index row")?;
    conn.execute("DELETE FROM idx.events_fts WHERE rowid = ?1", [row])
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO idx.events_fts (rowid, text) VALUES (?1, 'zebra')",
        [row],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[then(
    regex = r"^the output reports the purge pending, naming the ranges whose search index entries it has still to remove, and names no failed check; verify audits it and counts it once as a failed operation, and repairs nothing$"
)]
fn purge_pending_reported(w: &mut World) -> Result<(), String> {
    let (conn, writer) = damaged(w)?;
    let out = last_run(w)?;
    let shown = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let range = purged_range(&writer);
    if !shown.contains(&format!("pending purges: {range}")) {
        return Err(format!("verify does not name the range {range}: {shown}"));
    }
    if shown.contains("parinn verify: ") {
        return Err(format!("verify names a failed check: {shown}"));
    }
    let audited = crate::record_search::audited(w, "purge_unfinished")?;
    if audited != 1 {
        return Err(format!("{audited} audit entries count purge_unfinished"));
    }
    let kept: i64 = conn
        .query_row(
            "SELECT count(*) FROM purge_journal WHERE target = ?1",
            [range],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if kept != 1 {
        return Err(format!("verify repaired: the journal holds {kept} steps"));
    }
    let vacuum: i64 = conn
        .query_row(
            "SELECT count(*) FROM purge_journal WHERE kind = 'vacuum'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let residue: i64 = conn
        .query_row(
            "SELECT count(*) FROM idx.events_fts WHERE events_fts MATCH 'residue'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if vacuum != 1 || residue != 1 {
        return Err(format!(
            "verify repaired: {vacuum} vacuum steps and {residue} residue entries remain"
        ));
    }
    Ok(())
}

/// The ten-minute bound on an ingest marker, in milliseconds (ADM-09).
const MARKER_CEILING_MS: u64 = 10 * 60 * 1000;

/// The wall clock `parinn verify` reads, in milliseconds.
fn now_ms() -> Result<u64, String> {
    let since = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    u64::try_from(since.as_millis()).map_err(|e| e.to_string())
}

/// The run's first line consumed and its ingest marker left there at
/// `marked_at`, as an ingest stopped after its first line leaves it,
/// with the transcript holding more lines past it.
fn marker_after_first_line(w: &mut World, marked_at: u64) -> Result<(), String> {
    crate::recall::record_runs(w)?;
    let line = json!({"type": "atis-latch", "atis": "a line past the marker", "sessionId": "s"});
    append(w, "run-a", &line)?;
    let bytes = fs::read(transcript(w, "run-a")?).map_err(|e| e.to_string())?;
    let first = bytes
        .iter()
        .position(|b| *b == b'\n')
        .ok_or("the transcript holds no whole line")?
        + 1;
    let prefix = Digest::of(&[&bytes[..first]]).to_hex();
    let conn = Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE transcript_sources SET byte_offset = ?1, prefix_hash = ?2, lines = 1, seen = ?4,
             marked_at = ?3",
        rusqlite::params![first as i64, prefix, marked_at as i64, bytes.len() as i64],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[given(
    regex = r"^an ingest killed during ingestion left an ingest marker short of the transcript's end$"
)]
fn marker_fresh(w: &mut World) -> Result<(), String> {
    let now = now_ms()?;
    marker_after_first_line(w, now)
}

#[given(
    regex = r"^an ingest left an ingest marker short of the transcript's end over 10 minutes ago on the node's wall clock and no running MCP server holds the transcript$"
)]
fn marker_stale(w: &mut World) -> Result<(), String> {
    let then = now_ms()?.saturating_sub(MARKER_CEILING_MS + 60_000);
    marker_after_first_line(w, then)
}

#[given(regex = r"^the transcript holds fewer bytes than its ingest position$")]
fn transcript_shorter(w: &mut World) -> Result<(), String> {
    crate::recall::record_runs(w)?;
    let path = transcript(w, "run-a")?;
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    fs::write(&path, &bytes[..bytes.len() / 2]).map_err(|e| e.to_string())
}

#[given(
    regex = r"^the transcript's bytes up to its ingest position differ from its consumed-prefix hash$"
)]
fn prefix_altered(w: &mut World) -> Result<(), String> {
    let (conn, _) = damaged(w)?;
    conn.execute("UPDATE transcript_sources SET prefix_hash = ?1", [ZEROS])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// The bytes of the run's transcript, for the offset the line over the
/// limit starts at.
fn refused_line_offset(w: &mut World) -> Result<u64, String> {
    let len = fs::metadata(transcript(w, "run-a")?)
        .map_err(|e| e.to_string())?
        .len();
    Ok(len)
}

#[given(
    regex = r"^a transcript line over the 4 MiB limit was refused and the transcript is short at that line, until the limit is raised or the source is replaced$"
)]
fn line_refused(w: &mut World) -> Result<(), String> {
    crate::recall::record_runs(w)?;
    // Ingest consumed every line before it and stopped at its start: the
    // position stands there, as a refusal leaves it (SEC-05, REC-03).
    let mut text = fs::read(transcript(w, "run-a")?).map_err(|e| e.to_string())?;
    text.extend(vec![b'x'; (4 << 20) + 1]);
    text.push(b'\n');
    fs::write(transcript(w, "run-a")?, text).map_err(|e| e.to_string())
}

#[then(
    regex = r#"^the output reports the transcript "([^"]+)" short at a refused line, naming its offset and the counter `transcript_line_too_large`, and names no failed check$"#
)]
fn reports_refused(w: &mut World, name: String) -> Result<(), String> {
    let offset = refused_line_offset(w)? - ((4 << 20) + 2);
    let out = last_run(w)?;
    let shown = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let (_, after) = shown
        .split_once("short at a refused line")
        .ok_or_else(|| format!("verify reports no transcript short at a refused line: {shown}"))?;
    let at = format!("{name}.jsonl at offset {offset} ");
    if !after.contains(&at) || !after.contains("`transcript_line_too_large`") {
        return Err(format!(
            "verify does not name {name} at offset {offset} with the counter: {shown}"
        ));
    }
    if shown.contains("parinn verify: ") {
        return Err(format!("verify names a failed check: {shown}"));
    }
    Ok(())
}

#[then(
    regex = r#"^the output reports the transcript "([^"]+)" short of its end, which ingest resumes$"#
)]
fn reports_short(w: &mut World, name: String) -> Result<(), String> {
    let out = last_run(w)?;
    let shown = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let (_, after) = shown
        .split_once("short of their end, which ingest resumes: ")
        .ok_or_else(|| format!("verify reports no transcript short of its end: {shown}"))?;
    if after.contains(&format!("{name}.jsonl")) {
        return Ok(());
    }
    Err(format!(
        "verify does not name {name} short of its end: {shown}"
    ))
}

/// The start markers the scenario's running MCP servers hold, locked as
/// a server locks its own, until the process ends.
static HELD_MARKERS: std::sync::Mutex<Vec<fs::File>> = std::sync::Mutex::new(Vec::new());

#[given(
    regex = r"^a running MCP server's live ingest is short of the transcript's end and left no ingest marker yet$"
)]
fn server_holds_transcript(w: &mut World) -> Result<(), String> {
    crate::recall::record_runs(w)?;
    let line = json!({"type": "atis-latch", "atis": "a line the server has yet to ingest", "sessionId": "s"});
    append(w, "run-a", &line)?;
    let conn = Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    let path: String = conn
        .query_row("SELECT path FROM transcript_sources", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let dir = home(w)?.root().join("mcp");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    let marker = dir.join("running.marker");
    fs::write(&marker, path.as_bytes()).map_err(|e| e.to_string())?;
    fs::set_permissions(&marker, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    let file = fs::File::open(&marker).map_err(|e| e.to_string())?;
    file.lock().map_err(|e| e.to_string())?;
    HELD_MARKERS.lock().map_err(|e| e.to_string())?.push(file);
    Ok(())
}

#[given(regex = r"^deriving the search index again outlasts verify's deadline$")]
fn derivation_outlasts_deadline(w: &mut World) -> Result<(), String> {
    crate::recall::record_runs(w)?;
    set_env(w, "PARINN_TEST_CLOCK", "1000000");
    set_env(w, "PARINN_TEST_CLOCK_STEP", "61");
    Ok(())
}

#[then(
    regex = r"^the output says the search index was not derived again in time and is unchecked, names no failed check and does not report the search index passed; verify audits it and counts it once as timed out$"
)]
fn index_unchecked(w: &mut World) -> Result<(), String> {
    let out = last_run(w)?;
    let shown = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if !shown.contains("the search index was not derived again in time, so it is unchecked") {
        return Err(format!(
            "verify does not say the index is unchecked: {shown}"
        ));
    }
    if shown.contains("parinn verify: ") || shown.contains("search index passed") {
        return Err(format!("verify names a failed or passed check: {shown}"));
    }
    let timed_out = crate::record_search::audited(w, "verify_timed_out")?;
    if timed_out != 1 {
        return Err(format!("{timed_out} audit entries count verify_timed_out"));
    }
    Ok(())
}
