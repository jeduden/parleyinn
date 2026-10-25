//! The counters derived from the audit log (OPS-01, OPS-03): each
//! named counter counts the entries that name it, and its rise counts
//! those after the entry the last acknowledgement covered, which the
//! counter acknowledgement act records in the device seat (OWN-11). Every counter ParleyInn names
//! so far is a failure counter (OQ-50 (i)). Counting reads no clock: the
//! same entries always give the same counts (I10).

use std::collections::BTreeMap;

use record::digest::Digest;

use crate::entry::Entry;

/// Each counter's count, and its rise since the counters were last
/// acknowledged; a counter with no entry is absent from both.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts {
    pub total: BTreeMap<String, u64>,
    pub since_ack: BTreeMap<String, u64>,
}

impl Counts {
    /// Whether any failure counter rose since the counters were last
    /// acknowledged, which fails `parinn doctor` (OPS-03).
    #[must_use]
    pub fn risen(&self) -> bool {
        !self.since_ack.is_empty()
    }
}

/// The counts `entries` give, their rise counted after the entry the
/// counters were last acknowledged through, named by its number and
/// hash; all of them when they never were, or when `entries` no longer
/// hold that entry, as in a log that began again (I6).
#[must_use]
pub fn counts(entries: &[Entry], acknowledged: Option<(u64, Digest)>) -> Counts {
    let through = acknowledged
        .filter(|(n, hash)| entries.iter().any(|e| e.number == *n && e.hash == *hash))
        .map_or(0, |(n, _)| n);
    let mut out = Counts::default();
    for e in entries {
        if let Some(counter) = &e.counter {
            *out.total.entry(counter.clone()).or_default() += 1;
            if e.number > through {
                *out.since_ack.entry(counter.clone()).or_default() += 1;
            }
        }
    }

    out
}

#[cfg(test)]
mod tests;
