//! Segments (REC-19, ADM-05, REC-21): each writer's log in ranges, each
//! with its format version, kept beside the log. A segment is a unit of
//! the record, not a storage layout: no address, commitment or chain
//! head changes with it (REC-25). Each append extends its writer's open
//! segment, or opens one at its first new `seq`; closing it ends the
//! range. A segment of a newer format version than this parinn's is
//! refused, never rewritten.

use std::fmt;

use record::id::{Seq, WriterId};
use rusqlite::{Connection, params};

/// The format version of every segment this parinn writes.
pub const SEGMENT_VERSION: u32 = 1;

/// The segments' table, part of the store's schema.
pub(crate) const SCHEMA: &str = "
CREATE TABLE segments (
    writer TEXT NOT NULL,
    first_seq INTEGER NOT NULL,
    last_seq INTEGER NOT NULL,
    format_version INTEGER NOT NULL,
    closed INTEGER NOT NULL,
    PRIMARY KEY (writer, first_seq)
) STRICT, WITHOUT ROWID;
";

/// The columns a segment is read from.
const COLUMNS: &str = "SELECT writer, first_seq, last_seq, format_version, closed FROM segments";

/// One segment: a range of one writer, from `first` to `last`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub writer: WriterId,
    pub first: Seq,
    pub last: Seq,
    pub format_version: u32,
    /// Whether it has closed; the writer's newest may still be open.
    pub closed: bool,
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the segment of writer {} from seq {} to {}",
            self.writer, self.first, self.last
        )
    }
}

/// A segment row that does not parse: what of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BadRow(pub String);

/// One segment row as SQLite returns it.
type Row = (String, i64, i64, i64, bool);

/// The segment `row` is.
fn parse((writer, first, last, version, closed): Row) -> Result<Segment, BadRow> {
    let seq = |n: i64, what: &str| {
        u64::try_from(n)
            .ok()
            .and_then(Seq::new)
            .ok_or_else(|| BadRow(format!("segment {what} {n}")))
    };

    Ok(Segment {
        writer: WriterId::parse(&writer)
            .map_err(|_| BadRow(format!("segment writer {writer:?}")))?,
        first: seq(first, "first seq")?,
        last: seq(last, "last seq")?,
        format_version: u32::try_from(version)
            .map_err(|_| BadRow(format!("segment format version {version}")))?,
        closed,
    })
}

/// The segments `filter` selects, by writer then first seq. `filter`
/// names both `?1`, which is `arg`, and `?2`, this parinn's
/// [`SEGMENT_VERSION`].
pub(crate) fn select(
    conn: &Connection,
    filter: &str,
    arg: &str,
) -> rusqlite::Result<Vec<Result<Segment, BadRow>>> {
    let mut statement = conn.prepare(&format!("{COLUMNS} {filter} ORDER BY writer, first_seq"))?;
    let rows = statement.query_map(params![arg, SEGMENT_VERSION], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    })?;

    rows.map(|row| row.map(parse)).collect()
}

/// `writer`'s first segment of a newer format version, if it holds one.
pub(crate) fn newer(
    conn: &Connection,
    writer: WriterId,
) -> rusqlite::Result<Option<Result<Segment, BadRow>>> {
    Ok(select(
        conn,
        "WHERE writer = ?1 AND format_version > ?2",
        &writer.to_string(),
    )?
    .into_iter()
    .next())
}

/// Records that `writer` appended `first` to `last`: its open segment,
/// when that ends just before `first`, grows to `last`, else a segment
/// opens at `first`; closed after when `close`.
pub(crate) fn extend(
    conn: &Connection,
    writer: WriterId,
    first: Seq,
    last: Seq,
    close: bool,
) -> rusqlite::Result<()> {
    let id = writer.to_string();
    let (first, last) = (seq_sql(first), seq_sql(last));
    let grown = conn.execute(
        "UPDATE segments SET last_seq = ?3, closed = ?4
         WHERE writer = ?1 AND closed = 0 AND last_seq = ?2 - 1",
        params![id, first, last, close],
    )?;
    if grown == 0 {
        conn.execute(
            "INSERT INTO segments (writer, first_seq, last_seq, format_version, closed)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, first, last, SEGMENT_VERSION, close],
        )?;
    }

    Ok(())
}

/// Closes `writer`'s open segment, if it has one.
pub(crate) fn close(conn: &Connection, writer: WriterId) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE segments SET closed = 1 WHERE writer = ?1 AND closed = 0",
        [writer.to_string()],
    )
    .map(drop)
}

/// A seq as SQLite stores it: every seq is at most 2^53 - 1, so it
/// fits.
fn seq_sql(seq: Seq) -> i64 {
    seq.get() as i64
}

#[cfg(test)]
mod tests;
