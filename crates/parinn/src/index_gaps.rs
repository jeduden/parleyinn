//! Reports the events the search index leaves out (REC-11, I6): a writer
//! whose slot a smaller writer id holds, and a writer from seq 2^28 on.
//! Each is audited once per writer and class, as the counters
//! `search_index_writer_unindexed` and `search_index_seq_beyond_limit`,
//! and noted in the index before it is audited, so a failed note audits
//! nothing and no later ingest audits a gap twice. A gap whose entry the
//! audit log refused is un-noted, and stays unreported until an ingest
//! audits it.

use std::fmt;

use administration::home::Home;
use audit::entry::{Counter, Kind, Occurrence};
use store::{GapClass, IndexGap, Store, StoreError};

/// The entry for `gap`, the writer named by `writer`, its hex id.
fn occurrence<'a>(gap: IndexGap, writer: &'a str, operation: &'static str) -> Occurrence<'a> {
    let (counter, detail) = match gap.class {
        GapClass::WriterUnindexed => (
            Counter::SearchIndexWriterUnindexed,
            "search index leaves out the events of writer",
        ),
        GapClass::SeqBeyondLimit => (
            Counter::SearchIndexSeqBeyondLimit,
            "search index leaves out the events from seq 2^28 on of writer",
        ),
    };

    Occurrence::Keyed {
        kind: Kind::Dropped,
        counter,
        operation,
        detail,
        key: writer,
    }
}

/// Why a report of search index gaps fell short.
#[derive(Debug)]
pub(crate) enum ReportError {
    /// The store could not be opened or noted, so nothing was audited
    /// and the next ingest reports every gap.
    Unnoted(StoreError),
    /// The audit log refused `entries` entries and the note that their
    /// gaps were reported could not be taken back: they stay noted, not
    /// audited, until the index is rebuilt (I6).
    Unaudited { entries: usize, source: StoreError },
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unnoted(e) => write!(f, "{e}; the next ingest tries again"),
            Self::Unaudited { entries, source } => write!(
                f,
                "the audit log refused {entries} entries and their gaps stay noted as \
                 reported: {source}"
            ),
        }
    }
}

impl std::error::Error for ReportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unnoted(e) | Self::Unaudited { source: e, .. } => Some(e),
        }
    }
}

/// Audits each of `gaps` through `audit`, which says whether the audit
/// log took the entry, at most once each: the gaps are noted in the
/// index of `home`'s store first, so a failed note audits nothing and
/// the next ingest reports them all, and a gap whose entry the audit
/// log refused is un-noted, to be reported at the next ingest. Opens no
/// store when there is no gap. Returns how many entries the audit log
/// refused.
///
/// # Errors
///
/// Fails when the store cannot be opened or the gaps cannot be noted,
/// or when the note of a gap whose entry was refused cannot be taken
/// back.
pub(crate) fn report(
    home: &Home,
    gaps: &[IndexGap],
    operation: &'static str,
    audit: &mut dyn FnMut(&Occurrence<'_>) -> bool,
) -> Result<usize, ReportError> {
    if gaps.is_empty() {
        return Ok(0);
    }
    let store = Store::open(&home.store()).map_err(ReportError::Unnoted)?;
    store
        .mark_search_index_gaps_reported(gaps)
        .map_err(ReportError::Unnoted)?;
    let refused: Vec<IndexGap> = gaps
        .iter()
        .copied()
        .filter(|gap| !audit(&occurrence(*gap, &gap.writer.to_string(), operation)))
        .collect();
    store
        .unmark_search_index_gaps_reported(&refused)
        .map_err(|source| ReportError::Unaudited {
            entries: refused.len(),
            source,
        })?;

    Ok(refused.len())
}

/// What a report of gaps says on `stderr` or in the log: nothing when
/// every gap was audited.
pub(crate) fn problem(done: &Result<usize, ReportError>) -> Option<String> {
    match done {
        Ok(0) => None,
        Ok(n) => Some(format!(
            "the audit log refused {n} entries; the next ingest tries again"
        )),
        Err(e) => Some(e.to_string()),
    }
}

/// The entry for a lookup of the gaps that failed after an ingest
/// recorded its events.
pub(crate) fn lookup_failed(operation: &'static str) -> Occurrence<'static> {
    Occurrence::Degraded {
        kind: Kind::Failed,
        counter: Counter::SearchIndexGapLookupFailed,
        operation,
        detail: "the events the search index leaves out could not be looked up; \
                 the next ingest tries again",
    }
}

#[cfg(test)]
pub(crate) mod tests;
