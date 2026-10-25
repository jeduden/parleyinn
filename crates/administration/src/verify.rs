//! `parinn verify`'s check of the record (ADM-08's first part), read from
//! one snapshot of the store and checked against this node's device key:
//!
//! - every writer's chain (REC-10) and every cited head;
//! - every stored content against its commitment (REC-17);
//! - the inceptions of the node's device keys (PRV-12): a device seat a
//!   key certified whose log begins with that key's `minted` key event,
//!   or with a node identity change event naming it and then its
//!   `minted` (REC-24), the event's chain head naming an event of its
//!   writer and the key it replaces having an inception too;
//! - the node's genesis: following this node's device key back through
//!   the node identity changes leads to the device seat that created the
//!   personal room, whose create room act records the nonce the room's
//!   id derives from;
//! - every writer's seat certificate (PRV-11): it checks, names its own
//!   seat, was certified by a key with an inception, and the device seat
//!   holding that inception holds its certificate record, as each
//!   certificate record there names a seat that key certified;
//! - trusted provenance only in device seats;
//! - every origin the store keeps against a fresh derivation (PRV-13);
//! - every writer's segments, which tile its log, only its newest open,
//!   none of a newer format version (REC-19, ADM-05); every payload.
//!
//! The report names each seat with the seat kind and the ingest flag
//! its seat certificate names, read from that certificate alone, and
//! every writer chain that ended without a closed segment.
//!
//! The first failure is named by the event's full address, or by the
//! writer. A record that passes yields its causal order.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use key::certificate::{SeatCertificate, SeatKind};
use key::key_event::{KeyEvent, NodeIdentityChange};
use key::signing::PublicKey;
use record::chain::{self, ChainError, Link};
use record::commitment::commit;
use record::digest::{Digest, unhex};
use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, SeatId, WriterId};
use serde_json::Value;
use store::{Segment, Store, StoreError, Stored, StoredSeat};

use crate::AdminError;
use crate::node::{Node, begins_genesis};
use crate::origin;
use crate::repository;

/// What a verified record holds, and the causal order its cited heads
/// give.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub writers: usize,
    pub events: usize,
    pub order: Vec<Address>,
    /// The room each writer's seat is in.
    pub rooms: BTreeMap<WriterId, RoomId>,
    /// Each seat, as its seat certificate names it, by writer id.
    pub seats: Vec<SeatReport>,
    /// Each writer's open segment: a writer chain that ended without a
    /// closed segment, unless its writer still appends (REC-19).
    pub open_segments: Vec<Segment>,
}

/// A seat as `parinn verify` reports it: everything but its writer read
/// from its seat certificate alone, never from its events (PRV-11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatReport {
    pub writer: WriterId,
    pub seat: SeatId,
    pub room: RoomId,
    pub seat_kind: SeatKind,
    pub ingest_seat: bool,
    pub certificate: Value,
}

impl Report {
    /// The causal order, each event by its full address.
    #[must_use]
    pub fn full_order(&self) -> Vec<String> {
        self.order
            .iter()
            .map(|a| {
                self.rooms
                    .get(&a.writer)
                    .map_or_else(|| a.to_string(), |r| a.full(r))
            })
            .collect()
    }
}

/// Where and how the record fails verification.
#[derive(Debug)]
pub enum VerifyError {
    /// A writer's chain, or a head it cites, breaks at this event.
    Broken { address: String, reason: String },
    /// This event's stored content does not open its commitment, or was
    /// removed with no tombstone to record it.
    Content { address: String, removed: bool },
    /// The key event at this address does not check.
    KeyEvent { address: String, reason: String },
    /// The certificate record at this address names no seat the store
    /// holds.
    Record { address: String },
    /// The event at this address claims trusted provenance outside a
    /// device seat.
    Provenance { address: String, provenance: String },
    /// A writer's seat certificate does not check.
    Seat { writer: WriterId, reason: String },
    /// This node's genesis does not check.
    Genesis(&'static str),
    /// A derived artifact the store keeps differs from what the record
    /// and the node's key set derive (I10): the origin kept for this
    /// writer.
    Derived { writer: WriterId },
    /// A derived artifact the store keeps differs from what the record
    /// derives (I10): the identity kept for this worktree, named by its
    /// digest, or one the record holds and the store does not.
    DerivedWorktree { worktree: String },
    /// A segment is of a newer format version than this parinn's: it is
    /// refused and left as it is (ADM-05).
    NewerSegment(Segment),
    /// A writer's segments do not tile its log.
    Segments { writer: WriterId, reason: String },
    /// The payload of the event at this address cannot be read.
    Payload { address: String, reason: String },
    /// A stored value does not parse.
    Corrupt(StoreError),
    /// The store failed.
    Store(StoreError),
}

impl VerifyError {
    /// The failure's class, as `parinn verify --json` names it.
    #[must_use]
    pub fn class(&self) -> &'static str {
        match self {
            Self::Broken { .. } => "chain",
            Self::Content { .. } => "content",
            Self::KeyEvent { .. } => "key_event",
            Self::Record { .. } => "certificate_record",
            Self::Provenance { .. } => "provenance",
            Self::Seat { .. } => "seat",
            Self::Genesis(_) => "genesis",
            Self::Derived { .. } | Self::DerivedWorktree { .. } => "derived_artifact_consistency",
            Self::NewerSegment(_) => "newer_segment",
            Self::Segments { .. } => "segment",
            Self::Payload { .. } => "payload",
            Self::Corrupt(_) => "corrupt",
            Self::Store(_) => "store",
        }
    }

    /// The full address of the event the failure lies at, if it lies
    /// at one.
    #[must_use]
    pub fn address(&self) -> Option<&str> {
        match self {
            Self::Broken { address, .. }
            | Self::Content { address, .. }
            | Self::KeyEvent { address, .. }
            | Self::Record { address }
            | Self::Provenance { address, .. }
            | Self::Payload { address, .. } => Some(address),
            Self::Seat { .. }
            | Self::Genesis(_)
            | Self::Derived { .. }
            | Self::DerivedWorktree { .. }
            | Self::NewerSegment(_)
            | Self::Segments { .. }
            | Self::Corrupt(_)
            | Self::Store(_) => None,
        }
    }
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Broken { address, reason } => write!(f, "chain broken at {address}: {reason}"),
            Self::Content {
                address,
                removed: true,
            } => write!(
                f,
                "the content at {address} was removed, and no tombstone records it"
            ),
            Self::Content {
                address,
                removed: false,
            } => write!(f, "the content at {address} does not open its commitment"),
            Self::KeyEvent { address, reason } => {
                write!(f, "the key event at {address} does not check: {reason}")
            }
            Self::Record { address } => write!(
                f,
                "the certificate record at {address} names a seat this node does not hold"
            ),
            Self::Provenance {
                address,
                provenance,
            } => write!(
                f,
                "the event at {address} is of {provenance} provenance, which only a device seat records"
            ),
            Self::Seat { writer, reason } | Self::Segments { writer, reason } => {
                write!(f, "writer {writer}: {reason}")
            }
            Self::NewerSegment(s) => write!(f, "{}", StoreError::NewerSegment(s.clone())),
            Self::Payload { address, reason } => {
                write!(f, "the payload of {address} cannot be read: {reason}")
            }
            Self::Genesis(why) => f.write_str(why),
            Self::Derived { writer } => write!(
                f,
                "derived artifact consistency: the origin kept for writer {writer} differs from the record's: run parinn rebuild"
            ),
            Self::DerivedWorktree { worktree } => write!(
                f,
                "derived artifact consistency: the identity kept for worktree {worktree} differs from the record's: run parinn rebuild"
            ),
            Self::Corrupt(e) | Self::Store(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for VerifyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Corrupt(e) | Self::Store(e) => Some(e),
            _ => None,
        }
    }
}

impl From<StoreError> for VerifyError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::Corrupt { .. } => Self::Corrupt(e),
            e => Self::Store(e),
        }
    }
}

/// The provenance classes only a device seat records, and, for
/// structural events alone, a witnessed run's run seat: this node's own
/// operator and structural events (I2). A transcript line's
/// `harness_meta` is untrusted and recorded in an ingest seat (PRV-08);
/// where the launcher's trusted kind is recorded is M7's.
const TRUSTED: [Provenance; 2] = [Provenance::Operator, Provenance::Structural];

/// Each writer's log and segments as one snapshot read them.
type Logs = Vec<(
    WriterId,
    Result<Vec<Stored>, StoreError>,
    Result<Vec<Segment>, StoreError>,
)>;

/// Verifies the whole record in `store` against the node whose device
/// key is `node_key`.
///
/// # Errors
///
/// Fails at the first failure, naming it, or when the store fails.
pub fn verify(store: &Store, node_key: &PublicKey) -> Result<Report, VerifyError> {
    let (seats, writers, logs) = store.snapshot(|s| {
        let seats = s.seats()?;
        let writers = s.writers()?;
        let logs: Logs = writers
            .iter()
            .map(|w| (*w, s.log(*w), s.segments(*w)))
            .collect();
        Ok((seats, writers, logs))
    })?;
    let seats: BTreeMap<WriterId, StoredSeat> = seats.into_iter().map(|s| (s.writer, s)).collect();
    let full = |a: Address| {
        seats
            .get(&a.writer)
            .map_or_else(|| a.to_string(), |s| a.full(&s.room))
    };
    let broken = |e: ChainError| VerifyError::Broken {
        address: full(e.address),
        reason: e.fault.to_string(),
    };
    let mut held = BTreeMap::new();
    let mut open_segments = Vec::new();
    for (writer, log, segments) in logs {
        let log = log.map_err(|e| unread(e, &full))?;
        let segments = segments?;
        tile(writer, &segments, log.len() as u64)?;
        open_segments.extend(segments.last().filter(|s| !s.closed).cloned());
        let links: Vec<Link> = log.iter().map(|s| s.link.clone()).collect();
        chain::check_writer(writer, &links).map_err(broken)?;
        if let Some(s) = log.iter().find(|s| !opens(s)) {
            return Err(VerifyError::Content {
                address: full(s.link.header.address),
                removed: s.content.is_none() && s.commitment_key.is_none(),
            });
        }
        held.insert(writer, log);
    }
    let record: BTreeMap<WriterId, Vec<Link>> = held
        .iter()
        .map(|(w, log)| (*w, log.iter().map(|s| s.link.clone()).collect()))
        .collect();
    chain::check_heads(&record).map_err(broken)?;
    let certs = certificates(&writers, &seats)?;
    let inceptions = inceptions(&certs, &held, &full)?;
    genesis(node_key, &certs, &held, &inceptions)?;
    let recorded = certificate_records(&held, &certs, &inceptions, &full)?;
    seats_counted(&certs, &inceptions, &recorded)?;
    for (writer, log) in &held {
        let cert = certs.get(writer);
        let device = cert.is_some_and(|c| c.seat_kind == SeatKind::Device);
        // A witnessed run's MCP server records its hook observations in
        // its run seat (REC-19): structural, never an operator's.
        let witnessed = cert.is_some_and(|c| c.seat_kind == SeatKind::Run && !c.ingest_seat);
        events(log, device, witnessed, &full)?;
    }
    let order = chain::causal_order(&record).map_err(broken)?;
    kept_origins_agree(store, node_key)?;

    Ok(Report {
        writers: writers.len(),
        events: order.len(),
        order,
        rooms: certs.iter().map(|(w, c)| (*w, c.room)).collect(),
        seats: certs.iter().map(|(w, c)| seat_report(*w, c)).collect(),
        open_segments,
    })
}

/// Checks that every origin the store keeps is the one the record and
/// the node's key set derive (PRV-13, I10). A writer with no kept origin
/// is not yet derived, and a store that keeps none holds no claim.
fn kept_origins_agree(store: &Store, node_key: &PublicKey) -> Result<(), VerifyError> {
    let derived = origin::derive(store, node_key)?;
    for (writer, kept) in store.origins()? {
        if derived.get(&writer).map(|o| o.as_str()) != Some(kept.as_str()) {
            return Err(VerifyError::Derived { writer });
        }
    }

    Ok(())
}

/// Checks that the identity this node keeps for each of its worktrees is
/// the one its record derives, and that it keeps one for every worktree
/// the record binds (LANE-02, I10, I6): a store restored without its
/// table reports it rather than lose change detection silently. A store
/// that never made the table keeps none. It needs the node's device key,
/// which the names derive from, so it runs beside [`verify`], not in it.
///
/// # Errors
///
/// Fails naming the first worktree whose kept identity differs, or when
/// the store fails.
pub fn kept_worktrees_agree(store: &Store, node: &Node) -> Result<(), VerifyError> {
    let derived = repository::derive_worktrees(store, node).map_err(derive_failed)?;
    let kept = store.worktree_identities()?;
    let worktrees: BTreeSet<&String> = kept.keys().chain(derived.keys()).collect();
    for worktree in worktrees {
        if kept.get(worktree) != derived.get(worktree) {
            return Err(VerifyError::DerivedWorktree {
                worktree: worktree.clone(),
            });
        }
    }

    Ok(())
}

/// Why the worktrees' identities could not be derived, as verification
/// names it: the store failed, or the record held something it cannot
/// read.
fn derive_failed(e: AdminError) -> VerifyError {
    match e {
        AdminError::Store(e) => VerifyError::Store(e),
        other => VerifyError::Corrupt(StoreError::Corrupt {
            address: None,
            what: other.to_string(),
        }),
    }
}

/// Why a writer's log could not be read, as verification names it.
fn unread(e: StoreError, full: &dyn Fn(Address) -> String) -> VerifyError {
    match e {
        StoreError::Corrupt {
            address: Some(a),
            what,
        } => VerifyError::Broken {
            address: full(a),
            reason: format!("the stored header does not parse: {what}"),
        },
        StoreError::NewerSegment(s) => VerifyError::NewerSegment(s),
        StoreError::Payload { address, source } => VerifyError::Payload {
            address: full(address),
            reason: source.to_string(),
        },
        e => e.into(),
    }
}

/// Checks that `writer`'s `segments`, in seq order, tile its log of
/// `len` events: each starts where the one before it ended, the last
/// ends at its newest event, and only the last may be open.
fn tile(writer: WriterId, segments: &[Segment], len: u64) -> Result<(), VerifyError> {
    let fail = |reason: String| VerifyError::Segments { writer, reason };
    let mut next = 1;
    for (i, s) in segments.iter().enumerate() {
        let first = s.first.get();
        if first < next {
            return Err(fail(format!(
                "the segment from seq {first} overlaps the one before it"
            )));
        }
        if first > next {
            return Err(fail(format!("no segment holds seq {next}")));
        }
        if !s.closed && i + 1 < segments.len() {
            return Err(fail(format!(
                "the segment from seq {first} is open, though a later one follows it"
            )));
        }
        if s.last.get() > len {
            return Err(fail(format!(
                "a segment runs past its newest event, seq {len}"
            )));
        }
        next = s.last.get() + 1;
    }
    if next <= len {
        return Err(fail(format!("no segment holds seq {next}")));
    }

    Ok(())
}

/// Checks that each seat was certified by a key with an inception, and
/// that its certificate record stands in that key's device seat.
fn seats_counted(
    certs: &BTreeMap<WriterId, SeatCertificate>,
    inceptions: &BTreeMap<Digest, Inception>,
    recorded: &BTreeSet<WriterId>,
) -> Result<(), VerifyError> {
    for (writer, cert) in certs {
        let seat_error = |reason: &str| VerifyError::Seat {
            writer: *writer,
            reason: reason.to_owned(),
        };
        if !inceptions.contains_key(&cert.certifying_key.fingerprint()) {
            return Err(seat_error(
                "its seat certificate was certified by a key with no inception in this node's record",
            ));
        }
        if !recorded.contains(writer) {
            return Err(seat_error(
                "no certificate record in the device seat of the key that certified it holds its seat certificate",
            ));
        }
    }

    Ok(())
}

/// The report of `writer`'s seat, read from its certificate `cert`.
fn seat_report(writer: WriterId, cert: &SeatCertificate) -> SeatReport {
    SeatReport {
        writer,
        seat: cert.seat_id(),
        room: cert.room,
        seat_kind: cert.seat_kind,
        ingest_seat: cert.ingest_seat,
        certificate: cert.to_value(),
    }
}

/// Whether a stored event's content opens its commitment. Content
/// removed with its key does not: it is a loss until purge records a
/// tombstone for it (ADM-07).
fn opens(stored: &Stored) -> bool {
    match (&stored.content, &stored.commitment_key) {
        (Some(content), Some(key)) => {
            commit(key, content).is_ok_and(|c| c == stored.link.header.commitment)
        }
        _ => false,
    }
}

/// Every writer's seat certificate, each checked and naming its own
/// seat.
fn certificates(
    writers: &[WriterId],
    seats: &BTreeMap<WriterId, StoredSeat>,
) -> Result<BTreeMap<WriterId, SeatCertificate>, VerifyError> {
    let mut out = BTreeMap::new();
    for writer in writers {
        let seat_error = |reason: String| VerifyError::Seat {
            writer: *writer,
            reason,
        };
        let seat = seats
            .get(writer)
            .ok_or_else(|| seat_error("no seat certificate names it".to_owned()))?;
        let cert = SeatCertificate::from_value(&seat.certificate)
            .and_then(|c| c.check().map(|()| c))
            .map_err(|e| seat_error(format!("its seat certificate: {e}")))?;
        if cert.writer_id() != *writer || cert.seat_id() != seat.seat || cert.room != seat.room {
            return Err(seat_error(
                "its seat certificate names another seat".to_owned(),
            ));
        }
        out.insert(*writer, cert);
    }

    Ok(out)
}

/// A device key's inception as the record holds it.
#[derive(Debug, Clone, Copy)]
struct Inception {
    key: PublicKey,
    /// The device seat whose log holds it.
    writer: WriterId,
    /// The fingerprint of the key it replaces, and the node identity
    /// change event naming it; none for a genesis log's.
    replaces: Option<(Digest, Address)>,
}

/// The inceptions the device seats' logs hold, by their key's
/// fingerprint: a `minted` first, or a node identity change event and
/// then a `minted`, each of the key that certified the seat.
fn inceptions(
    certs: &BTreeMap<WriterId, SeatCertificate>,
    held: &BTreeMap<WriterId, Vec<Stored>>,
    full: &dyn Fn(Address) -> String,
) -> Result<BTreeMap<Digest, Inception>, VerifyError> {
    let mut out = BTreeMap::new();
    let devices = certs
        .iter()
        .filter(|(_, c)| c.seat_kind == SeatKind::Device);
    for (writer, cert) in devices {
        let log = held.get(writer).map_or(&[][..], Vec::as_slice);
        let key = cert.certifying_key;
        let replaces = if begins_genesis(log.first(), &key) {
            None
        } else if let Some(change) = identity_change(log, &key) {
            Some(change?)
        } else {
            continue;
        };
        if let Some((_, at)) = replaces {
            check_chain_head(log, held, full, at)?;
        }
        out.entry(key.fingerprint()).or_insert(Inception {
            key,
            writer: *writer,
            replaces: replaces.map(|(e, at)| (e.replaces, at)),
        });
    }
    for inception in out.values() {
        if let Some((replaced, at)) = inception.replaces
            && !out.contains_key(&replaced)
        {
            return Err(VerifyError::KeyEvent {
                address: full(at),
                reason: "it replaces a key with no inception in this node's record".to_owned(),
            });
        }
    }

    Ok(out)
}

/// The node identity change event that begins `log`, when its second
/// event is the `minted` of `key` and the event names that key.
fn identity_change(
    log: &[Stored],
    key: &PublicKey,
) -> Option<Result<(NodeIdentityChange, Address), VerifyError>> {
    let first = log
        .first()
        .filter(|s| s.link.header.kind == EventKind::NodeIdentityChange)?;
    let event = first
        .content
        .as_ref()
        .and_then(|c| NodeIdentityChange::from_value(c).ok())?;
    (event.key == key.fingerprint() && begins_genesis(log.get(1), key))
        .then(|| Ok((event, first.link.header.address)))
}

/// Checks that the node identity change event at `at`, the first of
/// `log`, carries a chain head naming an event of its writer.
fn check_chain_head(
    log: &[Stored],
    held: &BTreeMap<WriterId, Vec<Stored>>,
    full: &dyn Fn(Address) -> String,
    at: Address,
) -> Result<(), VerifyError> {
    let head = log
        .first()
        .and_then(|s| s.content.as_ref())
        .and_then(|c| NodeIdentityChange::from_value(c).ok())
        .map(|e| e.chain_head);
    let named = head.is_some_and(|h| {
        let index = usize::try_from(h.address.seq.get() - 1).unwrap_or(usize::MAX);
        held.get(&h.address.writer)
            .and_then(|l| l.get(index))
            .is_some_and(|s| s.link.hash == h.head)
    });
    if named {
        Ok(())
    } else {
        Err(VerifyError::KeyEvent {
            address: full(at),
            reason: "its chain head names no event of its writer".to_owned(),
        })
    }
}

/// Checks this node's genesis: `node_key`'s inception, followed back
/// through the keys each replaces, leads to a genesis log's inception,
/// whose device seat's create room act records the nonce its room's id
/// derives from.
fn genesis(
    node_key: &PublicKey,
    certs: &BTreeMap<WriterId, SeatCertificate>,
    held: &BTreeMap<WriterId, Vec<Stored>>,
    inceptions: &BTreeMap<Digest, Inception>,
) -> Result<WriterId, VerifyError> {
    let mut inception = inceptions
        .get(&node_key.fingerprint())
        .ok_or(VerifyError::Genesis(
            "no device seat of this node's device key holds its inception",
        ))?;
    // Every replaced key has an inception (`inceptions` checked it), so
    // only a cycle runs past the number of inceptions.
    let mut steps = 0;
    while let Some((replaced, _)) = inception.replaces {
        steps += 1;
        inception = inceptions
            .get(&replaced)
            .filter(|_| steps <= inceptions.len())
            .ok_or(VerifyError::Genesis(
                "this node's device key leads back to no genesis",
            ))?;
    }
    let log = held.get(&inception.writer).map_or(&[][..], Vec::as_slice);
    let cert = certs.get(&inception.writer);
    let room = log
        .get(1)
        .filter(|s| s.link.header.kind == EventKind::CreateRoom)
        .and_then(|s| s.content.as_ref())
        .zip(cert)
        .and_then(|(act, cert)| derived_room(&cert.seat_key, act).filter(|r| *r == cert.room));
    room.map(|_| inception.writer).ok_or(VerifyError::Genesis(
        "the personal room's id does not derive from its device seat's key and the nonce its create room act records",
    ))
}

/// The room a create room act names, when its id derives from the seat
/// key `seat_key` and the nonce the act records.
fn derived_room(seat_key: &PublicKey, act: &Value) -> Option<RoomId> {
    let nonce = act["nonce"].as_str().and_then(|n| unhex(n).ok())?;
    let room = RoomId::derive(seat_key.as_bytes(), &nonce);

    (act["room"].as_str() == Some(room.to_string().as_str())).then_some(room)
}

/// The writers whose seat certificates the certificate records hold:
/// those in the device seat holding each key's inception, each of which
/// must name, by its writer, its seat and its certificate's digest, a
/// seat that key certified.
fn certificate_records(
    held: &BTreeMap<WriterId, Vec<Stored>>,
    certs: &BTreeMap<WriterId, SeatCertificate>,
    inceptions: &BTreeMap<Digest, Inception>,
    full: &dyn Fn(Address) -> String,
) -> Result<BTreeSet<WriterId>, VerifyError> {
    let mut out = BTreeSet::new();
    for inception in inceptions.values() {
        let log = held.get(&inception.writer).map_or(&[][..], Vec::as_slice);
        let records = log
            .iter()
            .filter(|s| s.link.header.kind == EventKind::CertificateRecord);
        for stored in records {
            let content = stored.content.as_ref().unwrap_or(&Value::Null);
            let writer = content["writer"]
                .as_str()
                .and_then(|w| WriterId::parse(w).ok());
            let named = writer
                .and_then(|w| certs.get(&w).map(|c| (w, c)))
                .filter(|(_, c)| {
                    c.certifying_key == inception.key
                        && content["certificate"].as_str() == Some(c.digest().to_hex().as_str())
                        && content["seat"].as_str() == Some(c.seat_id().to_string().as_str())
                });
            let (writer, _) = named.ok_or_else(|| VerifyError::Record {
                address: full(stored.link.header.address),
            })?;
            out.insert(writer);
        }
    }

    Ok(out)
}

/// Checks each event of one writer's `log`: a key event's content is a
/// key event, a node identity change event's is one, and trusted
/// provenance appears only when `device`, the writer a device seat's,
/// or, for structural provenance alone, when `witnessed`, the writer a
/// run seat's that is no ingest seat.
fn events(
    log: &[Stored],
    device: bool,
    witnessed: bool,
    full: &dyn Fn(Address) -> String,
) -> Result<(), VerifyError> {
    for stored in log {
        let header = &stored.link.header;
        let structural = witnessed && header.provenance == Provenance::Structural;
        if !device && !structural && TRUSTED.contains(&header.provenance) {
            return Err(VerifyError::Provenance {
                address: full(header.address),
                provenance: header.provenance.name(),
            });
        }
        // Content that is not there failed the commitment already.
        let content = stored.content.as_ref().unwrap_or(&Value::Null);
        let parsed = match header.kind {
            EventKind::KeyEvent => KeyEvent::from_value(content).map(drop),
            EventKind::NodeIdentityChange => NodeIdentityChange::from_value(content).map(drop),
            _ => Ok(()),
        };
        parsed.map_err(|e| VerifyError::KeyEvent {
            address: full(header.address),
            reason: e.to_string(),
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
