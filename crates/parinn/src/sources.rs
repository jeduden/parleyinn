//! What `parinn status`, `parinn verify` and `parinn ingest` say of the
//! transcript sources (REC-04, REC-08): each source an ingest marker
//! holds repeated lines of, each source that disappeared before it was
//! fully ingested, and how far `parinn verify` finds each short of its
//! end (ADM-09).

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use administration::home::Home;
use record::digest::{Digest, Running};
use serde_json::{Value, json};
use store::{Source, Store, StoreError};

use crate::event::inline;

/// The transcript sources `parinn status` reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourcesStatus {
    /// The path of each source an ingest marker holds.
    pub(crate) held: Vec<String>,
    /// The path of each source gone before it was fully ingested.
    pub(crate) lost: Vec<String>,
}

/// The transcript sources of the store in `home`, read without changing
/// it; none while there is no store.
///
/// # Errors
///
/// Fails when the store cannot be read, or is of a newer schema version.
pub(crate) fn read(home: &Home) -> Result<Option<SourcesStatus>, StoreError> {
    let path = home.store();
    if !path.exists() {
        return Ok(None);
    }
    // A store its first opener was stopped before it made a schema in,
    // such as a hook handler past its hook budget, holds nothing yet.
    let Some(store) = Store::open_read_only_made(&path)? else {
        return Ok(None);
    };

    Ok(Some(of(&store)?))
}

/// The transcript sources `store` holds, as `parinn status` reports them.
///
/// # Errors
///
/// Fails when the store cannot be read.
pub(crate) fn of(store: &Store) -> Result<SourcesStatus, StoreError> {
    let sources = store.sources()?;

    Ok(SourcesStatus {
        held: sources
            .iter()
            .filter(|s| s.held_since.is_some())
            .map(|s| s.path.clone())
            .collect(),
        lost: lost(&sources).into_iter().map(|s| s.path.clone()).collect(),
    })
}

/// Each of `sources` that disappeared before it was fully ingested: its
/// file gone while bytes it held were unread, an ingest marker held part
/// of it, or ingest already found it so (REC-08).
pub(crate) fn lost(sources: &[Source]) -> Vec<&Source> {
    sources
        .iter()
        .filter(|s| {
            let gone = std::fs::symlink_metadata(Path::new(&s.path)).is_err();
            s.lost || (gone && (!s.position.complete() || s.held_since.is_some()))
        })
        .collect()
}

/// How long a source an ingest stopped short of stays short, its
/// ingest marker at its position, before `parinn verify` fails it: 10
/// minutes after the ingest left the marker, on either side of verify's
/// clock (ADM-09). An ingest killed mid-way leaves the record whole up
/// to its marker, from which the next `parinn ingest` resumes (REC-13,
/// ENG-06); a marker nothing resumed within the bound no longer counts.
pub(crate) const MARKER_CEILING_MS: u64 = 10 * 60 * 1000;

/// Why a source an ingest stopped short of longer ago than
/// [`MARKER_CEILING_MS`] fails.
pub(crate) const STALE: &str = "was left short of its end by an ingest over 10 minutes ago and \
                                nothing resumed it: parinn ingest resumes it";

/// Why a source that no longer holds the bytes ingested from it fails
/// (REC-07, I1).
pub(crate) const REPLACED: &str =
    "no longer begins with the bytes ingested from it: cut, replaced or rewritten since";

/// Why a source a whole line lies past the position of fails, while no
/// ingest marker or running MCP server accounts for it.
const UNSERVED: &str = "holds lines the record lacks and no running MCP server ingests them";

/// How many bytes a scan reads at a time.
const CHUNK: usize = 64 * 1024;

/// What `parinn verify` reads of a transcript source's file: the
/// SHA-256 of its bytes up to the ingest position, none when it holds
/// fewer, and whether a whole line lies past them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Scanned {
    pub(crate) prefix: Option<Digest>,
    pub(crate) unread_line: bool,
}

/// What `parinn verify` finds at a transcript source's path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Found {
    Gone,
    Unreadable,
    File(Scanned),
}

/// Reads `bytes` once: hashes the first `offset`, then looks for a line
/// break past them. Only the consumed prefix is hashed, a chunk at a
/// time, so a large transcript costs one read and a chunk of memory.
///
/// # Errors
///
/// Fails when a read does.
pub(crate) fn scan(mut bytes: impl Read, offset: u64) -> io::Result<Scanned> {
    let mut hash = Running::default();
    let mut buffer = vec![0; CHUNK];
    let mut left = offset;
    let mut unread_line = false;
    loop {
        let n = bytes.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        let taken = usize::try_from(left).map_or(n, |l| l.min(n));
        hash.update(&buffer[..taken]);
        left -= taken as u64;
        if buffer[taken..n].contains(&b'\n') {
            unread_line = true;
            break;
        }
    }

    Ok(Scanned {
        prefix: (left == 0).then(|| hash.digest()),
        unread_line,
    })
}

/// What is at `path` now, its first `offset` bytes hashed.
pub(crate) fn look(path: &Path, offset: u64) -> Found {
    match fs::File::open(path) {
        Ok(file) => scan(file, offset).map_or(Found::Unreadable, Found::File),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Found::Gone,
        Err(_) => Found::Unreadable,
    }
}

/// How far `parinn verify` finds the transcript sources short of their
/// ends (ADM-09).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Completeness {
    /// Sources short of their end that ingest resumes: a running MCP
    /// server holds the session binding of each, or an ingest left its
    /// ingest marker at its position within [`MARKER_CEILING_MS`].
    pub(crate) short: Vec<String>,
    /// Sources short of their end that nothing will finish, or that no
    /// longer hold what was ingested from them: the path and why.
    pub(crate) failed: Vec<(String, &'static str)>,
}

/// What `parinn verify` finds of one source.
enum Outcome {
    Holds,
    Short,
    Fails(&'static str),
}

/// Which of `sources` fall short, given the transcripts the running MCP
/// servers hold, what `look` finds at each path now, and the time now on
/// verify's clock, in milliseconds (REC-13, REC-07, REC-08). A source
/// whose file no longer begins with the bytes ingested from it fails
/// (I1). One with a whole line past its ingest position is short while a
/// server holds it, or while an ingest that stopped short of its end, or
/// began and consumed nothing of it, left its ingest marker there less
/// than [`MARKER_CEILING_MS`] ago; else it fails. One that disappeared
/// before it was fully ingested fails. Bytes past the last line break
/// are a line still being written, which no ingest consumes yet.
///
/// The clock is that of `parinn verify`, a command: no derived artifact
/// reads it (I10).
pub(crate) fn completeness(
    sources: &[Source],
    served: &[PathBuf],
    look: &dyn Fn(&Path, u64) -> Found,
    now_ms: u64,
) -> Completeness {
    let mut found = Completeness::default();
    let gone = lost(sources);
    for source in sources {
        let outcome = if gone.contains(&source) {
            Outcome::Fails("disappeared before it was fully ingested")
        } else {
            outcome(
                source,
                look(Path::new(&source.path), source.position.offset),
                served,
                now_ms,
            )
        };
        match outcome {
            Outcome::Holds => {}
            Outcome::Short => found.short.push(source.path.clone()),
            Outcome::Fails(why) => found.failed.push((source.path.clone(), why)),
        }
    }

    found
}

/// What `parinn verify` finds of `source`, given what is at its path.
fn outcome(source: &Source, at: Found, served: &[PathBuf], now_ms: u64) -> Outcome {
    let at = match at {
        // The record does not depend on a file fully ingested (REC-08).
        Found::Gone => return Outcome::Holds,
        Found::Unreadable => return Outcome::Fails("cannot be read to check it"),
        Found::File(at) => at,
    };
    if at.prefix.as_ref() != Some(&source.position.prefix) {
        return Outcome::Fails(REPLACED);
    }
    if !at.unread_line {
        return Outcome::Holds;
    }
    if served.iter().any(|s| s == Path::new(&source.path)) {
        return Outcome::Short;
    }
    let marked = !source.position.complete() || source.position.offset == 0;
    if !marked {
        return Outcome::Fails(UNSERVED);
    }
    if source.marked_at.abs_diff(now_ms) > MARKER_CEILING_MS {
        return Outcome::Fails(STALE);
    }

    Outcome::Short
}

impl Completeness {
    /// What `parinn verify` says of the sources that fail, one clause
    /// each, every path shown safe for a terminal (T16); none while they
    /// all hold.
    pub(crate) fn failure(&self) -> Option<String> {
        if self.failed.is_empty() {
            return None;
        }
        let each: Vec<String> = self
            .failed
            .iter()
            .map(|(path, why)| format!("{} {why}", inline(path)))
            .collect();

        Some(format!(
            "transcript sources fall short: {}",
            each.join("; ")
        ))
    }

    /// What `parinn verify` adds for sources short of their end: nothing
    /// when none is.
    pub(crate) fn short_text(&self) -> String {
        if self.short.is_empty() {
            return String::new();
        }

        format!(
            "; transcript sources short of their end, which ingest resumes: {}",
            shown(&self.short)
        )
    }
}

/// `paths` joined, each shown safe for a terminal (T16).
fn shown(paths: &[String]) -> String {
    paths
        .iter()
        .map(|p| inline(p))
        .collect::<Vec<_>>()
        .join(", ")
}

impl SourcesStatus {
    /// The sources as `parinn status --json` shows them.
    pub(crate) fn json(&self) -> Value {
        json!({"held": self.held, "lost": self.lost})
    }

    /// The lines `parinn status` shows: none while no source is held or
    /// lost.
    pub(crate) fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.held.is_empty() {
            lines.push(format!(
                "repeated lines held at an ingest marker: {}",
                shown(&self.held)
            ));
        }
        if !self.lost.is_empty() {
            lines.push(lost_text(&self.lost));
        }

        lines
    }
}

/// What `parinn status` and `parinn verify` say of the sources `lost`.
pub(crate) fn lost_text(lost: &[String]) -> String {
    format!(
        "transcript sources gone before they were fully ingested: {}",
        shown(lost)
    )
}

#[cfg(test)]
mod tests;
