//! The store (§8.1, §8.2): `store.db`, which holds the record's writer
//! logs and the seats that write them, through SQLite. Each writer's
//! events are appended inside an immediate transaction, a hundred at a
//! time, which alone assigns their seq and chains them (REC-06,
//! REC-10). The store records its schema version, the header's field
//! set and its encoding in `meta` (§8.3), and refuses a store of a
//! newer version before anything in it changes (ADM-05).
//!
//! - [`payload`]: content above the payload threshold, kept in the
//!   payload store beside `store.db` (REC-09).
//! - [`hook`]: the hook observations and prompt commitments the hook
//!   handlers hold beside the record (REC-19, PRV-08).
//! - [`segment`]: each writer's log in segments, each with its format
//!   version; a core append closes the segment it ends (REC-19).
//! - [`index`]: the search index over event text, kept in the append
//!   transaction and searched under a deadline (REC-11, RCL-02).

pub mod hook;
mod origin;
pub mod payload;
mod refusal;
mod repository;
pub mod search_index;
pub mod segment;
pub mod source;

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::time::Duration;

use record::canonical;
use record::chain::Link;
pub use record::commitment::CommitmentKey;
use record::commitment::{Commitment, commit_encoded};
use record::digest::Digest;
use record::event::{EventKind, Provenance};
pub use record::fork::Fork;
pub use record::header::{ENCODING, EventHeader, FIELD_SET, SeenHead};
pub use record::id::{Address, RoomId, SeatId, Seq, WriterId};
use rusqlite::config::DbConfig;
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params,
};
use serde_json::Value;

pub use hook::{Held, Observation, PromptCommitment};
pub use payload::{Disk, OsDisk, Payloads};
pub use search::rank::Hit;
pub use search_index::{GapClass, IndexGap, Search};
pub use segment::{SEGMENT_VERSION, Segment};
pub use source::{IngestedRun, Place, Placed, Position, Source};

/// The store's schema version (ADM-05, ADM-11).
pub const SCHEMA_VERSION: u32 = 1;

/// The payload threshold `payload.threshold_bytes` defaults to: content
/// whose canonical encoding is longer goes to the payload store
/// (REC-09).
pub const DEFAULT_PAYLOAD_THRESHOLD: usize = 8192;

/// How many events one append transaction commits.
pub const BATCH: usize = 100;

/// How long a write waits for another connection's lock (ENG-03).
const BUSY: Duration = Duration::from_secs(5);

/// Whether the store has its schema yet.
const HAS_META: &str = "SELECT count(*) FROM sqlite_master WHERE name = 'meta'";

/// The store's schema version (ADM-05, ADM-11).
const VERSION: &str = "SELECT value FROM meta WHERE key = 'schema_version'";

/// Every writer of the record and its newest seq, walking the events'
/// key writer by writer.
const NEWEST_SEQS: &str = "WITH RECURSIVE w(writer) AS (
        SELECT min(writer) FROM events
        UNION ALL
        SELECT (SELECT min(writer) FROM events WHERE writer > w.writer) FROM w
        WHERE w.writer IS NOT NULL
    ), all_writers(writer) AS (
        SELECT writer FROM w WHERE writer IS NOT NULL UNION SELECT writer FROM seats
    )
    SELECT writer, (SELECT max(seq) FROM events e WHERE e.writer = a.writer)
    FROM all_writers a ORDER BY writer";

/// The heads one event cites.
const CITED: &str = "SELECT other_writer, other_seq, other_hash FROM event_heads
     WHERE writer = ?1 AND seq = ?2 ORDER BY other_writer";

/// The tables of schema version 1.
const SCHEMA: &str = "
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL) STRICT;
CREATE TABLE seats (
    writer TEXT PRIMARY KEY,
    seat TEXT NOT NULL UNIQUE,
    room TEXT NOT NULL,
    certificate TEXT NOT NULL
) STRICT;
CREATE TABLE events (
    writer TEXT NOT NULL,
    seq INTEGER NOT NULL,
    kind TEXT NOT NULL,
    provenance TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    commitment TEXT NOT NULL,
    hash TEXT NOT NULL,
    content TEXT,
    commitment_key TEXT,
    preview TEXT,
    payload TEXT,
    payload_size INTEGER,
    transcript_timestamp TEXT,
    PRIMARY KEY (writer, seq)
) STRICT;
CREATE TABLE event_heads (
    writer TEXT NOT NULL,
    seq INTEGER NOT NULL,
    other_writer TEXT NOT NULL,
    other_seq INTEGER NOT NULL,
    other_hash TEXT NOT NULL,
    PRIMARY KEY (writer, seq, other_writer)
) STRICT, WITHOUT ROWID;
";

/// What can go wrong with the store.
#[derive(Debug)]
pub enum StoreError {
    /// SQLite refused an operation.
    Sqlite(rusqlite::Error),
    /// The store file cannot be made or read.
    Io(std::io::Error),
    /// The store file can be read by others: its mode, refused rather
    /// than quietly fixed (SEC-02).
    Permissions { mode: u32 },
    /// The store is of a newer schema version than this parinn's.
    Newer { found: u32 },
    /// The store records a header field set or encoding this parinn does
    /// not write, so its hashes would not check here.
    Format { key: String, found: String },
    /// A stored value does not parse; the event it belongs to, if any.
    Corrupt {
        address: Option<Address>,
        what: String,
    },
    /// A new event's content has no canonical encoding.
    Encoding(canonical::Error),
    /// A writer has used every seq.
    Full(WriterId),
    /// A segment is of a newer format version than this parinn's: it is
    /// refused and left as it is (ADM-05).
    NewerSegment(Segment),
    /// The payload of the event at `address` cannot be written or read.
    Payload {
        address: Address,
        source: std::io::Error,
    },
    /// Another ingest moved the ingest position of the transcript source
    /// generation `source` past where this append starts, so its lines
    /// are recorded already: the append is refused whole, for the caller
    /// to read the source again (REC-06, REC-13).
    PositionMoved { source: i64 },
    /// The store has no search index: one a parinn that never wrote to
    /// it made.
    NoIndex,
    /// A search ran past its deadline and was cancelled (SEC-04).
    Cancelled,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(e) => write!(f, "the store: {e}"),
            Self::Io(e) => write!(f, "the store file: {e}"),
            Self::Permissions { mode } => write!(
                f,
                "the store file can be read by others (mode {mode:o}): fix its permissions to 600 (SEC-02)"
            ),
            Self::Newer { found } => write!(
                f,
                "the store is schema version {found}, newer than this parinn's {SCHEMA_VERSION}"
            ),
            Self::Format { key, found } => write!(
                f,
                "the store records {key} {found:?}, which this parinn does not write"
            ),
            Self::Corrupt {
                address: Some(a),
                what,
            } => write!(f, "the stored event {a} is corrupt: {what}"),
            Self::Corrupt {
                address: None,
                what,
            } => write!(f, "the store is corrupt: {what}"),
            Self::Encoding(e) => write!(f, "{e}"),
            Self::Full(w) => write!(f, "writer {w} has no seq left"),
            Self::NewerSegment(s) => write!(
                f,
                "{s} is format version {}, newer than this parinn's {SEGMENT_VERSION}",
                s.format_version
            ),
            Self::Payload { address, source } => {
                write!(f, "the payload of {address}: {source}")
            }
            Self::PositionMoved { source } => write!(
                f,
                "another ingest moved the ingest position of transcript source {source} first"
            ),
            Self::NoIndex => f.write_str("the store has no search index: open it for writing once"),
            Self::Cancelled => f.write_str("the search ran past its deadline and was cancelled"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sqlite(e) => Some(e),
            Self::Io(e) | Self::Payload { source: e, .. } => Some(e),
            Self::Encoding(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

/// An event to append: everything but what its writer's append
/// transaction assigns.
#[derive(Debug, Clone)]
pub struct NewEvent {
    pub kind: EventKind,
    pub provenance: Provenance,
    pub content: Value,
    pub commitment_key: CommitmentKey,
    /// The transcript timestamp of the line it records, kept as
    /// metadata outside the header; none when the line carries none
    /// (REC-12).
    pub transcript_timestamp: Option<String>,
}

/// A stored event: its header and hash, and its content and commitment
/// key, which purge removes (§8.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stored {
    pub link: Link,
    /// Its content, read back from the payload store when it went there.
    pub content: Option<Value>,
    pub commitment_key: Option<CommitmentKey>,
    /// Where its content went when above the payload threshold.
    pub payload: Option<PayloadReference>,
    pub transcript_timestamp: Option<String>,
}

/// What an event keeps of content it put in the payload store: the
/// payload's name and size, and the content's preview (REC-09).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadReference {
    pub name: String,
    /// The payload's length in bytes: its content's canonical encoding.
    pub size: u64,
    pub preview: String,
}

/// A seat that writes in this store: its writer, id, room and seat
/// certificate, kept as the certificate's own JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSeat {
    pub writer: WriterId,
    pub seat: SeatId,
    pub room: RoomId,
    pub certificate: Value,
}

/// An open store.
pub struct Store {
    conn: Connection,
    payloads: Payloads,
    /// Content whose canonical encoding is longer goes to the payload
    /// store.
    threshold: usize,
}

impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store")
            .field("path", &self.conn.path())
            .finish()
    }
}

/// A seq as SQLite stores it: every seq is at most 2^53 - 1, so it
/// fits.
fn sql(seq: Seq) -> i64 {
    seq.get() as i64
}

/// The seq SQLite holds as `n`, if it is one.
fn seq_of(n: i64) -> Option<Seq> {
    u64::try_from(n).ok().and_then(Seq::new)
}

impl Store {
    /// Opens the store at `path`, creating it, readable only by its
    /// owner, when it does not exist.
    ///
    /// # Errors
    ///
    /// Fails when SQLite cannot open it, when it is of a newer format
    /// version, or when its version does not parse.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        create_owner_only(path)?;
        let mut store = Self::over(Connection::open(path)?, path)?;
        // A store of a newer version is refused before anything in it
        // changes, its journal mode included (ADM-05).
        if store.has_meta()? {
            store.check_format()?;
        }
        store.conn.pragma_update(None, "journal_mode", "WAL")?;
        store.conn.pragma_update(None, "synchronous", "FULL")?;
        store.migrate()?;
        store.attach_index(true)?;
        store.ensure_index()?;

        Ok(store)
    }

    /// Opens the store at `path` for reading alone: it is never created
    /// and nothing in it changes.
    ///
    /// # Errors
    ///
    /// Fails when there is no store, when SQLite cannot open it, when it
    /// holds no schema, or when it is of a newer schema version.
    pub fn open_read_only(path: &Path) -> Result<Self, StoreError> {
        Self::open_read_only_made(path)?.ok_or_else(|| StoreError::Corrupt {
            address: None,
            what: "it holds no schema".to_owned(),
        })
    }

    /// Opens the store at `path` for reading alone, as
    /// [`open_read_only`](Self::open_read_only) does; none when it holds
    /// no schema yet, as a store whose first opener was stopped before it
    /// made one, a hook handler past its budget say, holds nothing.
    ///
    /// # Errors
    ///
    /// Fails when there is no store, when SQLite cannot open it, or when
    /// it is of a newer schema version.
    pub fn open_read_only_made(path: &Path) -> Result<Option<Self>, StoreError> {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let store = Self::over(Connection::open_with_flags(path, flags)?, path)?;
        if !store.has_meta()? {
            return Ok(None);
        }
        store.check_format()?;
        store.attach_index(false)?;

        Ok(Some(store))
    }

    /// Attaches the search index's file, beside the store's: a writer
    /// makes it, readable only by its owner, where it is missing; a
    /// reader attaches the one there is (SEC-02).
    fn attach_index(&self, writable: bool) -> Result<(), StoreError> {
        let path = self
            .conn
            .path()
            .map(Path::new)
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("."))
            .join(search_index::FILE);
        if writable {
            create_owner_only(&path)?;
        } else if !path.exists() {
            return Ok(());
        }
        owner_only(&path)?;

        search_index::attach(&self.conn, &path, writable)
    }

    /// The store over `conn`, the file at `path` once checked readable
    /// only by its owner, its payload store beside it.
    fn over(conn: Connection, path: &Path) -> Result<Self, StoreError> {
        owner_only(path)?;
        conn.busy_timeout(BUSY)?;
        let dir = path.parent().unwrap_or_else(|| Path::new("."));

        Ok(Self {
            conn,
            payloads: Payloads::new(dir.join("payloads"), Box::new(OsDisk)),
            threshold: DEFAULT_PAYLOAD_THRESHOLD,
        })
    }

    /// This store, its content above `threshold` bytes of canonical
    /// encoding going to the payload store (`payload.threshold_bytes`).
    #[must_use]
    pub fn with_payload_threshold(mut self, threshold: usize) -> Self {
        self.threshold = threshold;
        self
    }

    /// This store, its connection leaving the write-ahead log to the
    /// next opener as it closes instead of checkpointing it into the
    /// store file: for a short-lived process, such as a hook handler
    /// within its hook budget (NFR-02). Every commit is durable
    /// already, so nothing is lost; the next opener, or SQLite's own
    /// automatic checkpoint, moves the log into the store file.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn without_checkpoint_on_close(self) -> Result<Self, StoreError> {
        self.conn
            .set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)?;
        Ok(self)
    }

    /// This store, its payloads written and read through `disk`.
    #[must_use]
    pub fn with_disk(mut self, disk: Box<dyn Disk>) -> Self {
        let dir = self
            .conn
            .path()
            .and_then(|p| Path::new(p).parent())
            .unwrap_or_else(|| Path::new("."))
            .join("payloads");
        self.payloads = Payloads::new(dir, disk);
        self
    }

    /// Whether the store has its schema yet.
    fn has_meta(&self) -> Result<bool, StoreError> {
        Ok(self.conn.query_row(HAS_META, [], |r| r.get::<_, bool>(0))?)
    }

    /// Creates the schema in a new store, or checks an existing one's
    /// version.
    fn migrate(&mut self) -> Result<(), StoreError> {
        // Reading needs no write lock; only creating the schema does,
        // and that checks again under it.
        if self.has_meta()? {
            return self.check_format();
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        // Another opener may have made the store since the first look.
        let fresh = !tx.query_row(HAS_META, [], |r| r.get::<_, bool>(0))?;
        fresh.then(|| create(&tx)).transpose()?;
        tx.commit()?;

        self.check_format()
    }

    /// Gives a store the search index it lacks, or holds out of step with
    /// its record, derived from the record (REC-11, I10): its file
    /// deleted, one a parinn before the index made, or one a crash between
    /// the two files' commits left behind.
    fn ensure_index(&self) -> Result<(), StoreError> {
        if search_index::exists(&self.conn)? && self.index_is_current()? {
            return Ok(());
        }
        self.conn.execute_batch("BEGIN IMMEDIATE")?;
        // Another opener may build it meanwhile: its tables are made
        // only if missing, and a rebuild is the same whoever runs it.
        let built = self.create_index();
        self.finish(built)
    }

    /// Whether the index holds what the record does. It reads each
    /// writer's newest seq by the record's key, never a scan of the
    /// events, as a hook handler's open stays within its budget (NFR-02).
    fn index_is_current(&self) -> Result<bool, StoreError> {
        search_index::is_current(&self.conn, &self.newest_seqs()?)
    }

    /// Each writer's newest seq, none counted as 0, by the record's key.
    fn newest_seqs(&self) -> Result<Vec<(WriterId, u64)>, StoreError> {
        let mut statement = self.conn.prepare(NEWEST_SEQS)?;
        let rows = statement.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?))
        })?;
        let mut heads = Vec::new();
        for row in rows {
            let (text, newest) = row?;
            let writer = WriterId::parse(&text).map_err(|_| StoreError::Corrupt {
                address: None,
                what: format!("writer id {text:?}"),
            })?;
            heads.push((
                writer,
                newest.and_then(|n| u64::try_from(n).ok()).unwrap_or(0),
            ));
        }

        Ok(heads)
    }

    /// Makes the index's tables and settings, and fills them from the
    /// record.
    fn create_index(&self) -> Result<(), StoreError> {
        self.conn.execute_batch(search_index::SCHEMA)?;
        search_index::write_settings(&self.conn)?;

        self.reindex().map(drop)
    }

    /// Ends the transaction begun on this connection: committed when
    /// `done` is well, else rolled back.
    fn finish<T>(&self, done: Result<T, StoreError>) -> Result<T, StoreError> {
        match done {
            Ok(out) => {
                self.conn.execute_batch("COMMIT")?;
                Ok(out)
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    /// Derives the search index again from the record alone, in one
    /// transaction, and returns how many events it read (I10).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, when an event cannot be read, or when
    /// the store has no search index.
    pub fn rebuild_search_index(&mut self) -> Result<u64, StoreError> {
        if !search_index::exists(&self.conn)? {
            return Err(StoreError::NoIndex);
        }
        self.conn.execute_batch("BEGIN IMMEDIATE")?;
        let done = self.reindex();
        self.finish(done)
    }

    /// Empties the index and fills it from every writer's log, inside a
    /// transaction already begun.
    fn reindex(&self) -> Result<u64, StoreError> {
        search_index::clear(&self.conn)?;
        let mut count = 0;
        for writer in self.writers()? {
            let mut indexing = search_index::Indexing::open(&self.conn, writer)?;
            let mut after = 0;
            loop {
                let chunk =
                    self.rows_limited(writer, "AND seq > ?2", after, search_index::CHUNK)?;
                let Some(last) = chunk.last() else { break };
                after = sql(last.link.header.address.seq);
                index_chunk(&mut indexing, &chunk)?;
                count += chunk.len() as u64;
            }
            indexing.done_to(u64::try_from(after).unwrap_or(0))?;
        }

        Ok(count)
    }

    /// Searches the index (RCL-02): the hits `search` asks for, best
    /// first, under its deadline (SEC-04).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, when the store has no search index, or
    /// with [`StoreError::Cancelled`] when the deadline passes.
    pub fn search(&self, search: &Search<'_>) -> Result<Vec<Hit>, StoreError> {
        search_index::run(&self.conn, search, None)
    }

    /// [`search`](Self::search) over the matches `keep` keeps, chosen
    /// before the ranking cuts them (RCL-02).
    ///
    /// # Errors
    ///
    /// As [`search`](Self::search).
    pub fn search_where(
        &self,
        search: &Search<'_>,
        keep: &dyn Fn(Address) -> bool,
    ) -> Result<Vec<Hit>, StoreError> {
        search_index::run(&self.conn, search, Some(keep))
    }

    /// The search index's canonical digest (§8.3): the same for any two
    /// stores that hold the same writer logs, however they came to
    /// hold them.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the store has no search index.
    pub fn search_index_digest(&self) -> Result<Digest, StoreError> {
        if !search_index::exists(&self.conn)? {
            return Err(StoreError::NoIndex);
        }

        search_index::digest(&self.conn)
    }

    /// The writers whose events the search index leaves out, because
    /// another writer's id holds their slot.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or an id does not parse.
    pub fn search_index_unindexed_writers(&self) -> Result<Vec<WriterId>, StoreError> {
        search_index::unindexed_writers(&self.conn)?
            .iter()
            .map(|text| {
                WriterId::parse(text).map_err(|_| StoreError::Corrupt {
                    address: None,
                    what: format!("writer id {text:?}"),
                })
            })
            .collect()
    }

    /// The events the search index leaves out (REC-11, I6): a writer
    /// whose slot another holds, and a writer from seq 2^28 on.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or an id does not parse.
    pub fn search_index_gaps(&self) -> Result<Vec<IndexGap>, StoreError> {
        search_index::gaps(&self.conn, &self.newest_seqs()?)
    }

    /// The gaps of the search index not yet reported.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, an id does not parse or the store has no
    /// search index.
    pub fn unreported_search_index_gaps(&self) -> Result<Vec<IndexGap>, StoreError> {
        search_index::unreported(&self.conn, self.search_index_gaps()?)
    }

    /// Notes that `gaps` were reported, so the next ingest does not
    /// report them again.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the store has no search index.
    pub fn mark_search_index_gaps_reported(&self, gaps: &[IndexGap]) -> Result<(), StoreError> {
        search_index::mark_reported(&self.conn, gaps)
    }

    /// Takes back the note that `gaps` were reported, for the ones whose
    /// report did not reach the audit log.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the store has no search index.
    pub fn unmark_search_index_gaps_reported(&self, gaps: &[IndexGap]) -> Result<(), StoreError> {
        search_index::unmark_reported(&self.conn, gaps)
    }

    /// Checks the store's schema version, header field set and encoding
    /// against this parinn's (§8.3, ADM-05).
    fn check_format(&self) -> Result<(), StoreError> {
        let version: String = self.conn.query_row(VERSION, [], |r| r.get(0))?;
        let found: u32 = version.parse().map_err(|_| StoreError::Corrupt {
            address: None,
            what: format!("schema_version {version:?}"),
        })?;
        if found > SCHEMA_VERSION {
            return Err(StoreError::Newer { found });
        }
        for (key, want) in [
            ("header_field_set", FIELD_SET),
            ("header_encoding", ENCODING),
        ] {
            let found = self.meta(key)?.unwrap_or_default();
            if found != want {
                return Err(StoreError::Format {
                    key: key.to_owned(),
                    found,
                });
            }
        }

        Ok(())
    }

    /// Runs `read` on one snapshot of the store: every read it makes
    /// sees the store as it stood when the snapshot began, whatever
    /// another connection appends meanwhile.
    ///
    /// # Errors
    ///
    /// Fails when SQLite cannot begin or end the snapshot, or with what
    /// `read` fails with.
    pub fn snapshot<T>(
        &self,
        read: impl FnOnce(&Self) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        self.conn.execute_batch("BEGIN DEFERRED")?;
        // The snapshot is fixed by the first read.
        let begun = self.conn.query_row(HAS_META, [], |r| r.get::<_, bool>(0));
        let out = begun.map_err(StoreError::from).and_then(|_| read(self));
        self.conn.execute_batch("COMMIT")?;

        out
    }

    /// The value `meta` holds under `key`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn meta(&self, key: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
            .optional()?)
    }

    /// Records a seat.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, its writer or seat already recorded.
    pub fn add_seat(&mut self, seat: &StoredSeat) -> Result<(), StoreError> {
        insert_seat(&self.conn, seat)
    }

    /// Records `seat` and appends `new_events` to `writer`, in one
    /// immediate transaction: both land, or neither does.
    ///
    /// # Errors
    ///
    /// Fails, recording nothing, when SQLite does, when the seat is
    /// already recorded, or when a new event's content has no canonical
    /// encoding.
    pub fn add_seat_and_append(
        &mut self,
        seat: &StoredSeat,
        writer: WriterId,
        new_events: &[NewEvent],
    ) -> Result<Vec<Address>, StoreError> {
        let out = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        insert_seat(&tx, seat)?;
        let out = out.append_chunk(&tx, writer, new_events, true)?;
        tx.commit()?;

        Ok(out)
    }

    /// The seat `writer` belongs to.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the row does not parse.
    pub fn seat(&self, writer: WriterId) -> Result<Option<StoredSeat>, StoreError> {
        Ok(self
            .seat_rows("WHERE writer = ?1", &writer.to_string())?
            .pop())
    }

    /// Every seat, by writer id.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn seats(&self) -> Result<Vec<StoredSeat>, StoreError> {
        self.seat_rows("WHERE ?1 = ?1", "")
    }

    fn seat_rows(&self, filter: &str, arg: &str) -> Result<Vec<StoredSeat>, StoreError> {
        let mut statement = self.conn.prepare(&format!(
            "SELECT writer, seat, room, certificate FROM seats {filter} ORDER BY writer"
        ))?;
        let rows = statement.query_map([arg], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        rows.map(|row| {
            let (writer, seat, room, certificate) = row?;
            let corrupt = |what: &str| StoreError::Corrupt {
                address: None,
                what: format!("seat {what}"),
            };
            Ok(StoredSeat {
                writer: WriterId::parse(&writer).map_err(|_| corrupt("writer"))?,
                seat: SeatId::parse(&seat).map_err(|_| corrupt("seat"))?,
                room: RoomId::parse(&room).map_err(|_| corrupt("room"))?,
                certificate: serde_json::from_str(&certificate)
                    .map_err(|_| corrupt("certificate"))?,
            })
        })
        .collect()
    }

    /// Every writer with an event or a seat, by writer id.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or an id does not parse.
    pub fn writers(&self) -> Result<Vec<WriterId>, StoreError> {
        let mut statement = self
            .conn
            .prepare("SELECT writer FROM events UNION SELECT writer FROM seats ORDER BY 1")?;
        let rows = statement.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|row| {
            let text = row?;
            WriterId::parse(&text).map_err(|_| StoreError::Corrupt {
                address: None,
                what: format!("writer id {text:?}"),
            })
        })
        .collect()
    }

    /// Appends `new_events` to `writer`'s log, a hundred to an immediate
    /// transaction, and closes the segment they end in the last: the
    /// core's append. Each transaction reads the writer's newest event,
    /// numbers and chains its events after it, and has its first event
    /// cite every other writer's head that advanced since the writer
    /// last cited it. Content above the payload threshold goes to the
    /// payload store first. Returns the new events' addresses.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, when a new event's content has no canonical
    /// encoding, when a payload cannot be written, when the writer runs
    /// out of seqs, or when it holds a segment of a newer format version.
    /// The transaction the failure falls in is rolled back; those before
    /// it stay.
    pub fn append(
        &mut self,
        writer: WriterId,
        new_events: &[NewEvent],
    ) -> Result<Vec<Address>, StoreError> {
        self.append_closing(writer, new_events, true)
    }

    /// Appends `new_events` to `writer`'s log as [`append`](Self::append)
    /// does, leaving the segment they end in open, for the writer to
    /// close later (REC-19).
    ///
    /// # Errors
    ///
    /// As [`append`](Self::append).
    pub fn append_open(
        &mut self,
        writer: WriterId,
        new_events: &[NewEvent],
    ) -> Result<Vec<Address>, StoreError> {
        self.append_closing(writer, new_events, false)
    }

    fn append_closing(
        &mut self,
        writer: WriterId,
        new_events: &[NewEvent],
        close: bool,
    ) -> Result<Vec<Address>, StoreError> {
        let mut out = Vec::with_capacity(new_events.len());
        let chunks = new_events.chunks(BATCH).count();
        let to = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        for (n, chunk) in new_events.chunks(BATCH).enumerate() {
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            out.extend(to.append_chunk(&tx, writer, chunk, close && n + 1 == chunks)?);
            tx.commit()?;
        }

        Ok(out)
    }

    /// Refuses the part of `offered`, a history brought in from elsewhere,
    /// that differs from an event this store holds at the same (writer,
    /// seq): the first head seen wins, so the held events stay as they
    /// are and the offered ones are never stored as events. Each fork is
    /// recorded, by writer, seq range and the two digests, and the new
    /// ones returned for the caller to count and audit (PEER-10, REC-21,
    /// I1, I6). A fork recorded already is not new.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, a held writer holds a segment of a newer
    /// format version, or a held row does not parse.
    pub fn refuse_forks(&mut self, offered: &[Link]) -> Result<Vec<Fork>, StoreError> {
        let mut writers: Vec<WriterId> = offered.iter().map(|l| l.header.address.writer).collect();
        writers.sort();
        writers.dedup();
        let mut held = Vec::new();
        for writer in writers {
            held.extend(self.log(writer)?.into_iter().map(|s| s.link));
        }
        let forks = record::fork::find(&held, offered);
        if forks.is_empty() {
            return Ok(forks);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute_batch(refusal::SCHEMA)?;
        let new = refusal::record(&tx, &forks)?;
        tx.commit()?;

        Ok(new)
    }

    /// Every fork this store refused, by writer then first seq (PEER-10).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn refused_forks(&self) -> Result<Vec<Fork>, StoreError> {
        refusal::all(&self.conn)
    }

    /// Every writer's kept origin, by writer: a derived artifact (PRV-13,
    /// I10) that decides nothing, kept so `parinn verify` can compare it
    /// to a fresh derivation. None while `parinn rebuild` has not written
    /// it.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a writer id does not parse.
    pub fn origins(&self) -> Result<BTreeMap<WriterId, String>, StoreError> {
        origin::all(&self.conn)
    }

    /// Replaces the kept origins with `origins`, in one transaction.
    ///
    /// # Errors
    ///
    /// Fails, changing nothing, when SQLite does.
    pub fn replace_origins(
        &mut self,
        origins: &BTreeMap<WriterId, String>,
    ) -> Result<(), StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        origin::replace(&tx, origins)?;
        tx.commit()?;

        Ok(())
    }

    /// The identity, as its caller named it, the worktree `worktree`
    /// last resolved to (LANE-02).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn worktree_identity(&self, worktree: &str) -> Result<Option<String>, StoreError> {
        repository::identity(&self.conn, worktree)
    }

    /// Keeps `identity` as what `worktree` resolves to (LANE-02).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn set_worktree_identity(
        &mut self,
        worktree: &str,
        identity: &str,
    ) -> Result<(), StoreError> {
        repository::set_identity(&self.conn, worktree, identity)?;

        Ok(())
    }

    /// What the mark `mark` holds: a node-local note of what this node
    /// already reported or looked at, not derived from the record
    /// (LANE-02).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn worktree_mark(&self, mark: &str) -> Result<Option<String>, StoreError> {
        repository::mark(&self.conn, mark)
    }

    /// Makes the mark `mark` hold `value` (LANE-02).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn set_worktree_mark(&mut self, mark: &str, value: &str) -> Result<(), StoreError> {
        repository::set_mark(&self.conn, mark, value)?;

        Ok(())
    }

    /// Clears the mark `mark`; nothing, when it was never made (LANE-02).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn forget_worktree_mark(&mut self, mark: &str) -> Result<(), StoreError> {
        repository::forget_mark(&self.conn, mark)?;

        Ok(())
    }

    /// Appends the bind `event` to `writer`'s log and keeps `identity` as
    /// what `worktree` resolves to, in one transaction, provided the
    /// worktree still resolves to `expected`: a bind another process
    /// made since the caller looked changes nothing and returns `None`
    /// (LANE-02).
    ///
    /// # Errors
    ///
    /// As [`append`](Self::append); nothing changes then.
    pub fn bind_worktree(
        &mut self,
        writer: WriterId,
        event: &NewEvent,
        worktree: &str,
        expected: Option<&str>,
        identity: &str,
    ) -> Result<Option<Address>, StoreError> {
        let to = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if repository::identity(&tx, worktree)?.as_deref() != expected {
            return Ok(None);
        }
        let address = to.append_chunk(&tx, writer, std::slice::from_ref(event), true)?;
        repository::set_identity(&tx, worktree, identity)?;
        tx.commit()?;

        Ok(address.into_iter().next())
    }

    /// The identity each worktree resolves to, as derived from the
    /// record (LANE-02, I10).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn worktree_identities(&self) -> Result<BTreeMap<String, String>, StoreError> {
        repository::identities(&self.conn)
    }

    /// Replaces the worktree identities with `identities` in one
    /// transaction, leaving the marks alone (LANE-02, I10).
    ///
    /// # Errors
    ///
    /// Fails, changing nothing, when SQLite does.
    pub fn replace_worktree_identities(
        &mut self,
        identities: &BTreeMap<String, String>,
    ) -> Result<(), StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        repository::replace(&tx, identities)?;
        tx.commit()?;

        Ok(())
    }

    /// Closes `writer`'s open segment, if it has one (REC-19).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, or when the writer holds a segment of a
    /// newer format version.
    pub fn close_segment(&mut self, writer: WriterId) -> Result<(), StoreError> {
        refuse_newer(&self.conn, writer)?;

        Ok(segment::close(&self.conn, writer)?)
    }

    /// `writer`'s segments, in seq order.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn segments(&self, writer: WriterId) -> Result<Vec<Segment>, StoreError> {
        self.select_segments("WHERE writer = ?1 AND ?2 = ?2", &writer.to_string())
    }

    /// Every open segment, by writer: each writer chain that ended
    /// without a closed segment, unless its writer still appends.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn open_segments(&self) -> Result<Vec<Segment>, StoreError> {
        self.select_segments("WHERE closed = 0 AND ?1 = ?1 AND ?2 = ?2", "")
    }

    /// Every segment of a newer format version than this parinn's, by
    /// writer (ADM-05).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn newer_segments(&self) -> Result<Vec<Segment>, StoreError> {
        self.select_segments("WHERE format_version > ?2 AND ?1 = ?1", "")
    }

    fn select_segments(&self, filter: &str, arg: &str) -> Result<Vec<Segment>, StoreError> {
        segment::select(&self.conn, filter, arg)?
            .into_iter()
            .map(|row| {
                row.map_err(|bad| StoreError::Corrupt {
                    address: None,
                    what: bad.0,
                })
            })
            .collect()
    }

    /// The newest generation of the transcript source at `path`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or its row does not parse.
    pub fn source(&self, path: &str) -> Result<Option<Source>, StoreError> {
        let found = source::select(
            &self.conn,
            &format!("{} AND path = ?1", source::NEWEST),
            path,
        );

        found.map(|mut s| s.pop())
    }

    /// Every generation of the transcript source at `path`, oldest
    /// first.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn generations(&self, path: &str) -> Result<Vec<Source>, StoreError> {
        source::select(&self.conn, "WHERE path = ?1 ORDER BY generation", path)
    }

    /// The newest generation of every transcript source, by path.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn sources(&self) -> Result<Vec<Source>, StoreError> {
        source::select(
            &self.conn,
            &format!("{} AND ?1 = ?1 ORDER BY path", source::NEWEST),
            "",
        )
    }

    /// Starts the next generation of the transcript source at `path`,
    /// which lies in the transcript directory `directory`, its first
    /// when none was read (REC-07).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn start_generation(&mut self, path: &str, directory: &str) -> Result<Source, StoreError> {
        source::start(&self.conn, path, directory)
    }

    /// Moves the position of the transcript source generation `id` to
    /// `position`, appending nothing; never back from where another
    /// ingest moved it.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn advance(&mut self, id: i64, position: &Position) -> Result<(), StoreError> {
        source::advance(&self.conn, id, position)
    }

    /// Marks the transcript source generation `id` held at its ingest
    /// marker since `since`, or, with none, no longer held (REC-04).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn hold(&mut self, id: i64, since: Option<u64>) -> Result<(), StoreError> {
        source::hold(&self.conn, id, since)
    }

    /// Marks the transcript source generation `id` gone before it was
    /// fully ingested (REC-08).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn lose(&mut self, id: i64) -> Result<(), StoreError> {
        source::lose(&self.conn, id)
    }

    /// Appends `new_events` to `writer`'s log in one transaction, as
    /// [`append`](Self::append) does, keeping each one's place in the
    /// transcript source generation `id` and moving its position to
    /// `position` in the same transaction (REC-03); the writer's segment
    /// closes after each event `closes` marks `true` (REC-19).
    ///
    /// # Errors
    ///
    /// As [`append`](Self::append), and [`StoreError::PositionMoved`]
    /// when another ingest moved the position past the first line;
    /// nothing is appended or moved then.
    pub fn append_lines(
        &mut self,
        writer: WriterId,
        new_events: &[NewEvent],
        places: &[Place],
        closes: &[bool],
        id: i64,
        position: &Position,
    ) -> Result<Vec<Address>, StoreError> {
        let to = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        // An ingest that read the source before another appended starts
        // behind the position that one moved: its lines are recorded.
        let first = places.first().map_or(position.offset, |p| p.offset);
        if source::offset(&tx, id)? > first {
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
        source::place(&tx, id, &out, places)?;
        source::advance(&tx, id, position)?;
        tx.commit()?;

        Ok(out)
    }

    /// The events of the transcript source generation `id`, in line
    /// order.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn placed(&self, id: i64) -> Result<Vec<Placed>, StoreError> {
        source::placed(&self.conn, id)
    }

    /// The event that first recorded the line `uuid` names in the
    /// transcript directory `directory`, read from a transcript source
    /// other than `path`; never a repeat reference (REC-04).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn line_owner(
        &self,
        directory: &str,
        uuid: &str,
        path: &str,
    ) -> Result<Option<Address>, StoreError> {
        source::owner(&self.conn, directory, uuid, path)
    }

    /// Keeps `run`, an ingested run (REC-02): for its transcript source
    /// and agent id, it takes over from any kept before, whose writer
    /// gets nothing more (REC-19).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, or when its writer is already kept.
    pub fn add_run(&mut self, run: &IngestedRun) -> Result<(), StoreError> {
        source::add_run(&self.conn, run)
    }

    /// The ingested run of the transcript source at `path` and the
    /// harness's agent id `agent_id`, none for the main run: the one
    /// kept last.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn ingested_run(
        &self,
        path: &str,
        agent_id: Option<&str>,
    ) -> Result<Option<IngestedRun>, StoreError> {
        let found = source::runs(
            &self.conn,
            "WHERE path = ?1 AND ifnull(agent_id, '') = ?2",
            path,
            agent_id.unwrap_or_default(),
        );

        found.map(|mut r| r.pop())
    }

    /// Every ingested run, in the order they were kept.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn ingested_runs(&self) -> Result<Vec<IngestedRun>, StoreError> {
        source::runs(&self.conn, "WHERE ?1 = ?2", "", "")
    }

    /// The event at `address`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or its row does not parse.
    pub fn event(&self, address: Address) -> Result<Option<Stored>, StoreError> {
        Ok(self
            .rows(address.writer, "AND seq = ?2", sql(address.seq))?
            .pop())
    }

    /// The preview of the event at `address` (REC-09): the first bytes
    /// of its content's text, at most [`payload::PREVIEW_BYTES`], cut at
    /// a character boundary. It is read from the row alone, so an event
    /// whose content went to the payload store shows the preview its
    /// row keeps; none for an event that is not there.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the row does not parse.
    pub fn preview(&self, address: Address) -> Result<Option<String>, StoreError> {
        let row = self
            .conn
            .query_row(
                PREVIEW_ROW,
                params![address.writer.to_string(), sql(address.seq)],
                |r| {
                    Ok((
                        r.get::<_, Option<String>>(0)?,
                        r.get::<_, Option<String>>(1)?,
                    ))
                },
            )
            .optional()?;
        let Some((content, preview)) = row else {
            return Ok(None);
        };
        let Some(content) = content else {
            return Ok(Some(preview.unwrap_or_default()));
        };
        let value: Value = serde_json::from_str(&content).map_err(|_| StoreError::Corrupt {
            address: Some(address),
            what: "content".to_owned(),
        })?;

        Ok(Some(payload::preview(&value, &content)))
    }

    /// The kind and provenance class of the event at `address`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the row does not parse.
    pub fn kind_and_provenance(
        &self,
        address: Address,
    ) -> Result<Option<(EventKind, Provenance)>, StoreError> {
        let row = self
            .conn
            .query_row(
                CLASS_ROW,
                params![address.writer.to_string(), sql(address.seq)],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?;
        let corrupt = |what: &str| StoreError::Corrupt {
            address: Some(address),
            what: what.to_owned(),
        };

        row.map(|(kind, provenance)| {
            Ok((
                EventKind::parse(&kind).ok_or_else(|| corrupt("kind"))?,
                Provenance::parse(&provenance).ok_or_else(|| corrupt("provenance"))?,
            ))
        })
        .transpose()
    }

    /// `writer`'s log, in seq order.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn log(&self, writer: WriterId) -> Result<Vec<Stored>, StoreError> {
        self.rows(writer, "AND ?2 = ?2", 0)
    }

    fn rows(&self, writer: WriterId, filter: &str, seq: i64) -> Result<Vec<Stored>, StoreError> {
        self.rows_limited(writer, filter, seq, i64::MAX)
    }

    /// At most `limit` of the rows `rows` reads.
    fn rows_limited(
        &self,
        writer: WriterId,
        filter: &str,
        seq: i64,
        limit: i64,
    ) -> Result<Vec<Stored>, StoreError> {
        refuse_newer(&self.conn, writer)?;
        let mut statement = self.conn.prepare(&format!(
            "SELECT seq, kind, provenance, prev_hash, commitment, hash, content, commitment_key,
                    preview, payload, payload_size, transcript_timestamp
             FROM events WHERE writer = ?1 {filter} ORDER BY seq LIMIT ?3"
        ))?;
        let rows = statement.query_map(params![writer.to_string(), seq, limit], |r| {
            Ok(Row {
                seq: r.get(0)?,
                kind: r.get(1)?,
                provenance: r.get(2)?,
                prev_hash: r.get(3)?,
                commitment: r.get(4)?,
                hash: r.get(5)?,
                content: r.get(6)?,
                commitment_key: r.get(7)?,
                preview: r.get(8)?,
                payload: r.get(9)?,
                payload_size: r.get(10)?,
                transcript_timestamp: r.get(11)?,
            })
        })?;
        let rows: Vec<Row> = rows.collect::<Result<_, _>>()?;
        rows.into_iter()
            .map(|row| self.stored(writer, row))
            .collect()
    }

    fn stored(&self, writer: WriterId, mut row: Row) -> Result<Stored, StoreError> {
        let seq = seq_of(row.seq).ok_or_else(|| StoreError::Corrupt {
            address: None,
            what: format!("seq {} of writer {writer}", row.seq),
        })?;
        let address = Address::new(writer, seq);
        let corrupt = |what: &str| StoreError::Corrupt {
            address: Some(address),
            what: what.to_owned(),
        };
        let digest = |text: &str, what: &str| {
            Digest::from_hex(text).map_err(|e| corrupt(&format!("{what}: {e}")))
        };
        let header = EventHeader {
            address,
            kind: EventKind::parse(&row.kind).ok_or_else(|| corrupt("kind"))?,
            provenance: Provenance::parse(&row.provenance).ok_or_else(|| corrupt("provenance"))?,
            prev_hash: digest(&row.prev_hash, "prev_hash")?,
            heads: self.heads(address)?,
            commitment: Commitment::from_hex(&row.commitment)
                .map_err(|e| corrupt(&format!("commitment: {e}")))?,
        };
        let payload = payload_reference(&mut row).map_err(corrupt)?;
        let text = match &payload {
            Some(p) => Some(self.read_payload(address, &p.name)?),
            None => row.content.take(),
        };
        let content = text
            .map(|text| serde_json::from_str(&text).map_err(|e| corrupt(&format!("content: {e}"))))
            .transpose()?;
        let commitment_key = row
            .commitment_key
            .map(|text| {
                CommitmentKey::from_hex(&text).map_err(|e| corrupt(&format!("commitment_key: {e}")))
            })
            .transpose()?;

        Ok(Stored {
            link: Link {
                header,
                hash: digest(&row.hash, "hash")?,
            },
            content,
            commitment_key,
            payload,
            transcript_timestamp: row.transcript_timestamp,
        })
    }

    /// The text of the payload `name` of the event at `address`.
    fn read_payload(&self, address: Address, name: &str) -> Result<String, StoreError> {
        let bytes = self
            .payloads
            .read(name)
            .map_err(|source| StoreError::Payload { address, source })?;

        String::from_utf8(bytes).map_err(|_| StoreError::Corrupt {
            address: Some(address),
            what: "content: its payload is not UTF-8".to_owned(),
        })
    }

    /// The heads the event at `address` cites.
    fn heads(&self, address: Address) -> Result<Vec<SeenHead>, StoreError> {
        let mut statement = self.conn.prepare(CITED)?;
        let rows =
            statement.query_map(params![address.writer.to_string(), sql(address.seq)], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?;
        rows.map(|row| {
            let (writer, seq, hash) = row?;
            let corrupt = |what: &str| StoreError::Corrupt {
                address: Some(address),
                what: format!("cited head {what}"),
            };
            let writer = WriterId::parse(&writer).map_err(|_| corrupt("writer"))?;
            let seq = seq_of(seq).ok_or_else(|| corrupt("seq"))?;
            let head = Digest::from_hex(&hash).map_err(|_| corrupt("hash"))?;
            Ok(SeenHead::new(Address::new(writer, seq), head))
        })
        .collect()
    }
}

/// Where an append writes: the payload store, for content above the
/// payload threshold.
struct Out<'a> {
    payloads: &'a Payloads,
    threshold: usize,
}

impl Out<'_> {
    /// Appends one transaction's new events to `writer`, writing each
    /// payload first, and extends its segment, closed after when
    /// `close`.
    fn append_chunk(
        &self,
        tx: &Transaction<'_>,
        writer: WriterId,
        new_events: &[NewEvent],
        close: bool,
    ) -> Result<Vec<Address>, StoreError> {
        refuse_newer(tx, writer)?;
        let (mut next, mut prev_hash) = newest(tx, writer)?;
        let mut heads = advanced_heads(tx, writer)?;
        let mut out = Vec::with_capacity(new_events.len());
        let mut inserts = Inserts {
            event: tx.prepare(INSERT_EVENT)?,
            head: tx.prepare(INSERT_HEAD)?,
        };
        let mut indexing = search_index::Indexing::open(tx, writer)?;
        for new in new_events {
            let seq = Seq::new(next).ok_or(StoreError::Full(writer))?;
            let address = Address::new(writer, seq);
            let canonical = canonical::encode(&new.content).map_err(StoreError::Encoding)?;
            let header = EventHeader {
                address,
                kind: new.kind,
                provenance: new.provenance.clone(),
                prev_hash,
                heads: std::mem::take(&mut heads),
                commitment: commit_encoded(&new.commitment_key, &canonical),
            };
            let hash = header.hash();
            let body = self.body(address, new, canonical)?;
            insert_event(&mut inserts, &header, hash, new, &body)?;
            indexing.add(seq, &new.content)?;
            out.push(address);
            prev_hash = hash;
            next += 1;
        }
        indexing.done_to(next - 1)?;
        if let (Some(first), Some(last)) = (out.first(), out.last()) {
            segment::extend(tx, writer, first.seq, last.seq, close)?;
        }

        Ok(out)
    }

    /// Where the event at `address` keeps the content whose canonical
    /// encoding is `canonical`: inline, or, above the payload threshold,
    /// in the payload store, written now, with the row keeping its
    /// reference.
    fn body(
        &self,
        address: Address,
        new: &NewEvent,
        canonical: String,
    ) -> Result<Body, StoreError> {
        if canonical.len() <= self.threshold {
            return Ok(Body::Inline(canonical));
        }
        let reference = PayloadReference {
            name: payload::name(&new.commitment_key),
            size: canonical.len() as u64,
            preview: payload::preview(&new.content, &canonical),
        };
        self.payloads
            .write(&reference.name, canonical.as_bytes())
            .map_err(|source| StoreError::Payload { address, source })?;

        Ok(Body::Payload(reference))
    }
}

/// One event row as SQLite returns it.
struct Row {
    seq: i64,
    kind: String,
    provenance: String,
    prev_hash: String,
    commitment: String,
    hash: String,
    content: Option<String>,
    commitment_key: Option<String>,
    preview: Option<String>,
    payload: Option<String>,
    payload_size: Option<i64>,
    transcript_timestamp: Option<String>,
}

/// The payload reference `row` holds, taken out of it: none, or a name,
/// size and preview together.
fn payload_reference(row: &mut Row) -> Result<Option<PayloadReference>, &'static str> {
    match (row.payload.take(), row.payload_size, row.preview.take()) {
        (None, None, None) => Ok(None),
        (Some(name), Some(size), Some(preview)) => Ok(Some(PayloadReference {
            name,
            size: u64::try_from(size).map_err(|_| "payload_size")?,
            preview,
        })),
        _ => Err("payload reference: its name, size and preview stand together"),
    }
}

/// Where a new event's content goes.
enum Body {
    /// Inline in its row: its canonical encoding.
    Inline(String),
    /// In the payload store, its row keeping the reference.
    Payload(PayloadReference),
}

/// Inserts an event's row.
const INSERT_EVENT: &str =
    "INSERT INTO events (writer, seq, kind, provenance, prev_hash, commitment, hash, content,
                         commitment_key, preview, payload, payload_size, transcript_timestamp)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)";

/// Inserts one head an event cites.
const INSERT_HEAD: &str =
    "INSERT INTO event_heads (writer, seq, other_writer, other_seq, other_hash)
     VALUES (?1, ?2, ?3, ?4, ?5)";

/// The statements an append transaction inserts its events with,
/// prepared once for all of them.
struct Inserts<'t> {
    event: rusqlite::Statement<'t>,
    head: rusqlite::Statement<'t>,
}

/// Inserts the row of the new event `new`, whose header is `header` and
/// hash `hash`, its content kept as `body`, and the heads it cites.
fn insert_event(
    inserts: &mut Inserts<'_>,
    header: &EventHeader,
    hash: Digest,
    new: &NewEvent,
    body: &Body,
) -> Result<(), StoreError> {
    let id = header.address.writer.to_string();
    let seq = sql(header.address.seq);
    let (content, reference) = match body {
        Body::Inline(text) => (Some(text.as_str()), None),
        Body::Payload(r) => (None, Some(r)),
    };
    inserts.event.execute(params![
        id,
        seq,
        header.kind.as_str(),
        header.provenance.name(),
        header.prev_hash.to_hex(),
        header.commitment.to_string(),
        hash.to_hex(),
        content,
        new.commitment_key.to_hex(),
        reference.map(|r| r.preview.as_str()),
        reference.map(|r| r.name.as_str()),
        reference.map(|r| sql_size(r.size)),
        new.transcript_timestamp,
    ])?;
    for seen in &header.heads {
        let inserted = inserts.head.execute(params![
            id,
            seq,
            seen.address.writer.to_string(),
            sql(seen.address.seq),
            seen.head.to_hex()
        ]);
        inserted?;
    }

    Ok(())
}

/// A payload's size as SQLite stores it: a payload is no longer than a
/// string in memory, so it fits.
fn sql_size(size: u64) -> i64 {
    i64::try_from(size).unwrap_or(i64::MAX)
}

/// Refuses `writer` when it holds a segment of a newer format version
/// (ADM-05).
fn refuse_newer(conn: &Connection, writer: WriterId) -> Result<(), StoreError> {
    match segment::newer(conn, writer)? {
        None => Ok(()),
        Some(Ok(newer)) => Err(StoreError::NewerSegment(newer)),
        Some(Err(bad)) => Err(StoreError::Corrupt {
            address: None,
            what: bad.0,
        }),
    }
}

/// The seq `writer`'s next event takes and the hash it chains to.
fn newest(tx: &Transaction<'_>, writer: WriterId) -> Result<(u64, Digest), StoreError> {
    let newest: Option<(i64, String)> = tx
        .query_row(
            "SELECT seq, hash FROM events WHERE writer = ?1 ORDER BY seq DESC LIMIT 1",
            [writer.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    match newest {
        Some((seq, hash)) => Ok((
            u64::try_from(seq).unwrap_or(0) + 1,
            Digest::from_hex(&hash).map_err(|e| StoreError::Corrupt {
                address: seq_of(seq).map(|s| Address::new(writer, s)),
                what: format!("hash: {e}"),
            })?,
        )),
        None => Ok((1, Digest::ZERO)),
    }
}

/// Inserts the row of `seat`.
fn insert_seat(conn: &Connection, seat: &StoredSeat) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO seats (writer, seat, room, certificate) VALUES (?1, ?2, ?3, ?4)",
        params![
            seat.writer.to_string(),
            seat.seat.to_string(),
            seat.room.to_string(),
            seat.certificate.to_string()
        ],
    )?;

    Ok(())
}

/// Makes the store's tables and records its format in `meta`.
fn create(tx: &rusqlite::Transaction<'_>) -> Result<(), StoreError> {
    tx.execute_batch(SCHEMA)?;
    tx.execute_batch(segment::SCHEMA)?;
    tx.execute_batch(source::SCHEMA)?;
    tx.execute_batch(hook::SCHEMA)?;
    let mut insert = tx.prepare("INSERT INTO meta (key, value) VALUES (?1, ?2)")?;
    for (key, value) in [
        ("schema_version", SCHEMA_VERSION.to_string()),
        ("header_field_set", FIELD_SET.to_owned()),
        ("header_encoding", ENCODING.to_owned()),
    ] {
        insert.execute(params![key, value])?;
    }

    Ok(())
}

/// Indexes the events of `chunk` that still have their content.
fn index_chunk(
    indexing: &mut search_index::Indexing<'_>,
    chunk: &[Stored],
) -> Result<(), StoreError> {
    for stored in chunk {
        let seq = stored.link.header.address.seq;
        stored
            .content
            .iter()
            .try_for_each(|c| indexing.add(seq, c))?;
    }

    Ok(())
}

/// The columns a preview reads, and those a hit's classes.
const PREVIEW_ROW: &str = "SELECT content, preview FROM events WHERE writer = ?1 AND seq = ?2";
const CLASS_ROW: &str = "SELECT kind, provenance FROM events WHERE writer = ?1 AND seq = ?2";

/// Every other writer's newest event, of a writer whose seat is in
/// `writer`'s room, that lies beyond what `writer` last cited of it. A
/// writer never cites a head of another room, so no room's record
/// names another's writers, and a node holding one room verifies its
/// heads.
fn advanced_heads(tx: &Transaction<'_>, writer: WriterId) -> Result<Vec<SeenHead>, StoreError> {
    let mut statement = tx.prepare(
        "SELECT e.writer, e.seq, e.hash FROM events e
         JOIN (SELECT writer, max(seq) AS seq FROM events WHERE writer != ?1 GROUP BY writer) n
           ON e.writer = n.writer AND e.seq = n.seq
         WHERE (SELECT room FROM seats s WHERE s.writer = e.writer)
               IS (SELECT room FROM seats s WHERE s.writer = ?1)
           AND e.seq > coalesce(
           (SELECT max(other_seq) FROM event_heads h WHERE h.writer = ?1 AND h.other_writer = e.writer), 0)
         ORDER BY e.writer",
    )?;
    let rows = statement.query_map([writer.to_string()], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    rows.map(|row| {
        let (other, seq, hash) = row?;
        let address = WriterId::parse(&other)
            .ok()
            .zip(seq_of(seq))
            .map(|(w, s)| Address::new(w, s));
        let corrupt = || StoreError::Corrupt {
            address,
            what: "the head it would cite".to_owned(),
        };
        let head = Digest::from_hex(&hash).map_err(|_| corrupt())?;
        Ok(SeenHead::new(address.ok_or_else(corrupt)?, head))
    })
    .collect()
}

/// Creates the store file, readable only by its owner, before SQLite
/// opens it, so neither it nor the WAL and shared-memory files SQLite
/// makes beside it with its mode are ever readable by others (§8.1).
/// An existing file is left as it is.
fn create_owner_only(path: &Path) -> Result<(), StoreError> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(false);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path).map(drop).map_err(StoreError::Io)
}

/// Refuses a store file others can read (SEC-02), leaving it as it is
/// so the principal sees that it was exposed.
fn owner_only(path: &Path) -> Result<(), StoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)
            .map_err(StoreError::Io)?
            .permissions()
            .mode()
            & 0o777;
        if mode & 0o077 != 0 {
            return Err(StoreError::Permissions { mode });
        }
    }
    #[cfg(not(unix))]
    let _ = path;

    Ok(())
}

#[cfg(test)]
mod tests;
