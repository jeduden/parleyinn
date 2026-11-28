use super::*;
use crate::home::Home;
use crate::node::Node;
use crate::node_identity::IdentitySource;
use key::key_event::NodeIdentityChange;
use key::random::OsEntropy;
use key::signing::{DeviceKey, RecoveryKey, SeatKey};
use record::commitment::CommitmentKey;
use record::digest::Digest;
use record::event::{EventKind, Provenance};
use record::id::Seq;
use serde_json::json;
use store::{NewEvent, StoredSeat};
use testkit::TempDir;

struct Fixture {
    _dir: TempDir,
    home: Home,
    store: Store,
    node: Node,
    ingest: StoredSeat,
}

fn new_event(n: u8) -> NewEvent {
    NewEvent {
        kind: EventKind::UnparsedLine,
        provenance: Provenance::Unparsed,
        content: json!({"line": n, "text": format!("line {n}")}),
        commitment_key: CommitmentKey::from_bytes([n; 32]),
        transcript_timestamp: None,
    }
}

/// A node with one ingest seat holding three events.
fn fixture() -> Fixture {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("machine-id"), "first").unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![dir.path().join("machine-id")]));
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    let ingest = node
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .unwrap();
    store
        .append(ingest.writer, &[new_event(1), new_event(2), new_event(3)])
        .unwrap();
    Fixture {
        _dir: dir,
        home,
        store,
        node,
        ingest,
    }
}

impl Fixture {
    fn sql(&self, statement: &str) {
        let conn = rusqlite::Connection::open(self.home.store()).unwrap();
        conn.execute_batch(statement).unwrap();
    }

    fn at(&self, seq: u64) -> Address {
        Address::new(self.ingest.writer, Seq::new(seq).unwrap())
    }
}

#[test]
fn an_intact_record_verifies_and_counts_its_writers_and_events() {
    let f = fixture();

    let report = verify(&f.store, &f.node.device().public()).unwrap();

    assert_eq!((report.writers, report.events), (2, 7));
    assert_eq!(report.order.len(), 7);
    let ingest: Vec<u64> = report
        .order
        .iter()
        .filter(|a| a.writer == f.ingest.writer)
        .map(|a| a.seq.get())
        .collect();
    assert_eq!(ingest, [1, 2, 3]);
    let full = report.full_order();
    assert_eq!(full.len(), 7);
    assert!(full.contains(&f.at(3).full(&f.node.personal_room())));
    let bare = Report {
        rooms: BTreeMap::new(),
        ..report
    };
    assert_eq!(bare.full_order()[0], bare.order[0].to_string());
}

#[test]
fn a_changed_header_byte_is_named_by_its_full_address() {
    let f = fixture();
    let stored = f.store.event(f.at(2)).unwrap().unwrap();
    let mut flipped = stored.link.header.commitment.to_string();
    let last = if flipped.ends_with('0') { "1" } else { "0" };
    flipped.replace_range(63.., last);
    f.sql(&format!(
        "UPDATE events SET commitment = '{flipped}' WHERE seq = 2 AND writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "chain broken at {}: the stored hash does not match the event's header",
            f.at(2).full(&f.node.personal_room())
        )
    );
}

#[test]
fn content_that_does_not_open_its_commitment_is_named() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE events SET content = '{{\"line\":3,\"text\":\"other\"}}' WHERE seq = 3 AND writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "the content at {} does not open its commitment",
            f.at(3).full(&f.node.personal_room())
        )
    );
}

#[test]
fn content_removed_with_no_tombstone_is_named() {
    // Purge, which records a tombstone for what it removes, lands with
    // ADM-07; until then removed content is a loss verify reports (I1).
    let f = fixture();
    f.sql(&format!(
        "UPDATE events SET content = NULL, commitment_key = NULL WHERE seq = 3 AND writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "the content at {} was removed, and no tombstone records it",
            f.at(3).full(&f.node.personal_room())
        )
    );
}

#[test]
fn a_commitment_key_without_its_content_does_not_open_its_commitment() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE events SET content = NULL WHERE seq = 3 AND writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert!(
        err.to_string().ends_with("does not open its commitment"),
        "{err}"
    );
}

#[test]
fn a_row_that_does_not_parse_is_named() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE events SET kind = 'x' WHERE seq = 2 AND writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert!(
        err.to_string().starts_with(&format!(
            "chain broken at {}: ",
            f.at(2).full(&f.node.personal_room())
        )),
        "{err}"
    );
}

#[test]
fn a_changed_cited_head_changes_the_hash_of_the_event_citing_it() {
    let f = fixture();
    f.sql("UPDATE event_heads SET other_seq = 1");

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "chain broken at {}: the stored hash does not match the event's header",
            f.at(1).full(&f.node.personal_room())
        )
    );
}

#[test]
fn a_cited_head_naming_no_event_is_named_even_when_the_chain_is_rehashed() {
    let f = fixture();
    // The ingest seat's first event cited the device seat's fourth
    // event; point it at the first, and rehash the ingest seat's chain
    // after it so only the cited head is wrong.
    f.sql("UPDATE event_heads SET other_seq = 1");
    let mut prev = Digest::ZERO;
    for stored in f.store.log(f.ingest.writer).unwrap() {
        let mut header = stored.link.header.clone();
        header.prev_hash = prev;
        prev = header.hash();
        f.sql(&format!(
            "UPDATE events SET prev_hash = '{}', hash = '{prev}' WHERE writer = '{}' AND seq = {}",
            header.prev_hash, f.ingest.writer, header.address.seq
        ));
    }

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert!(
        err.to_string()
            .ends_with("which no event of another writer matches"),
        "{err}"
    );
}

#[test]
fn a_writer_without_a_seat_certificate_is_named() {
    let mut f = fixture();
    let stray = WriterId::derive(&[9; 32]);
    f.store.append(stray, &[new_event(1)]).unwrap();

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!("writer {stray}: no seat certificate names it")
    );
}

#[test]
fn a_changed_seat_certificate_is_named() {
    let f = fixture();
    let mut cert = f.ingest.certificate.clone();
    cert["ingest_seat"] = json!(false);
    f.sql(&format!(
        "UPDATE seats SET certificate = '{cert}' WHERE writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "writer {}: its seat certificate: a seat_certificate signature does not verify",
            f.ingest.writer
        )
    );
}

#[test]
fn a_seat_certificate_naming_another_seat_or_room_is_named() {
    let f = fixture();
    let other = RoomId::derive(&[1; 32], &[1; 16]);
    f.sql(&format!(
        "UPDATE seats SET room = '{other}' WHERE writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "writer {}: its seat certificate names another seat",
            f.ingest.writer
        )
    );
}

#[test]
fn a_seat_certificate_no_certificate_record_holds_is_named() {
    let mut f = fixture();
    let seat = SeatKey::mint(&mut OsEntropy).unwrap();
    let cert = SeatCertificate::issue(
        f.node.device(),
        &seat,
        f.node.personal_room(),
        SeatKind::Run,
        true,
    )
    .unwrap();
    let record = StoredSeat {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    };
    f.store.add_seat(&record).unwrap();

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "writer {}: no certificate record in the device seat of the key that certified it holds its seat certificate",
            record.writer
        )
    );
}

#[test]
fn a_seat_certificate_from_a_key_with_no_minted_key_event_is_named() {
    let mut f = fixture();
    let stranger = DeviceKey::mint(&mut OsEntropy).unwrap();
    let seat = SeatKey::mint(&mut OsEntropy).unwrap();
    let cert = SeatCertificate::issue(
        &stranger,
        &seat,
        f.node.personal_room(),
        SeatKind::Run,
        true,
    )
    .unwrap();
    let record = StoredSeat {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    };
    f.store.add_seat(&record).unwrap();

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "writer {}: its seat certificate was certified by a key with no inception in this node's record",
            record.writer
        )
    );
}

#[test]
fn a_removed_key_event_is_named_as_lost() {
    let f = fixture();
    let device = f.node.device_writer();
    f.sql(&format!("UPDATE events SET content = NULL, commitment_key = NULL WHERE seq = 1 AND writer = '{device}'"));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "the content at {} was removed, and no tombstone records it",
            Address::new(device, Seq::FIRST).full(&f.node.personal_room())
        )
    );
}

#[test]
fn a_seat_certificate_that_does_not_parse_is_named() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE seats SET certificate = '{{}}' WHERE writer = '{}'",
        f.ingest.writer
    ));
    assert!(
        verify(&f.store, &f.node.device().public())
            .unwrap_err()
            .to_string()
            .contains("its seat certificate: malformed")
    );
}

#[test]
fn a_store_error_is_passed_on() {
    let f = fixture();
    f.sql("UPDATE seats SET room = 'x'");

    assert!(matches!(
        verify(&f.store, &f.node.device().public()),
        Err(VerifyError::Corrupt(_))
    ));
    let broken = VerifyError::Broken {
        address: "a".into(),
        reason: "r".into(),
    };
    assert!(std::error::Error::source(&broken).is_none());
    let err = VerifyError::Store(store::StoreError::Full(f.ingest.writer));
    assert!(std::error::Error::source(&err).is_some());
    assert!(err.to_string().contains("no seq left"));
}

#[test]
fn a_key_event_whose_content_is_no_key_event_is_named() {
    let mut f = fixture();
    let device = f.node.device_writer();
    f.store
        .append(
            device,
            &[NewEvent {
                kind: EventKind::KeyEvent,
                provenance: Provenance::Structural,
                content: json!({}),
                commitment_key: CommitmentKey::from_bytes([1; 32]),
                transcript_timestamp: None,
            }],
        )
        .unwrap();

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert!(
        err.to_string()
            .contains("does not check: malformed key event"),
        "{err}"
    );
}

#[test]
fn content_without_its_commitment_key_does_not_open_its_commitment() {
    let f = fixture();
    f.sql("UPDATE events SET commitment_key = NULL WHERE seq = 3");

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert!(
        err.to_string().ends_with("does not open its commitment"),
        "{err}"
    );
}

#[test]
fn a_store_error_reading_a_log_is_passed_on() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE events SET seq = -1 WHERE seq = 3 AND writer = '{}'",
        f.ingest.writer
    ));

    assert!(matches!(
        verify(&f.store, &f.node.device().public()),
        Err(VerifyError::Corrupt(_))
    ));
}

#[test]
fn a_record_checked_against_another_nodes_key_has_no_genesis() {
    let f = fixture();
    let stranger = DeviceKey::mint(&mut OsEntropy).unwrap();

    let err = verify(&f.store, &stranger.public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        "no device seat of this node's device key holds its inception"
    );
    assert_eq!((err.class(), err.address()), ("genesis", None));
}

#[test]
fn the_personal_room_derives_from_the_device_seat_key_and_the_acts_nonce() {
    let key = SeatKey::mint(&mut OsEntropy).unwrap().public();
    let room = RoomId::derive(key.as_bytes(), &[7; 16]);
    let act = json!({"room": room.to_string(), "nonce": "07".repeat(16)});

    assert_eq!(derived_room(&key, &act), Some(room));
    let other = json!({"room": room.to_string(), "nonce": "08".repeat(16)});
    assert_eq!(derived_room(&key, &other), None);
    assert_eq!(derived_room(&key, &json!({"room": room.to_string()})), None);
}

#[test]
fn a_create_room_act_whose_room_does_not_derive_is_named() {
    let f = fixture();
    // Another create room act stands second in a rebuilt genesis: the
    // whole device seat is rewritten so only the room is wrong.
    let device = f.node.device_writer();
    let stored = f
        .store
        .event(Address::new(device, Seq::new(2).unwrap()))
        .unwrap()
        .unwrap();
    let mut act = stored.content.clone().unwrap();
    act["nonce"] = json!("00".repeat(16));
    let key = stored.commitment_key.clone().unwrap();
    let commitment = record::commitment::commit(&key, &act).unwrap();
    f.sql(&format!(
        "UPDATE events SET content = '{act}', commitment = '{commitment}' WHERE writer = '{device}' AND seq = 2"
    ));
    rehash(&f, device);
    // The ingest seat cites the device seat's head: follow it.
    f.sql(
        "UPDATE event_heads SET other_hash = (SELECT hash FROM events e \
         WHERE e.writer = event_heads.other_writer AND e.seq = event_heads.other_seq)",
    );
    rehash(&f, f.ingest.writer);

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        "the personal room's id does not derive from its device seat's key and the nonce its create room act records"
    );
}

/// Rewrites `writer`'s chain over the headers the store now holds.
fn rehash(f: &Fixture, writer: WriterId) {
    let mut prev = Digest::ZERO;
    for stored in f.store.log(writer).unwrap() {
        let mut header = stored.link.header.clone();
        header.prev_hash = prev;
        prev = header.hash();
        f.sql(&format!(
            "UPDATE events SET prev_hash = '{}', hash = '{prev}' WHERE writer = '{writer}' AND seq = {}",
            header.prev_hash, header.address.seq
        ));
    }
}

#[test]
fn a_certificate_record_naming_a_seat_this_node_does_not_hold_is_named() {
    let mut f = fixture();
    let device = f.node.device_writer();
    let stray = WriterId::derive(&[9; 32]);
    let at = f
        .store
        .append(
            device,
            &[NewEvent {
                kind: EventKind::CertificateRecord,
                provenance: Provenance::Structural,
                content: json!({"certificate": "0".repeat(64), "seat": "0".repeat(64), "writer": stray.to_string()}),
                commitment_key: CommitmentKey::from_bytes([1; 32]),
                transcript_timestamp: None,
            }],
        )
        .unwrap()[0];

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    let full = at.full(&f.node.personal_room());
    assert_eq!(
        err.to_string(),
        format!("the certificate record at {full} names a seat this node does not hold")
    );
    assert_eq!(
        (err.class(), err.address()),
        ("certificate_record", Some(full.as_str()))
    );
}

#[test]
fn trusted_provenance_outside_a_device_seat_is_named() {
    let mut f = fixture();
    let at = f
        .store
        .append(
            f.ingest.writer,
            &[NewEvent {
                kind: EventKind::UnparsedLine,
                provenance: Provenance::Operator,
                content: json!({"line": 4, "text": "x"}),
                commitment_key: CommitmentKey::from_bytes([1; 32]),
                transcript_timestamp: None,
            }],
        )
        .unwrap()[0];

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "the event at {} is of operator provenance, which only a device seat records",
            at.full(&f.node.personal_room())
        )
    );
    assert_eq!(err.class(), "provenance");
}

#[test]
fn a_witnessed_run_seat_records_structural_events_but_no_operator_s() {
    let mut f = fixture();
    let key = key::signing::SeatKey::mint(&mut key::random::OsEntropy).unwrap();
    let seat = f
        .node
        .certify_run_seat(&mut f.store, &key, &mut key::random::OsEntropy)
        .unwrap();
    let event = |provenance| NewEvent {
        kind: EventKind::HookObservation,
        provenance,
        content: json!({"hook": "SessionStart"}),
        commitment_key: CommitmentKey::from_bytes([2; 32]),
        transcript_timestamp: None,
    };
    f.store
        .append(seat.writer, &[event(Provenance::Structural)])
        .unwrap();
    assert!(verify(&f.store, &f.node.device().public()).is_ok());

    f.store
        .append(seat.writer, &[event(Provenance::Operator)])
        .unwrap();
    let err = verify(&f.store, &f.node.device().public()).unwrap_err();
    assert_eq!(err.class(), "provenance");
}

#[test]
fn structural_provenance_in_an_ingest_seat_is_named() {
    let mut f = fixture();
    f.store
        .append(
            f.ingest.writer,
            &[NewEvent {
                kind: EventKind::HookObservation,
                provenance: Provenance::Structural,
                content: json!({"hook": "SessionStart"}),
                commitment_key: CommitmentKey::from_bytes([3; 32]),
                transcript_timestamp: None,
            }],
        )
        .unwrap();

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(err.class(), "provenance");
}

#[test]
fn a_transcripts_untrusted_harness_meta_verifies_in_an_ingest_seat() {
    // A transcript line's harness_meta is untrusted (PRV-08): only what
    // the launcher records is the trusted kind.
    let mut f = fixture();
    f.store
        .append(
            f.ingest.writer,
            &[NewEvent {
                kind: EventKind::TranscriptLine,
                provenance: Provenance::HarnessMeta,
                content: json!({"line": 4, "text": "{\"type\":\"mode\"}"}),
                commitment_key: CommitmentKey::from_bytes([4; 32]),
                transcript_timestamp: None,
            }],
        )
        .unwrap();

    assert!(verify(&f.store, &f.node.device().public()).is_ok());
}

#[test]
fn each_failure_names_its_class_and_where_it_is() {
    let at = |e: VerifyError| (e.class(), e.address().map(str::to_owned));
    let a = || "a".to_owned();
    let r = || "r".to_owned();
    let w = WriterId::derive(&[1; 32]);

    assert_eq!(
        at(VerifyError::Broken {
            address: a(),
            reason: r()
        }),
        ("chain", Some(a()))
    );
    assert_eq!(
        at(VerifyError::Content {
            address: a(),
            removed: true
        }),
        ("content", Some(a()))
    );
    assert_eq!(
        at(VerifyError::KeyEvent {
            address: a(),
            reason: r()
        }),
        ("key_event", Some(a()))
    );
    assert_eq!(
        at(VerifyError::Provenance {
            address: a(),
            provenance: r()
        }),
        ("provenance", Some(a()))
    );
    assert_eq!(
        at(VerifyError::Seat {
            writer: w,
            reason: r()
        }),
        ("seat", None)
    );
    assert_eq!(
        at(VerifyError::Corrupt(store::StoreError::Full(w))),
        ("corrupt", None)
    );
    assert_eq!(
        at(VerifyError::Store(store::StoreError::Full(w))),
        ("store", None)
    );
    assert!(matches!(
        VerifyError::from(store::StoreError::Full(w)),
        VerifyError::Store(_)
    ));
}

#[test]
fn the_report_names_each_seat_by_its_seat_certificate() {
    let f = fixture();

    let report = verify(&f.store, &f.node.device().public()).unwrap();

    let device = report
        .seats
        .iter()
        .find(|s| s.writer == f.node.device_writer())
        .unwrap();
    assert_eq!(
        (device.seat_kind, device.ingest_seat, device.room),
        (SeatKind::Device, false, f.node.personal_room())
    );
    let ingest = report
        .seats
        .iter()
        .find(|s| s.writer == f.ingest.writer)
        .unwrap();
    assert_eq!(
        (ingest.seat_kind, ingest.ingest_seat, ingest.seat),
        (SeatKind::Run, true, f.ingest.seat)
    );
    assert_eq!(ingest.certificate, f.ingest.certificate);
    assert_eq!(report.seats.len(), 2);
}

/// The fixture's node moved to another machine: a new device key and a
/// new device seat that begins with the node identity change.
fn moved(f: &mut Fixture) -> Node {
    let root = f.home.root().parent().unwrap().to_path_buf();
    std::fs::write(root.join("machine-id"), "second").unwrap();
    Node::open(&f.home, &mut f.store, &mut OsEntropy).unwrap()
}

#[test]
fn a_record_whose_node_identity_changed_verifies_against_the_new_key() {
    let mut f = fixture();
    let node = moved(&mut f);
    let ingest = node
        .certify_ingest_seat(&f.home, &mut f.store, &mut OsEntropy)
        .unwrap();

    let report = verify(&f.store, &node.device().public()).unwrap();

    assert_eq!(report.seats.len(), 4);
    let new = report
        .seats
        .iter()
        .find(|s| s.writer == node.device_writer())
        .unwrap();
    assert_eq!(
        (new.seat_kind, new.room),
        (SeatKind::Device, f.node.personal_room())
    );
    assert!(report.seats.iter().any(|s| s.writer == ingest.writer));
    // The old key's record still verifies against the old key.
    assert!(verify(&f.store, &f.node.device().public()).is_ok());
}

/// Records a device seat of a fresh device key in the personal room
/// that begins with `change`, then the key's `minted`: a node identity
/// change as an appending process could forge it.
fn forged_change(
    f: &mut Fixture,
    change: impl FnOnce(&DeviceKey) -> NodeIdentityChange,
) -> (DeviceKey, Address) {
    let device = DeviceKey::mint(&mut OsEntropy).unwrap();
    forged_change_of(f, device, change)
}

/// [`forged_change`] for the device key `device`.
fn forged_change_of(
    f: &mut Fixture,
    device: DeviceKey,
    change: impl FnOnce(&DeviceKey) -> NodeIdentityChange,
) -> (DeviceKey, Address) {
    let recovery = RecoveryKey::mint(&mut OsEntropy).unwrap();
    let seat = SeatKey::mint(&mut OsEntropy).unwrap();
    let cert = SeatCertificate::issue(
        &device,
        &seat,
        f.node.personal_room(),
        SeatKind::Device,
        false,
    )
    .unwrap();
    let record = StoredSeat {
        writer: cert.writer_id(),
        seat: cert.seat_id(),
        room: cert.room,
        certificate: cert.to_value(),
    };
    let minted = key::key_event::KeyEvent::minted(&device, &recovery.public()).unwrap();
    let events = [
        NewEvent {
            kind: EventKind::NodeIdentityChange,
            provenance: Provenance::Structural,
            content: change(&device).to_value(),
            commitment_key: CommitmentKey::from_bytes([1; 32]),
            transcript_timestamp: None,
        },
        NewEvent {
            kind: EventKind::KeyEvent,
            provenance: Provenance::Structural,
            content: minted.to_value(),
            commitment_key: CommitmentKey::from_bytes([2; 32]),
            transcript_timestamp: None,
        },
        NewEvent {
            kind: EventKind::CertificateRecord,
            provenance: Provenance::Structural,
            content: json!({
                "certificate": cert.digest().to_hex(),
                "seat": cert.seat_id().to_string(),
                "writer": cert.writer_id().to_string(),
            }),
            commitment_key: CommitmentKey::from_bytes([3; 32]),
            transcript_timestamp: None,
        },
    ];
    let at = f
        .store
        .add_seat_and_append(&record, record.writer, &events)
        .unwrap()[0];
    (device, at)
}

mod later;
