//! What this node keeps locally of the repositories its worktrees hold
//! (LANE-02). The worktrees' table holds, per worktree, the identity it
//! resolves to; both are named by a keyed digest the caller made, so no
//! name here reveals a worktree or an identity off this node. It is
//! derived from the record's bind events and acts (I10; `parinn rebuild`
//! writes it again). The marks' table holds what this node already
//! reported or looked at; it is the node's own and not derived.
//! Neither replicates.

use std::collections::BTreeMap;

use rusqlite::{Connection, OptionalExtension};

use crate::StoreError;

/// One of the two tables: its name, its key column and its value column.
#[derive(Clone, Copy)]
struct Table {
    name: &'static str,
    key: &'static str,
    value: &'static str,
}

/// The worktrees' table: derived.
const WORKTREES: Table = Table {
    name: "repository_worktrees",
    key: "worktree",
    value: "identity",
};

/// The marks' table: the node's own.
const MARKS: Table = Table {
    name: "repository_worktree_marks",
    key: "mark",
    value: "value",
};

/// Whether the store has made a table.
const MADE: &str = "SELECT count(*) FROM sqlite_master WHERE name = ?1";

impl Table {
    fn schema(self) -> String {
        let Self { name, key, value } = self;

        format!(
            "CREATE TABLE IF NOT EXISTS {name} ({key} TEXT PRIMARY KEY, {value} TEXT NOT NULL) STRICT, WITHOUT ROWID;"
        )
    }

    fn made(self, conn: &Connection) -> Result<bool, StoreError> {
        let made = conn.query_row(MADE, [self.name], |r| r.get::<_, bool>(0))?;

        Ok(made)
    }

    fn get(self, conn: &Connection, key: &str) -> Result<Option<String>, StoreError> {
        if !self.made(conn)? {
            return Ok(None);
        }
        let Self {
            name,
            key: k,
            value,
        } = self;

        Ok(conn
            .query_row(
                &format!("SELECT {value} FROM {name} WHERE {k} = ?1"),
                [key],
                |r| r.get(0),
            )
            .optional()?)
    }

    fn set(self, conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
        conn.execute_batch(&self.schema())?;
        let Self {
            name,
            key: k,
            value: v,
        } = self;
        conn.execute(
            &format!("INSERT OR REPLACE INTO {name} ({k}, {v}) VALUES (?1, ?2)"),
            [key, value],
        )
        .map(|_| ())
    }

    fn forget(self, conn: &Connection, key: &str) -> rusqlite::Result<()> {
        conn.execute_batch(&self.schema())?;
        let Self { name, key: k, .. } = self;
        conn.execute(&format!("DELETE FROM {name} WHERE {k} = ?1"), [key])
            .map(|_| ())
    }
}

/// The identity the worktree `worktree` resolves to.
pub(crate) fn identity(conn: &Connection, worktree: &str) -> Result<Option<String>, StoreError> {
    WORKTREES.get(conn, worktree)
}

/// Keeps `identity` as what `worktree` resolves to.
pub(crate) fn set_identity(
    conn: &Connection,
    worktree: &str,
    identity: &str,
) -> rusqlite::Result<()> {
    WORKTREES.set(conn, worktree, identity)
}

/// Every worktree with the identity it resolves to.
pub(crate) fn identities(conn: &Connection) -> Result<BTreeMap<String, String>, StoreError> {
    if !WORKTREES.made(conn)? {
        return Ok(BTreeMap::new());
    }
    let mut statement =
        conn.prepare("SELECT worktree, identity FROM repository_worktrees ORDER BY worktree")?;
    let rows = statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;

    Ok(rows.collect::<Result<_, _>>()?)
}

/// Replaces every row with `identities`, inside the transaction `conn`
/// is in. The marks stay.
pub(crate) fn replace(
    conn: &Connection,
    identities: &BTreeMap<String, String>,
) -> rusqlite::Result<()> {
    conn.execute_batch(&WORKTREES.schema())?;
    conn.execute("DELETE FROM repository_worktrees", [])?;
    for (worktree, identity) in identities {
        set_identity(conn, worktree, identity)?;
    }

    Ok(())
}

/// What the mark `mark` holds.
pub(crate) fn mark(conn: &Connection, mark: &str) -> Result<Option<String>, StoreError> {
    MARKS.get(conn, mark)
}

/// Makes the mark `mark` hold `value`.
pub(crate) fn set_mark(conn: &Connection, mark: &str, value: &str) -> rusqlite::Result<()> {
    MARKS.set(conn, mark, value)
}

/// Clears the mark `mark`.
pub(crate) fn forget_mark(conn: &Connection, mark: &str) -> rusqlite::Result<()> {
    MARKS.forget(conn, mark)
}

#[cfg(test)]
mod tests;
