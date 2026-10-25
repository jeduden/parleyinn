//! What `parinn status`, `parinn verify` and `parinn ingest` say of the
//! transcript sources (REC-04, REC-08): each source an ingest marker
//! holds repeated lines of, and each source that disappeared before it
//! was fully ingested.

use std::path::{Path, PathBuf};

use administration::home::Home;
use serde_json::{Value, json};
use store::{Source, Store, StoreError};

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

/// How far `parinn verify` finds the transcript sources short of their
/// ends (ADM-09).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Completeness {
    /// Sources past their ingest position that a running MCP server
    /// holds the session binding of: it ingests the rest.
    pub(crate) pending: Vec<String>,
    /// Sources short of their end that nothing will finish: the path
    /// and why.
    pub(crate) failed: Vec<(String, &'static str)>,
}

/// Which of `sources` are short of their ends, given the transcripts the
/// running MCP servers hold and the length of a file now (REC-13,
/// REC-07, REC-08): one past its ingest position is pending while a
/// server holds it, else it fails; one that disappeared before it was
/// fully ingested fails. A source an ingest stopped short of, or began
/// and consumed nothing of, is pending too: it left an ingest marker at
/// its position, from which the next `parinn ingest` resumes (REC-13,
/// ENG-06).
pub(crate) fn completeness(
    sources: &[Source],
    served: &[PathBuf],
    length: &dyn Fn(&Path) -> Option<u64>,
) -> Completeness {
    let mut found = Completeness::default();
    let gone = lost(sources);
    for source in sources {
        let path = Path::new(&source.path);
        if gone.contains(&source) {
            found.failed.push((
                source.path.clone(),
                "disappeared before it was fully ingested",
            ));
        } else if length(path).is_some_and(|len| len > source.position.offset) {
            let stopped_short = !source.position.complete() || source.position.offset == 0;
            if stopped_short || served.iter().any(|s| s == path) {
                found.pending.push(source.path.clone());
            } else {
                found.failed.push((
                    source.path.clone(),
                    "holds lines the record lacks and no running MCP server ingests them",
                ));
            }
        }
    }

    found
}

impl Completeness {
    /// What `parinn verify` says of the sources that fail, one clause
    /// each; none while they all hold.
    pub(crate) fn failure(&self) -> Option<String> {
        if self.failed.is_empty() {
            return None;
        }
        let each: Vec<String> = self
            .failed
            .iter()
            .map(|(path, why)| format!("{path} {why}"))
            .collect();

        Some(format!(
            "transcript sources fall short: {}",
            each.join("; ")
        ))
    }

    /// What `parinn verify` adds for sources still pending: nothing when
    /// none is.
    pub(crate) fn pending_text(&self) -> String {
        if self.pending.is_empty() {
            return String::new();
        }

        format!(
            "; transcript sources pending, a running MCP server ingests them: {}",
            self.pending.join(", ")
        )
    }
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
                self.held.join(", ")
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
        lost.join(", ")
    )
}

#[cfg(test)]
mod tests;
