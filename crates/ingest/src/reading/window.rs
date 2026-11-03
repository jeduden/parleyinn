//! The window of a transcript source one ingest reads, and what it needs
//! of the source around it: the ingest marker row, the identifiers its
//! lines name and the hash of what an earlier read consumed (SEC-05,
//! REC-13, SEC-31).

use std::path::Path;

use administration::home::Home;
use redaction::Redactor;
use redaction::account::HarnessAccountIdentifiers;
use store::source::Zeroed;
use store::{Source, Store};

use super::plan::{PARSED_LINE_LIMIT, json_heap_bound};
use super::{Resume, as_index};
use crate::IngestError;
use crate::read::bounded::Refused;
use crate::read::bounded::{LINE_LIMIT, Limits};
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
    let limit = open::IDENTIFIER_LIMIT;
    // The buffered window is the source this ingest will actually
    // record. Learn it before reopening the pathname: if the path was
    // replaced after the guarded read, a later scan must not make its
    // identifiers disappear (SEC-08).
    for line in lines {
        if json_heap_bound(line.text) > PARSED_LINE_LIMIT {
            return Err(IngestError::SourceMemoryLimit(path.to_path_buf()));
        }
        if !learn_within(&mut ids, line.text, limit) {
            return Err(IngestError::IdentifierLimit(path.to_path_buf()));
        }
    }
    if view.base == 0 && !view.resumable && view.refused.is_none() {
        return Ok(Learned {
            high_water: view
                .high_water
                .saturating_add(open::LEARNING_RESERVE)
                .saturating_add(ids.byte_len().saturating_mul(2)),
            ids,
        });
    }
    let learning = open::LEARNING_WINDOW_LIMIT;
    let reserve = open::LEARNING_RESERVE;
    let (mut identifiers_exceeded, mut structure_exceeded) = (false, false);
    let limits = Limits {
        line: LINE_LIMIT,
        source: learning,
    };
    open::for_each_line_with_limits(path, 0, u64::MAX, limits, |text| {
        if !identifiers_exceeded && !structure_exceeded {
            if json_heap_bound(text) > PARSED_LINE_LIMIT {
                structure_exceeded = true;
            } else if !learn_within(&mut ids, text, limit) {
                identifiers_exceeded = true;
            }
        }
    })
    .map_err(IngestError::read(path))?;
    if structure_exceeded {
        return Err(IngestError::SourceMemoryLimit(path.to_path_buf()));
    }
    if identifiers_exceeded {
        return Err(IngestError::IdentifierLimit(path.to_path_buf()));
    }

    Ok(Learned {
        high_water: view
            .high_water
            .saturating_add(reserve)
            .saturating_add(ids.byte_len().saturating_mul(2)),
        ids,
    })
}

/// Learns only a line that can hold one of the two harness fields. Most
/// transcript lines carry neither; avoiding a general JSON tree for
/// them keeps the whole-source SEC-08 pass linear and bounded.
fn learn_within(ids: &mut HarnessAccountIdentifiers, text: &str, limit: usize) -> bool {
    if !text.contains("\"userEmail\"") && !text.contains("\"organizationUuid\"") {
        return true;
    }

    ids.learn_within(text, limit)
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

#[cfg(test)]
mod tests {
    use super::*;
    use testkit::TempDir;

    #[test]
    fn a_window_s_identifiers_are_learned_before_the_path_is_reopened() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("changed.jsonl");
        std::fs::write(&path, "{\"type\":\"other\"}\n").unwrap();
        let id = "0f0e0d0c-0000-0000-0000-000000000001";
        let text = format!(
            "{{\"attachment\":{{\"type\":\"credential_org\",\"organizationUuid\":\"{id}\"}}}}\n"
        );
        let view = View {
            text: &text,
            base: 7,
            seen: text.len() as u64 + 7,
            refused: None,
            resumable: true,
            high_water: text.len(),
        };
        let lines = read::lines_at(&text, view.base);
        let base = Redactor::new(&[]).unwrap().0;

        let (redactor, _) = redacting(&base, &path, &view, &lines).unwrap();

        assert_eq!(redactor.redact_text(id).text, "[REDACTED:organization_id]");
    }
}
