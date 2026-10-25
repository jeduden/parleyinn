//! Purge (ADM-07, REC-17): the store's part of erasing the content of
//! address ranges. The caller decides what may be purged and builds the
//! act and tombstone events; the store makes the erasure one
//! transaction with their append, so a range is never erased without a
//! tombstone, nor a tombstone recorded for content that stays.
//!
//! What the transaction erases of each event: its content, its
//! commitment key, its preview, its payload reference and its
//! transcript timestamp. The event's row, header, commitment and hash
//! stay, so its writer's chain still checks (REC-10, REC-17); without
//! the key the commitment confirms no guess at the content. The prompt
//! commitment a purged `user` line used up goes with it.
//!
//! What cannot be done inside the transaction is journalled in it and
//! finished after it, by [`Store::finish_purge`], which the next purge,
//! ingest or rebuild also runs when a crash cut the first attempt short:
//! the search index's postings of the erased events, the payload files,
//! and, once both are gone, a checkpoint of the write-ahead logs and a
//! vacuum of both database files with SQLite's secure delete on, so no
//! free page, log frame or index segment keeps the content (SEC-14).

use record::event::EventKind;
use record::id::{Address, Seq, WriterId};
use rusqlite::{Connection, TransactionBehavior, params};
use serde_json::Value;

use crate::{NewEvent, Out, Store, StoreError, search_index, seq_of, sql};

/// The journal of what a purge still has to finish.
pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS purge_journal (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    target TEXT NOT NULL,
    UNIQUE (kind, target)
) STRICT;
";

const PAYLOAD: &str = "payload";
const INDEX: &str = "index";
const VACUUM: &str = "vacuum";

/// A range of one writer's events to erase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Erase {
    pub writer: WriterId,
    pub first: Seq,
    pub last: Seq,
}

/// What a purge did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purged {
    /// The events appended with the erasure, in order.
    pub appended: Vec<Address>,
    /// How many events the ranges held.
    pub events: u64,
}

/// What finishing a purge did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Finished {
    /// Payload files removed.
    pub payloads: u64,
    /// Events whose postings left the search index.
    pub postings: u64,
    /// Whether the logs were checkpointed and both files vacuumed; not
    /// when another connection held a log open, which leaves the
    /// vacuum for the next attempt.
    pub vacuumed: bool,
    /// Whether anything was left to finish.
    pub nothing: bool,
}

impl Store {
    /// Erases the content of `ranges` and appends `record` to `device`'s
    /// log in one transaction, then finishes the purge
    /// ([`finish_purge`](Self::finish_purge)). `record` holds the
    /// principal act and the tombstones; none of their content is
    /// searched.
    ///
    /// # Errors
    ///
    /// Fails, erasing and recording nothing, when SQLite does or an
    /// event of `record` has no canonical encoding. The finishing steps
    /// that fail afterwards stay journalled: the error is returned, the
    /// erasure stands.
    pub fn purge(
        &mut self,
        device: WriterId,
        record: &[NewEvent],
        ranges: &[Erase],
    ) -> Result<Purged, StoreError> {
        let to = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        secure_delete(&self.conn)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute_batch(SCHEMA)?;
        let mut events = 0;
        for range in ranges {
            events += erase(&tx, range)?;
        }
        let appended = if record.is_empty() {
            Vec::new()
        } else {
            to.append_chunk(&tx, device, record, true)?
        };
        journal(&tx, VACUUM, "")?;
        tx.commit()?;
        self.finish_purge()?;

        Ok(Purged { appended, events })
    }

    /// Finishes what a purge journalled and a crash may have cut short:
    /// the search index's postings of the erased events, the payload
    /// files, then the checkpoint and the vacuum. Nothing is left once
    /// it returns [`Finished::vacuumed`], and a store that never purged
    /// has nothing to finish.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a payload file cannot be removed; the
    /// step stays journalled for the next attempt.
    pub fn finish_purge(&mut self) -> Result<Finished, StoreError> {
        if !has_journal(&self.conn)? {
            return Ok(Finished {
                nothing: true,
                ..Finished::default()
            });
        }
        let mut done = Finished::default();
        done.postings = self.finish_postings()?;
        done.payloads = self.finish_payloads()?;
        done.vacuumed = self.finish_vacuum()?;
        done.nothing = done.postings == 0 && done.payloads == 0 && !done.vacuumed;

        Ok(done)
    }

    /// Whether a purge has steps still to finish.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn purge_pending(&self) -> Result<bool, StoreError> {
        if !has_journal(&self.conn)? {
            return Ok(false);
        }

        Ok(self
            .conn
            .query_row("SELECT count(*) FROM purge_journal", [], |r| {
                r.get::<_, bool>(0)
            })?)
    }

    /// Takes the journalled ranges out of the search index.
    fn finish_postings(&self) -> Result<u64, StoreError> {
        let mut count = 0;
        for (id, target) in journalled(&self.conn, INDEX)? {
            if let Some((writer, first, last)) = parse_range(&target) {
                count += search_index::remove(&self.conn, writer, first, last)?;
            }
            self.conn
                .execute("DELETE FROM purge_journal WHERE id = ?1", [id])?;
        }

        Ok(count)
    }

    /// Removes the journalled payload files.
    fn finish_payloads(&self) -> Result<u64, StoreError> {
        let mut count = 0;
        for (id, name) in journalled(&self.conn, PAYLOAD)? {
            self.payloads.remove(&name).map_err(StoreError::Io)?;
            self.conn
                .execute("DELETE FROM purge_journal WHERE id = ?1", [id])?;
            count += 1;
        }

        Ok(count)
    }

    /// Checkpoints both logs and vacuums both files, once the steps
    /// before it are done.
    fn finish_vacuum(&mut self) -> Result<bool, StoreError> {
        let journalled = journalled(&self.conn, VACUUM)?;
        if journalled.is_empty() {
            return Ok(false);
        }
        secure_delete(&self.conn)?;
        self.conn.execute_batch("PRAGMA temp_store = MEMORY")?;
        if !checkpoint(&self.conn)? {
            return Ok(false);
        }
        for schema in schemas(&self.conn)? {
            self.conn.execute_batch(&format!("VACUUM {schema}"))?;
        }
        if !checkpoint(&self.conn)? {
            return Ok(false);
        }
        self.conn
            .execute("DELETE FROM purge_journal WHERE kind = ?1", [VACUUM])?;

        Ok(true)
    }
}

/// Whether the store made the journal.
fn has_journal(conn: &Connection) -> Result<bool, StoreError> {
    Ok(conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE name = 'purge_journal'",
        [],
        |r| r.get::<_, bool>(0),
    )?)
}

/// Journals `target` for a step of `kind`, once.
fn journal(conn: &Connection, kind: &str, target: &str) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR IGNORE INTO purge_journal (kind, target) VALUES (?1, ?2)",
        params![kind, target],
    )?;

    Ok(())
}

/// The journalled targets of `kind`, with their ids, oldest first.
fn journalled(conn: &Connection, kind: &str) -> Result<Vec<(i64, String)>, StoreError> {
    let mut statement =
        conn.prepare("SELECT id, target FROM purge_journal WHERE kind = ?1 ORDER BY id")?;
    let rows = statement.query_map([kind], |r| Ok((r.get(0)?, r.get(1)?)))?;

    Ok(rows.collect::<Result<_, _>>()?)
}

/// Erases the events of `range` and journals what is left to finish;
/// returns how many events the range held.
fn erase(conn: &Connection, range: &Erase) -> Result<u64, StoreError> {
    let writer = range.writer.to_string();
    let (first, last) = (sql(range.first), sql(range.last));
    let mut statement = conn
        .prepare("SELECT seq, payload FROM events WHERE writer = ?1 AND seq BETWEEN ?2 AND ?3")?;
    let rows = statement.query_map(params![writer, first, last], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?))
    })?;
    let held: Vec<(i64, Option<String>)> = rows.collect::<Result<_, _>>()?;
    for (_, payload) in &held {
        if let Some(name) = payload {
            journal(conn, PAYLOAD, name)?;
        }
    }
    used_prompts(conn, range)?;
    conn.execute(
        "UPDATE events SET content = NULL, commitment_key = NULL, preview = NULL,
             payload = NULL, payload_size = NULL, transcript_timestamp = NULL
         WHERE writer = ?1 AND seq BETWEEN ?2 AND ?3",
        params![writer, first, last],
    )?;
    if !held.is_empty() {
        journal(conn, INDEX, &format!("{writer}:{first}-{last}"))?;
    }

    Ok(held.len() as u64)
}

/// Erases the key and commitment of each prompt commitment a prompt
/// match of an event in `range` used up: with its key, the commitment
/// would confirm a guess at the prompt the purged `user` line held.
fn used_prompts(conn: &Connection, range: &Erase) -> Result<(), StoreError> {
    let mut statement =
        conn.prepare("SELECT content FROM events WHERE kind = ?1 AND content IS NOT NULL")?;
    let rows = statement.query_map([EventKind::PromptMatch.as_str()], |r| r.get::<_, String>(0))?;
    for row in rows {
        let Ok(content) = serde_json::from_str::<Value>(&row?) else {
            continue;
        };
        if !names(&content["event"], range) {
            continue;
        }
        if let Some(id) = content["prompt_id"].as_str() {
            conn.execute(
                "UPDATE prompt_commitments SET commitment_key = '', commitment = ''
                 WHERE prompt_id = ?1 AND used = 1",
                [id],
            )?;
        }
    }

    Ok(())
}

/// Whether `event`, an address as text, lies in `range`.
fn names(event: &Value, range: &Erase) -> bool {
    let Some((writer, seq)) = event.as_str().and_then(|t| t.rsplit_once(':')) else {
        return false;
    };
    let seq = seq.parse::<u64>().ok().and_then(Seq::new);

    WriterId::parse(writer).ok() == Some(range.writer)
        && seq.is_some_and(|s| range.first <= s && s <= range.last)
}

/// The writer and seqs a journalled range names.
fn parse_range(target: &str) -> Option<(WriterId, Seq, Seq)> {
    let (writer, seqs) = target.split_once(':')?;
    let (first, last) = seqs.split_once('-')?;
    let seq = |text: &str| seq_of(text.parse().ok()?);

    Some((WriterId::parse(writer).ok()?, seq(first)?, seq(last)?))
}

/// Checkpoints and truncates the write-ahead log of the store and, when
/// attached, of the search index; whether neither was held open.
fn checkpoint(conn: &Connection) -> Result<bool, StoreError> {
    let mut whole = true;
    for schema in schemas(conn)? {
        let busy: i64 = conn.query_row(
            &format!("PRAGMA {schema}.wal_checkpoint(TRUNCATE)"),
            [],
            |r| r.get(0),
        )?;
        whole &= busy == 0;
    }

    Ok(whole)
}

/// The databases the connection holds: the store, and the search index
/// when it is attached.
fn schemas(conn: &Connection) -> Result<Vec<&'static str>, StoreError> {
    let mut schemas = vec!["main"];
    if search_index::attached(conn)? {
        schemas.push(search_index::SCHEMA_NAME);
    }

    Ok(schemas)
}

/// Has SQLite overwrite the pages it frees, in every database the
/// connection holds (SEC-14).
fn secure_delete(conn: &Connection) -> Result<(), StoreError> {
    for schema in schemas(conn)? {
        conn.execute_batch(&format!("PRAGMA {schema}.secure_delete = ON"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
