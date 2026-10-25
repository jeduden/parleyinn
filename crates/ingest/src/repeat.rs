//! A line a harness fork repeats (REC-04): one carrying the `uuid` of a
//! line in another transcript of the same transcript directory. It is
//! recorded once, in the writer of the run it belongs to: the run of
//! the harness session no fork observation names, else the run whose
//! writer first recorded it on this node. Every other run records a
//! repeat reference to that event's address when the line's content
//! equals it but for the harness session's own fields; other content is
//! its own event, a conflict. A fork's repeated line read before its
//! owner's is held at an ingest marker until the owner's transcript is
//! ingested, up to [`HOLD_MS`], or released once that transcript is
//! gone.

use record::event::Provenance;
use record::id::Address;
use serde_json::Value;

/// How long an ingest marker holds a fork's repeated lines, in
/// milliseconds (REC-04's 30 s).
pub const HOLD_MS: u64 = 30_000;

/// The fields a harness session writes into each of its lines, which a
/// fork rewrites in the lines it repeats: no part of the line's content.
const SESSION_FIELDS: [&str; 2] = ["sessionId", "promptId"];

/// Why a repeated line's owner is the run it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// A fork observation names the other run's harness session.
    ForkObservation,
    /// No fork observation decides it: the owner first recorded it.
    FirstRecorded,
}

/// The event already holding a line with the same `uuid`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owner {
    pub address: Address,
    pub provenance: Provenance,
    /// The redacted line it holds.
    pub text: String,
    /// Whether a fork observation names its run's harness session.
    pub fork: bool,
}

/// What is known of one line with a `uuid` when it is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub owner: Option<Owner>,
    /// Whether a fork observation names this line's harness session.
    pub fork: bool,
    /// Whether another transcript of the directory holds the `uuid`.
    pub in_sibling: bool,
    /// Since when an ingest marker holds this source, on ingest's clock.
    pub held_since: Option<u64>,
    /// Now, on ingest's clock, in milliseconds.
    pub now: u64,
}

/// What becomes of one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Its own event in its run's writer.
    Own,
    /// Its own event, though another holds its `uuid` with other
    /// content: counted and audited.
    Conflict,
    /// A repeat reference to the owner's event.
    Reference {
        owner: Address,
        provenance: Provenance,
        rule: Rule,
    },
    /// Held at an ingest marker until its owner's transcript is read.
    Hold,
    /// Its own event, released from a hold: audited.
    Released,
}

/// What becomes of a line whose redacted text is `redacted`, given
/// `facts`.
#[must_use]
pub fn decide(facts: &Facts, redacted: &str) -> Decision {
    match &facts.owner {
        // A hold released the owner's line into the fork's own writer;
        // this run keeps its own copy, never a reference to the fork.
        Some(owner) if owner.fork && !facts.fork => Decision::Own,
        Some(owner) if same_content(&owner.text, redacted) => Decision::Reference {
            owner: owner.address,
            provenance: owner.provenance.clone(),
            rule: if facts.fork && !owner.fork {
                Rule::ForkObservation
            } else {
                Rule::FirstRecorded
            },
        },
        Some(_) => Decision::Conflict,
        None => unowned(facts),
    }
}

/// What becomes of a line no recorded event holds yet.
fn unowned(facts: &Facts) -> Decision {
    let expired = facts
        .held_since
        .is_some_and(|since| facts.now.saturating_sub(since) >= HOLD_MS);
    match (facts.fork, facts.in_sibling) {
        (true, true) if !expired => Decision::Hold,
        (true, _) if facts.held_since.is_some() => Decision::Released,
        _ => Decision::Own,
    }
}

/// Whether the redacted lines `a` and `b` hold the same content: the
/// same JSON value but for the harness session's own fields, or, when
/// either does not parse, the same text.
#[must_use]
pub fn same_content(a: &str, b: &str) -> bool {
    let parse = |text: &str| -> Option<Value> {
        let mut value: Value = serde_json::from_str(text).ok()?;
        if let Some(fields) = value.as_object_mut() {
            for field in SESSION_FIELDS {
                fields.remove(field);
            }
        }
        Some(value)
    };
    match (parse(a), parse(b)) {
        (Some(a), Some(b)) => a == b,
        _ => a == b,
    }
}

#[cfg(test)]
mod tests;
