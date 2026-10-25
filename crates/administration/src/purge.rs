//! Purge (ADM-07, REC-17, SEC-22): what a range of one writer's events
//! may lose, and the tombstone that records it. The store erases the
//! content, the commitment key, the preview and the payload of each
//! event (`store::Store::purge`); this decides which ranges it may, and
//! builds what the record keeps in their place.
//!
//! A tombstone is a structural event of the device seat. It holds the
//! range, how many events it held, why it was purged and a digest over
//! those events' commitments in order, never content; the chained
//! headers keep the commitments themselves, and without its key a
//! commitment confirms no guess (REC-17). `parinn verify` accepts an
//! event without content only under a tombstone that holds its
//! commitment.
//!
//! M1 purges content only: a range holding an event the key set or room
//! state derives from is refused, since erasing it would change what
//! derives from the record (ADM-07). A range of a writer with a pending
//! ingest marker is refused too, until `parinn ingest` brings its run
//! in.

use key::random::{Entropy, draw};
use record::commitment::CommitmentKey;
use record::digest::Digest;
use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, Seq, WriterId};
use serde_json::{Value, json};
use store::{Erase, NewEvent, Store};

use crate::AdminError;
use crate::node::Node;
use crate::quarantine::device_writers;

/// The kinds of event whose content a purge may erase: a transcript
/// line, a line the parser could not read, a hook observation and a
/// recall. Every other kind holds no content, or is one the key set or
/// room state derives from (ADM-07).
pub const PURGEABLE: [EventKind; 4] = [
    EventKind::TranscriptLine,
    EventKind::UnparsedLine,
    EventKind::HookObservation,
    EventKind::Recall,
];

/// The label a tombstone's digest of commitments is taken under.
const DIGEST_LABEL: &[u8] = b"parinn tombstone commitments v1\0";

/// Why a range was purged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// The principal purged it (ADM-07).
    Purged,
    /// The retention policy `policy`, a settings key, aged it out
    /// (REC-15, SEC-22).
    Retention { policy: String },
}

impl Reason {
    /// The word a tombstone and the envelope show.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Purged => "purged",
            Self::Retention { .. } => "retention",
        }
    }
}

/// A tombstone: what the record keeps of a purged range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tombstone {
    pub writer: WriterId,
    pub first: Seq,
    pub last: Seq,
    /// How many events the range held.
    pub count: u64,
    pub reason: String,
    /// The retention policy, when one purged the range.
    pub policy: Option<String>,
    /// A digest over the commitments of the events, in seq order.
    pub commitments: Digest,
}

impl Tombstone {
    /// The tombstone as its event holds it.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut value = json!({
            "writer": self.writer.to_string(),
            "first": self.first.get(),
            "last": self.last.get(),
            "count": self.count,
            "reason": self.reason,
            "commitments": self.commitments.to_hex(),
        });
        if let Some(policy) = &self.policy {
            value["policy"] = json!(policy);
        }

        value
    }

    /// The tombstone `value` holds; none for content that is not one.
    #[must_use]
    pub fn from_value(value: &Value) -> Option<Self> {
        let seq = |name: &str| Seq::new(value.get(name)?.as_u64()?);

        Some(Self {
            writer: WriterId::parse(value.get("writer")?.as_str()?).ok()?,
            first: seq("first")?,
            last: seq("last")?,
            count: value.get("count")?.as_u64()?,
            reason: value.get("reason")?.as_str()?.to_owned(),
            policy: value
                .get("policy")
                .and_then(Value::as_str)
                .map(str::to_owned),
            commitments: Digest::from_hex(value.get("commitments")?.as_str()?).ok()?,
        })
    }

    /// Whether the tombstone covers the event at `address`.
    #[must_use]
    pub fn covers(&self, address: Address) -> bool {
        address.writer == self.writer && self.first <= address.seq && address.seq <= self.last
    }
}

/// The tombstones the device seats of `room`, the node's personal room,
/// record, in the order they were recorded: what says an event without
/// content was purged and not lost (ADM-07, RCL-06).
///
/// # Errors
///
/// Fails when the store does.
pub fn recorded(store: &Store, room: RoomId) -> Result<Vec<Tombstone>, AdminError> {
    let mut out = Vec::new();
    for device in device_writers(store, room)? {
        let found = store.contents(device, EventKind::Tombstone)?;
        out.extend(
            found
                .iter()
                .filter_map(|(_, content)| Tombstone::from_value(content)),
        );
    }

    Ok(out)
}

/// The digest over `commitments`, as a tombstone holds it.
#[must_use]
pub fn commitments_digest(commitments: &[String]) -> Digest {
    let mut parts: Vec<&[u8]> = vec![DIGEST_LABEL];
    for commitment in commitments {
        parts.push(commitment.as_bytes());
        parts.push(b"\0");
    }

    Digest::of(&parts)
}

/// Why a range may not be purged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The writer holds no event in the range.
    NothingHeld,
    /// The event at `address` is of a kind the key set or room state
    /// derives from, or one that holds no content (ADM-07).
    Kept { address: Address, kind: EventKind },
    /// The writer's run still has lines past its ingest marker or hook
    /// observations held at it (ADM-07, REC-19).
    PendingIngest { writer: WriterId },
}

impl Refusal {
    /// The cause as the audit entry's detail and key name it.
    #[must_use]
    pub fn detail(&self) -> &'static str {
        match self {
            Self::NothingHeld => "refused: the range holds no event",
            Self::Kept { .. } => "refused: the range holds an event of a kind a purge keeps",
            Self::PendingIngest { .. } => {
                "refused: the range lies in a run with a pending ingest marker"
            }
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NothingHeld => f.write_str("the writer holds no event in the range"),
            Self::Kept { address, kind } => write!(
                f,
                "{address} is a {} event, which a purge keeps: the key set or room state derives from it",
                kind.as_str()
            ),
            Self::PendingIngest { writer } => write!(
                f,
                "writer {writer} has a pending ingest marker: run parinn ingest to bring its run in first"
            ),
        }
    }
}

/// A range ready to purge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub erase: Erase,
    pub tombstone: Tombstone,
    /// How many of its events still hold content.
    pub held: u64,
}

/// The outcome of planning a purge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Planned {
    Ready(Plan),
    Refused(Vec<Refusal>),
}

/// Plans the purge of `writer`'s events `first` to `last` under
/// `reason`: the range is cut to the events there are, and refused for
/// each cause that applies (ADM-07).
///
/// # Errors
///
/// Fails when the store does.
pub fn plan(
    store: &Store,
    writer: WriterId,
    first: Seq,
    last: Seq,
    reason: &Reason,
) -> Result<Planned, AdminError> {
    let events = store.range_events(writer, first, last)?;
    let (Some(head), Some(tail)) = (events.first(), events.last()) else {
        return Ok(Planned::Refused(vec![Refusal::NothingHeld]));
    };
    let mut refusals: Vec<Refusal> = events
        .iter()
        .filter(|e| !PURGEABLE.contains(&e.kind))
        .map(|e| Refusal::Kept {
            address: Address::new(writer, e.seq),
            kind: e.kind,
        })
        .collect();
    if pending_ingest(store, writer)? {
        refusals.push(Refusal::PendingIngest { writer });
    }
    let held = events.iter().filter(|e| !e.erased).count() as u64;
    if refusals.is_empty() && held == 0 {
        refusals.push(Refusal::NothingHeld);
    }
    if !refusals.is_empty() {
        return Ok(Planned::Refused(refusals));
    }
    let commitments: Vec<String> = events.iter().map(|e| e.commitment.to_string()).collect();
    let policy = match reason {
        Reason::Purged => None,
        Reason::Retention { policy } => Some(policy.clone()),
    };

    Ok(Planned::Ready(Plan {
        erase: Erase {
            writer,
            first: head.seq,
            last: tail.seq,
        },
        tombstone: Tombstone {
            writer,
            first: head.seq,
            last: tail.seq,
            count: events.len() as u64,
            reason: reason.as_str().to_owned(),
            policy,
            commitments: commitments_digest(&commitments),
        },
        held,
    }))
}

/// Whether `writer` records a run whose transcript has lines past the
/// ingest marker, or hook observations held at it that never reached
/// the record (ADM-07, REC-19).
fn pending_ingest(store: &Store, writer: WriterId) -> Result<bool, AdminError> {
    let Some(run) = store
        .ingested_runs()?
        .into_iter()
        .find(|r| r.writer == writer)
    else {
        return Ok(false);
    };
    let unread = store
        .sources()?
        .iter()
        .any(|s| s.path == run.path && !s.position.complete());
    let held = store
        .hook_observations()?
        .iter()
        .any(|h| !h.appended && h.observation.transcript_path == run.path);

    Ok(unread || held)
}

/// What a purge recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Done {
    /// The principal act, when a principal took one.
    pub act: Option<Address>,
    pub tombstone: Address,
    /// How many events lost their content.
    pub erased: u64,
}

/// Purges the planned range: records `act` if there is one, then the
/// tombstone, in the node's device seat, in the transaction that erases
/// the range's content (ADM-07).
///
/// # Errors
///
/// Fails, changing nothing, when no key can be drawn or the store
/// fails; the steps after the transaction stay journalled when they
/// fail (`store::Store::finish_purge`).
pub fn carry_out(
    store: &mut Store,
    node: &Node,
    plan: &Plan,
    act: Option<NewEvent>,
    entropy: &mut dyn Entropy,
) -> Result<Done, AdminError> {
    let tombstone = NewEvent {
        kind: EventKind::Tombstone,
        provenance: Provenance::Structural,
        content: plan.tombstone.to_value(),
        commitment_key: CommitmentKey::from_bytes(draw(entropy)?),
        transcript_timestamp: None,
    };
    let record: Vec<NewEvent> = act.into_iter().chain([tombstone]).collect();
    let acted = record.len() == 2;
    let purged = store.purge(node.device_writer(), &record, &[plan.erase])?;
    let mut appended = purged.appended.into_iter();
    let act = if acted { appended.next() } else { None };
    let tombstone = appended
        .next()
        .ok_or(AdminError::Node("the store recorded no tombstone"))?;

    Ok(Done {
        act,
        tombstone,
        erased: purged.events,
    })
}

#[cfg(test)]
mod tests;
