//! Origin (PRV-13, RCL-09): where an event came from, derived from the
//! record and the node's key set alone, never read from a stored value,
//! and never changed by a quarantine or its release (I10). In PRV-13's
//! order an event is `bundle` or `peer` when an arrival event this node
//! recorded names it, `ingested` in the writer of an ingest seat, then
//! `witnessed`, else `peer`. M1 records no arrival event (backup restore
//! M3, peers M8, bundles M9), so only `ingested`, `witnessed` and the
//! `peer` fallback arise; [`Origin::Bundle`] is reserved for arrival.
//!
//! A writer reads `ingested` or `witnessed` only when a key of this
//! node's own certified its seat and that key's device seat holds the
//! seat's certificate record (PRV-12). A node's own keys are the device
//! keys whose log begins with their `minted` and that no node identity
//! change replaced: after a node identity change (REC-24) the replaced
//! key's writers read `peer`, and so do the new key's until a confirm
//! counted under PRV-12 names its inception, which no M1 command makes.

use std::collections::{BTreeMap, BTreeSet};

use key::certificate::{SeatCertificate, SeatKind};
use key::key_event::NodeIdentityChange;
use key::signing::PublicKey;
use record::digest::Digest;
use record::event::EventKind;
use record::id::{Address, Seq, WriterId};
use store::{Store, StoreError, Stored, StoredSeat};

use crate::node::begins_genesis;

/// Where an event came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    Witnessed,
    Ingested,
    Bundle,
    Peer,
}

impl Origin {
    /// The origin's name, as the envelope and the kept origins show it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Witnessed => "witnessed",
            Self::Ingested => "ingested",
            Self::Bundle => "bundle",
            Self::Peer => "peer",
        }
    }
}

/// This node's key set, as far as origin reads it: the device keys of
/// its own, and the certificate records each one's device seat holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeySet {
    /// The certificate records of each own device key's device seat, by
    /// the key's fingerprint: the certificate's digest and its writer.
    records: BTreeMap<Digest, BTreeSet<(String, String)>>,
}

impl KeySet {
    /// The key set `store` holds for the node whose device key is
    /// `node_key`: that key alone, and only while its device seat begins
    /// with its `minted` and no node identity change replaced it. A
    /// device log of another node's, held in the store, is no key of
    /// this one's.
    ///
    /// # Errors
    ///
    /// Fails when the store does.
    pub fn load(store: &Store, node_key: &PublicKey) -> Result<Self, StoreError> {
        let fingerprint = node_key.fingerprint();
        let mut device = None;
        let mut replaced = false;
        for seat in store.seats()? {
            let Some(cert) = SeatCertificate::from_value(&seat.certificate)
                .ok()
                .filter(|c| c.seat_kind == SeatKind::Device && c.check().is_ok())
            else {
                continue;
            };
            let first = store.event(Address::new(seat.writer, Seq::FIRST))?;
            if cert.certifying_key == *node_key && begins_genesis(first.as_ref(), node_key) {
                device = Some(seat.writer);
            }
            replaced |= first
                .as_ref()
                .filter(|s| s.link.header.kind == EventKind::NodeIdentityChange)
                .and_then(|s| s.content.as_ref())
                .and_then(|c| NodeIdentityChange::from_value(c).ok())
                .is_some_and(|change| change.replaces == fingerprint);
        }
        let mut records = BTreeMap::new();
        if let (Some(writer), false) = (device, replaced) {
            records.insert(fingerprint, certificate_records(&store.log(writer)?));
        }

        Ok(Self { records })
    }

    /// The origin of the writer of `seat`.
    #[must_use]
    pub fn origin_of(&self, seat: &StoredSeat) -> Origin {
        let certified = SeatCertificate::from_value(&seat.certificate)
            .ok()
            .filter(|c| c.check().is_ok() && c.writer_id() == seat.writer && c.room == seat.room);
        let Some(cert) = certified else {
            return Origin::Peer;
        };
        let recorded = self
            .records
            .get(&cert.certifying_key.fingerprint())
            .is_some_and(|r| r.contains(&(cert.digest().to_hex(), seat.writer.to_string())));
        match (recorded, cert.ingest_seat) {
            (false, _) => Origin::Peer,
            (true, true) => Origin::Ingested,
            (true, false) => Origin::Witnessed,
        }
    }
}

/// Whether the certificate of `seat` names a device seat, as opposed to
/// a run seat: only a device seat records the operator and structural
/// events the trust policy trusts (PRV-02).
#[must_use]
pub fn is_device_seat(seat: &StoredSeat) -> bool {
    SeatCertificate::from_value(&seat.certificate).is_ok_and(|c| c.seat_kind == SeatKind::Device)
}

/// The certificate records `log` holds: each one's certificate digest
/// and writer.
fn certificate_records(log: &[Stored]) -> BTreeSet<(String, String)> {
    log.iter()
        .filter(|s| s.link.header.kind == EventKind::CertificateRecord)
        .filter_map(|s| s.content.as_ref())
        .filter_map(|c| Some((c["certificate"].as_str()?, c["writer"].as_str()?)))
        .map(|(digest, writer)| (digest.to_owned(), writer.to_owned()))
        .collect()
}

/// The origin of every writer with a seat in `store`, for the node whose
/// device key is `node_key`.
///
/// # Errors
///
/// Fails when the store does.
pub fn derive(
    store: &Store,
    node_key: &PublicKey,
) -> Result<BTreeMap<WriterId, Origin>, StoreError> {
    let keys = KeySet::load(store, node_key)?;

    Ok(store
        .seats()?
        .iter()
        .map(|seat| (seat.writer, keys.origin_of(seat)))
        .collect())
}

#[cfg(test)]
mod tests;
