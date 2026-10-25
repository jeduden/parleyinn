//! What `parinn status`, `parinn verify` and `parinn ingest` say of the
//! transcript sources (REC-04, REC-08): each source an ingest marker
//! holds repeated lines of, and each source that disappeared before it
//! was fully ingested.

use std::path::Path;

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
