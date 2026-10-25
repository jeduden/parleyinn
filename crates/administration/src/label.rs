//! Writer labels (RCL-08): a writer's short display alias, `A` and a
//! number. A device seat assigns one by a structural event in its own
//! writer, the first time an address is asked for; the label is `A`
//! and the rank of its assignment, so the record alone gives the
//! labels (I10) and no two writers share one.
//!
//! The assignments go in the order the device seat certified the
//! seats, a run's and an ingest seat's before any device seat's, so the
//! first run seat a node starts is `A1`.

use std::collections::BTreeSet;

use key::random::{Entropy, draw};
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::WriterId;
use serde_json::json;
use store::{NewEvent, Store};

use crate::AdminError;
use crate::origin::is_device_seat;

/// Assigns a label to every writer of a seat in `store` that has none,
/// recording each assignment as a structural event in `device`, the
/// device seat's writer. Returns how many it assigned; none when every
/// writer is labelled.
///
/// # Errors
///
/// Fails when the random source or the store fails.
pub fn assign_missing(
    store: &mut Store,
    device: WriterId,
    entropy: &mut dyn Entropy,
) -> Result<usize, AdminError> {
    let labelled = store.writer_labels(device)?;
    let seats = store.seats()?;
    if seats.iter().all(|s| labelled.contains_key(&s.writer)) {
        return Ok(0);
    }
    let (devices, runs): (Vec<_>, Vec<_>) = seats.iter().partition(|s| is_device_seat(s));
    let mut order: Vec<WriterId> = Vec::new();
    let mut seen = BTreeSet::new();
    let certified = store.certified_writers(device)?;
    let run_writers: BTreeSet<WriterId> = runs.iter().map(|s| s.writer).collect();
    let certified = certified.into_iter().filter(|w| run_writers.contains(w));
    // Seats the device seat certified come in its order, any other
    // seat after them by writer id (seats read back in that order).
    for writer in certified
        .chain(runs.iter().map(|s| s.writer))
        .chain(devices.iter().map(|s| s.writer))
    {
        if seen.insert(writer) && !labelled.contains_key(&writer) {
            order.push(writer);
        }
    }
    let mut events = Vec::with_capacity(order.len());
    for writer in &order {
        events.push(NewEvent {
            kind: EventKind::WriterLabel,
            provenance: Provenance::Structural,
            content: json!({"writer": writer.to_string()}),
            commitment_key: CommitmentKey::from_bytes(draw(entropy)?),
            transcript_timestamp: None,
        });
    }
    store.append(device, &events)?;

    Ok(order.len())
}

#[cfg(test)]
mod tests;
