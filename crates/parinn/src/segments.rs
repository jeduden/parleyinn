//! What `parinn status` and `parinn verify` say of the store's segments
//! (REC-19, ADM-05): each writer chain that ended without a closed
//! segment, and each segment refused as newer than this parinn's format
//! version.

use administration::home::Home;
use serde_json::{Value, json};
use store::{Segment, Store, StoreError};

/// The segments `parinn status` reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SegmentsStatus {
    /// Each writer's open segment.
    pub(crate) open: Vec<Segment>,
    /// Each segment of a newer format version, refused.
    pub(crate) newer: Vec<Segment>,
}

/// The segments of the store in `home`, read without changing it; none
/// while there is no store.
///
/// # Errors
///
/// Fails when the store cannot be read, or is of a newer schema version.
pub(crate) fn read(home: &Home) -> Result<Option<SegmentsStatus>, StoreError> {
    let path = home.store();
    if !path.exists() {
        return Ok(None);
    }
    // A store its first opener was stopped before it made a schema in,
    // such as a hook handler past its hook budget, holds nothing yet.
    let Some(store) = Store::open_read_only_made(&path)? else {
        return Ok(None);
    };

    Ok(Some(SegmentsStatus {
        open: store.open_segments()?,
        newer: store.newer_segments()?,
    }))
}

impl SegmentsStatus {
    /// The segments as `parinn status --json` shows them.
    pub(crate) fn json(&self) -> Value {
        let newer: Vec<Value> = self
            .newer
            .iter()
            .map(|s| {
                let mut v = range_json(s);
                v["format_version"] = json!(s.format_version);
                v
            })
            .collect();

        json!({"open": open_json(&self.open), "newer": newer})
    }

    /// The lines `parinn status` shows: none while every segment closed
    /// and none is newer.
    pub(crate) fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.open.is_empty() {
            lines.push(format!(
                "writer chains ended without a closed segment: {}",
                open_list(&self.open)
            ));
        }
        lines.extend(
            self.newer
                .iter()
                .map(|s| format!("refused: {}", StoreError::NewerSegment(s.clone()))),
        );

        lines
    }
}

/// Each open segment, as `parinn verify --json` and `parinn status --json`
/// show it.
pub(crate) fn open_json(open: &[Segment]) -> Value {
    open.iter().map(range_json).collect()
}

/// A segment's writer and range.
fn range_json(s: &Segment) -> Value {
    json!({"writer": s.writer.to_string(), "first": s.first.get(), "last": s.last.get()})
}

/// Each open segment's writer and range, for a line of text.
pub(crate) fn open_list(open: &[Segment]) -> String {
    open.iter()
        .map(|s| format!("{} (seq {} to {})", s.writer, s.first, s.last))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests;
