//! The forks this node refused (PEER-10, REC-21): two different events
//! at one (writer, seq), the first head seen kept and the later one
//! refused. A refusal names the writer, the seq range and the two
//! digests, never content. The events table holds the first head alone,
//! so a refused history is recorded here, not rewritten into the record
//! (I1). The table is made when the first refusal is recorded; a store
//! without it holds none.

use record::digest::Digest;
use record::fork::Fork;
use record::id::{Seq, WriterId};
use rusqlite::{Connection, params};

use crate::StoreError;

/// The refusals' table.
pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS fork_refusals (
    writer TEXT NOT NULL,
    first_seq INTEGER NOT NULL,
    last_seq INTEGER NOT NULL,
    held TEXT NOT NULL,
    offered TEXT NOT NULL,
    PRIMARY KEY (writer, first_seq, offered)
) STRICT, WITHOUT ROWID;
";

/// Whether the store has made the table.
const HAS_TABLE: &str = "SELECT count(*) FROM sqlite_master WHERE name = 'fork_refusals'";

/// One refusal row as SQLite returns it.
type Row = (String, i64, i64, String, String);

/// Records `forks`, those not recorded yet; returns the ones that were
/// new.
pub(crate) fn record(conn: &Connection, forks: &[Fork]) -> rusqlite::Result<Vec<Fork>> {
    let mut new = Vec::new();
    for fork in forks {
        let added = conn.execute(
            "INSERT OR IGNORE INTO fork_refusals (writer, first_seq, last_seq, held, offered)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                fork.writer.to_string(),
                fork.first.get() as i64,
                fork.last.get() as i64,
                fork.held.to_hex(),
                fork.offered.to_hex()
            ],
        )?;
        if added > 0 {
            new.push(*fork);
        }
    }

    Ok(new)
}

/// Every refusal, by writer then first seq; none while no table.
pub(crate) fn all(conn: &Connection) -> Result<Vec<Fork>, StoreError> {
    if !conn.query_row(HAS_TABLE, [], |r| r.get::<_, bool>(0))? {
        return Ok(Vec::new());
    }
    let mut statement = conn.prepare(
        "SELECT writer, first_seq, last_seq, held, offered FROM fork_refusals
         ORDER BY writer, first_seq, offered",
    )?;
    let rows = statement.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    })?;

    rows.map(|row| parse(row?)).collect()
}

/// The fork `row` is.
fn parse((writer, first, last, held, offered): Row) -> Result<Fork, StoreError> {
    let corrupt = |what: String| StoreError::Corrupt {
        address: None,
        what,
    };
    let seq = |n: i64, what: &str| {
        u64::try_from(n)
            .ok()
            .and_then(Seq::new)
            .ok_or_else(|| corrupt(format!("fork {what} {n}")))
    };
    let digest =
        |text: &str| Digest::from_hex(text).map_err(|_| corrupt(format!("fork digest {text:?}")));

    Ok(Fork {
        writer: WriterId::parse(&writer).map_err(|_| corrupt(format!("fork writer {writer:?}")))?,
        first: seq(first, "first seq")?,
        last: seq(last, "last seq")?,
        held: digest(&held)?,
        offered: digest(&offered)?,
    })
}

#[cfg(test)]
mod tests;
