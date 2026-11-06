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
//!
//! A purge keeps each purged line's position and length, and recomputes
//! the consumed-prefix hash with those bytes zeroed (SEC-31, ADM-07):
//! ingest, resume and `parinn verify` zero the same spans, so the hash
//! confirms no guess at the purged bytes and the source still resumes.

use std::collections::BTreeSet;

use record::digest::{Digest, Running};
use record::event::EventKind;
use record::id::{Address, WriterId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::{NewEvent, Out, Store, StoreError, seq_of, sql};

pub(crate) mod purged;
pub use purged::Transcripts;

impl Store {
    /// This store, each ingest marker it leaves, as a generation starts
    /// or a position moves, recorded as left at `ms`, in milliseconds on
    /// ingest's clock (ADM-09). The store reads no clock itself.
    #[must_use]
    pub fn with_marker_time(mut self, ms: u64) -> Self {
        self.marker_time = ms;
        self
    }
}

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
    marked_at INTEGER NOT NULL,
    UNIQUE (path, generation)
) STRICT;
CREATE TABLE transcript_places (
    event INTEGER PRIMARY KEY,
    source INTEGER NOT NULL,
    line INTEGER NOT NULL,
    byte_offset INTEGER NOT NULL,
    byte_length INTEGER NOT NULL,
    purged INTEGER NOT NULL DEFAULT 0,
    uuid TEXT,
    UNIQUE (source, line)
) STRICT;
CREATE INDEX transcript_places_uuid ON transcript_places (uuid) WHERE uuid IS NOT NULL;
CREATE TABLE IF NOT EXISTS transcript_purged_copies (
    source INTEGER NOT NULL,
    line INTEGER NOT NULL,
    byte_offset INTEGER NOT NULL,
    byte_length INTEGER NOT NULL,
    uuid TEXT,
    PRIMARY KEY (source, line)
) STRICT, WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS transcript_purged_copies_uuid
    ON transcript_purged_copies (uuid);
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
                             seen, held_since, lost, marked_at FROM transcript_sources";

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

/// The bytes one transcript line takes: where it starts and its length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub offset: u64,
    pub length: u64,
}

/// A prefix hash that no bytes produce: what a purge leaves of a source
/// whose file it could not read, or no longer begins with what ingest
/// consumed, so the hash confirms nothing of the purged bytes (SEC-31).
/// The source then reads as rewritten.
#[must_use]
pub fn unconfirmable() -> Digest {
    Digest::from_bytes([0; 32])
}

/// A running SHA-256 of a transcript's bytes, in order from the first,
/// that takes the bytes of each purged line as zeros: the consumed-prefix
/// hash as ingest, resume and `parinn verify` compute it (SEC-31).
pub struct Zeroed<'a> {
    running: Running,
    spans: &'a [Span],
    at: u64,
}

impl<'a> Zeroed<'a> {
    /// A hash of nothing yet, zeroing `spans`.
    #[must_use]
    pub fn new(spans: &'a [Span]) -> Self {
        Self {
            running: Running::default(),
            spans,
            at: 0,
        }
    }

    /// Takes the next `bytes` of the transcript.
    pub fn update(&mut self, bytes: &[u8]) {
        let end = self.at + bytes.len() as u64;
        let mut from = 0;
        for span in self.spans {
            let (start, stop) = (
                span.offset.max(self.at),
                (span.offset + span.length).min(end),
            );
            if start >= stop {
                continue;
            }
            let (start, stop) = (as_index(start - self.at), as_index(stop - self.at));
            // A span another overlaps zeroes only what it adds.
            let start = start.max(from);
            if start >= stop {
                continue;
            }
            self.running.update(&bytes[from..start]);
            let mut zeroes = stop - start;
            const ZEROES: [u8; 4096] = [0; 4096];
            while zeroes > 0 {
                let take = zeroes.min(ZEROES.len());
                self.running.update(&ZEROES[..take]);
                zeroes -= take;
            }
            from = stop;
        }
        self.running.update(&bytes[from..]);
        self.at = end;
    }

    /// The SHA-256 of the bytes taken.
    #[must_use]
    pub fn digest(&self) -> Digest {
        self.running.digest()
    }
}

/// A byte count as an index.
fn as_index(n: u64) -> usize {
    usize::try_from(n).unwrap_or(usize::MAX)
}

/// The consumed-prefix hash of `bytes`'s first `offset`, `spans` zeroed;
/// none when `bytes` hold fewer (SEC-31, REC-07).
#[must_use]
pub fn prefix_digest(bytes: &[u8], offset: u64, spans: &[Span]) -> Option<Digest> {
    let mut hash = Zeroed::new(spans);
    hash.update(bytes.get(..as_index(offset))?);

    Some(hash.digest())
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
    /// The bytes of each purged line, which the prefix hash takes as
    /// zeros (SEC-31), in offset order.
    pub purged: Vec<Span>,
    /// When an ingest marker began to hold its repeated lines, in
    /// milliseconds on ingest's clock (REC-04).
    pub held_since: Option<u64>,
    /// Whether ingest found it gone before it was fully ingested
    /// (REC-08).
    pub lost: bool,
    /// When ingest last left its ingest marker at its position, by
    /// starting the generation or moving the position, in milliseconds
    /// on ingest's clock: how long ago an ingest that stopped short of
    /// its end did so (ADM-09).
    pub marked_at: u64,
}

/// Where in its transcript source an event's line lies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// Its line number, from 1.
    pub line: u64,
    /// The byte offset its line starts at.
    pub offset: u64,
    /// How many bytes its line takes, its line break included.
    pub length: u64,
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

impl IngestedRun {
    /// The run's name, as a tool call's finding gives it: its harness
    /// session, and the harness's agent id for a subagent's run.
    #[must_use]
    pub fn run_name(&self) -> String {
        match &self.agent_id {
            Some(agent) => format!("{}/{agent}", self.harness_session),
            None => self.harness_session.clone(),
        }
    }
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
    i64,
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
    let (id, path, generation, directory, offset, prefix, lines, seen, held, lost, marked) = row;
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
        purged: Vec::new(),
        held_since: held.map(count),
        lost,
        marked_at: count(marked),
    })
}

/// The bytes of each purged line of source `id`, in offset order.
fn spans(conn: &Connection, id: i64) -> Result<Vec<Span>, StoreError> {
    let mut statement = conn.prepare(
        "SELECT byte_offset, byte_length FROM transcript_places
         WHERE source = ?1 AND purged = 1 ORDER BY byte_offset",
    )?;
    let rows = statement.query_map([id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
    let mut spans = Vec::new();
    for row in rows {
        let (offset, length) = row?;
        spans.push(Span {
            offset: count(offset),
            length: count(length),
        });
    }
    let query = "SELECT count(*) FROM sqlite_master WHERE name = 'transcript_purged_copies'";
    let has_copies: bool = conn.query_row(query, [], |r| r.get(0))?;
    if has_copies {
        let mut statement = conn.prepare(
            "SELECT byte_offset, byte_length FROM transcript_purged_copies
             WHERE source = ?1 ORDER BY byte_offset",
        )?;
        let rows = statement.query_map([id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            let (offset, length) = row?;
            spans.push(Span {
                offset: count(offset),
                length: count(length),
            });
        }
        spans.sort_by_key(|span| (span.offset, span.length));
        spans.dedup();
    }

    Ok(spans)
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
            r.get(10)?,
        ))
    })?;
    let rows: Vec<SourceRow> = rows.collect::<Result<_, _>>()?;

    let mut sources = rows
        .into_iter()
        .map(source)
        .collect::<Result<Vec<_>, _>>()?;
    for source in &mut sources {
        source.purged = spans(conn, source.id)?;
    }

    Ok(sources)
}

/// The newest generation of each source `filter` selects.
pub(crate) const NEWEST: &str = "WHERE generation = (SELECT max(generation)
     FROM transcript_sources t WHERE t.path = transcript_sources.path)";

/// Starts the next generation of the source at `path`, in `directory`,
/// nothing read of it, its ingest marker left at `at`.
pub(crate) fn start(
    conn: &Connection,
    path: &str,
    directory: &str,
    at: u64,
) -> Result<Source, StoreError> {
    let position = Position::start();
    let next = "SELECT ifnull(max(generation), 0) + 1 FROM transcript_sources WHERE path = ?1";
    let generation: Result<i64, _> = conn.query_row(next, [path], |r| r.get(0));
    let generation = generation?;
    let inserted = conn.execute(
        "INSERT INTO transcript_sources
             (path, generation, directory, byte_offset, prefix_hash, lines, seen, held_since, lost,
              marked_at)
         VALUES (?1, ?2, ?3, 0, ?4, 0, 0, NULL, 0, ?5)",
        params![
            path,
            generation,
            directory,
            position.prefix.to_hex(),
            sql_count(at)
        ],
    );
    inserted?;

    Ok(Source {
        id: conn.last_insert_rowid(),
        path: path.to_owned(),
        generation: count(generation),
        directory: directory.to_owned(),
        position,
        purged: Vec::new(),
        held_since: None,
        lost: false,
        marked_at: at,
    })
}

/// Moves source `id`'s position to `position`, never back, its ingest
/// marker left there at `at`.
pub(crate) fn advance(
    conn: &Connection,
    id: i64,
    position: &Position,
    at: u64,
) -> Result<(), StoreError> {
    let updated = conn.execute(
        "UPDATE transcript_sources SET byte_offset = ?2, prefix_hash = ?3, lines = ?4, seen = ?5,
             marked_at = ?6
         WHERE id = ?1 AND byte_offset <= ?2",
        params![
            id,
            sql_count(position.offset),
            position.prefix.to_hex(),
            sql_count(position.lines),
            sql_count(position.seen),
            sql_count(at)
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
        "INSERT INTO transcript_places (event, source, line, byte_offset, byte_length, uuid)
         VALUES ((SELECT rowid FROM events WHERE writer = ?1 AND seq = ?2), ?3, ?4, ?5, ?6, ?7)",
    )?;
    for (address, place) in addresses.iter().zip(places) {
        let inserted = insert.execute(params![
            address.writer.to_string(),
            sql(address.seq),
            id,
            sql_count(place.line),
            sql_count(place.offset),
            sql_count(place.length),
            place.uuid
        ]);
        inserted?;
    }

    Ok(())
}

/// Keeps the positions, lengths and harness UUIDs of refused copies of
/// purged lines. Their bytes are zeroed in the source's consumed-prefix
/// hash, so the record keeps nothing derived from their content (REC-04,
/// SEC-31).
pub(crate) fn refuse_copies(
    conn: &Connection,
    id: i64,
    places: &[Place],
) -> Result<(), StoreError> {
    if places.is_empty() {
        return Ok(());
    }
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS transcript_purged_copies (
             source INTEGER NOT NULL,
             line INTEGER NOT NULL,
             byte_offset INTEGER NOT NULL,
             byte_length INTEGER NOT NULL,
             uuid TEXT,
             PRIMARY KEY (source, line)
         ) STRICT, WITHOUT ROWID;
         CREATE INDEX IF NOT EXISTS transcript_purged_copies_uuid
             ON transcript_purged_copies (uuid);",
    )?;
    let mut insert = conn.prepare(
        "INSERT OR IGNORE INTO transcript_purged_copies
             (source, line, byte_offset, byte_length, uuid)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for place in places {
        insert.execute(params![
            id,
            sql_count(place.line),
            sql_count(place.offset),
            sql_count(place.length),
            place.uuid
        ])?;
    }

    Ok(())
}

impl Store {
    /// The ingested run `writer` keeps, none when it keeps none: a
    /// read by the table's primary key.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn ingested_run_of(&self, writer: WriterId) -> Result<Option<IngestedRun>, StoreError> {
        runs(
            &self.conn,
            "WHERE writer = ?1 AND ?2 = ?2",
            &writer.to_string(),
            "",
        )
        .map(|mut r| r.pop())
    }

    /// The position, length and harness UUID left for each purged line
    /// of source `id`, with no event association (SEC-31).
    pub fn purged_places(&self, id: i64) -> Result<Vec<Place>, StoreError> {
        purged_places(&self.conn, id)
    }

    /// The harness UUIDs left by purged lines in any transcript source
    /// of `directory`, an earlier generation of a rewritten transcript
    /// included, ready for an ingest to test in memory (REC-04, REC-07,
    /// SEC-31).
    pub fn purged_line_uuids(&self, directory: &str) -> Result<BTreeSet<String>, StoreError> {
        purged_line_uuids(&self.conn, directory)
    }

    /// Appends transcript lines and atomically keeps the positions of
    /// refused copies of purged lines which the new position crosses.
    /// The copies' bytes are zeroed in the position's hash (REC-04,
    /// SEC-31).
    #[expect(
        clippy::too_many_arguments,
        reason = "the appended lines and refused copies share one source position transaction"
    )]
    pub fn append_lines_refusing(
        &mut self,
        writer: WriterId,
        new_events: &[NewEvent],
        places: &[Place],
        closes: &[bool],
        id: i64,
        position: &Position,
        refused: &[Place],
    ) -> Result<Vec<Address>, StoreError> {
        let to = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let first = places
            .first()
            .into_iter()
            .chain(refused.first())
            .map(|p| p.offset)
            .min()
            .unwrap_or(position.offset);
        if offset(&tx, id)? > first {
            return Err(StoreError::PositionMoved { source: id });
        }
        let mut out = Vec::with_capacity(new_events.len());
        let mut start = 0;
        for i in 0..new_events.len() {
            let close = closes.get(i).copied().unwrap_or(false);
            if close || i + 1 == new_events.len() {
                out.extend(to.append_chunk(&tx, writer, &new_events[start..=i], close)?);
                start = i + 1;
            }
        }
        refuse_copies(&tx, id, refused)?;
        place(&tx, id, &out, places)?;
        advance(&tx, id, position, self.marker_time)?;
        tx.commit()?;

        Ok(out)
    }

    /// Moves a source past refused copies of purged lines, keeping their
    /// positions and zeroed hash in one transaction (REC-04, SEC-31).
    pub fn advance_refusing(
        &mut self,
        id: i64,
        position: &Position,
        refused: &[Place],
    ) -> Result<(), StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let first = refused.first().map_or(position.offset, |p| p.offset);
        if offset(&tx, id)? > first {
            return Err(StoreError::PositionMoved { source: id });
        }
        refuse_copies(&tx, id, refused)?;
        advance(&tx, id, position, self.marker_time)?;
        tx.commit()?;

        Ok(())
    }
}

/// The allowed residue of each purged line of source `id`, in line
/// order. An older store without the residue table holds none.
fn purged_places(conn: &Connection, id: i64) -> Result<Vec<Place>, StoreError> {
    let query = "SELECT count(*) FROM sqlite_master WHERE name = 'transcript_purged_copies'";
    if !conn.query_row(query, [], |row| row.get::<_, bool>(0))? {
        return Ok(Vec::new());
    }
    let mut statement = conn.prepare(
        "SELECT line, byte_offset, byte_length, uuid FROM transcript_purged_copies
         WHERE source = ?1 ORDER BY line",
    )?;
    let rows = statement.query_map([id], |row| {
        Ok(Place {
            line: count(row.get::<_, i64>(0)?),
            offset: count(row.get::<_, i64>(1)?),
            length: count(row.get::<_, i64>(2)?),
            uuid: row.get(3)?,
        })
    })?;

    Ok(rows.collect::<Result<_, _>>()?)
}

/// The events of source `id`, in line order.
pub(crate) fn placed(conn: &Connection, id: i64) -> Result<Vec<Placed>, StoreError> {
    let statement = conn.prepare(
        "SELECT p.line, p.byte_offset, p.byte_length, p.uuid, e.writer, e.seq, e.kind
         FROM transcript_places p JOIN events e ON e.rowid = p.event
         WHERE p.source = ?1 ORDER BY p.line",
    );
    let mut statement = statement?;
    let rows = statement.query_map([id], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, String>(6)?,
        ))
    })?;
    rows.map(|row| {
        let (line, offset, length, uuid, writer, seq, kind) = row?;
        Ok(Placed {
            place: Place {
                line: count(line),
                offset: count(offset),
                length: count(length),
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

/// The allowed residue UUIDs of purged lines in the transcript sources of
/// `directory`. An older store without the residue table holds none.
fn purged_line_uuids(conn: &Connection, directory: &str) -> Result<BTreeSet<String>, StoreError> {
    let query = "SELECT count(*) FROM sqlite_master WHERE name = 'transcript_purged_copies'";
    if !conn.query_row(query, [], |row| row.get::<_, bool>(0))? {
        return Ok(BTreeSet::new());
    }
    let mut statement = conn.prepare(
        "SELECT DISTINCT p.uuid FROM transcript_purged_copies p
         JOIN transcript_sources s ON s.id = p.source
         WHERE p.uuid IS NOT NULL AND s.directory = ?1
         ORDER BY p.uuid",
    )?;
    let rows = statement.query_map([directory], |row| row.get(0))?;

    Ok(rows.collect::<Result<_, _>>()?)
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
