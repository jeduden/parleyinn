//! The search index and room state in one SQLite file, derived only
//! from the events a node holds and its key set (I10, ADM-08).
//!
//! Built from nothing in (writer, seq) order, in one transaction, so
//! the same held writers and key set give the same bytes whatever order
//! their segments arrived in.

use std::path::Path;

use rusqlite::{Connection, OpenFlags, Transaction, params};
use sha2::{Digest, Sha256};

use crate::node::Node;
use crate::record::{Address, Event, Segment};
use crate::room::RoomState;
use crate::{Result, hex};

const SCHEMA: &str = "
PRAGMA journal_mode = OFF;
PRAGMA synchronous = OFF;
CREATE TABLE events(
    id INTEGER PRIMARY KEY,
    writer TEXT NOT NULL,
    seq INTEGER NOT NULL,
    kind TEXT NOT NULL,
    provenance TEXT NOT NULL,
    origin TEXT NOT NULL,
    trust TEXT NOT NULL,
    commitment TEXT NOT NULL,
    hash TEXT NOT NULL,
    run TEXT,
    tool TEXT,
    ts INTEGER,
    text TEXT,
    UNIQUE(writer, seq)
);
CREATE VIRTUAL TABLE events_fts USING fts5(text, content = 'events', content_rowid = 'id');
CREATE TABLE branch_links(
    room TEXT NOT NULL,
    repository TEXT NOT NULL,
    branch TEXT NOT NULL,
    writer TEXT NOT NULL,
    seq INTEGER NOT NULL,
    commitment TEXT NOT NULL,
    PRIMARY KEY(room, repository, branch)
) WITHOUT ROWID;
CREATE TABLE conflicts(
    void_writer TEXT NOT NULL,
    void_seq INTEGER NOT NULL,
    stands_writer TEXT NOT NULL,
    stands_seq INTEGER NOT NULL,
    rule TEXT NOT NULL,
    PRIMARY KEY(void_writer, void_seq)
) WITHOUT ROWID;
";

/// Builds the index at `path`, which must not exist yet.
pub fn build(path: &Path, node: &Node, room: &RoomState) -> Result<()> {
    build_with(path, node, room, Ids::Arrival)
}

/// How an event's row id is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ids {
    /// The next id in insertion order: a rebuild inserts in (writer,
    /// seq) order, an incremental index in arrival order.
    Arrival,
    /// From the event's hash, the same in every order.
    FromHash,
}

/// `build` with a choice of row ids.
pub fn build_with(path: &Path, node: &Node, room: &RoomState, ids: Ids) -> Result<()> {
    let mut conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA)?;
    let tx = conn.transaction()?;
    insert_events(&tx, node, node.events(), ids)?;
    insert_room(&tx, room)?;
    tx.commit()?;
    Ok(())
}

/// Inserts events with their origin and trust on `node`; an address
/// already indexed is skipped.
fn insert_events<'a>(
    tx: &Transaction,
    node: &Node,
    events: impl IntoIterator<Item = &'a Event>,
    ids: Ids,
) -> Result<()> {
    let mut event = tx.prepare(
        "INSERT OR IGNORE INTO events(writer, seq, kind, provenance, origin, trust, commitment, hash, run, tool, ts, text, id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
    )?;
    let mut fts = tx.prepare("INSERT INTO events_fts(rowid, text) VALUES (?1, ?2)")?;
    for e in events {
        let h = &e.header;
        let content = e.body.as_ref().map(|b| &b.content);
        let field = |k: &str| content.and_then(|c| c.get(k));
        let text = field("text").and_then(|v| v.as_str());
        let added = event.execute(params![
            h.writer,
            h.seq as i64,
            h.kind,
            h.provenance,
            node.origin(&h.writer),
            node.trust(e),
            hex(&h.commitment),
            hex(&e.hash),
            field("run").and_then(|v| v.as_str()),
            field("tool").and_then(|v| v.as_str()),
            field("ts").and_then(|v| v.as_i64()),
            text,
            match ids {
                Ids::Arrival => None,
                Ids::FromHash => Some(i64::from_be_bytes(hash_prefix(&e.hash)) & i64::MAX),
            },
        ])?;
        if let (1, Some(t)) = (added, text) {
            fts.execute(params![tx.last_insert_rowid(), t])?;
        }
    }
    Ok(())
}

fn hash_prefix(h: &[u8; 32]) -> [u8; 8] {
    let mut b = [0u8; 8];
    b.copy_from_slice(&h[..8]);
    b
}

/// Replaces the room state tables.
fn insert_room(tx: &Transaction, room: &RoomState) -> Result<()> {
    tx.execute_batch("DELETE FROM branch_links; DELETE FROM conflicts;")?;
    let mut link = tx.prepare("INSERT INTO branch_links VALUES (?1, ?2, ?3, ?4, ?5, ?6)")?;
    for l in &room.links {
        link.execute(params![
            l.room,
            l.repository,
            l.branch,
            l.act.writer,
            l.act.seq as i64,
            hex(&l.commitment)
        ])?;
    }
    let mut conflict = tx.prepare("INSERT INTO conflicts VALUES (?1, ?2, ?3, ?4, ?5)")?;
    for c in &room.conflicts {
        conflict.execute(params![
            c.void.writer,
            c.void.seq as i64,
            c.stands.writer,
            c.stands.seq as i64,
            c.rule
        ])?;
    }
    Ok(())
}

/// The other way to keep an index, measured against `build`: add each
/// segment's events as it arrives, in arrival order.
pub struct Incremental {
    conn: Connection,
    ids: Ids,
}

impl Incremental {
    pub fn create(path: &Path, ids: Ids) -> Result<Incremental> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Incremental { conn, ids })
    }

    /// Indexes the segment's events as `node` now holds them.
    pub fn add(&mut self, node: &Node, segment: &Segment) -> Result<()> {
        let held: Vec<&Event> = segment
            .events
            .iter()
            .filter_map(|e| node.event(&e.address()))
            .collect();
        let tx = self.conn.transaction()?;
        insert_events(&tx, node, held, self.ids)?;
        tx.commit()?;
        Ok(())
    }

    pub fn set_room(&mut self, room: &RoomState) -> Result<()> {
        let tx = self.conn.transaction()?;
        insert_room(&tx, room)?;
        tx.commit()?;
        Ok(())
    }
}

/// Merges the FTS5 index into one segment, then rewrites the file.
pub fn optimize_and_vacuum(path: &Path) -> Result<()> {
    Connection::open(path)?
        .execute_batch("INSERT INTO events_fts(events_fts) VALUES ('optimize'); VACUUM;")?;
    Ok(())
}

/// SHA-256 over the index's logical content: every event row but its
/// row id, in (writer, seq) order, and the room state.
pub fn logical_digest(path: &Path) -> Result<[u8; 32]> {
    let conn = open(path)?;
    let mut h = Sha256::new();
    for sql in [
        "SELECT writer, seq, kind, provenance, origin, trust, commitment, hash, run, tool, ts, text
         FROM events ORDER BY writer, seq",
        "SELECT * FROM branch_links ORDER BY room, repository, branch",
        "SELECT * FROM conflicts ORDER BY void_writer, void_seq",
    ] {
        let mut q = conn.prepare(sql)?;
        let n = q.column_count();
        let mut rows = q.query([])?;
        while let Some(r) = rows.next()? {
            for i in 0..n {
                let v: rusqlite::types::Value = r.get(i)?;
                h.update(format!("{v:?}\u{1f}").as_bytes());
            }
            h.update(b"\n");
        }
    }
    Ok(h.finalize().into())
}

/// Purges one event's content in place, as an index kept incrementally
/// would: blanks its row and deletes its FTS5 entry. `secure_delete`
/// sets SQLite's pragma of that name, `fts5_secure_delete` FTS5's own
/// option of that name.
pub fn purge_in_place(
    path: &Path,
    a: &Address,
    secure_delete: bool,
    fts5_secure_delete: bool,
) -> Result<()> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "secure_delete", secure_delete)?;
    if fts5_secure_delete {
        conn.execute(
            "INSERT INTO events_fts(events_fts, rank) VALUES ('secure-delete', 1)",
            [],
        )?;
    }
    let tx = conn.unchecked_transaction()?;
    let (id, text): (i64, Option<String>) = tx.query_row(
        "SELECT id, text FROM events WHERE writer = ?1 AND seq = ?2",
        params![a.writer, a.seq as i64],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if let Some(t) = text {
        tx.execute(
            "INSERT INTO events_fts(events_fts, rowid, text) VALUES ('delete', ?1, ?2)",
            params![id, t],
        )?;
    }
    tx.execute(
        "UPDATE events SET run = NULL, tool = NULL, ts = NULL, text = NULL WHERE id = ?1",
        [id],
    )?;
    tx.commit()?;
    Ok(())
}

/// The bytes FTS5 keeps in its segment blobs.
pub fn fts5_data_bytes(path: &Path) -> Result<i64> {
    let conn = open(path)?;
    Ok(conn.query_row(
        "SELECT coalesce(sum(length(block)), 0) FROM events_fts_data",
        [],
        |r| r.get(0),
    )?)
}

fn open(path: &Path) -> Result<Connection> {
    Ok(Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?)
}

fn address(writer: String, seq: i64) -> Address {
    Address {
        writer,
        seq: seq as u64,
    }
}

/// One indexed event's identity and trust, as the index holds them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub address: Address,
    pub provenance: String,
    pub origin: String,
    pub trust: String,
}

/// Every indexed event, in (writer, seq) order.
pub fn rows(path: &Path) -> Result<Vec<Row>> {
    let conn = open(path)?;
    let mut q = conn.prepare(
        "SELECT writer, seq, provenance, origin, trust FROM events ORDER BY writer, seq",
    )?;
    let rows = q.query_map([], |r| {
        Ok(Row {
            address: address(r.get(0)?, r.get(1)?),
            provenance: r.get(2)?,
            origin: r.get(3)?,
            trust: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// The addresses whose text matches the phrase.
pub fn phrase_matches(path: &Path, phrase: &str) -> Result<Vec<Address>> {
    let conn = open(path)?;
    let mut q = conn.prepare(
        "SELECT e.writer, e.seq FROM events_fts f JOIN events e ON e.id = f.rowid
         WHERE events_fts MATCH ?1 ORDER BY e.writer, e.seq",
    )?;
    let quoted = format!("\"{}\"", phrase.replace('"', "\"\""));
    let rows = q.query_map([quoted], |r| Ok(address(r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// The standing branch links the index holds, as (branch, act).
pub fn branch_links(path: &Path) -> Result<Vec<(String, Address)>> {
    let conn = open(path)?;
    let mut q = conn.prepare(
        "SELECT branch, writer, seq FROM branch_links ORDER BY room, repository, branch",
    )?;
    let rows = q.query_map([], |r| Ok((r.get(0)?, address(r.get(1)?, r.get(2)?))))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}
