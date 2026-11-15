//! The steps of ADM-09: `parinn verify` exits 3 and names the failed
//! check for each kind of damage. Each damage is done to the home of an
//! agent's recorded run, by hand, as an attacker or a failing disk
//! would do it: a line the transcript gained, a row's hash, a payload
//! file's byte, an object in the store, the index's ranking or one of
//! its entries.
//!
//! The row that alters a landmark row cannot run in M1: no landmark is
//! recorded before M2 (LMK-01), so there is none to alter. Its step
//! fails with that finding, and the scenario stays `@pending`. The rows
//! of a source an ingest resumes, of a stale ingest marker and of the
//! consumed prefix, and of an index derivation past verify's deadline,
//! have no steps yet: they wait with it. The two rows of a purge a kill
//! stopped after its erasure was recorded run here.

use std::fs;

use ::record::id::{Address, Seq, WriterId};
use cucumber::{given, then};
use rusqlite::Connection;
use serde_json::json;

use crate::World;
use crate::erasure::purge_run_events;
use crate::record::{home, last_run, run_core};
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
    Err("no landmark is recorded before M2 (LMK-01): the store holds none to alter".to_owned())
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
    Ok(())
}
