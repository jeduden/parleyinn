use std::fs;

use super::*;
use crate::home::Home;
use crate::node::Node;
use crate::node_identity::IdentitySource;
use key::random::OsEntropy;
use key::signing::SeatKey;
use record::id::RoomId;
use testkit::TempDir;

fn home(dir: &TempDir) -> Home {
    fs::write(dir.path().join("machine-id"), "first\n").unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![dir.path().join("machine-id")]));
    home.create().unwrap();
    home
}

fn origin(store: &Store, node: &Node, writer: WriterId) -> Origin {
    derive(store, &node.device().public()).unwrap()[&writer]
}

#[test]
fn the_device_seat_and_a_run_seat_a_key_of_this_node_certified_are_witnessed() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    let run = node
        .certify_run_seat(&mut store, &key, &mut OsEntropy)
        .unwrap();

    assert_eq!(
        origin(&store, &node, node.device_writer()),
        Origin::Witnessed
    );
    assert_eq!(origin(&store, &node, run.writer), Origin::Witnessed);
}

#[test]
fn an_ingest_seat_reads_ingested() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let ingest = node
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();

    assert_eq!(origin(&store, &node, ingest.writer), Origin::Ingested);
}

#[test]
fn a_seat_whose_certificate_record_the_device_seat_lacks_reads_peer() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    let cert = SeatCertificate::issue(
        node.device(),
        &key,
        node.personal_room(),
        SeatKind::Run,
        false,
    )
    .unwrap();
    let seat = StoredSeat {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    };
    store.add_seat(&seat).unwrap();

    assert_eq!(origin(&store, &node, seat.writer), Origin::Peer);
}

#[test]
fn a_seat_another_key_certified_reads_peer() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let other = key::signing::DeviceKey::mint(&mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    let cert =
        SeatCertificate::issue(&other, &key, node.personal_room(), SeatKind::Run, false).unwrap();
    let seat = StoredSeat {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    };
    store.add_seat(&seat).unwrap();

    assert_eq!(origin(&store, &node, seat.writer), Origin::Peer);
}

#[test]
fn a_seat_whose_certificate_does_not_read_or_names_another_writer_reads_peer() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let mut seat = store.seat(node.device_writer()).unwrap().unwrap();
    let keys = KeySet::load(&store, &node.device().public()).unwrap();
    assert_eq!(keys.origin_of(&seat), Origin::Witnessed);

    seat.room = RoomId::derive(&[9; 32], &[9; 16]);
    assert_eq!(keys.origin_of(&seat), Origin::Peer);
    seat.certificate = serde_json::json!({"unrelated": true});
    assert_eq!(keys.origin_of(&seat), Origin::Peer);
}

#[test]
fn after_a_node_identity_change_no_writer_reads_witnessed_or_ingested() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let old = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let ingest = old
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();
    fs::write(dir.path().join("machine-id"), "second\n").unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    let run = node
        .certify_run_seat(&mut store, &key, &mut OsEntropy)
        .unwrap();

    let origins = derive(&store, &node.device().public()).unwrap();

    assert_eq!(origins.len(), 4);
    assert!(origins.values().all(|o| *o == Origin::Peer));
    assert_eq!(origins[&old.device_writer()], Origin::Peer);
    assert_eq!(origins[&ingest.writer], Origin::Peer);
    assert_eq!(origins[&run.writer], Origin::Peer);
}

#[test]
fn the_origins_names_are_fixed() {
    assert_eq!(
        [
            Origin::Witnessed,
            Origin::Ingested,
            Origin::Bundle,
            Origin::Peer
        ]
        .map(Origin::as_str),
        ["witnessed", "ingested", "bundle", "peer"]
    );
}

#[test]
fn a_second_genesis_log_in_the_store_is_no_key_of_this_node() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    // Another node's whole genesis, as a later plan may store it.
    let other_dir = TempDir::new().unwrap();
    let other_home =
        crate::home::Home::locate(Some(other_dir.path().join(".parinn").as_os_str()), None)
            .unwrap();
    other_home.create().unwrap();
    let mut other_store = Store::open(&other_home.store()).unwrap();
    let other = Node::open(&other_home, &mut other_store, &mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    let foreign = other
        .certify_run_seat(&mut other_store, &key, &mut OsEntropy)
        .unwrap();
    for seat in other_store.seats().unwrap() {
        store.add_seat(&seat).unwrap();
        for stored in other_store.log(seat.writer).unwrap() {
            let event = store::NewEvent {
                kind: stored.link.header.kind,
                provenance: stored.link.header.provenance.clone(),
                content: stored.content.clone().unwrap(),
                commitment_key: stored.commitment_key.clone().unwrap(),
                transcript_timestamp: None,
            };
            store.append(seat.writer, &[event]).unwrap();
        }
    }

    let keys = KeySet::load(&store, &node.device().public()).unwrap();
    let foreign_seat = store.seat(foreign.writer).unwrap().unwrap();

    assert_eq!(keys.origin_of(&foreign_seat), Origin::Peer);
}

#[test]
fn a_device_seat_is_told_from_a_run_seat_by_its_certificate() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let key = SeatKey::mint(&mut OsEntropy).unwrap();
    let run = node
        .certify_run_seat(&mut store, &key, &mut OsEntropy)
        .unwrap();

    assert!(is_device_seat(
        &store.seat(node.device_writer()).unwrap().unwrap()
    ));
    assert!(!is_device_seat(&run));
}
