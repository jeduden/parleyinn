//! The items the recall tools return: one event's closed fields beside
//! its content (§9.3). Every field but the content derives from the
//! record and the node's key set, never from a stored value (PRV-02,
//! PRV-13); the content is the event's own, and reaches the model only
//! as one JSON string inside the envelope (I2).

use administration::origin::{KeySet, is_device_seat};
use envelope::{IntegrityStatus, Item, Run, TrustLevel};
use record::commitment::commit;
use record::digest::Digest;
use record::id::{Address, Seq, WriterId};
use serde_json::Value;
use store::{Hit, Stored, StoredSeat};

use crate::expand::CONTINUED;
use crate::scope::Scope;
use crate::{NO_INDEX, Seated, Seats, Server, UNREADABLE, shown_origin, trust_of};

const NO_SEAT: &str = "the event's writer has no seat";

impl Server {
    /// The seat that wrote for `writer` and where its events came from,
    /// looked up once for the rest of the call.
    pub(crate) fn seated<'s>(
        &self,
        keys: &KeySet,
        writer: WriterId,
        seats: &'s mut Seats,
    ) -> Result<&'s Seated, String> {
        if let std::collections::btree_map::Entry::Vacant(vacant) = seats.entry(writer) {
            let seat = self
                .store
                .seat(writer)
                .map_err(|_| NO_INDEX.to_owned())?
                .ok_or(NO_SEAT)?;
            vacant.insert(Seated {
                author: seat.seat.to_string(),
                origin: shown_origin(keys.origin_of(&seat)),
                device: is_device_seat(&seat),
                room: seat.room,
            });
        }

        seats.get(&writer).ok_or_else(|| NO_SEAT.to_owned())
    }

    /// The trust level of the event at `address`.
    pub(crate) fn trust_at(
        &self,
        address: Address,
        keys: &KeySet,
        seats: &mut Seats,
    ) -> Result<TrustLevel, String> {
        let (kind, provenance) = self
            .store
            .kind_and_provenance(address)
            .map_err(|_| NO_INDEX.to_owned())?
            .ok_or_else(|| format!("no event at {address}"))?;
        let seated = self.seated(keys, address.writer, seats)?;

        Ok(trust_of(&provenance, kind, seated.origin, seated.device))
    }

    /// The item for one hit: the event's closed fields, its score and
    /// the start of its text.
    pub(crate) fn hit_item(
        &self,
        hit: &Hit,
        keys: &KeySet,
        scope: &Scope,
        seats: &mut Seats,
    ) -> Result<Item, String> {
        let unreadable = |_| NO_INDEX.to_owned();
        let address = hit.address;
        let (kind, provenance) = self
            .store
            .kind_and_provenance(address)
            .map_err(unreadable)?
            .ok_or_else(|| format!("no event at {address}"))?;
        let preview = self
            .store
            .preview(address)
            .map_err(unreadable)?
            .unwrap_or_default();
        let seated = self.seated(keys, address.writer, seats)?;

        Ok(Item {
            address: address.full(&seated.room),
            writer: address.writer.to_string(),
            author: seated.author.clone(),
            origin: seated.origin,
            run: run_of(scope, address.writer),
            kind: kind.as_str().to_owned(),
            provenance: provenance.name(),
            trust: trust_of(&provenance, kind, seated.origin, seated.device),
            flags: Vec::new(),
            integrity: IntegrityStatus::Unverified,
            score: Some(hit.score),
            content: preview,
        })
    }

    /// The item for a stored event, its content from the character
    /// `offset` on. The event is served only once it checks against the
    /// record (RCL-03).
    pub(crate) fn event_item(
        &self,
        stored: &Stored,
        seat: &StoredSeat,
        keys: &KeySet,
        scope: &Scope,
        offset: usize,
    ) -> Result<Item, String> {
        let header = &stored.link.header;
        let full = header.address.full(&seat.room);
        let Some(content) = &stored.content else {
            return Err(format!("the content at {full} was removed"));
        };
        if !self.checks(stored).map_err(|_| UNREADABLE.to_owned())? {
            return Err(format!(
                "the event at {full} does not check against the record: run parinn verify"
            ));
        }
        let origin = shown_origin(keys.origin_of(seat));
        let text = content_text(content);
        let mut flags = Vec::new();
        let content = if offset > 0 {
            flags.push(CONTINUED.to_owned());
            text.chars().skip(offset).collect()
        } else {
            text
        };

        Ok(Item {
            address: full,
            writer: header.address.writer.to_string(),
            author: seat.seat.to_string(),
            origin,
            run: run_of(scope, header.address.writer),
            kind: header.kind.as_str().to_owned(),
            provenance: header.provenance.name(),
            trust: trust_of(
                &header.provenance,
                header.kind,
                origin,
                is_device_seat(seat),
            ),
            flags,
            integrity: IntegrityStatus::Unsigned,
            score: None,
            content,
        })
    }

    /// Whether `stored` checks against the record: its content opens
    /// its commitment, its header hashes to its stored hash, and its
    /// `prev_hash` is its predecessor's stored hash.
    fn checks(&self, stored: &Stored) -> Result<bool, store::StoreError> {
        let header = &stored.link.header;
        let opens = match (&stored.content, &stored.commitment_key) {
            (Some(content), Some(key)) => {
                commit(key, content).is_ok_and(|c| c == header.commitment)
            }
            _ => false,
        };
        let prev = match Seq::new(header.address.seq.get() - 1) {
            Some(seq) => self
                .store
                .event(Address::new(header.address.writer, seq))?
                .map(|p| p.link.hash),
            None => Some(Digest::ZERO),
        };

        Ok(opens && header.hash() == stored.link.hash && prev == Some(header.prev_hash))
    }
}

/// Whether an event of `writer` belongs to the calling run.
fn run_of(scope: &Scope, writer: WriterId) -> Run {
    if scope.is_current(writer) {
        Run::Current
    } else {
        Run::Other
    }
}

/// An event's content as the text a model reads: its `text` when it has
/// one, else its JSON.
fn content_text(content: &Value) -> String {
    match content {
        Value::Object(fields) if fields.get("text").is_some_and(Value::is_string) => {
            fields["text"].as_str().unwrap_or_default().to_owned()
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests;
