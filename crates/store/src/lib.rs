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
//! - [`Store::purge`]: erasing the content of address ranges with the
//!   tombstones that record it, finished by [`Store::finish_purge`]
//!   (ADM-07, REC-17).
//! - [`segment`]: each writer's log in segments, each with its format
//!   version; a core append closes the segment it ends (REC-19).
//! - [`index`]: the search index over event text, kept in the append
//!   transaction and searched under a deadline (REC-11, RCL-02).

pub mod hook;
mod index_and_seats;
mod integrity;
mod origin;
pub mod payload;
mod purge;
mod recall;
mod refusal;
mod repository;
pub mod search_index;
pub mod segment;
pub mod source;
mod sources_and_previews;

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
pub use purge::{Erase, Finished, Purged};
pub use recall::{EventFacts, RangeEvent, RecallReach, Stats};
pub use search::rank::Hit;
pub use search_index::{GapClass, IndexGap, Search};
pub use segment::{SEGMENT_VERSION, Segment};
pub use source::{IngestedRun, Place, Placed, Position, PurgedLines, Source};

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
    /// When, in milliseconds on ingest's clock, an ingest marker this
    /// store leaves is left (ADM-09).
    marker_time: u64,
    transcripts: Option<source::Transcripts>,
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
            marker_time: 0,
            transcripts: None,
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
        // The index opens for the writer's first event: a writer with no
        // events claims no slot, nor evicts the writer holding it (I10).
        let mut indexing = None;
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
            let open = match indexing.as_mut() {
                Some(open) => open,
                None => indexing.insert(search_index::Indexing::open(tx, writer)?),
            };
            open.add(seq, &new.content)?;
            out.push(address);
            prev_hash = hash;
            next += 1;
        }
        if let Some(open) = &indexing {
            open.done_to(next - 1)?;
        }
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
