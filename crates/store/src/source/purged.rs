//! What a purge does to the transcript sources the purged lines were
//! read from (SEC-31, ADM-07, REC-17): it marks each purged line's place
//! and recomputes the source's consumed-prefix hash with the bytes of
//! each purged line zeroed. Ingest, resume and `parinn verify` zero the
//! same spans, so the hash confirms no guess at the purged bytes and the
//! source still resumes; only each purged line's position and length
//! remain. The harness's transcript file is only read.

use std::io;
use std::path::Path;

use record::digest::Digest;
use rusqlite::{Connection, Transaction, params};

use super::{Source, Span, count, select, unconfirmable};
use crate::purge::Erase;
use crate::{Store, StoreError, sql};

/// How a purge reads a transcript's bytes: the guarded read the harness's
/// transcripts get (SEC-18, ENG-03), injected by the program that links
/// it.
pub type Transcripts = fn(&Path, u64, &[Span], &[Span]) -> io::Result<Option<(Digest, Digest)>>;

impl Store {
    /// This store, a purge reading the transcripts it recomputes the
    /// consumed-prefix hashes over through `read` (SEC-31, ADM-07); with
    /// none, the hash confirms nothing.
    #[must_use]
    pub fn with_transcripts(mut self, read: Transcripts) -> Self {
        self.transcripts = Some(read);
        self
    }
}

/// A source's consumed-prefix hash, recomputed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rehash {
    id: i64,
    offset: u64,
    old: Digest,
    new: Digest,
}

/// The places of the events in `range`, as the spans they take, by source.
const PLACES: &str = "SELECT p.source, p.byte_offset, p.byte_length
    FROM transcript_places p JOIN events e ON e.rowid = p.event
    WHERE e.writer = ?1 AND e.seq BETWEEN ?2 AND ?3 ORDER BY p.source, p.byte_offset";

/// The spans the purge of `ranges` takes out of each source, by source id.
fn erasing(conn: &Connection, ranges: &[Erase]) -> Result<Vec<(i64, Vec<Span>)>, StoreError> {
    let mut by_source: Vec<(i64, Vec<Span>)> = Vec::new();
    let mut statement = conn.prepare(PLACES)?;
    for range in ranges {
        let args = params![range.writer.to_string(), sql(range.first), sql(range.last)];
        let rows = statement.query_map(args, |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in rows {
            let (id, offset, length) = row?;
            let span = Span {
                offset: count(offset),
                length: count(length),
            };
            match by_source.iter_mut().find(|(s, _)| *s == id) {
                Some((_, spans)) => spans.push(span),
                None => by_source.push((id, vec![span])),
            }
        }
    }

    Ok(by_source)
}

/// `old` and `new` spans together, in offset order, each once.
fn merged(old: &[Span], new: &[Span]) -> Vec<Span> {
    let mut all: Vec<Span> = old.iter().chain(new).copied().collect();
    all.sort_by_key(|s| (s.offset, s.length));
    all.dedup();

    all
}

/// The hash of `source`'s consumed prefix with `spans` zeroed, read from
/// its transcript; none-confirming when no reader is injected or the
/// transcript no longer begins with the bytes the stored hash covers.
///
/// # Errors
///
/// Fails when the reader cannot read the transcript.
fn recomputed(source: &Source, spans: &[Span], read: Option<Transcripts>) -> io::Result<Digest> {
    let Some(read) = read else {
        return Ok(unconfirmable());
    };
    let path = Path::new(&source.path);
    let offset = source.position.offset;
    Ok(read(path, offset, &source.purged, spans)?
        .filter(|(held, _)| *held == source.position.prefix)
        .map_or_else(unconfirmable, |(_, recomputed)| recomputed))
}

/// The recomputed hash of each source the purge of `ranges` takes lines
/// out of, from the transcript files as they are now, and how many
/// sources' transcripts could not be read: their hash confirms nothing
/// (I6).
pub(crate) fn plan(
    conn: &Connection,
    ranges: &[Erase],
    read: Option<Transcripts>,
) -> Result<(Vec<Rehash>, u64), StoreError> {
    let mut out = Vec::new();
    let mut unreadable = 0;
    for (id, erased) in erasing(conn, ranges)? {
        let found = select(conn, "WHERE id = CAST(?1 AS INTEGER)", &id.to_string())?;
        for source in found {
            let spans = merged(&source.purged, &erased);
            let new = recomputed(&source, &spans, read).unwrap_or_else(|_| {
                unreadable += 1;
                unconfirmable()
            });
            out.push(Rehash {
                id,
                offset: source.position.offset,
                old: source.position.prefix,
                new,
            });
        }
    }

    Ok((out, unreadable))
}

/// Moves the allowed residue of the events in `range` away from their
/// event-linked places, then removes those associations (SEC-31).
pub(crate) fn mark(conn: &Connection, range: &Erase) -> Result<(), StoreError> {
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
    conn.execute(
        "INSERT OR IGNORE INTO transcript_purged_copies
             (source, line, byte_offset, byte_length, uuid)
         SELECT p.source, p.line, p.byte_offset, p.byte_length, p.uuid
         FROM transcript_places p JOIN events e ON e.rowid = p.event
         WHERE e.writer = ?1 AND e.seq BETWEEN ?2 AND ?3",
        params![range.writer.to_string(), sql(range.first), sql(range.last)],
    )?;
    conn.execute(
        "DELETE FROM transcript_places WHERE event IN
         (SELECT rowid FROM events WHERE writer = ?1 AND seq BETWEEN ?2 AND ?3)",
        params![range.writer.to_string(), sql(range.first), sql(range.last)],
    )?;

    Ok(())
}

/// Sets each recomputed hash, in the purge's transaction.
///
/// # Errors
///
/// Fails, the purge with it, when a source's position moved since the
/// hash was computed: an ingest read on, so its hash covers bytes the
/// recomputation did not.
pub(crate) fn apply(tx: &Transaction<'_>, rehashes: &[Rehash]) -> Result<(), StoreError> {
    for r in rehashes {
        let moved = tx.execute(
            "UPDATE transcript_sources SET prefix_hash = ?4
             WHERE id = ?1 AND byte_offset = ?2 AND prefix_hash = ?3",
            params![
                r.id,
                super::sql_count(r.offset),
                r.old.to_hex(),
                r.new.to_hex()
            ],
        )?;
        if moved == 0 {
            return Err(StoreError::PositionMoved { source: r.id });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
