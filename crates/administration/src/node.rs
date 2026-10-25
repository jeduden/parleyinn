//! The node in a home. At its first start it mints its device key, the
//! recovery key its genesis log commits to, and its device seat's key in
//! the personal room, and keeps each in its own entry of the home's key
//! store (SEC-10). That seat's writer records, in one transaction, the
//! genesis log's `minted` key event, then the personal room's create
//! room act, then the certificate record of its own seat certificate
//! (PRV-11, PRV-12).
//!
//! Before each later start's first append it checks the node identity
//! (REC-24). On a change it mints a new device key, recovery key and
//! device seat key, and the new device seat, in the same personal room,
//! records a node identity change event naming the replaced and the new
//! key and the chain head of the key event log it leaves, then the new
//! key's `minted`, then its certificate record (SEC-27). Nothing is
//! appended under an old key again. A start finds its node by the
//! device key's inception.
//!
//! A start stages its new keys in staged entries of the key store and
//! writes the home's staged file, which names what it staged; then it
//! records, places the keys, the device key last, and removes both. A
//! start that finds the staged file finishes what an interrupted one
//! began, so the store never names a key the home lost, and a crash
//! never leaves a home with a device key but no inception.
//!
//! A device key missing from the key store is told apart from a key
//! store that is locked or cannot be read; neither starts a second
//! genesis (PRV-12).

use std::fs;

use key::KeyError;
use key::certificate::{SeatCertificate, SeatKind};
use key::file::{read_secret, sync_directory, write_secret};
use key::key_event::{KeyEvent, NodeIdentityChange};
use key::key_store::{Entry, KeyStore};
use key::random::{Entropy, draw};
use key::signing::{DeviceKey, PublicKey, RecoveryKey, SeatKey};
use record::chain::SeenHead;
use record::commitment::CommitmentKey;
use record::digest::{Digest, hex, unhex};
use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, Seq, WriterId};
use serde_json::{Value, json};
use store::{NewEvent, Store, Stored, StoredSeat};

use crate::AdminError;
use crate::home::Home;
use crate::node_identity::{self, IdentitySource};

/// A node identity change a start recorded (REC-24): the fingerprints of
/// the device key it replaced and of the new one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentityChange {
    pub replaced: Digest,
    pub key: Digest,
}

/// The node: its device key, its device seat's writer and its personal
/// room.
#[derive(Debug)]
pub struct Node {
    device: DeviceKey,
    device_writer: WriterId,
    room: RoomId,
    change: Option<IdentityChange>,
}

impl Node {
    /// The node in `home`, its keys in the home's key store: started for
    /// the first time when the home holds no device key and the store no
    /// seat, moved to new keys when the node identity changed, and
    /// finished when a start was interrupted.
    ///
    /// # Errors
    ///
    /// Fails when a key cannot be minted, kept or read, when the device
    /// key is missing but the store holds seats, when the staged start
    /// does not read, when the store fails, or when the home holds a
    /// device key whose inception the store lacks.
    pub fn open(
        home: &Home,
        store: &mut Store,
        entropy: &mut dyn Entropy,
    ) -> Result<Self, AdminError> {
        Self::open_in(home, home.key_store().as_ref(), store, entropy)
    }

    /// [`open`](Self::open), its keys in `keys`.
    ///
    /// # Errors
    ///
    /// As [`open`](Self::open).
    pub fn open_in(
        home: &Home,
        keys: &dyn KeyStore,
        store: &mut Store,
        entropy: &mut dyn Entropy,
    ) -> Result<Self, AdminError> {
        if !home.staged().exists() {
            prepare(home, keys, store, entropy)?;
        }
        let change = match Staged::load(home, keys)? {
            Some(staged) => staged.finish(home, keys, store, entropy)?,
            None => None,
        };
        let mut node = Self::find_in(keys, store)?;
        node.change = change;

        Ok(node)
    }

    /// The node whose device key `home`'s key store holds, by the device
    /// seat whose log holds that key's inception. It writes nothing, so
    /// it finds only a node a start made. A seat whose certificate does
    /// not read is passed over: it cannot be the seat the device key
    /// certified, and `parinn verify` reports it.
    ///
    /// # Errors
    ///
    /// Fails when the device key cannot be read, the store fails, or
    /// the store holds no inception of the device key.
    pub fn find(home: &Home, store: &Store) -> Result<Self, AdminError> {
        Self::find_in(home.key_store().as_ref(), store)
    }

    /// [`find`](Self::find), the device key read from `keys`.
    ///
    /// # Errors
    ///
    /// As [`find`](Self::find).
    pub fn find_in(keys: &dyn KeyStore, store: &Store) -> Result<Self, AdminError> {
        let device = DeviceKey::fetch(keys, &Entry::device())?;
        let key = device.public();
        for seat in store.seats()? {
            let Ok(cert) = SeatCertificate::from_value(&seat.certificate) else {
                continue;
            };
            if cert.seat_kind != SeatKind::Device || cert.certifying_key != key {
                continue;
            }
            let first = store.event(Address::new(seat.writer, Seq::FIRST))?;
            let second_seq = Seq::FIRST.next().unwrap_or(Seq::FIRST);
            let second = store.event(Address::new(seat.writer, second_seq))?;
            if holds_inception(first.as_ref(), second.as_ref(), &key) {
                return Ok(Self {
                    device,
                    device_writer: seat.writer,
                    room: seat.room,
                    change: None,
                });
            }
        }

        Err(AdminError::Node(
            "the home holds a device key but the store no device seat it certified",
        ))
    }

    /// The device key.
    #[must_use]
    pub fn device(&self) -> &DeviceKey {
        &self.device
    }

    /// The device seat's writer.
    #[must_use]
    pub fn device_writer(&self) -> WriterId {
        self.device_writer
    }

    /// The personal room.
    #[must_use]
    pub fn personal_room(&self) -> RoomId {
        self.room
    }

    /// The node identity change this start recorded, if it recorded one.
    #[must_use]
    pub fn identity_change(&self) -> Option<IdentityChange> {
        self.change
    }

    /// Starts an ingest seat in the personal room: mints its key, keeps
    /// it in the home's key store, certifies it with the device key, and
    /// records the seat and the certificate record in the device seat in
    /// one transaction.
    ///
    /// # Errors
    ///
    /// Fails when the key cannot be minted or kept, or the store fails.
    pub fn certify_ingest_seat(
        &self,
        home: &Home,
        store: &mut Store,
        entropy: &mut dyn Entropy,
    ) -> Result<StoredSeat, AdminError> {
        let seat = SeatKey::mint(entropy)?;
        let cert = SeatCertificate::issue(&self.device, &seat, self.room, SeatKind::Run, true)?;
        seat.keep(home.key_store().as_ref(), &Entry::seat(cert.seat_id()))?;
        self.record_seat(store, &cert, entropy)
    }

    /// Starts a run seat in the personal room for the key `seat`, which
    /// its caller keeps in memory alone, as a witnessed run's MCP server
    /// does (SEC-10): certifies it with the device key, naming it a run
    /// seat and no ingest seat (PRV-11), and records the seat and the
    /// certificate record in the device seat in one transaction.
    ///
    /// # Errors
    ///
    /// Fails when the random source or the store fails.
    pub fn certify_run_seat(
        &self,
        store: &mut Store,
        seat: &SeatKey,
        entropy: &mut dyn Entropy,
    ) -> Result<StoredSeat, AdminError> {
        let cert = SeatCertificate::issue(&self.device, seat, self.room, SeatKind::Run, false)?;
        self.record_seat(store, &cert, entropy)
    }

    /// Records the seat `cert` certifies, and its certificate record in
    /// the device seat, in one transaction.
    fn record_seat(
        &self,
        store: &mut Store,
        cert: &SeatCertificate,
        entropy: &mut dyn Entropy,
    ) -> Result<StoredSeat, AdminError> {
        let record = seat_record(cert);
        let new_events = [new_event(
            draw(entropy)?,
            EventKind::CertificateRecord,
            Provenance::Structural,
            certificate_record(cert),
        )];
        store.add_seat_and_append(&record, self.device_writer, &new_events)?;

        Ok(record)
    }
}

/// Stages a first start's keys when the key store holds no device key
/// and the store no seat, and a node identity change's when the device
/// key is there and the node identity changed.
fn prepare(
    home: &Home,
    keys: &dyn KeyStore,
    store: &Store,
    entropy: &mut dyn Entropy,
) -> Result<(), AdminError> {
    match keys.get(&Entry::device()) {
        Err(e) if e.is_missing() => {
            if !store.seats()?.is_empty() {
                return Err(AdminError::DeviceKeyMissing);
            }
            Staged::mint(entropy, true)?.stage(home, keys)
        }
        Err(e) => Err(e.into()),
        Ok(_) => match node_identity::changed(home)? {
            Some(_) => Staged::mint(entropy, false)?.stage(home, keys),
            None => Ok(()),
        },
    }
}

/// The device key `parinn verify` checks the record in `store` against:
/// the one `home`'s key store holds, or, while an interrupted first
/// start is staged, the key it staged, which its next start places, as
/// a crash can leave it (ENG-06). The flag says the key is staged.
/// `None` while no start has recorded anything: no key is placed and
/// the store holds no seat, so the next start makes the node.
///
/// # Errors
///
/// Fails when the device key cannot be read, or is missing beside a
/// record with no first start staged, or when the staged start or the
/// store fails.
pub fn verifying_key(home: &Home, store: &Store) -> Result<Option<(PublicKey, bool)>, AdminError> {
    let keys = home.key_store();
    let missing = match DeviceKey::fetch(keys.as_ref(), &Entry::device()) {
        Ok(device) => return Ok(Some((device.public(), false))),
        Err(e) if e.is_missing() => e,
        Err(e) => return Err(e.into()),
    };
    if store.seats()?.is_empty() {
        return Ok(None);
    }
    match Staged::load(home, keys.as_ref())? {
        Some(staged) if staged.nonce.is_some() => Ok(Some((staged.device.public(), true))),
        _ => Err(missing.into()),
    }
}

/// What a start staged: a first start's keys and its personal room's
/// nonce, or a node identity change's keys.
struct Staged {
    /// The personal room's nonce, for a first start alone.
    nonce: Option<[u8; 16]>,
    device: DeviceKey,
    recovery: RecoveryKey,
    seat: SeatKey,
}

/// The staged file's text for a node identity change.
const CHANGE: &str = "change";
/// The staged file's text for a first start, before its nonce.
const GENESIS: &str = "genesis ";

impl Staged {
    /// Fresh keys from `entropy`, with a nonce for a first start.
    fn mint(entropy: &mut dyn Entropy, genesis: bool) -> Result<Self, AdminError> {
        Ok(Self {
            device: DeviceKey::mint(entropy)?,
            recovery: RecoveryKey::mint(entropy)?,
            seat: SeatKey::mint(entropy)?,
            nonce: if genesis { Some(draw(entropy)?) } else { None },
        })
    }

    /// Keeps the keys in their staged entries, clearing what an earlier
    /// start left there, then writes the staged file last.
    fn stage(&self, home: &Home, keys: &dyn KeyStore) -> Result<(), AdminError> {
        for entry in staged_entries() {
            keys.remove(&entry)?;
        }
        self.device.keep(keys, &Entry::staged(&Entry::device()))?;
        self.recovery
            .keep(keys, &Entry::staged(&Entry::recovery()))?;
        self.seat.keep(keys, &Entry::staged_seat())?;
        let text = match self.nonce {
            Some(nonce) => format!("{GENESIS}{}", hex(&nonce)),
            None => CHANGE.to_owned(),
        };
        write_secret(&home.staged(), &text)?;
        sync_directory(&home.staged()).map_err(|source| AdminError::io(&home.staged(), source))
    }

    /// What `home` staged, none when it staged nothing.
    fn load(home: &Home, keys: &dyn KeyStore) -> Result<Option<Self>, AdminError> {
        let path = home.staged();
        let text = match read_secret(&path) {
            Err(KeyError::File { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            text => text?,
        };
        let nonce =
            match text.strip_prefix(GENESIS) {
                Some(nonce) => Some(unhex(nonce).map_err(|_| {
                    AdminError::Node("the staged genesis nonce is not 32 hex digits")
                })?),
                None if text == CHANGE => None,
                None => {
                    return Err(AdminError::Node(
                        "the staged start names neither a genesis nor a change",
                    ));
                }
            };

        Ok(Some(Self {
            nonce,
            device: DeviceKey::fetch(keys, &Entry::staged(&Entry::device()))?,
            recovery: RecoveryKey::fetch(keys, &Entry::staged(&Entry::recovery()))?,
            seat: SeatKey::fetch(keys, &Entry::staged_seat())?,
        }))
    }

    /// Records what was staged, unless an earlier start recorded it,
    /// places the keys, records the node identity the home now starts
    /// under, and clears the staging.
    fn finish(
        &self,
        home: &Home,
        keys: &dyn KeyStore,
        store: &mut Store,
        entropy: &mut dyn Entropy,
    ) -> Result<Option<IdentityChange>, AdminError> {
        let (record, change) = match self.nonce {
            Some(nonce) => (self.record_genesis(store, entropy, nonce)?, None),
            None => {
                let (record, change) = self.record_change(keys, store, entropy)?;
                (record, Some(change))
            }
        };
        self.place(keys, &record)?;
        if *home.identity() != IdentitySource::Unsupported {
            let observed = home.identity().observe();
            node_identity::record(home, node_identity::digest(observed.as_deref()))?;
        }
        for entry in staged_entries() {
            keys.remove(&entry)?;
        }
        let path = home.staged();
        fs::remove_file(&path).map_err(|source| AdminError::io(&path, source))?;
        sync_directory(&path).map_err(|source| AdminError::io(&path, source))?;

        Ok(change)
    }

    /// The device seat's certificate in `room`. Its signatures are
    /// deterministic, so every start that resumes issues the same one.
    fn certificate(&self, room: RoomId) -> Result<SeatCertificate, AdminError> {
        SeatCertificate::issue(&self.device, &self.seat, room, SeatKind::Device, false)
            .map_err(AdminError::from)
    }

    /// The writer of the device seat it starts, whose id derives from the
    /// seat key alone.
    fn writer(&self) -> WriterId {
        WriterId::derive(self.seat.public().as_bytes())
    }

    /// Records a first start's device seat and its first three events in
    /// one transaction, unless an earlier start recorded them.
    fn record_genesis(
        &self,
        store: &mut Store,
        entropy: &mut dyn Entropy,
        nonce: [u8; 16],
    ) -> Result<StoredSeat, AdminError> {
        let room = RoomId::derive(self.seat.public().as_bytes(), &nonce);
        let cert = self.certificate(room)?;
        let record = seat_record(&cert);
        if store.seat(record.writer)?.is_some() {
            return Ok(record);
        }
        let minted = KeyEvent::minted(&self.device, &self.recovery.public())?;
        let room_act = json!({"room": room.to_string(), "nonce": hex(&nonce)});
        let new_events = [
            new_event(
                draw(entropy)?,
                EventKind::KeyEvent,
                Provenance::Structural,
                minted.to_value(),
            ),
            new_event(
                draw(entropy)?,
                EventKind::CreateRoom,
                Provenance::Operator,
                room_act,
            ),
            new_event(
                draw(entropy)?,
                EventKind::CertificateRecord,
                Provenance::Structural,
                certificate_record(&cert),
            ),
        ];
        store.add_seat_and_append(&record, record.writer, &new_events)?;

        Ok(record)
    }

    /// Records a node identity change's device seat in the personal room
    /// and its first three events in one transaction, unless an earlier
    /// start recorded them: the node identity change event, the new
    /// key's `minted` and the new seat's certificate record.
    fn record_change(
        &self,
        keys: &dyn KeyStore,
        store: &mut Store,
        entropy: &mut dyn Entropy,
    ) -> Result<(StoredSeat, IdentityChange), AdminError> {
        if let Some(record) = store.seat(self.writer())? {
            let first = store.event(Address::new(record.writer, Seq::FIRST))?;
            let event = first
                .as_ref()
                .and_then(|s| s.content.as_ref())
                .and_then(|c| NodeIdentityChange::from_value(c).ok())
                .ok_or(AdminError::Node(
                    "the staged change's device seat holds no node identity change event",
                ))?;
            return Ok((record, change_of(&event)));
        }
        let old = Node::find_in(keys, store)?;
        let head = store
            .log(old.device_writer)?
            .last()
            .map(|s| SeenHead::new(s.link.header.address, s.link.hash))
            .ok_or(AdminError::Node(
                "the device seat the change leaves holds no event",
            ))?;
        let event = NodeIdentityChange {
            replaces: old.device.public().fingerprint(),
            key: self.device.public().fingerprint(),
            chain_head: head,
        };
        let cert = self.certificate(old.room)?;
        let record = seat_record(&cert);
        let minted = KeyEvent::minted(&self.device, &self.recovery.public())?;
        let new_events = [
            new_event(
                draw(entropy)?,
                EventKind::NodeIdentityChange,
                Provenance::Structural,
                event.to_value(),
            ),
            new_event(
                draw(entropy)?,
                EventKind::KeyEvent,
                Provenance::Structural,
                minted.to_value(),
            ),
            new_event(
                draw(entropy)?,
                EventKind::CertificateRecord,
                Provenance::Structural,
                certificate_record(&cert),
            ),
        ];
        store.add_seat_and_append(&record, record.writer, &new_events)?;

        Ok((record, change_of(&event)))
    }

    /// Places the keys in their entries, the device key last: the seat's
    /// unless it is there, and the recovery and device keys in place of
    /// any other key there.
    fn place(&self, keys: &dyn KeyStore, record: &StoredSeat) -> Result<(), AdminError> {
        let seat = Entry::seat(record.seat);
        match keys.get(&seat) {
            Err(e) if e.is_missing() => self.seat.keep(keys, &seat)?,
            Err(e) => return Err(e.into()),
            Ok(_) => {}
        }
        let recovery = Entry::recovery();
        if RecoveryKey::fetch(keys, &recovery).map(|k| k.public()).ok()
            != Some(self.recovery.public())
        {
            replace(keys, &recovery, |e| self.recovery.keep(keys, e))?;
        }
        let device = Entry::device();
        if DeviceKey::fetch(keys, &device).map(|k| k.public()).ok() != Some(self.device.public()) {
            replace(keys, &device, |e| self.device.keep(keys, e))?;
        }

        Ok(())
    }
}

/// Empties `entry` and keeps a key there through `keep`.
fn replace(
    keys: &dyn KeyStore,
    entry: &Entry,
    keep: impl FnOnce(&Entry) -> Result<(), KeyError>,
) -> Result<(), AdminError> {
    keys.remove(entry)?;

    Ok(keep(entry)?)
}

/// The change a node identity change event records.
fn change_of(event: &NodeIdentityChange) -> IdentityChange {
    IdentityChange {
        replaced: event.replaces,
        key: event.key,
    }
}

/// The staged entries a start keeps its new keys in.
fn staged_entries() -> [Entry; 3] {
    [
        Entry::staged(&Entry::device()),
        Entry::staged(&Entry::recovery()),
        Entry::staged_seat(),
    ]
}

/// Whether `first` and `second`, a device seat's first two events, hold
/// the inception of `key`: its `minted` first, as the genesis log's, or
/// a node identity change event naming it, then its `minted`.
fn holds_inception(first: Option<&Stored>, second: Option<&Stored>, key: &PublicKey) -> bool {
    if begins_genesis(first, key) {
        return true;
    }
    let names_key = first
        .filter(|s| s.link.header.kind == EventKind::NodeIdentityChange)
        .and_then(|s| s.content.as_ref())
        .and_then(|c| NodeIdentityChange::from_value(c).ok())
        .is_some_and(|e| e.key == key.fingerprint());

    names_key && begins_genesis(second, key)
}

/// Whether `event` is the `minted` key event of `key`.
pub(crate) fn begins_genesis(event: Option<&Stored>, key: &PublicKey) -> bool {
    event
        .filter(|s| s.link.header.kind == EventKind::KeyEvent)
        .and_then(|s| s.content.as_ref())
        .and_then(|c| KeyEvent::from_value(c).ok())
        .is_some_and(|e| e.check_minted(key).is_ok())
}

/// The store's record of the seat `cert` certifies.
fn seat_record(cert: &SeatCertificate) -> StoredSeat {
    StoredSeat {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    }
}

/// A certificate record's content: the certificate's digest and the
/// seat and writer it names.
fn certificate_record(cert: &SeatCertificate) -> Value {
    json!({
        "certificate": cert.digest().to_hex(),
        "seat": cert.seat_id().to_string(),
        "writer": cert.writer_id().to_string(),
    })
}

/// A new event of `content`, under the commitment key `key`.
fn new_event(key: [u8; 32], kind: EventKind, provenance: Provenance, content: Value) -> NewEvent {
    NewEvent {
        kind,
        provenance,
        content,
        commitment_key: CommitmentKey::from_bytes(key),
        transcript_timestamp: None,
    }
}

#[cfg(test)]
mod tests;
