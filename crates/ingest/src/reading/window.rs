//! The window of a transcript source one ingest reads, and what it needs
//! of the source around it: the ingest marker row, the identifiers its
//! lines name and the hash of what an earlier read consumed (SEC-05,
//! REC-13, SEC-31).

use std::path::Path;

use administration::home::Home;
use redaction::Redactor;
use redaction::account::HarnessAccountIdentifiers;
use serde_json::Value;
use store::source::Zeroed;
use store::{Source, Store};

use super::{Resume, as_index};
use crate::IngestError;
use crate::read::bounded::Refused;
use crate::read::bounded::{LINE_LIMIT, Limits, SOURCE_LIMIT};
use crate::read::{self, Line, open};

/// The window of a transcript source one ingest reads (SEC-05, REC-13).
pub(crate) struct View<'t> {
    /// Its lines, whole.
    pub(crate) text: &'t str,
    /// The byte offset of the source it starts at: past 0 only when a
    /// longer source resumes at its ingest marker.
    pub(crate) base: u64,
    /// The bytes the source held when read.
    pub(crate) seen: u64,
    /// The line over the line ceiling the window ends before.
    pub(crate) refused: Option<Refused>,
    /// Whether the source holds more lines past the window to resume at.
    pub(crate) resumable: bool,
    /// The most bytes of the source the read held at once.
    pub(crate) high_water: usize,
}

/// Each of `lines` parsed as JSON, null where it is none.
pub(super) fn parse(lines: &[Line<'_>]) -> Vec<Value> {
    lines
        .iter()
        .map(|l| serde_json::from_str(l.text).unwrap_or(Value::Null))
        .collect()
}

/// `redactor` with the harness account identifiers the transcript names
/// among its rules (SEC-08).
pub(super) fn redacting(
    redactor: &Redactor,
    path: &Path,
    view: &View<'_>,
    lines: &[Line<'_>],
) -> Result<(Redactor, usize), IngestError> {
    let learned = learned(path, view, lines)?;

    redactor
        .with_identifiers(&learned.ids)
        .map_err(IngestError::Redaction)
        .map(|redactor| (redactor, learned.high_water))
}

struct Learned {
    ids: HarnessAccountIdentifiers,
    high_water: usize,
}

/// The harness account identifiers the transcript's lines name: those
/// of `lines`, or, for a source read in windows, of every line of it, a
/// line at a time (SEC-08, SEC-05).
fn learned(path: &Path, view: &View<'_>, lines: &[Line<'_>]) -> Result<Learned, IngestError> {
    let mut ids = HarnessAccountIdentifiers::default();
    if view.base == 0 && !view.resumable && view.refused.is_none() {
        let limit = SOURCE_LIMIT.saturating_sub(view.high_water);
        for line in lines {
            if !ids.learn_within(line.text, limit) {
                return Err(IngestError::IdentifierLimit(path.to_path_buf()));
            }
        }
        return Ok(Learned {
            high_water: view.high_water.saturating_add(ids.byte_len()),
            ids,
        });
    }
    let learning = open::LEARNING_WINDOW_LIMIT;
    let reserve = open::LEARNING_RESERVE;
    let limit = SOURCE_LIMIT.saturating_sub(view.high_water.saturating_add(reserve));
    let mut exceeded = false;
    let limits = Limits {
        line: LINE_LIMIT,
        source: learning,
    };
    open::for_each_line_with_limits(path, 0, u64::MAX, limits, |text| {
        if !exceeded && !ids.learn_within(text, limit) {
            exceeded = true;
        }
    })
    .map_err(IngestError::read(path))?;
    if exceeded {
        return Err(IngestError::IdentifierLimit(path.to_path_buf()));
    }

    Ok(Learned {
        high_water: view
            .high_water
            .saturating_add(reserve)
            .saturating_add(ids.byte_len()),
        ids,
    })
}

/// The ingest marker row of the source at `path`, read without changing
/// the store; none while there is no store or no row.
pub(crate) fn peek(home: &Home, path: &Path) -> Result<Option<Source>, IngestError> {
    let file = home.store();
    if !file.exists() {
        return Ok(None);
    }
    let Some(store) = Store::open_read_only_made(&file)? else {
        return Ok(None);
    };

    Ok(store.source(&path.to_string_lossy())?)
}

/// The running hash of what `resume` consumed of the source, its purged
/// lines zeroed: of `bytes` when they begin the source, else of the
/// source at `path`, a chunk at a time (SEC-31, SEC-05).
pub(super) fn consumed<'s>(
    resume: &Resume,
    bytes: &[u8],
    path: &Path,
    base: u64,
    purged: &'s [store::source::Span],
) -> Result<Zeroed<'s>, IngestError> {
    if base == 0 {
        let mut running = Zeroed::new(purged);
        running.update(
            bytes
                .get(..as_index(resume.position.offset))
                .unwrap_or_default(),
        );
        return Ok(running);
    }
    let file = read::regular_file(path).map_err(IngestError::read(path))?;
    let prefix =
        open::zeroed(file, resume.position.offset, purged).map_err(IngestError::read(path))?;
    let moved = store::StoreError::PositionMoved {
        source: resume.source.id,
    };

    prefix.ok_or(IngestError::Store(moved))
}
