//! Hook observations and prompt commitments (REC-19, PRV-08): the state
//! the hook handlers keep beside the record. Until a run's MCP server
//! holds its run-seat key, a hook handler appends nothing for the run:
//! it holds what it observed at the transcript's ingest marker, at the
//! offset where its hook fired, for whoever appends the run next. A
//! `SessionStart` hook observation is the harness session's session
//! binding; one no MCP server took within 30 s is counted once.
//!
//! A prompt commitment is kept here alone, never in a segment, and is
//! used up by the first line carrying its `prompt_id`.

use record::commitment::{Commitment, CommitmentKey};
use record::id::{Address, WriterId};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::{NewEvent, Out, Store, StoreError};

/// The tables of hook observations and prompt commitments, part of the
/// store's schema.
pub(crate) const SCHEMA: &str = "
CREATE TABLE hook_observations (
    id INTEGER PRIMARY KEY,
    hook TEXT NOT NULL,
    harness_session TEXT NOT NULL,
    transcript_path TEXT NOT NULL,
    cwd TEXT NOT NULL,
    byte_offset INTEGER,
    source TEXT,
    prompt_id TEXT,
    attended INTEGER NOT NULL,
    harnesses INTEGER,
    nested INTEGER,
    harness_process INTEGER,
    at INTEGER NOT NULL,
    appended INTEGER NOT NULL,
    binding_counted INTEGER NOT NULL,
    taken INTEGER NOT NULL
) STRICT;
CREATE INDEX hook_observations_session ON hook_observations (harness_session);
CREATE TABLE prompt_commitments (
    harness_session TEXT NOT NULL,
    prompt_id TEXT NOT NULL,
    commitment_key TEXT NOT NULL,
    commitment TEXT NOT NULL,
    used INTEGER NOT NULL,
    PRIMARY KEY (harness_session, prompt_id)
) STRICT;
";

/// The columns a hook observation is read from.
/// How many session bindings were taken (`?1` is 1) or not (0).
const COUNT_BINDINGS: &str =
    "SELECT count(*) FROM hook_observations WHERE hook = 'SessionStart' AND taken = ?1";

const OBSERVATION: &str = "SELECT id, hook, harness_session, transcript_path, cwd, byte_offset,
    source, prompt_id, attended, harnesses, nested, harness_process, at, appended, taken
    FROM hook_observations";

/// What one hook handler observed: only the hook input's structural
/// fields, never a prompt's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// The hook it answered.
    pub hook: String,
    pub harness_session: String,
    /// The transcript path the hook input named: with the harness
    /// session id, the session binding.
    pub transcript_path: String,
    /// The working directory the hook input named: the run's worktree.
    pub cwd: String,
    /// The transcript's size when the hook fired, when it lies within
    /// the transcript roots: where it is held.
    pub offset: Option<u64>,
    /// How the harness session started (`SessionStart`).
    pub source: Option<String>,
    pub prompt_id: Option<String>,
    /// Whether the handler found the attended mark (PRV-04).
    pub attended: bool,
    /// The harness processes among the handler's ancestors, and whether
    /// that makes the harness session nested; none when unread.
    pub ancestry: Option<(u64, bool)>,
    /// The nearest harness process among them, by its process id: with
    /// `SessionStart`, the harness process whose MCP server takes this
    /// session binding (REC-13).
    pub harness: Option<u32>,
    /// When it was held, in seconds since the Unix epoch.
    pub at: u64,
}

/// A hook observation the store holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    pub id: i64,
    pub observation: Observation,
    /// Whether it reached the record.
    pub appended: bool,
    /// Whether an MCP server took it as its session binding (REC-13).
    pub taken: bool,
}

/// A prompt commitment (PRV-08).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptCommitment {
    pub harness_session: String,
    pub prompt_id: String,
    pub key: CommitmentKey,
    pub commitment: Commitment,
    /// Whether a line carrying its `prompt_id` used it up.
    pub used: bool,
}

/// A count, offset or time SQLite holds, which is never negative.
fn count(n: i64) -> u64 {
    u64::try_from(n).unwrap_or(0)
}

/// A count, offset or time as SQLite stores it.
fn sql_count(n: u64) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// One hook observation row as SQLite returns it.
type Row = (
    i64,
    String,
    String,
    String,
    String,
    Option<i64>,
    Option<String>,
    Option<String>,
    bool,
    Option<i64>,
    Option<bool>,
    Option<i64>,
    i64,
    bool,
    bool,
);

fn held(row: Row) -> Held {
    let (id, hook, session, path, cwd, offset, source, prompt_id, attended) = (
        row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8,
    );
    let (harnesses, nested, harness, at, appended, taken) =
        (row.9, row.10, row.11, row.12, row.13, row.14);

    Held {
        id,
        observation: Observation {
            hook,
            harness_session: session,
            transcript_path: path,
            cwd,
            offset: offset.map(count),
            source,
            prompt_id,
            attended,
            ancestry: harnesses.zip(nested).map(|(h, n)| (count(h), n)),
            harness: harness.and_then(|h| u32::try_from(h).ok()),
            at: count(at),
        },
        appended,
        taken,
    }
}

fn observations(conn: &Connection, filter: &str) -> Result<Vec<Held>, StoreError> {
    let mut statement = conn.prepare(&format!("{OBSERVATION} {filter} ORDER BY id"))?;
    let rows = statement.query_map([], |r| {
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
            r.get(11)?,
            r.get(12)?,
            r.get(13)?,
            r.get(14)?,
        ))
    })?;
    let rows: Vec<Row> = rows.collect::<Result<_, _>>()?;

    Ok(rows.into_iter().map(held).collect())
}

impl Store {
    /// Holds `observation` at its transcript's ingest marker, and
    /// `commitment` beside it when given, in one transaction (REC-19).
    /// A second commitment for the same harness session and `prompt_id`
    /// leaves the first.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does; nothing is held then.
    pub fn hold_observation(
        &mut self,
        observation: &Observation,
        commitment: Option<&PromptCommitment>,
    ) -> Result<i64, StoreError> {
        let o = observation;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO hook_observations (hook, harness_session, transcript_path, cwd,
                 byte_offset, source, prompt_id, attended, harnesses, nested, harness_process,
                 at, appended, binding_counted, taken)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0, 0, 0)",
            params![
                o.hook,
                o.harness_session,
                o.transcript_path,
                o.cwd,
                o.offset.map(sql_count),
                o.source,
                o.prompt_id,
                o.attended,
                o.ancestry.map(|(h, _)| sql_count(h)),
                o.ancestry.map(|(_, n)| n),
                o.harness,
                sql_count(o.at),
            ],
        )?;
        let id = tx.last_insert_rowid();
        if let Some(c) = commitment {
            tx.execute(
                "INSERT OR IGNORE INTO prompt_commitments
                     (harness_session, prompt_id, commitment_key, commitment, used)
                 VALUES (?1, ?2, ?3, ?4, 0)",
                params![
                    c.harness_session,
                    c.prompt_id,
                    c.key.to_hex(),
                    c.commitment.to_string()
                ],
            )?;
        }
        tx.commit()?;

        Ok(id)
    }

    /// Every hook observation the store holds, appended or not, in the
    /// order they were held.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn hook_observations(&self) -> Result<Vec<Held>, StoreError> {
        observations(&self.conn, "")
    }

    /// The prompt commitment of `prompt_id` in `harness_session`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn prompt_commitment(
        &self,
        harness_session: &str,
        prompt_id: &str,
    ) -> Result<Option<PromptCommitment>, StoreError> {
        let row: Option<(String, String, bool)> = self
            .conn
            .query_row(
                "SELECT commitment_key, commitment, used FROM prompt_commitments
                 WHERE harness_session = ?1 AND prompt_id = ?2",
                params![harness_session, prompt_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let corrupt = |what: String| StoreError::Corrupt {
            address: None,
            what: format!("prompt commitment of {prompt_id:?}: {what}"),
        };
        row.map(|(key, commitment, used)| {
            Ok(PromptCommitment {
                harness_session: harness_session.to_owned(),
                prompt_id: prompt_id.to_owned(),
                key: CommitmentKey::from_hex(&key).map_err(|e| corrupt(e.to_string()))?,
                commitment: Commitment::from_hex(&commitment)
                    .map_err(|e| corrupt(e.to_string()))?,
                used,
            })
        })
        .transpose()
    }

    /// Appends `new_events` to `writer`'s log in one transaction, as
    /// [`append`](Self::append) does, marking the hook observations
    /// `observations` appended and the prompt commitments `used`, each
    /// a harness session and `prompt_id`, used up in the same
    /// transaction.
    ///
    /// # Errors
    ///
    /// As [`append`](Self::append); nothing changes then.
    pub fn append_observed(
        &mut self,
        writer: WriterId,
        new_events: &[NewEvent],
        observations: &[i64],
        used: &[(String, String)],
    ) -> Result<Vec<Address>, StoreError> {
        let to = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let out = if new_events.is_empty() {
            Vec::new()
        } else {
            to.append_chunk(&tx, writer, new_events, true)?
        };
        for id in observations {
            tx.execute(
                "UPDATE hook_observations SET appended = 1 WHERE id = ?1",
                [id],
            )?;
        }
        for (session, prompt_id) in used {
            tx.execute(
                "UPDATE prompt_commitments SET used = 1
                 WHERE harness_session = ?1 AND prompt_id = ?2",
                params![session, prompt_id],
            )?;
        }
        tx.commit()?;

        Ok(out)
    }

    /// Every `SessionStart` hook observation held before `before`, in
    /// seconds since the Unix epoch, whose session binding no MCP server
    /// took and whose failure is not counted yet (REC-19).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn untaken_bindings(&self, before: u64) -> Result<Vec<Held>, StoreError> {
        observations(
            &self.conn,
            &format!(
                "WHERE hook = 'SessionStart' AND binding_counted = 0 AND taken = 0 AND at < {}",
                sql_count(before)
            ),
        )
    }

    /// How many `SessionStart` session bindings an MCP server took, and
    /// how many it did not (REC-13, REC-19).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn binding_counts(&self) -> Result<(u64, u64), StoreError> {
        let count = |taken: u8| -> Result<u64, StoreError> {
            let n: i64 = self.conn.query_row(COUNT_BINDINGS, [taken], |r| r.get(0))?;

            Ok(u64::try_from(n).unwrap_or_default())
        };

        Ok((count(1)?, count(0)?))
    }

    /// Takes every session binding the `SessionStart` hook handlers of
    /// the harness process `harness` recorded that no MCP server took,
    /// marking each taken in one transaction, so no other MCP server
    /// takes it and none is counted untaken (REC-13, REC-19). Returns
    /// them in the order they were held.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does; nothing is taken then.
    pub fn take_bindings(&mut self, harness: u32) -> Result<Vec<Held>, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let filter =
            format!("WHERE hook = 'SessionStart' AND taken = 0 AND harness_process = {harness}");
        let mut taken = observations(&tx, &filter)?;
        tx.execute(
            &format!("UPDATE hook_observations SET taken = 1 {filter}"),
            [],
        )?;
        tx.commit()?;
        for held in &mut taken {
            held.taken = true;
        }

        Ok(taken)
    }

    /// Marks the session binding of hook observation `id` counted as a
    /// failure; returns whether this call did, so of every reader only
    /// one counts it.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn count_binding(&mut self, id: i64) -> Result<bool, StoreError> {
        let changed = self.conn.execute(
            "UPDATE hook_observations SET binding_counted = 1
             WHERE id = ?1 AND binding_counted = 0",
            [id],
        )?;

        Ok(changed == 1)
    }
}

#[cfg(test)]
mod tests;
