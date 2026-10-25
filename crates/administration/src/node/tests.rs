use std::fs;

use super::*;
use key::key_store::FileKeyStore;
use key::random::OsEntropy;
use record::event::{EventKind, Provenance};
use record::id::SeatId;
use testkit::TempDir;

/// A home in `dir`, its node identity read from `dir/machine-id`, which
/// holds "first".
fn home(dir: &TempDir) -> Home {
    fs::write(dir.path().join("machine-id"), "first\n").unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![dir.path().join("machine-id")]));
    home.create().unwrap();
    home
}

/// The home's key store, as a file store this test can look into.
fn files(home: &Home) -> FileKeyStore {
    FileKeyStore::new(home.keys())
}

fn kinds(log: &[Stored]) -> Vec<(EventKind, Provenance)> {
    log.iter()
        .map(|s| (s.link.header.kind, s.link.header.provenance.clone()))
        .collect()
}

#[test]
fn the_first_start_mints_the_device_key_and_records_the_personal_room() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();

    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    let log = store.log(node.device_writer()).unwrap();
    assert_eq!(
        kinds(&log),
        [
            (EventKind::KeyEvent, Provenance::Structural),
            (EventKind::CreateRoom, Provenance::Operator),
            (EventKind::CertificateRecord, Provenance::Structural),
        ]
    );
    let keys = files(&home);
    let device = DeviceKey::fetch(&keys, &Entry::device()).unwrap();
    let minted = KeyEvent::from_value(log[0].content.as_ref().unwrap()).unwrap();
    assert!(minted.check_minted(&device.public()).is_ok());
    let seat = store.seat(node.device_writer()).unwrap().unwrap();
    let cert = SeatCertificate::from_value(&seat.certificate).unwrap();
    assert_eq!(
        (cert.seat_kind, cert.ingest_seat, cert.room),
        (SeatKind::Device, false, node.personal_room())
    );
    assert_eq!(
        log[1].content.as_ref().unwrap()["room"],
        node.personal_room().to_string()
    );
    assert_eq!(
        log[2].content.as_ref().unwrap()["certificate"],
        cert.digest().to_hex()
    );
    assert!(RecoveryKey::fetch(&keys, &Entry::recovery()).is_ok());
    assert!(SeatKey::fetch(&keys, &Entry::seat(seat.seat)).is_ok());
    assert_eq!(node.identity_change(), None);
    assert!(!home.staged().exists());
    assert_eq!(
        fs::read_to_string(home.node_identity()).unwrap(),
        node_identity::digest(Some("first")).to_hex()
    );
}

#[test]
fn no_key_reaches_the_store() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    node.certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();
    drop(store);

    let bytes = fs::read(home.store()).unwrap();
    let keys = fs::read_dir(home.keys()).unwrap();
    for entry in keys {
        let secret = fs::read_to_string(entry.unwrap().path()).unwrap();
        let found = bytes.windows(secret.len()).any(|w| w == secret.as_bytes());
        assert!(!found, "a key's seed is in the store");
    }
}

#[test]
fn the_room_id_derives_from_the_device_seat_key_and_the_recorded_nonce() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    let log = store.log(node.device_writer()).unwrap();
    let seat = store.seat(node.device_writer()).unwrap().unwrap();
    let cert = SeatCertificate::from_value(&seat.certificate).unwrap();
    let nonce = log[1].content.as_ref().unwrap()["nonce"]
        .as_str()
        .unwrap()
        .to_owned();

    let bytes: [u8; 16] = unhex(&nonce).unwrap();
    assert_eq!(
        RoomId::derive(cert.seat_key.as_bytes(), &bytes),
        node.personal_room()
    );
}

#[test]
fn a_later_start_finds_the_same_node() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let first = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    let again = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert_eq!(
        (again.device_writer(), again.personal_room()),
        (first.device_writer(), first.personal_room())
    );
    assert_eq!(again.device().public(), first.device().public());
    assert_eq!(again.identity_change(), None);
    assert_eq!(store.log(first.device_writer()).unwrap().len(), 3);
}

#[test]
fn a_device_key_without_its_device_seat_is_refused() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    fs::remove_file(home.store()).unwrap();
    let mut empty = Store::open(&home.store()).unwrap();
    // A seat the device key certified, but no device seat.
    let device = DeviceKey::fetch(&files(&home), &Entry::device()).unwrap();
    let seat = SeatKey::mint(&mut OsEntropy).unwrap();
    let room = RoomId::derive(&[1; 32], &[1; 16]);
    let cert = SeatCertificate::issue(&device, &seat, room, SeatKind::Run, true).unwrap();
    empty.add_seat(&seat_record(&cert)).unwrap();

    let err = Node::open(&home, &mut empty, &mut OsEntropy).unwrap_err();

    assert_eq!(
        err.to_string(),
        "the home holds a device key but the store no device seat it certified"
    );
}

#[test]
fn a_device_key_missing_beside_a_record_starts_no_second_genesis() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    files(&home).remove(&Entry::device()).unwrap();

    let err = Node::open(&home, &mut store, &mut OsEntropy).unwrap_err();

    assert!(matches!(err, AdminError::DeviceKeyMissing), "{err}");
    assert_eq!(
        err.to_string(),
        "the device key is missing from the key store, but the store holds this node's record; no second genesis starts"
    );
    assert_eq!(store.seats().unwrap().len(), 1);
    assert_eq!(store.log(node.device_writer()).unwrap().len(), 3);
}

#[test]
fn a_device_key_that_cannot_be_read_is_not_missing_and_records_nothing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let path = files(&home).path(&Entry::device());
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();

    let err = Node::open(&home, &mut store, &mut OsEntropy).unwrap_err();

    match &err {
        AdminError::Key(e) => assert!(!e.is_missing(), "{e}"),
        other => panic!("want a key error, got {other}"),
    }
    assert_eq!(store.seats().unwrap().len(), 1);
}

#[test]
fn certify_starts_an_ingest_seat_in_the_personal_room_and_records_its_certificate() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    let seat = node
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();

    let cert = SeatCertificate::from_value(&seat.certificate).unwrap();
    assert!(cert.check().is_ok());
    assert_eq!((cert.seat_kind, cert.ingest_seat), (SeatKind::Run, true));
    assert_eq!(
        (seat.room, cert.certifying_key),
        (node.personal_room(), node.device().public())
    );
    assert_eq!(store.seat(seat.writer).unwrap(), Some(seat.clone()));
    assert!(SeatKey::fetch(&files(&home), &Entry::seat(seat.seat)).is_ok());
    let log = store.log(node.device_writer()).unwrap();
    assert_eq!(log.len(), 4);
    assert_eq!(log[3].link.header.kind, EventKind::CertificateRecord);
    assert_eq!(
        log[3].content.as_ref().unwrap()["certificate"],
        cert.digest().to_hex()
    );
    assert_eq!(
        log[3].content.as_ref().unwrap()["writer"],
        seat.writer.to_string()
    );
}

#[test]
fn a_run_seat_is_certified_as_a_run_seat_and_no_ingest_seat_and_its_key_is_not_kept() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();

    let seat = node
        .certify_run_seat(&mut store, &key, &mut OsEntropy)
        .unwrap();

    let cert = SeatCertificate::from_value(&seat.certificate).unwrap();
    assert!(cert.check().is_ok());
    assert_eq!(
        (cert.seat_kind, cert.ingest_seat, cert.seat_key),
        (SeatKind::Run, false, key.public())
    );
    assert_eq!(
        seat.seat,
        SeatId::derive(&node.personal_room(), key.public().as_bytes())
    );
    assert!(
        SeatKey::fetch(&files(&home), &Entry::seat(seat.seat))
            .unwrap_err()
            .is_missing()
    );
    let log = store.log(node.device_writer()).unwrap();
    assert_eq!(
        log[3].content.as_ref().unwrap()["writer"],
        seat.writer.to_string()
    );
    assert!(crate::verify::verify(&store, &node.device().public()).is_ok());
}

#[test]
fn a_failure_on_the_way_is_returned() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    fs::write(files(&home).path(&Entry::device()), "x").unwrap();

    assert!(matches!(
        Node::open(&home, &mut store, &mut OsEntropy),
        Err(AdminError::Key(_))
    ));
    let other = TempDir::new().unwrap();
    let fresh = self::home(&other);
    assert!(matches!(
        Node::open(&fresh, &mut store, &mut Failing),
        Err(AdminError::Key(_))
    ));
}

/// A random source that always fails.
struct Failing;

impl key::random::Entropy for Failing {
    fn fill(&mut self, _: &mut [u8]) -> Result<(), key::KeyError> {
        Err(key::KeyError::Entropy("none".into()))
    }
}

#[test]
fn a_store_that_fails_mid_start_is_returned() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    // A seat certificate stored by hand under an unparseable value
    // makes the device seat unreadable, so no device seat is found.
    let conn = rusqlite::Connection::open(home.store()).unwrap();
    conn.execute("UPDATE seats SET certificate = '{}'", [])
        .unwrap();

    assert!(matches!(
        Node::open(&home, &mut store, &mut OsEntropy),
        Err(AdminError::Node(_))
    ));
    conn.execute("UPDATE seats SET certificate = '{'", [])
        .unwrap();
    assert!(matches!(
        Node::open(&home, &mut store, &mut OsEntropy),
        Err(AdminError::Store(_))
    ));
    assert!(matches!(
        node.certify_ingest_seat(&home, &mut store, &mut Failing),
        Err(AdminError::Key(_))
    ));
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    assert!(matches!(
        node.certify_run_seat(&mut store, &key, &mut Failing),
        Err(AdminError::Key(_))
    ));
}

#[test]
fn a_first_start_interrupted_after_staging_completes_on_the_next_start() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let keys = files(&home);
    let mut store = Store::open(&home.store()).unwrap();
    Staged::mint(&mut OsEntropy, true)
        .unwrap()
        .stage(&home, &keys)
        .unwrap();
    assert!(keys.get(&Entry::device()).unwrap_err().is_missing());

    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert!(keys.get(&Entry::device()).is_ok());
    assert!(!home.staged().exists());
    for entry in staged_entries() {
        assert!(keys.get(&entry).unwrap_err().is_missing());
    }
    assert_eq!(store.log(node.device_writer()).unwrap().len(), 3);
    assert!(crate::verify::verify(&store, &node.device().public()).is_ok());
}

#[test]
fn a_first_start_interrupted_after_recording_places_its_keys_on_the_next_start() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let keys = files(&home);
    let mut store = Store::open(&home.store()).unwrap();
    let staged = Staged::mint(&mut OsEntropy, true).unwrap();
    staged.stage(&home, &keys).unwrap();
    let record = staged
        .record_genesis(&mut store, &mut OsEntropy, staged.nonce.unwrap())
        .unwrap();
    // A crash after the seat key was placed, before the others.
    staged.seat.keep(&keys, &Entry::seat(record.seat)).unwrap();

    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert_eq!(node.device().public(), staged.device.public());
    assert_eq!(store.log(node.device_writer()).unwrap().len(), 3);
    assert!(!home.staged().exists());
    assert!(crate::verify::verify(&store, &node.device().public()).is_ok());
}

#[test]
fn a_staged_start_that_does_not_read_is_refused() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let keys = files(&home);
    let mut store = Store::open(&home.store()).unwrap();
    Staged::mint(&mut OsEntropy, true)
        .unwrap()
        .stage(&home, &keys)
        .unwrap();
    fs::remove_file(home.staged()).unwrap();
    write_secret(&home.staged(), "genesis zz").unwrap();

    let err = Node::open(&home, &mut store, &mut OsEntropy).unwrap_err();
    assert_eq!(
        err.to_string(),
        "the staged genesis nonce is not 32 hex digits"
    );

    fs::remove_file(home.staged()).unwrap();
    write_secret(&home.staged(), "neither").unwrap();
    let err = Node::open(&home, &mut store, &mut OsEntropy).unwrap_err();
    assert_eq!(
        err.to_string(),
        "the staged start names neither a genesis nor a change"
    );

    fs::remove_file(home.staged()).unwrap();
    fs::create_dir(home.staged()).unwrap();
    assert!(matches!(
        Node::open(&home, &mut store, &mut OsEntropy),
        Err(AdminError::Key(_))
    ));
}

#[test]
fn a_first_start_whose_random_source_fails_records_nothing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();

    let err = Node::open(&home, &mut store, &mut Failing).unwrap_err();

    assert!(
        matches!(err, AdminError::Key(key::KeyError::Entropy(_))),
        "{err}"
    );
    assert!(store.seats().unwrap().is_empty());
}

#[test]
fn the_personal_room_is_the_one_whose_device_seat_holds_the_genesis() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    // Device seats the device key certified in other rooms, as joins
    // will add, with no genesis of their own; the store lists seats by
    // writer, so at least one must come before the genesis. A genesis
    // writer is random and may sort low, so seats are added until one
    // sorts before it, never a fixed few.
    let mut before = 0;
    let mut n = 0u32;
    while before == 0 || n < 5 {
        let seat = SeatKey::mint(&mut OsEntropy).unwrap();
        let mut tag = [0u8; 16];
        tag[..4].copy_from_slice(&n.to_le_bytes());
        let room = RoomId::derive(&[0; 32], &tag);
        let cert =
            SeatCertificate::issue(node.device(), &seat, room, SeatKind::Device, false).unwrap();
        before += usize::from(cert.writer_id() < node.device_writer());
        store.add_seat(&seat_record(&cert)).unwrap();
        n += 1;
    }

    let again = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert_eq!(
        (again.device_writer(), again.personal_room()),
        (node.device_writer(), node.personal_room())
    );
}

#[test]
fn find_reads_the_node_a_start_made_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    assert!(matches!(Node::find(&home, &store), Err(AdminError::Key(_))));
    assert!(!home.staged().exists());
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    let found = Node::find(&home, &store).unwrap();

    assert_eq!(
        (found.device_writer(), found.personal_room()),
        (node.device_writer(), node.personal_room())
    );
}

#[test]
fn find_passes_over_a_seat_whose_certificate_does_not_read() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    // It sorts before every other writer, so find meets it first.
    let first = WriterId::parse(&"0".repeat(64)).unwrap();
    let device = store.seat(node.device_writer()).unwrap().unwrap();
    store
        .add_seat(&StoredSeat {
            writer: first,
            seat: SeatId::derive(&device.room, &[9; 32]),
            certificate: serde_json::json!({}),
            ..device
        })
        .unwrap();

    let found = Node::find(&home, &store).unwrap();

    assert_eq!(found.device_writer(), node.device_writer());
}

#[test]
fn a_leftover_staged_entry_is_cleared_before_a_start_stages_again() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let keys = files(&home);
    let mut store = Store::open(&home.store()).unwrap();
    // A crash after staging a key, before the staged file was written.
    DeviceKey::mint(&mut OsEntropy)
        .unwrap()
        .keep(&keys, &Entry::staged(&Entry::device()))
        .unwrap();

    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert!(crate::verify::verify(&store, &node.device().public()).is_ok());
    let path = keys.path(&Entry::staged(&Entry::device()));
    fs::create_dir_all(path.join("x")).unwrap();
    let staged = Staged::mint(&mut OsEntropy, true).unwrap();
    assert!(matches!(
        staged.stage(&home, &keys),
        Err(AdminError::Key(_))
    ));
}

/// The length of each of `writers`' logs.
fn lengths(store: &Store, writers: &[WriterId]) -> Vec<usize> {
    writers
        .iter()
        .map(|w| store.log(*w).unwrap().len())
        .collect()
}

/// Checks that `node`'s device seat begins with the node identity change
/// leaving `old`'s device seat at its head, then the new key's `minted`,
/// which it returns, then its certificate record.
fn holds_the_change(store: &Store, old: &Node, node: &Node) -> KeyEvent {
    let log = store.log(node.device_writer()).unwrap();
    assert_eq!(
        kinds(&log),
        [
            (EventKind::NodeIdentityChange, Provenance::Structural),
            (EventKind::KeyEvent, Provenance::Structural),
            (EventKind::CertificateRecord, Provenance::Structural),
        ]
    );
    let change = NodeIdentityChange::from_value(log[0].content.as_ref().unwrap()).unwrap();
    let old_head = store
        .log(old.device_writer())
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(
        change.chain_head,
        SeenHead::new(old_head.link.header.address, old_head.link.hash)
    );
    let minted = KeyEvent::from_value(log[1].content.as_ref().unwrap()).unwrap();
    assert!(minted.check_minted(&node.device().public()).is_ok());
    minted
}

/// The scenario's change: the node identity file now holds "second".
fn move_machine(dir: &TempDir) {
    fs::write(dir.path().join("machine-id"), "second\n").unwrap();
}

#[test]
fn a_changed_node_identity_mints_new_keys_and_a_new_device_seat_in_the_personal_room() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let keys = files(&home);
    let mut store = Store::open(&home.store()).unwrap();
    let old = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let ingest = old
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();
    let old_logs = lengths(&store, &[old.device_writer(), ingest.writer]);
    let old_recovery = RecoveryKey::fetch(&keys, &Entry::recovery()).unwrap();
    move_machine(&dir);

    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert_ne!(node.device().public(), old.device().public());
    assert_ne!(node.device_writer(), old.device_writer());
    assert_eq!(node.personal_room(), old.personal_room());
    assert_eq!(
        node.identity_change(),
        Some(IdentityChange {
            replaced: old.device().public().fingerprint(),
            key: node.device().public().fingerprint(),
        })
    );
    let minted = holds_the_change(&store, &old, &node);
    let new_recovery = RecoveryKey::fetch(&keys, &Entry::recovery()).unwrap();
    assert_ne!(new_recovery.public(), old_recovery.public());
    assert_eq!(minted.recovery, new_recovery.public().digest());
    // Nothing is appended under an old key.
    assert_eq!(
        lengths(&store, &[old.device_writer(), ingest.writer]),
        old_logs
    );
    assert!(!home.staged().exists());
    assert_eq!(
        fs::read_to_string(home.node_identity()).unwrap(),
        node_identity::digest(Some("second")).to_hex()
    );
    assert!(crate::verify::verify(&store, &node.device().public()).is_ok());

    let again = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    assert_eq!(again.identity_change(), None);
    assert_eq!(again.device_writer(), node.device_writer());
}

#[test]
fn an_unavailable_node_identity_counts_as_a_change_once() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let old = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    fs::remove_file(dir.path().join("machine-id")).unwrap();

    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let again = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert!(node.identity_change().is_some());
    assert_ne!(node.device_writer(), old.device_writer());
    assert_eq!(again.identity_change(), None);
    assert_eq!(again.device_writer(), node.device_writer());
}

#[test]
fn a_change_interrupted_after_recording_finishes_with_one_node_identity_change_event() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let keys = files(&home);
    let mut store = Store::open(&home.store()).unwrap();
    let old = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    move_machine(&dir);
    let staged = Staged::mint(&mut OsEntropy, false).unwrap();
    staged.stage(&home, &keys).unwrap();
    staged
        .record_change(&keys, &mut store, &mut OsEntropy)
        .unwrap();
    // A crash after the device key was removed, before the new one was
    // kept.
    keys.remove(&Entry::device()).unwrap();

    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert_eq!(node.device().public(), staged.device.public());
    assert_eq!(
        node.identity_change().map(|c| c.replaced),
        Some(old.device().public().fingerprint())
    );
    let changes = store
        .seats()
        .unwrap()
        .iter()
        .filter(|s| {
            store
                .log(s.writer)
                .unwrap()
                .first()
                .is_some_and(|e| e.link.header.kind == EventKind::NodeIdentityChange)
        })
        .count();
    assert_eq!(changes, 1);
    assert!(crate::verify::verify(&store, &node.device().public()).is_ok());
}

#[test]
fn a_staged_change_whose_device_seat_holds_no_change_is_refused() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let keys = files(&home);
    let mut store = Store::open(&home.store()).unwrap();
    let old = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let staged = Staged::mint(&mut OsEntropy, false).unwrap();
    staged.stage(&home, &keys).unwrap();
    // A device seat of the staged seat key that begins otherwise.
    let cert = staged.certificate(old.personal_room()).unwrap();
    store.add_seat(&seat_record(&cert)).unwrap();

    let err = Node::open(&home, &mut store, &mut OsEntropy).unwrap_err();

    assert_eq!(
        err.to_string(),
        "the staged change's device seat holds no node identity change event"
    );
}

#[test]
fn an_unsupported_node_identity_is_never_found_changed() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir).with_identity(IdentitySource::Unsupported);
    let mut store = Store::open(&home.store()).unwrap();
    let old = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert_eq!(node.device_writer(), old.device_writer());
    assert!(!home.node_identity().exists());
}

#[test]
fn a_start_interrupted_after_placing_every_key_clears_its_staging() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let keys = files(&home);
    let mut store = Store::open(&home.store()).unwrap();
    let staged = Staged::mint(&mut OsEntropy, true).unwrap();
    staged.stage(&home, &keys).unwrap();
    let record = staged
        .record_genesis(&mut store, &mut OsEntropy, staged.nonce.unwrap())
        .unwrap();
    staged.place(&keys, &record).unwrap();

    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert_eq!(node.device().public(), staged.device.public());
    assert!(!home.staged().exists());
}

#[test]
fn a_seat_entry_that_cannot_be_read_stops_the_placing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let keys = files(&home);
    let mut store = Store::open(&home.store()).unwrap();
    let staged = Staged::mint(&mut OsEntropy, true).unwrap();
    staged.stage(&home, &keys).unwrap();
    let record = staged
        .record_genesis(&mut store, &mut OsEntropy, staged.nonce.unwrap())
        .unwrap();
    fs::create_dir(keys.path(&Entry::seat(record.seat))).unwrap();

    let err = Node::open(&home, &mut store, &mut OsEntropy).unwrap_err();

    match err {
        AdminError::Key(e) => assert!(e.is_unavailable(), "{e}"),
        other => panic!("want a key error, got {other}"),
    }
}

#[test]
fn the_verifying_key_is_the_device_key_once_a_start_finished() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();

    assert_eq!(
        verifying_key(&home, &store).unwrap(),
        Some((node.device().public(), false))
    );
}

#[test]
fn a_first_start_interrupted_before_it_recorded_has_no_verifying_key_yet() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let store = Store::open(&home.store()).unwrap();
    Staged::mint(&mut OsEntropy, true)
        .unwrap()
        .stage(&home, &files(&home))
        .unwrap();

    assert_eq!(verifying_key(&home, &store).unwrap(), None);
}

#[test]
fn a_first_start_interrupted_after_it_recorded_is_verified_against_its_staged_key() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let keys = files(&home);
    let mut store = Store::open(&home.store()).unwrap();
    let staged = Staged::mint(&mut OsEntropy, true).unwrap();
    staged.stage(&home, &keys).unwrap();
    staged
        .record_genesis(&mut store, &mut OsEntropy, staged.nonce.unwrap())
        .unwrap();

    let (key, is_staged) = verifying_key(&home, &store).unwrap().unwrap();

    assert_eq!((key, is_staged), (staged.device.public(), true));
    assert!(crate::verify::verify(&store, &key).is_ok());
}

#[test]
fn a_missing_device_key_beside_a_record_with_no_first_start_staged_has_no_verifying_key() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    assert_eq!(
        verifying_key(&home, &store).unwrap(),
        None,
        "nothing recorded"
    );
    Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    fs::remove_file(files(&home).path(&Entry::device())).unwrap();

    let err = verifying_key(&home, &store).unwrap_err();
    assert!(
        matches!(&err, AdminError::Key(e) if e.is_missing()),
        "{err}"
    );

    Staged::mint(&mut OsEntropy, false)
        .unwrap()
        .stage(&home, &files(&home))
        .unwrap();
    assert!(matches!(
        verifying_key(&home, &store),
        Err(AdminError::Key(_))
    ));
}

#[cfg(unix)]
#[test]
fn a_device_key_that_cannot_be_read_has_no_verifying_key() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let file = files(&home).path(&Entry::device());
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();

    let err = verifying_key(&home, &store).unwrap_err();

    assert!(
        matches!(&err, AdminError::Key(e) if e.is_unavailable()),
        "{err}"
    );
}
