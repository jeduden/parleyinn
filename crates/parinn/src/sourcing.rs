//! What an ingest found of its transcript source, audited (REC-04,
//! REC-07).

use std::io::Write;

use audit::entry::{Counter, Kind, Occurrence};

use crate::Context;

/// Audits what an ingest found of its transcript source, one entry
/// each, naming the source by its redacted name: a new transcript
/// generation (REC-07); each line repeating a `uuid` with other
/// content, each repeat owner no fork observation chose, a hold begun,
/// each line released from one and each refused for naming a purged
/// event (REC-04).
pub(super) fn audit(ctx: &Context, ingested: &ingest::Ingested, stderr: &mut dyn Write) {
    let keyed = |kind, counter, detail| Occurrence::Keyed {
        kind,
        counter,
        operation: "ingest",
        detail,
        key: &ingested.name,
    };
    let occurrences = [
        (
            usize::from(ingested.new_generation),
            keyed(
                Kind::Truncated,
                Counter::NewTranscriptGeneration,
                "new transcript generation of",
            ),
        ),
        (
            ingested.conflicts,
            keyed(
                Kind::Rejected,
                Counter::RepeatConflicts,
                "repeated uuid with other content, recorded as its own event, in",
            ),
        ),
        (
            ingested.owners_chosen,
            keyed(
                Kind::Coalesced,
                Counter::RepeatOwnerChosen,
                "repeated lines owned by the run that first recorded them, repeated in",
            ),
        ),
        (
            usize::from(ingested.hold_started),
            keyed(
                Kind::Truncated,
                Counter::RepeatHeld,
                "repeated lines held at an ingest marker in",
            ),
        ),
        (
            ingested.released,
            keyed(
                Kind::TimedOut,
                Counter::RepeatReleased,
                "repeated line released into its own run in",
            ),
        ),
        (
            ingested.refused,
            keyed(
                Kind::Rejected,
                Counter::RepeatOfPurgedRefused,
                "refused a repeated line whose `uuid` names a purged event, in",
            ),
        ),
    ];
    for (times, occurrence) in &occurrences {
        for _ in 0..*times {
            if let Err(e) = ctx.audit(occurrence) {
                let _ = writeln!(stderr, "parinn ingest: audit the transcript source: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests;
