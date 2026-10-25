//! What `parinn verify` checks of the store's own shape (ADM-09): no
//! schema object the store holds is one ParleyInn does not ship, the
//! search index's configuration is the shipped one, and the index is
//! what the record derives (I10). Each reads the store and changes
//! nothing that stays.

use std::collections::BTreeSet;

use rusqlite::Connection;

use crate::{SCHEMA, StoreError};
use crate::{hook, origin, purge, refusal, repository, search_index, segment, source};

/// Every schema object a store of this build can hold, as `(schema,
/// name)`: those its schemas create when made in a store of their own.
fn shipped() -> Result<BTreeSet<(String, String)>, StoreError> {
    let conn = Connection::open_in_memory()?;
    conn.execute("ATTACH DATABASE ':memory:' AS idx", [])?;
    for schema in [
        SCHEMA,
        segment::SCHEMA,
        source::SCHEMA,
        hook::SCHEMA,
        origin::SCHEMA,
        refusal::SCHEMA,
        purge::SCHEMA,
        search_index::SCHEMA,
    ] {
        conn.execute_batch(schema)?;
    }
    for schema in repository::schemas() {
        conn.execute_batch(&schema)?;
    }

    objects(&conn)
}

/// The `(schema, name)` of every object `conn`'s main database and its
/// attached index hold, SQLite's own left out.
fn objects(conn: &Connection) -> Result<BTreeSet<(String, String)>, StoreError> {
    let mut found = BTreeSet::new();
    let mut schemas = vec!["main"];
    if search_index::attached(conn)? {
        schemas.push(search_index::SCHEMA_NAME);
    }
    for schema in schemas {
        let mut names = conn.prepare(&format!(
            "SELECT name FROM {schema}.sqlite_master WHERE name NOT LIKE 'sqlite\\_%' ESCAPE '\\'"
        ))?;
        for name in names.query_map([], |r| r.get::<_, String>(0))? {
            found.insert((schema.to_owned(), name?));
        }
    }

    Ok(found)
}

/// The schema objects `conn` holds beyond those this build ships, as
/// `schema.name`.
pub(crate) fn foreign(conn: &Connection) -> Result<Vec<String>, StoreError> {
    let shipped = shipped()?;

    Ok(objects(conn)?
        .difference(&shipped)
        .map(|(schema, name)| format!("{schema}.{name}"))
        .collect())
}

/// What the configuration of the index's FTS5 table holds of its ranking
/// function: nothing, in a new index. Its other rows (the version, the
/// secure delete a purge turns on) are no ranking.
const RANKING: &str = "SELECT k, v FROM idx.events_fts_config WHERE k = 'rank' ORDER BY k";

/// The index's rows that set its ranking function.
fn ranking(conn: &Connection) -> Result<Vec<(String, String)>, StoreError> {
    let mut rows = conn.prepare(RANKING)?;
    let found = rows
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, rusqlite::types::Value>(1)
                    .map(|v| format!("{v:?}"))?,
            ))
        })?
        .collect::<Result<_, _>>()?;

    Ok(found)
}

/// Whether the index ranks as a new index does.
pub(crate) fn ranking_is_shipped(conn: &Connection) -> Result<bool, StoreError> {
    let fresh = Connection::open_in_memory()?;
    fresh.execute("ATTACH DATABASE ':memory:' AS idx", [])?;
    fresh.execute_batch(search_index::SCHEMA)?;

    Ok(ranking(conn)? == ranking(&fresh)?)
}

#[cfg(test)]
mod tests;
