//! Retention (REC-15 in part, SEC-22): a retention policy, set per room
//! and provenance class by the principal's configuration or managed
//! policy, has the node purge what it holds of that class once it is
//! older than the policy's window. The purge is the node's own act, not
//! a principal's: it leaves a tombstone naming the policy, is audited
//! and is counted (ADM-07, SEC-22). The default policy keeps everything.
//!
//! An event ages from the first time mark after it in causal order,
//! never from a transcript timestamp (REC-15). A time mark is a
//! structural event the node records in its device seat at intervals
//! ([`mark`]), and it follows an event in causal order when a device seat
//! event at or before it cites a head of the event's writer at or past
//! the event (PRV-12), or, for the device seat's own events, when it
//! lies after them. An event no mark follows has no age yet.
//!
//! M1 records marks only while a retention policy is set, and applies
//! the policies when `parinn ingest` runs; the rest of REC-15 is M5's.

use key::random::{Entropy, draw};
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, Seq, WriterId};
use serde_json::json;
use store::{NewEvent, Store};

use crate::AdminError;
use crate::node::Node;
use crate::purge::{self, Done, PURGEABLE, Planned, Reason, Refusal};
use crate::quarantine::device_writers;

/// How long, in seconds, a time mark waits for the next.
pub const MARK_EVERY: u64 = 3_600;

/// Seconds in a day.
pub const DAY: u64 = 86_400;

/// A retention policy: events of `class` in `room` older than `days`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// The settings key that sets it, which the purge names.
    pub key: String,
    /// The room it covers; none covers every room.
    pub room: Option<RoomId>,
    pub class: Provenance,
    pub days: u64,
}

/// A range the policies have aged out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aged {
    pub writer: WriterId,
    pub first: Seq,
    pub last: Seq,
    /// The key of the policy that aged it.
    pub policy: String,
}

/// Records a time mark in `node`'s device seat when the last one is at
/// least [`MARK_EVERY`] seconds before `now`, or there is none; the
/// mark's address when it recorded one.
///
/// # Errors
///
/// Fails when no key can be drawn or the store fails.
pub fn mark(
    store: &mut Store,
    node: &Node,
    now: u64,
    entropy: &mut dyn Entropy,
) -> Result<Option<Address>, AdminError> {
    let marks = marks(store, node.device_writer())?;
    if marks
        .last()
        .is_some_and(|(_, last)| now < last.saturating_add(MARK_EVERY))
    {
        return Ok(None);
    }
    let event = NewEvent {
        kind: EventKind::TimeMark,
        provenance: Provenance::Structural,
        content: json!({"time": now}),
        commitment_key: CommitmentKey::from_bytes(draw(entropy)?),
        transcript_timestamp: None,
    };

    Ok(store
        .append(node.device_writer(), &[event])?
        .into_iter()
        .next())
}

/// The time marks of `device`'s log, with their seqs and the node's
/// clock they read, in seq order.
fn marks(store: &Store, device: WriterId) -> Result<Vec<(Seq, u64)>, AdminError> {
    Ok(store
        .contents(device, EventKind::TimeMark)?
        .into_iter()
        .filter_map(|(seq, content)| Some((seq, content["time"].as_u64()?)))
        .collect())
}

/// The seqs of one writer that the first mark after them dates alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Block {
    first: u64,
    last: u64,
    /// The clock of the first time mark after them.
    time: u64,
}

/// The blocks of `writer`'s events the marks of `device` date.
fn blocks(
    store: &Store,
    device: WriterId,
    writer: WriterId,
    marks: &[(Seq, u64)],
) -> Result<Vec<Block>, AdminError> {
    let mut out = Vec::new();
    let mut before = 0;
    if writer == device {
        for (seq, time) in marks {
            out.push(Block {
                first: before + 1,
                last: seq.get() - 1,
                time: *time,
            });
            before = seq.get();
        }
        return Ok(out);
    }
    for (citing, cited) in store.citations(device, writer)? {
        let Some((_, time)) = marks.iter().find(|(seq, _)| *seq >= citing) else {
            break;
        };
        if cited.get() > before {
            out.push(Block {
                first: before + 1,
                last: cited.get(),
                time: *time,
            });
            before = cited.get();
        }
    }

    Ok(out)
}

/// The policy covering events of `class` in `room`: the room's own, else
/// the policy for every room.
fn covering<'a>(policies: &'a [Policy], room: RoomId, class: &Provenance) -> Option<&'a Policy> {
    let of = |wanted: Option<RoomId>| {
        policies
            .iter()
            .find(|p| p.room == wanted && p.class == *class)
    };

    of(Some(room)).or_else(|| of(None))
}

/// The ranges `policies` have aged out of the node's rooms at `now`: the
/// events of a covered class whose window has passed since the first
/// mark after them, that still hold content and that a purge may erase,
/// grouped into runs of one writer and one policy. `personal` is the
/// node's personal room, whose device seats record the marks.
///
/// # Errors
///
/// Fails when the store does.
pub fn aged(
    store: &Store,
    personal: RoomId,
    policies: &[Policy],
    now: u64,
) -> Result<Vec<Aged>, AdminError> {
    let devices = device_writers(store, personal)?;
    let mut found = Vec::new();
    for seat in store.seats()? {
        let covered = policies
            .iter()
            .any(|p| p.room.is_none_or(|r| r == seat.room));
        let Some(head) = store.head(seat.writer)?.filter(|_| covered) else {
            continue;
        };
        let mut dated = Vec::new();
        for device in &devices {
            let marks = marks(store, *device)?;
            dated.extend(blocks(store, *device, seat.writer, &marks)?);
        }
        let mut run: Option<Aged> = None;
        for event in store.range_events(seat.writer, Seq::FIRST, head)? {
            let policy = covering(policies, seat.room, &event.provenance).filter(|p| {
                !event.erased
                    && PURGEABLE.contains(&event.kind)
                    && dated.iter().any(|b| {
                        b.first <= event.seq.get()
                            && event.seq.get() <= b.last
                            && now > b.time.saturating_add(p.days.saturating_mul(DAY))
                    })
            });
            match (policy, run.as_mut()) {
                (Some(p), Some(open))
                    if open.policy == p.key && open.last.next() == Some(event.seq) =>
                {
                    open.last = event.seq;
                }
                (policy, _) => {
                    found.extend(run.take());
                    run = policy.map(|p| Aged {
                        writer: seat.writer,
                        first: event.seq,
                        last: event.seq,
                        policy: p.key.clone(),
                    });
                }
            }
        }
        found.extend(run);
    }

    Ok(found)
}

/// What applying the policies did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Applied {
    /// The ranges purged, each with the policy behind it.
    pub purged: Vec<(Aged, Done)>,
    /// The ranges a purge refused, with the causes (ADM-07).
    pub refused: Vec<(Aged, Vec<Refusal>)>,
}

/// Applies `policies` at `now`: records a time mark if one is due, then
/// purges every range the policies have aged out, each as the node's own
/// purge naming its policy (SEC-22). With no policy it does nothing: the
/// default policy keeps everything (REC-15).
///
/// # Errors
///
/// Fails when no key can be drawn or the store fails; what was purged
/// before stays purged.
pub fn apply(
    store: &mut Store,
    node: &Node,
    policies: &[Policy],
    now: u64,
    entropy: &mut dyn Entropy,
) -> Result<Applied, AdminError> {
    let mut applied = Applied::default();
    if policies.is_empty() {
        return Ok(applied);
    }
    mark(store, node, now, entropy)?;
    for range in aged(store, node.personal_room(), policies, now)? {
        let reason = Reason::Retention {
            policy: range.policy.clone(),
        };
        match purge::plan(store, range.writer, range.first, range.last, &reason)? {
            Planned::Ready(plan) => {
                let done = purge::carry_out(store, node, &plan, None, entropy)?;
                applied.purged.push((range, done));
            }
            Planned::Refused(why) => applied.refused.push((range, why)),
        }
    }

    Ok(applied)
}

#[cfg(test)]
mod tests;
