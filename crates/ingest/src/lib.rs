//! `parinn ingest` (REC-01 to REC-04, REC-07, REC-08, REC-22): reads a
//! transcript source into the record, incrementally and idempotently.
//!
//! - Each transcript is split by run (REC-02): each line belongs to the
//!   run of the harness's agent id, a line carrying none to the run of
//!   the transcript's first entry. Each run is an ingested run with its
//!   own ingest seat in the personal room, certified by the node's
//!   device key, so its events derive origin `ingested` and stay
//!   untrusted (REC-22). A later ingest of the same transcript source
//!   appends to the same writer.
//! - Each transcript source keeps an ingest position (REC-03), which
//!   moves in the transaction that appends the lines it covers; a
//!   source that shrank, or whose consumed prefix changed, starts a new
//!   transcript generation (REC-07), every earlier event kept.
//! - A line a harness fork repeats is recorded once (REC-04, [`repeat`]).
//! - The transcript parser classes each line from its structure
//!   (PRV-08): a `transcript_line` of one provenance class, or an
//!   `unparsed_line` of provenance `unparsed` (REC-05); a line that
//!   drifts from its recorded shape is kept whole as `harness_text`
//!   (REC-26). A typed line is `user` only once a hook observation read
//!   its harness session's process ancestry, not nested (PRV-08).
//! - The hook observations the hook handlers held for the transcript's
//!   harness session ([`observed`]) reach the record with it, as
//!   structural events of the device seat, each prompt line's prompt
//!   match among them ([`prompt`], REC-19, PRV-08).
//!
//! The transcript is read before anything is recorded, so a transcript
//! that cannot be read records nothing; every line is redacted (SEC-08)
//! before it is written. The harness account identifiers are learnt
//! from the whole transcript first, so a line before the one that names
//! them is redacted too.

pub mod live;
pub mod observed;
pub mod prompt;
pub mod read;
mod reading;
pub mod repeat;
pub mod run;

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use administration::AdminError;
use administration::home::Home;
use key::KeyError;
use key::random::Entropy;
use record::id::{RoomId, SeatId, WriterId};
use redaction::Redactor;
use serde_json::{Value, json};
use store::StoreError;

/// One run an ingest read lines of (REC-02).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    /// Its ingest seat's writer.
    pub writer: WriterId,
    pub seat: SeatId,
    /// The harness's agent id; none for a harness session's main run.
    pub agent_id: Option<String>,
    /// The run its parent link ties a subagent's run to: the main run of
    /// its harness session, when that is known.
    pub parent: Option<WriterId>,
    /// How many events this ingest appended to it.
    pub events: usize,
}

/// What one ingest recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ingested {
    /// The writer of the transcript's own run: that of its first entry.
    pub writer: WriterId,
    pub seat: SeatId,
    /// The personal room its runs' seats are in.
    pub room: RoomId,
    /// Each run this ingest appended to, the transcript's own first.
    pub runs: Vec<RunReport>,
    /// How many events it appended, repeat references among them.
    pub events: usize,
    /// The transcript generation it read (REC-07).
    pub generation: u64,
    /// Whether it started that generation over an earlier one.
    pub new_generation: bool,
    /// Whether an ingest marker now holds repeated lines (REC-04).
    pub held: bool,
    /// Whether that hold began with this ingest.
    pub hold_started: bool,
    /// How many lines a hold released into their own run's writer.
    pub released: usize,
    /// How many repeat references it recorded.
    pub references: usize,
    /// How many lines repeat a `uuid` with other content.
    pub conflicts: usize,
    /// How many repeat owner events it recorded in the device seat.
    pub owners_chosen: usize,
    /// The rule of each redaction made, line by line.
    pub redactions: Vec<String>,
    /// How many lines it stored as `unparsed` events (REC-05).
    pub unparsed: usize,
    /// What an audit entry says of each format drift (REC-26), line by
    /// line: plain names only, each passed through the redactor.
    pub drifts: Vec<String>,
    /// The transcript's path redacted the way its lines were, the
    /// harness account identifiers it taught among the rules: the path
    /// as it may be logged (SEC-08).
    pub redacted_path: String,
    /// The transcript's file name without its extension, redacted the
    /// same way: what an audit entry names it by.
    pub name: String,
    /// How many hook observations it appended (REC-19).
    pub observations: usize,
    /// How many prompt lines matched their prompt commitment (PRV-08).
    pub matched: usize,
    /// How many prompt lines carried a `prompt_id` already used up.
    pub prompt_conflicts: usize,
    /// How many matched prompt lines' hook handlers found no attended
    /// mark in deployment mode `interactive` (PRV-04).
    pub not_attended: usize,
    /// How many witnessed lines repeating another transcript's event
    /// were recorded as their own, never repeat references (REC-04).
    pub witnessed_repeats: usize,
    /// The events the search index leaves out that no ingest has
    /// reported yet (REC-11, I6): the caller audits each, then notes it
    /// with `Store::mark_search_index_gaps_reported`.
    pub index_gaps: Vec<store::IndexGap>,
    /// Why the lookup of `index_gaps` failed, when it did: the ingest
    /// recorded its events all the same, and the caller counts and says
    /// the failure (I6).
    pub index_gaps_unread: Option<String>,
}

/// Why an ingest recorded nothing, or stopped part way.
#[derive(Debug)]
pub enum IngestError {
    /// The transcript cannot be read.
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A directory under a transcript root cannot be listed.
    List {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The transcript is not UTF-8.
    NotUtf8(PathBuf),
    /// The transcript holds no line.
    Empty(PathBuf),
    /// A harness account identifier is too long to redact.
    Redaction(redaction::PatternError),
    /// The random source failed.
    Key(KeyError),
    /// The home or the node failed.
    Admin(AdminError),
    /// The store failed.
    Store(StoreError),
    /// The store failed after `landed` of the `lines` lines read were
    /// recorded, the last in `writer`'s log; those stay, and the ingest
    /// position stands after them.
    Partial {
        writer: WriterId,
        landed: usize,
        lines: usize,
        source: StoreError,
    },
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "read the transcript {}: {source}", path.display())
            }
            Self::List { path, source } => {
                write!(f, "list the transcripts in {}: {source}", path.display())
            }
            Self::NotUtf8(path) => write!(f, "the transcript {} is not UTF-8", path.display()),
            Self::Empty(path) => write!(f, "the transcript {} holds no line", path.display()),
            Self::Redaction(e) => write!(f, "redact the transcript: {e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::Admin(e) => write!(f, "{e}"),
            Self::Store(e) => write!(f, "{e}"),
            Self::Partial {
                writer,
                landed,
                lines,
                source,
            } => write!(
                f,
                "writer {writer} recorded {landed} of {lines} lines before the store failed: {source}"
            ),
        }
    }
}

impl std::error::Error for IngestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } | Self::List { source, .. } => Some(source),
            Self::Redaction(e) => Some(e),
            Self::Key(e) => Some(e),
            Self::Admin(e) => Some(e),
            Self::Store(e) | Self::Partial { source: e, .. } => Some(e),
            Self::NotUtf8(_) | Self::Empty(_) => None,
        }
    }
}

impl From<StoreError> for IngestError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

/// The payload settings an ingest writes under (REC-09).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayloadSettings {
    /// Content whose canonical encoding is longer goes to the payload
    /// store (`payload.threshold_bytes`, REC-09).
    pub payload_threshold: usize,
}

impl Default for PayloadSettings {
    fn default() -> Self {
        Self {
            payload_threshold: store::DEFAULT_PAYLOAD_THRESHOLD,
        }
    }
}

/// What an ingest reads beside the transcript: its payload settings,
/// the harness sessions fork observations name beside those the store
/// holds (REC-04), the time on its clock, which only holds use, and the
/// deployment mode.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inputs {
    pub payload: PayloadSettings,
    /// The harness session ids a `SessionStart` hook observation with
    /// `source` = `fork` names.
    pub forks: BTreeSet<String>,
    /// Now, in milliseconds since the Unix epoch.
    pub now_ms: u64,
    /// Whether the deployment mode is `interactive` (PRV-04).
    pub interactive: bool,
}

/// Ingests the transcript at `path` as [`ingest_with`] does, with the
/// default [`Inputs`].
///
/// # Errors
///
/// As [`ingest_with`].
pub fn ingest_path(
    home: &Home,
    path: &Path,
    redactor: &Redactor,
    entropy: &mut dyn Entropy,
) -> Result<Ingested, IngestError> {
    ingest_with(home, path, redactor, &Inputs::default(), entropy)
}

/// Ingests the transcript source at `path`, a path resolved within the
/// allowed roots, into the node in `home`, starting the node first when
/// it has never started: the lines past its ingest position, each
/// redacted by `redactor` and the harness account identifiers the
/// transcript names, each appended to its run's writer. A writer closes
/// a segment at each turn end and subagent end the transcript records,
/// and at the end of what this ingest appends to it (REC-19). When the
/// MCP server ingesting the same source live got ahead of an append, the
/// source is read again from where it left off, up to [`ATTEMPTS`]
/// times; the report is the last read's (REC-06).
///
/// # Errors
///
/// Fails when the transcript cannot be read, is not UTF-8 or holds no
/// line, or when an identifier cannot be redacted, or the random
/// source, the home, the node or the store fails.
pub fn ingest_with(
    home: &Home,
    path: &Path,
    redactor: &Redactor,
    inputs: &Inputs,
    entropy: &mut dyn Entropy,
) -> Result<Ingested, IngestError> {
    let mut attempt = 1;
    loop {
        match ingest_once(home, path, redactor, inputs, entropy) {
            Err(e) if e.position_moved() && attempt < ATTEMPTS => attempt += 1,
            done => return done,
        }
    }
}

/// How many times [`ingest_with`] reads a source another ingest keeps
/// getting ahead of.
pub const ATTEMPTS: usize = 8;

impl IngestError {
    /// Whether another ingest moved the ingest position first.
    #[must_use]
    pub fn position_moved(&self) -> bool {
        matches!(
            self,
            Self::Store(StoreError::PositionMoved { .. })
                | Self::Partial {
                    source: StoreError::PositionMoved { .. },
                    ..
                }
        )
    }
}

/// One read of the source [`ingest_with`] ingests.
fn ingest_once(
    home: &Home,
    path: &Path,
    redactor: &Redactor,
    inputs: &Inputs,
    entropy: &mut dyn Entropy,
) -> Result<Ingested, IngestError> {
    let bytes = read::transcript(path).map_err(|source| IngestError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let text = String::from_utf8(bytes).map_err(|_| IngestError::NotUtf8(path.to_path_buf()))?;
    if text.is_empty() {
        return Err(IngestError::Empty(path.to_path_buf()));
    }

    reading::ingest(
        home,
        path,
        &text,
        redactor,
        inputs,
        &mut reading::Recorder::Cli,
        entropy,
    )
}

/// The transcripts under `root`: every `.jsonl` file in it or a
/// directory below it, in path order, a subagent's among them. A
/// symbolic link to a transcript is listed as the link, for the caller
/// to resolve within the allowed roots (SEC-18); a linked directory,
/// which could loop, is not descended. A root that does not exist holds
/// none.
///
/// # Errors
///
/// Fails when `root` or a directory below it cannot be listed.
pub fn transcripts(root: &Path) -> Result<Vec<PathBuf>, IngestError> {
    let mut found = Vec::new();
    match fs::symlink_metadata(root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        _ => walk(root, &mut found)?,
    }
    found.sort();

    Ok(found)
}

/// Adds each transcript in `dir` and the directories below it to
/// `found`.
fn walk(dir: &Path, found: &mut Vec<PathBuf>) -> Result<(), IngestError> {
    let failed = |source| IngestError::List {
        path: dir.to_path_buf(),
        source,
    };
    for entry in fs::read_dir(dir).map_err(failed)? {
        let entry = entry.map_err(failed)?;
        let path = entry.path();
        if entry.file_type().map_err(failed)?.is_dir() {
            walk(&path, found)?;
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            found.push(path);
        }
    }

    Ok(())
}

/// Whether the parsed transcript line `line` is the model reply that
/// ends a turn: an `assistant` line that stopped for any reason but a
/// tool call or a pause.
fn turn_end(line: &Value) -> bool {
    line["type"] == "assistant"
        && line["message"]["stop_reason"]
            .as_str()
            .is_some_and(|r| !matches!(r, "tool_use" | "pause_turn"))
}

/// Whether `text` is an RFC 3339 date and time, as a transcript
/// timestamp is: `YYYY-MM-DDTHH:MM:SS`, at most nine fraction digits,
/// then `Z` or an offset `+HH:MM`.
fn well_formed(text: &str) -> bool {
    let b = text.as_bytes();
    let digits =
        |r: std::ops::Range<usize>| b.get(r).is_some_and(|d| d.iter().all(u8::is_ascii_digit));
    let at = |i: usize, c: u8| b.get(i) == Some(&c);
    let date_time = digits(0..4)
        && at(4, b'-')
        && digits(5..7)
        && at(7, b'-')
        && digits(8..10)
        && at(10, b'T')
        && digits(11..13)
        && at(13, b':')
        && digits(14..16)
        && at(16, b':')
        && digits(17..19);
    let mut rest = b.get(19..).unwrap_or_default();
    if let Some(fraction) = rest.strip_prefix(b".") {
        let n = fraction.iter().take_while(|d| d.is_ascii_digit()).count();
        if n == 0 || n > 9 {
            return false;
        }
        rest = &fraction[n..];
    }
    let zone = rest == b"Z"
        || (rest.len() == 6
            && matches!(rest[0], b'+' | b'-')
            && rest[1..3].iter().all(u8::is_ascii_digit)
            && rest[3] == b':'
            && rest[4..6].iter().all(u8::is_ascii_digit));

    date_time && zone
}

/// A line's content: its number and redacted text, and how many
/// redactions the text holds when it holds any.
fn content(line: u64, text: &str, redactions: usize) -> Value {
    let mut content = json!({"line": line, "text": text});
    if redactions > 0 {
        content["redactions"] = json!(redactions);
    }

    content
}

#[cfg(test)]
mod tests;
