//! The search index (§8.2, REC-11, RCL-02, SEC-04): an FTS5 table over
//! each event's text, in a file of its own, [`FILE`], beside the
//! `store.db` that holds the record it derives from. It is a derived
//! artifact (I10): a function of the writer logs the store holds,
//! whatever order they arrived in. `Store::rebuild_search_index` derives
//! it again from the record alone, and so does the next opener of a
//! store whose index file is missing, or lags or leads the record (a
//! crash between the two files' commits). NFR-09 measures the record
//! and the index apart.
//!
//! - It keeps no text of its own (contentless): the record stays the one
//!   place content lives. An event's text is its content's `text`, in the
//!   payload store too, up to the indexing cap of [`DEFAULT_CAP`] bytes.
//! - A row's id derives from the event's address, never from arrival
//!   (S12): its seq in the high bits, a slot taken from its writer id in
//!   the low ones. Newer events of a writer so have the larger ids.
//!   Where two writers' ids share a slot the smaller writer id keeps it
//!   and the other's events are left out, whichever arrived first. A
//!   seq of [`SEQ_LIMIT`] or more is left out too. No layout of a 64-bit
//!   row id is free of such limits and the same for any arrival order
//!   (I10), so each is an [`IndexGap`]: named by `Store::search_index_gaps`
//!   and reported once, as the counters `search_index_writer_unindexed`
//!   and `search_index_seq_beyond_limit` (I6).
//! - Search ranks bounded: BM25 with length normalization over the
//!   newest [`RANKED_MATCHES`] matches after the filters (S2), damped per writer,
//!   under a deadline (SEC-04).

use std::path::Path;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use record::digest::Digest;
use record::id::{Address, RoomId, Seq, WriterId};
use rusqlite::types::Value as Sql;
use rusqlite::{Connection, ErrorCode, OptionalExtension, params, params_from_iter};
use search::Compiled;
use search::rank::{self, Candidate, Hit};
use serde_json::Value;

use crate::StoreError;

/// The index's file name, beside `store.db`.
pub const FILE: &str = "search_index";

/// The name the index file is attached to the store's connection under.
const SCHEMA_NAME: &str = "idx";

/// REC-11's indexing cap: the most bytes of one event's text indexed.
pub const DEFAULT_CAP: usize = 1 << 20;

/// How many of the newest matches a search ranks (S2).
pub const RANKED_MATCHES: usize = 2_000;

/// The tokenizer the index is built with, part of its content (§8.3).
pub const TOKENIZER: &str = "unicode61 remove_diacritics 2";

/// The bits of a row id that hold the writer's slot.
pub const SLOT_BITS: u32 = 35;

/// The slot's mask.
const SLOT_MASK: i64 = (1 << SLOT_BITS) - 1;

/// A seq from here on is not indexed: it would not fit a row id.
pub const SEQ_LIMIT: u64 = 1 << 28;

/// How long a search waits for its deadline watcher to end once the
/// search has (ENG-03).
const WATCHER_GRACE: Duration = Duration::from_secs(5);

/// How many events a rebuild reads at a time.
pub(crate) const CHUNK: i64 = 100;

/// The tables of the search index.
pub(crate) const SCHEMA: &str = "
CREATE VIRTUAL TABLE IF NOT EXISTS idx.events_fts USING fts5(
    text, content = '', contentless_delete = 1, tokenize = 'unicode61 remove_diacritics 2');
CREATE VIRTUAL TABLE IF NOT EXISTS idx.events_vocab USING fts5vocab(events_fts, 'instance');
CREATE TABLE IF NOT EXISTS idx.search_writers (
    slot INTEGER PRIMARY KEY,
    writer TEXT NOT NULL UNIQUE,
    indexed_to INTEGER NOT NULL DEFAULT 0
) STRICT;
CREATE TABLE IF NOT EXISTS idx.search_gaps_reported (
    writer TEXT NOT NULL,
    class TEXT NOT NULL,
    PRIMARY KEY (writer, class)
) STRICT;
CREATE TABLE IF NOT EXISTS idx.search_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;
";

/// Gives a slot to a writer, takes it from the writer that held it, and
/// drops that writer's rows.
const CLAIM_SLOT: &str = "INSERT INTO search_writers (slot, writer) VALUES (?1, ?2)";
const TAKE_SLOT: &str = "UPDATE search_writers SET writer = ?2 WHERE slot = ?1";
const DROP_SLOT_ROWS: &str = "DELETE FROM events_fts WHERE (rowid & ?1) = ?2";

/// Empties the index.
const CLEAR: &str =
    "INSERT INTO events_fts (events_fts) VALUES ('delete-all'); DELETE FROM search_writers;";

/// The writers with events but no slot.
const UNINDEXED: &str = "SELECT DISTINCT writer FROM events
    WHERE writer NOT IN (SELECT writer FROM search_writers) ORDER BY writer";

/// What the canonical digest reads of the index.
const SCHEMA_OBJECTS: &str = "SELECT sql FROM idx.sqlite_master
    WHERE name IN ('events_fts', 'events_vocab', 'search_writers') ORDER BY name";
const POSTINGS: &str =
    "SELECT term, doc, col, offset FROM events_vocab ORDER BY term, doc, col, offset";

/// The settings the index is built under, kept in the index file (§8.3).
pub(crate) fn settings() -> [(&'static str, String); 2] {
    [
        ("search_tokenizer", TOKENIZER.to_owned()),
        ("search_index.indexing_cap", DEFAULT_CAP.to_string()),
    ]
}

/// Whether the index file is attached to `conn`.
fn attached(conn: &Connection) -> Result<bool, StoreError> {
    let found = conn.query_row(
        "SELECT count(*) FROM pragma_database_list WHERE name = ?1",
        [SCHEMA_NAME],
        |r| r.get::<_, bool>(0),
    );

    Ok(found?)
}

/// Attaches the index file at `path` to `conn`. A writer sets the file's
/// journal to a write-ahead log and relaxes its sync: what a crash
/// loses of it the next opener derives again from the record.
pub(crate) fn attach(conn: &Connection, path: &Path, writable: bool) -> Result<(), StoreError> {
    let text = path.to_str().ok_or_else(|| {
        StoreError::Io(std::io::Error::other("the index file's path is not text"))
    })?;
    conn.execute("ATTACH DATABASE ?1 AS idx", [text])?;
    if writable {
        conn.execute_batch("PRAGMA idx.journal_mode = WAL; PRAGMA idx.synchronous = NORMAL")?;
    }

    Ok(())
}

/// Whether the store has its search index yet.
pub(crate) fn exists(conn: &Connection) -> Result<bool, StoreError> {
    if !attached(conn)? {
        return Ok(false);
    }
    let found = conn.query_row(
        "SELECT count(*) FROM idx.sqlite_master WHERE name = 'events_fts'",
        [],
        |r| r.get::<_, bool>(0),
    );

    Ok(found?)
}

/// Records the settings the index is built under.
pub(crate) fn write_settings(conn: &Connection) -> Result<(), StoreError> {
    for (key, value) in settings() {
        conn.execute(
            "INSERT OR REPLACE INTO idx.search_settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
    }

    Ok(())
}

/// Whether the index holds what the record does, given each writer's
/// newest seq (none for a writer with no events): every writer that
/// holds a slot is indexed to its newest event and no further, no slot
/// is held by a writer the record lacks, and a writer without a slot
/// has lost it to a smaller writer id.
pub(crate) fn is_current(conn: &Connection, heads: &[(WriterId, u64)]) -> Result<bool, StoreError> {
    let mut held = std::collections::BTreeMap::new();
    let mut statement = conn.prepare("SELECT writer, slot, indexed_to FROM search_writers")?;
    let rows = statement.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
        ))
    })?;
    for row in rows {
        let (writer, slot, indexed) = row?;
        held.insert(writer, (slot, indexed));
    }
    let owners: std::collections::BTreeMap<i64, &String> =
        held.iter().map(|(w, (slot, _))| (*slot, w)).collect();
    let mut known = 0;
    for (writer, newest) in heads {
        let name = writer.to_string();
        match held.get(&name) {
            Some((_, indexed)) => {
                known += 1;
                if u64::try_from(*indexed).ok() != Some(*newest) {
                    return Ok(false);
                }
            }
            None if *newest == 0 => {}
            None => match owners.get(&slot_of(*writer)) {
                Some(owner) if **owner < name => {}
                _ => return Ok(false),
            },
        }
    }

    Ok(known == held.len())
}

/// The slot a writer takes in row ids: five bytes of its id.
fn slot_of(writer: WriterId) -> i64 {
    let bytes = writer.digest().as_bytes();
    let wide = bytes[..5]
        .iter()
        .fold(0_u64, |n, b| (n << 8) | u64::from(*b));

    i64::try_from(wide).map_or(0, |n| n & SLOT_MASK)
}

/// The row id of the event at `seq` of the writer in `slot`.
fn row_id(slot: i64, seq: Seq) -> i64 {
    i64::try_from(seq.get()).map_or(0, |n| (n << SLOT_BITS) | slot)
}

/// The row id the index gives the event at `address`; none for a seq
/// from [`SEQ_LIMIT`] on, which is not indexed.
#[must_use]
pub fn row_id_of(address: Address) -> Option<i64> {
    (address.seq.get() < SEQ_LIMIT).then(|| row_id(slot_of(address.writer), address.seq))
}

/// An event's text as the index takes it: the content's `text`, cut to
/// at most `cap` bytes at a character boundary; none for content with
/// none.
#[must_use]
pub fn text_of(content: &Value, cap: usize) -> Option<&str> {
    let text = content.get("text")?.as_str()?;
    let mut end = text.len().min(cap);
    while !text.is_char_boundary(end) {
        end -= 1;
    }

    Some(&text[..end])
}

/// The index as one append transaction fills it, for one writer.
pub(crate) struct Indexing<'t> {
    insert: Option<rusqlite::Statement<'t>>,
    slot: i64,
    writer: String,
    conn: &'t Connection,
}

impl<'t> Indexing<'t> {
    /// Claims `writer`'s slot, or finds another writer holding it.
    pub(crate) fn open(conn: &'t Connection, writer: WriterId) -> Result<Self, StoreError> {
        let slot = slot_of(writer);
        let held: Option<String> = conn
            .query_row(
                "SELECT writer FROM search_writers WHERE slot = ?1",
                [slot],
                |r| r.get(0),
            )
            .optional()?;
        let mine = writer.to_string();
        let owner = match held {
            Some(held) if held < mine => false,
            Some(held) if held == mine => true,
            Some(_) => {
                // The smaller writer id keeps the slot: the one that
                // held it is left out, whichever arrived first.
                conn.execute(DROP_SLOT_ROWS, [SLOT_MASK, slot])?;
                conn.execute(TAKE_SLOT, params![slot, mine])?;
                true
            }
            None => {
                conn.execute(CLAIM_SLOT, params![slot, mine])?;
                true
            }
        };
        let insert = if owner {
            Some(conn.prepare("INSERT INTO events_fts (rowid, text) VALUES (?1, ?2)")?)
        } else {
            None
        };

        Ok(Self {
            insert,
            slot,
            writer: mine,
            conn,
        })
    }

    /// Records that the writer's events are indexed through `seq`.
    pub(crate) fn done_to(&self, seq: u64) -> Result<(), StoreError> {
        if self.insert.is_some() {
            self.conn.execute(
                "UPDATE search_writers SET indexed_to = ?2 WHERE writer = ?1",
                params![self.writer, i64::try_from(seq).unwrap_or(i64::MAX)],
            )?;
        }

        Ok(())
    }

    /// Indexes the event at `seq` whose content is `content`.
    pub(crate) fn add(&mut self, seq: Seq, content: &Value) -> Result<(), StoreError> {
        let (Some(insert), Some(text)) = (self.insert.as_mut(), text_of(content, DEFAULT_CAP))
        else {
            return Ok(());
        };
        if seq.get() < SEQ_LIMIT {
            insert.execute(params![row_id(self.slot, seq), text])?;
        }

        Ok(())
    }
}

/// How the search index leaves a writer's events out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GapClass {
    /// Another writer id holds its slot.
    WriterUnindexed,
    /// Its events from [`SEQ_LIMIT`] on do not fit a row id.
    SeqBeyondLimit,
}

impl GapClass {
    /// Its name in the note of what was reported.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::WriterUnindexed => "writer_unindexed",
            Self::SeqBeyondLimit => "seq_beyond_limit",
        }
    }
}

/// Events of one writer the search index leaves out, one class of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexGap {
    pub writer: WriterId,
    pub class: GapClass,
}

/// The gaps of the index, given each writer's newest seq (none for a
/// writer with no events): a writer with events and no slot, and a
/// writer whose newest seq is [`SEQ_LIMIT`] or more.
pub(crate) fn gaps(
    conn: &Connection,
    heads: &[(WriterId, u64)],
) -> Result<Vec<IndexGap>, StoreError> {
    let mut statement = conn.prepare("SELECT 1 FROM search_writers WHERE writer = ?1")?;
    let mut found = Vec::new();
    for (writer, newest) in heads.iter().filter(|(_, newest)| *newest > 0) {
        if !statement.exists([writer.to_string()])? {
            found.push(IndexGap {
                writer: *writer,
                class: GapClass::WriterUnindexed,
            });
        }
        if *newest >= SEQ_LIMIT {
            found.push(IndexGap {
                writer: *writer,
                class: GapClass::SeqBeyondLimit,
            });
        }
    }

    Ok(found)
}

/// Whether `gap` was reported before.
fn reported(conn: &Connection, gap: &IndexGap) -> Result<bool, StoreError> {
    Ok(conn.query_row(
        "SELECT count(*) FROM search_gaps_reported WHERE writer = ?1 AND class = ?2",
        params![gap.writer.to_string(), gap.class.name()],
        |r| r.get::<_, bool>(0),
    )?)
}

/// The gaps of `all` not reported yet.
pub(crate) fn unreported(
    conn: &Connection,
    all: Vec<IndexGap>,
) -> Result<Vec<IndexGap>, StoreError> {
    let mut out = Vec::new();
    for gap in all {
        if !reported(conn, &gap)? {
            out.push(gap);
        }
    }

    Ok(out)
}

/// Notes that `gaps` were reported.
pub(crate) fn mark_reported(conn: &Connection, gaps: &[IndexGap]) -> Result<(), StoreError> {
    for gap in gaps {
        conn.execute(
            "INSERT OR IGNORE INTO search_gaps_reported (writer, class) VALUES (?1, ?2)",
            params![gap.writer.to_string(), gap.class.name()],
        )?;
    }

    Ok(())
}

/// Takes back the note that `gaps` were reported, so the next ingest
/// reports them.
pub(crate) fn unmark_reported(conn: &Connection, gaps: &[IndexGap]) -> Result<(), StoreError> {
    for gap in gaps {
        conn.execute(
            "DELETE FROM search_gaps_reported WHERE writer = ?1 AND class = ?2",
            params![gap.writer.to_string(), gap.class.name()],
        )?;
    }

    Ok(())
}

/// Empties the index, leaving its tables.
pub(crate) fn clear(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(CLEAR)?;

    Ok(())
}

/// The writers a search leaves out: they have events but no slot.
pub(crate) fn unindexed_writers(conn: &Connection) -> Result<Vec<String>, StoreError> {
    let mut statement = conn.prepare(UNINDEXED)?;
    let rows = statement.query_map([], |r| r.get::<_, String>(0))?;

    Ok(rows.collect::<Result<_, _>>()?)
}

/// What a search asks of the index.
#[derive(Debug, Clone, Copy)]
pub struct Search<'a> {
    pub query: &'a Compiled,
    /// The room whose writers are searched.
    pub room: RoomId,
    /// One writer: a run, until runs derive from their seats.
    pub writer: Option<WriterId>,
    /// Provenance classes, by name; none means any.
    pub provenance: &'a [String],
    /// Event kinds, by name; none means any.
    pub kinds: &'a [String],
    /// Transcript times from here on.
    pub since: Option<&'a str>,
    /// Transcript times up to here, a date taking its whole day.
    pub until: Option<&'a str>,
    /// How many hits to return, at most [`rank::MAX_HITS`].
    pub k: usize,
    /// How long the search may run (SEC-04).
    pub deadline: Duration,
}

/// The SQL and arguments of the query for the newest matches for `search`.
/// With `limited`, only the newest [`RANKED_MATCHES`]; without, every
/// match, for a caller that keeps some of them.
fn ranked_matches(search: &Search<'_>, limited: bool) -> (String, Vec<Sql>) {
    let mut sql = String::from(
        "SELECT events_fts.rowid, bm25(events_fts), w.writer
         FROM events_fts
         JOIN search_writers w ON w.slot = (events_fts.rowid & 34359738367)
         JOIN seats s ON s.writer = w.writer
         JOIN events e ON e.writer = w.writer AND e.seq = (events_fts.rowid >> 35)
         WHERE events_fts MATCH ? AND s.room = ?",
    );
    let mut args = vec![
        Sql::Text(search.query.expression().to_owned()),
        Sql::Text(search.room.to_string()),
    ];
    if let Some(writer) = search.writer {
        sql.push_str(" AND w.writer = ?");
        args.push(Sql::Text(writer.to_string()));
    }
    for (column, values) in [("provenance", search.provenance), ("kind", search.kinds)] {
        if !values.is_empty() {
            let marks = vec!["?"; values.len()].join(", ");
            sql.push_str(&format!(" AND e.{column} IN ({marks})"));
            args.extend(values.iter().cloned().map(Sql::Text));
        }
    }
    if let Some(since) = search.since {
        sql.push_str(" AND e.transcript_timestamp >= ?");
        args.push(Sql::Text(since.to_owned()));
    }
    if let Some(until) = search.until {
        sql.push_str(" AND substr(e.transcript_timestamp, 1, length(?)) <= ?");
        args.extend([Sql::Text(until.to_owned()), Sql::Text(until.to_owned())]);
    }
    sql.push_str(" ORDER BY events_fts.rowid DESC");
    if limited {
        sql.push_str(&format!(" LIMIT {RANKED_MATCHES}"));
    }

    (sql, args)
}

/// The newest [`RANKED_MATCHES`] matches of `search` that `keep` keeps
/// (all, when it is `None`).
fn candidates(
    conn: &Connection,
    search: &Search<'_>,
    keep: Option<&dyn Fn(Address) -> bool>,
) -> Result<Vec<Candidate>, StoreError> {
    let (sql, args) = ranked_matches(search, keep.is_none());
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(args), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, f64>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, bm25, writer) = row?;
        let corrupt = || StoreError::Corrupt {
            address: None,
            what: "the search index names an event that is not there".to_owned(),
        };
        let writer = WriterId::parse(&writer).map_err(|_| corrupt())?;
        let seq = u64::try_from(id >> SLOT_BITS).ok().and_then(Seq::new);
        let address = Address::new(writer, seq.ok_or_else(corrupt)?);
        if keep.is_some_and(|keep| !keep(address)) {
            continue;
        }
        out.push(Candidate {
            address,
            relevance: -bm25,
        });
        if out.len() == RANKED_MATCHES {
            break;
        }
    }

    Ok(out)
}

/// Runs `work` on `conn`, interrupting the statement it runs once
/// `deadline` has passed: a deadline already passed cancels it before it
/// starts (SEC-04, ENG-03).
fn within<T>(
    conn: &Connection,
    deadline: Duration,
    work: impl FnOnce() -> Result<T, StoreError>,
) -> Result<T, StoreError> {
    if deadline.is_zero() {
        return Err(StoreError::Cancelled);
    }
    let handle = conn.get_interrupt_handle();
    let (done, wait) = mpsc::channel::<()>();
    let (ended, watched) = mpsc::channel::<()>();
    thread::spawn(move || {
        if wait.recv_timeout(deadline) == Err(RecvTimeoutError::Timeout) {
            handle.interrupt();
        }
        let _ = ended.send(());
    });
    let out = work();
    // The watcher is done once it hears the work has ended, or at once
    // if it fired first; wait for it, so it never interrupts a
    // statement that starts after this one.
    let _ = done.send(());
    let _ = watched.recv_timeout(WATCHER_GRACE);
    match out {
        Err(StoreError::Sqlite(rusqlite::Error::SqliteFailure(e, _)))
            if e.code == ErrorCode::OperationInterrupted =>
        {
            Err(StoreError::Cancelled)
        }
        other => other,
    }
}

/// Searches the index for `search`, cancelled at its deadline.
pub(crate) fn run(
    conn: &Connection,
    search: &Search<'_>,
    keep: Option<&dyn Fn(Address) -> bool>,
) -> Result<Vec<Hit>, StoreError> {
    if !exists(conn)? {
        return Err(StoreError::NoIndex);
    }
    let found = within(conn, search.deadline, || candidates(conn, search, keep))?;

    Ok(rank::rank(&found, search.k))
}

/// The canonical digest of the index's content (§8.3): its settings,
/// its schema, the slot each writer holds and every posting (term,
/// event, position), in key order, folded into one SHA-256. A row id
/// derives from an address, so no storage layout reaches it.
pub(crate) fn digest(conn: &Connection) -> Result<Digest, StoreError> {
    let mut state = Fold(Digest::of(&[b"parinn search index digest v1"]));
    for (key, _) in settings() {
        let value: String = conn.query_row(
            "SELECT value FROM idx.search_settings WHERE key = ?1",
            [key],
            |r| r.get(0),
        )?;
        state.add(&[key.as_bytes(), b"\0", value.as_bytes(), b"\0"]);
    }
    let mut schema = conn.prepare(SCHEMA_OBJECTS)?;
    for sql in schema.query_map([], |r| r.get::<_, String>(0))? {
        state.add(&[sql?.as_bytes(), b"\0"]);
    }
    let mut writers = conn.prepare("SELECT slot, writer FROM search_writers ORDER BY writer")?;
    for row in writers.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))? {
        let (slot, writer) = row?;
        state.add(&[&slot.to_be_bytes(), writer.as_bytes(), b"\0"]);
    }
    let mut postings = conn.prepare(POSTINGS)?;
    let rows = postings.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
        ))
    })?;
    for row in rows {
        let (term, doc, col, offset) = row?;
        let (doc, offset) = (doc.to_be_bytes(), offset.to_be_bytes());
        state.add(&[term.as_bytes(), b"\0", &doc, col.as_bytes(), b"\0", &offset]);
    }

    Ok(state.0)
}

/// A digest folded over the parts of each row in turn.
struct Fold(Digest);

impl Fold {
    fn add(&mut self, parts: &[&[u8]]) {
        let mut all: Vec<&[u8]> = vec![self.0.as_bytes().as_slice()];
        all.extend(parts);
        let next = Digest::of(&all);
        self.0 = next;
    }
}

#[cfg(test)]
mod tests;
