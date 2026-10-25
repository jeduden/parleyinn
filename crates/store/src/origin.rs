//! The origins this store keeps (PRV-13, I10): for each writer, the
//! origin a derivation from the record and the node's key set gave it,
//! kept as a derived artifact beside the record. No reader decides an
//! origin by it: `parinn verify` compares it to a fresh derivation and
//! `parinn rebuild` writes it again. The table is made when the first
//! origins are written; a store without it holds none.

use std::collections::BTreeMap;

use record::id::WriterId;
use rusqlite::Connection;

use crate::StoreError;

/// The origins' table.
pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS origins (
    writer TEXT PRIMARY KEY,
    origin TEXT NOT NULL
) STRICT, WITHOUT ROWID;
";

/// Whether the store has made the table.
const HAS_TABLE: &str = "SELECT count(*) FROM sqlite_master WHERE name = 'origins'";

/// Every writer's kept origin, by writer; none while no table.
pub(crate) fn all(conn: &Connection) -> Result<BTreeMap<WriterId, String>, StoreError> {
    if !conn.query_row(HAS_TABLE, [], |r| r.get::<_, bool>(0))? {
        return Ok(BTreeMap::new());
    }
    let mut statement = conn.prepare("SELECT writer, origin FROM origins ORDER BY writer")?;
    let rows = statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;

    rows.map(|row| {
        let (text, origin) = row?;
        let writer = WriterId::parse(&text).map_err(|_| StoreError::Corrupt {
            address: None,
            what: format!("origin writer {text:?}"),
        })?;
        Ok((writer, origin))
    })
    .collect()
}

/// Replaces the kept origins with `origins`, inside the transaction
/// `conn` is in.
pub(crate) fn replace(
    conn: &Connection,
    origins: &BTreeMap<WriterId, String>,
) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;
    conn.execute("DELETE FROM origins", [])?;
    for (writer, origin) in origins {
        conn.execute(
            "INSERT INTO origins (writer, origin) VALUES (?1, ?2)",
            [writer.to_string(), origin.clone()],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
