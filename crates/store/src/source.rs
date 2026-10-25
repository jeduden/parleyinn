//! Transcript sources (§8.2, REC-03, REC-07, REC-08): the state `parinn
//! ingest` keeps beside the record. Each transcript source has one row
//! per transcript generation, holding its ingest position: how far
//! ingest has read it, as a byte offset and the SHA-256 of the consumed
//! prefix, with the number of its last consumed line and how many bytes
//! it held when last read. Each event ingest appends keeps its place:
//! its source, generation, line, byte offset and the line's `uuid`. Each
//! ingested run is kept by its transcript source and the harness's agent
//! id (REC-02).
//!
//! The position moves in the same transaction as the events it covers,
//! so a kill neither drops nor repeats a line.

use record::digest::Digest;
use record::event::EventKind;
use record::id::{Address, WriterId};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::{StoreError, seq_of, sql};

/// The tables of transcript sources, their places and ingested runs,
/// part of the store's schema.
pub(crate) const SCHEMA: &str = "
CREATE TABLE transcript_sources (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL,
    generation INTEGER NOT NULL,
    directory TEXT NOT NULL,
    byte_offset INTEGER NOT NULL,
    prefix_hash TEXT NOT NULL,
    lines INTEGER NOT NULL,
    seen INTEGER NOT NULL,
    held_since INTEGER,
    lost INTEGER NOT NULL,
    UNIQUE (path, generation)
) STRICT;
CREATE TABLE transcript_places (
    event INTEGER PRIMARY KEY,
    source INTEGER NOT NULL,
    line INTEGER NOT NULL,
    byte_offset INTEGER NOT NULL,
    uuid TEXT,
    UNIQUE (source, line)
) STRICT;
CREATE INDEX transcript_places_uuid ON transcript_places (uuid) WHERE uuid IS NOT NULL;
CREATE TABLE ingested_runs (
    writer TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    harness_session TEXT NOT NULL,
    agent_id TEXT
) STRICT;
CREATE INDEX ingested_runs_agent ON ingested_runs (path, ifnull(agent_id, ''));
";

/// The columns a transcript source is read from.
const SOURCE: &str = "SELECT id, path, generation, directory, byte_offset, prefix_hash, lines,
                             seen, held_since, lost FROM transcript_sources";

/// How far ingest has read one generation of a transcript source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    /// The bytes consumed.
    pub offset: u64,
    /// SHA-256 of the consumed bytes.
    pub prefix: Digest,
    /// The number of the last line consumed; 0 before the first.
    pub lines: u64,
    /// How many bytes the source held when last read.
    pub seen: u64,
}

impl Position {
    /// A generation nothing has been read of.
    #[must_use]
    pub fn start() -> Self {
        Self {
            offset: 0,
            prefix: Digest::of(&[]),
            lines: 0,
            seen: 0,
        }
    }

    /// Whether every byte last seen was consumed.
    #[must_use]
    pub fn complete(&self) -> bool {
        self.offset >= self.seen
    }
}

/// One generation of a transcript source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Its row, which places name.
    pub id: i64,
    pub path: String,
    pub generation: u64,
    /// The transcript directory it lies in, within which a harness fork
    /// repeats lines (REC-04).
    pub directory: String,
    pub position: Position,
    /// When an ingest marker began to hold its repeated lines, in
    /// milliseconds on ingest's clock (REC-04).
    pub held_since: Option<u64>,
    /// Whether ingest found it gone before it was fully ingested
    /// (REC-08).
    pub lost: bool,
}

/// Where in its transcript source an event's line lies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// Its line number, from 1.
    pub line: u64,
    /// The byte offset its line starts at.
    pub offset: u64,
    /// The line's `uuid`, when it carries one.
    pub uuid: Option<String>,
}

/// An event's place, with its address and kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub place: Place,
    pub address: Address,
    pub kind: EventKind,
}

/// An ingested run (REC-02, REC-22): its ingest seat's writer, the
/// transcript source it is read from, its harness session and the
/// harness's agent id, none for a harness session's main run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestedRun {
    pub writer: WriterId,
    pub path: String,
    pub harness_session: String,
    pub agent_id: Option<String>,
}

/// One transcript source row as SQLite returns it.
type SourceRow = (
    i64,
    String,
    i64,
    String,
    i64,
    String,
    i64,
    i64,
    Option<i64>,
    bool,
);

/// A count or offset SQLite holds, which is never negative.
fn count(n: i64) -> u64 {
    u64::try_from(n).unwrap_or(0)
}

/// A count or offset as SQLite stores it.
fn sql_count(n: u64) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// The source `row` is.
fn source(row: SourceRow) -> Result<Source, StoreError> {
    let (id, path, generation, directory, offset, prefix, lines, seen, held, lost) = row;
    let prefix = Digest::from_hex(&prefix).map_err(|e| StoreError::Corrupt {
        address: None,
        what: format!("prefix hash of transcript source {path:?}: {e}"),
    })?;

    Ok(Source {
        id,
        path,
        generation: count(generation),
        directory,
        position: Position {
            offset: count(offset),
            prefix,
            lines: count(lines),
            seen: count(seen),
        },
        held_since: held.map(count),
        lost,
    })
}

/// The sources `filter` selects, `?1` being `arg`.
pub(crate) fn select(
    conn: &Connection,
    filter: &str,
    arg: &str,
) -> Result<Vec<Source>, StoreError> {
    let mut statement = conn.prepare(&format!("{SOURCE} {filter}"))?;
    let rows = statement.query_map([arg], |r| {
        Ok((
            r.get(0)?,
            r.get(1)?,
            r.get(2)?,
            r.get(3)?,
            r.get(4)?,
            r.get(5)?,
            r.get(6)?,
            r.get(7)?,
            r.get(8)?,
            r.get(9)?,
        ))
    })?;
    let rows: Vec<SourceRow> = rows.collect::<Result<_, _>>()?;

    rows.into_iter().map(source).collect()
}

/// The newest generation of each source `filter` selects.
pub(crate) const NEWEST: &str = "WHERE generation = (SELECT max(generation)
     FROM transcript_sources t WHERE t.path = transcript_sources.path)";

/// Starts the next generation of the source at `path`, in `directory`,
/// nothing read of it.
pub(crate) fn start(conn: &Connection, path: &str, directory: &str) -> Result<Source, StoreError> {
    let position = Position::start();
    let next = "SELECT ifnull(max(generation), 0) + 1 FROM transcript_sources WHERE path = ?1";
    let generation: Result<i64, _> = conn.query_row(next, [path], |r| r.get(0));
    let generation = generation?;
    let inserted = conn.execute(
        "INSERT INTO transcript_sources
             (path, generation, directory, byte_offset, prefix_hash, lines, seen, held_since, lost)
         VALUES (?1, ?2, ?3, 0, ?4, 0, 0, NULL, 0)",
        params![path, generation, directory, position.prefix.to_hex()],
    );
    inserted?;

    Ok(Source {
        id: conn.last_insert_rowid(),
        path: path.to_owned(),
        generation: count(generation),
        directory: directory.to_owned(),
        position,
        held_since: None,
        lost: false,
    })
}

/// Moves source `id`'s position to `position`, never back.
pub(crate) fn advance(conn: &Connection, id: i64, position: &Position) -> Result<(), StoreError> {
    let updated = conn.execute(
        "UPDATE transcript_sources SET byte_offset = ?2, prefix_hash = ?3, lines = ?4, seen = ?5
         WHERE id = ?1 AND byte_offset <= ?2",
        params![
            id,
            sql_count(position.offset),
            position.prefix.to_hex(),
            sql_count(position.lines),
            sql_count(position.seen)
        ],
    );

    updated.map(drop).map_err(StoreError::from)
}

/// The byte offset source `id`'s position stands at; 0 for no source.
pub(crate) fn offset(conn: &Connection, id: i64) -> Result<u64, StoreError> {
    let offset: Option<i64> = conn
        .query_row(
            "SELECT byte_offset FROM transcript_sources WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;

    Ok(offset.map_or(0, |o| u64::try_from(o).unwrap_or(0)))
}

/// Marks source `id` held at its ingest marker since `since`, or no
/// longer held.
pub(crate) fn hold(conn: &Connection, id: i64, since: Option<u64>) -> Result<(), StoreError> {
    let updated = conn.execute(
        "UPDATE transcript_sources SET held_since = ?2 WHERE id = ?1",
        params![id, since.map(sql_count)],
    );

    updated.map(drop).map_err(StoreError::from)
}

/// Marks source `id` found gone before it was fully ingested.
pub(crate) fn lose(conn: &Connection, id: i64) -> Result<(), StoreError> {
    conn.execute("UPDATE transcript_sources SET lost = 1 WHERE id = ?1", [id])?;

    Ok(())
}

/// Keeps the place of each event at `addresses` in source `id`.
pub(crate) fn place(
    tx: &Transaction<'_>,
    id: i64,
    addresses: &[Address],
    places: &[Place],
) -> Result<(), StoreError> {
    let mut insert = tx.prepare(
        "INSERT INTO transcript_places (event, source, line, byte_offset, uuid)
         VALUES ((SELECT rowid FROM events WHERE writer = ?1 AND seq = ?2), ?3, ?4, ?5, ?6)",
    )?;
    for (address, place) in addresses.iter().zip(places) {
        let inserted = insert.execute(params![
            address.writer.to_string(),
            sql(address.seq),
            id,
            sql_count(place.line),
            sql_count(place.offset),
            place.uuid
        ]);
        inserted?;
    }

    Ok(())
}

/// The events of source `id`, in line order.
pub(crate) fn placed(conn: &Connection, id: i64) -> Result<Vec<Placed>, StoreError> {
    let statement = conn.prepare(
        "SELECT p.line, p.byte_offset, p.uuid, e.writer, e.seq, e.kind
         FROM transcript_places p JOIN events e ON e.rowid = p.event
         WHERE p.source = ?1 ORDER BY p.line",
    );
    let mut statement = statement?;
    let rows = statement.query_map([id], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, String>(5)?,
        ))
    })?;
    rows.map(|row| {
        let (line, offset, uuid, writer, seq, kind) = row?;
        Ok(Placed {
            place: Place {
                line: count(line),
                offset: count(offset),
                uuid,
            },
            address: address(&writer, seq)?,
            kind: EventKind::parse(&kind).ok_or_else(|| StoreError::Corrupt {
                address: None,
                what: format!("kind {kind:?}"),
            })?,
        })
    })
    .collect()
}

/// The address `writer` and `seq` name.
fn address(writer: &str, seq: i64) -> Result<Address, StoreError> {
    let corrupt = || StoreError::Corrupt {
        address: None,
        what: format!("place of writer {writer:?} seq {seq}"),
    };
    let writer = WriterId::parse(writer).map_err(|_| corrupt())?;

    Ok(Address::new(writer, seq_of(seq).ok_or_else(corrupt)?))
}

/// The event that first recorded the line `uuid` names in `directory`,
/// from a transcript source other than `path`: never a repeat reference.
pub(crate) fn owner(
    conn: &Connection,
    directory: &str,
    uuid: &str,
    path: &str,
) -> Result<Option<Address>, StoreError> {
    let found: Option<(String, i64)> = conn
        .query_row(
            "SELECT e.writer, e.seq FROM transcript_places p
             JOIN events e ON e.rowid = p.event
             JOIN transcript_sources s ON s.id = p.source
             WHERE p.uuid = ?1 AND s.directory = ?2 AND s.path != ?3 AND e.kind != ?4
             ORDER BY p.event LIMIT 1",
            params![uuid, directory, path, EventKind::RepeatReference.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    found.map(|(w, s)| address(&w, s)).transpose()
}

/// The ingested runs `filter` selects, `?1` and `?2` being `a` and `b`.
pub(crate) fn runs(
    conn: &Connection,
    filter: &str,
    a: &str,
    b: &str,
) -> Result<Vec<IngestedRun>, StoreError> {
    let mut statement = conn.prepare(&format!(
        "SELECT writer, path, harness_session, agent_id FROM ingested_runs {filter} ORDER BY rowid"
    ))?;
    let rows = statement.query_map([a, b], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    })?;
    rows.map(|row| {
        let (writer, path, harness_session, agent_id) = row?;
        Ok(IngestedRun {
            writer: WriterId::parse(&writer).map_err(|_| StoreError::Corrupt {
                address: None,
                what: format!("ingested run writer {writer:?}"),
            })?,
            path,
            harness_session,
            agent_id,
        })
    })
    .collect()
}

/// Keeps `run`.
pub(crate) fn add_run(conn: &Connection, run: &IngestedRun) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO ingested_runs (writer, path, harness_session, agent_id) VALUES (?1, ?2, ?3, ?4)",
        params![
            run.writer.to_string(),
            run.path,
            run.harness_session,
            run.agent_id
        ],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests;
