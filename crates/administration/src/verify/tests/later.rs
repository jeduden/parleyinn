//! The verification's later tests: forged changes, worktree binds, purges and tombstones.
//! The rest of the module's tests, moved from `tests.rs` to keep each file in its limit.

use super::*;
use key::key_event::NodeIdentityChange;
use key::random::OsEntropy;
use key::signing::DeviceKey;
use record::chain::SeenHead;
use record::commitment::CommitmentKey;
use record::digest::Digest;
use record::event::{EventKind, Provenance};
use record::id::Seq;
use serde_json::json;
use store::NewEvent;

#[test]
fn a_node_identity_change_whose_chain_head_names_no_event_is_named() {
    let mut f = fixture();
    let old = f.node.device().public().fingerprint();
    let writer = f.node.device_writer();
    let (device, at) = forged_change(&mut f, |d| NodeIdentityChange {
        replaces: old,
        key: d.public().fingerprint(),
        chain_head: SeenHead::new(Address::new(writer, Seq::new(9).unwrap()), Digest::ZERO),
    });

    let err = verify(&f.store, &device.public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "the key event at {} does not check: its chain head names no event of its writer",
            at.full(&f.node.personal_room())
        )
    );
}

#[test]
fn a_node_identity_change_replacing_a_key_with_no_inception_is_named() {
    let mut f = fixture();
    let writer = f.node.device_writer();
    let head = f.store.log(writer).unwrap().last().unwrap().link.clone();
    let (device, at) = forged_change(&mut f, |d| NodeIdentityChange {
        replaces: Digest::ZERO,
        key: d.public().fingerprint(),
        chain_head: SeenHead::new(head.header.address, head.hash),
    });

    let err = verify(&f.store, &device.public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "the key event at {} does not check: it replaces a key with no inception in this node's record",
            at.full(&f.node.personal_room())
        )
    );
}

#[test]
fn a_node_identity_change_naming_another_key_holds_no_inception() {
    let mut f = fixture();
    let old = f.node.device().public().fingerprint();
    let writer = f.node.device_writer();
    let head = f.store.log(writer).unwrap().last().unwrap().link.clone();
    let (device, _) = forged_change(&mut f, |_| NodeIdentityChange {
        replaces: old,
        key: Digest::ZERO,
        chain_head: SeenHead::new(head.header.address, head.hash),
    });

    let err = verify(&f.store, &device.public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        "no device seat of this node's device key holds its inception"
    );
}

#[test]
fn a_node_identity_change_event_that_does_not_parse_is_named() {
    let mut f = fixture();
    let device = f.node.device_writer();
    f.store
        .append(
            device,
            &[NewEvent {
                kind: EventKind::NodeIdentityChange,
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
            .contains("does not check: malformed node identity change event"),
        "{err}"
    );
}

#[test]
fn a_certificate_record_naming_a_seat_another_key_certified_is_named() {
    let mut f = fixture();
    let node = moved(&mut f);
    // The new device seat records the old ingest seat, which the old key
    // certified.
    let cert = SeatCertificate::from_value(&f.ingest.certificate).unwrap();
    let at = f
        .store
        .append(
            node.device_writer(),
            &[NewEvent {
                kind: EventKind::CertificateRecord,
                provenance: Provenance::Structural,
                content: json!({
                    "certificate": cert.digest().to_hex(),
                    "seat": cert.seat_id().to_string(),
                    "writer": cert.writer_id().to_string(),
                }),
                commitment_key: CommitmentKey::from_bytes([1; 32]),
                transcript_timestamp: None,
            }],
        )
        .unwrap()[0];

    let err = verify(&f.store, &node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "the certificate record at {} names a seat this node does not hold",
            at.full(&f.node.personal_room())
        )
    );
}

#[test]
fn node_identity_changes_that_replace_each_other_lead_back_to_no_genesis() {
    let mut f = fixture();
    let writer = f.node.device_writer();
    let head = f.store.log(writer).unwrap().last().unwrap().link.clone();
    let head = SeenHead::new(head.header.address, head.hash);
    let a = DeviceKey::mint(&mut OsEntropy).unwrap();
    let b = DeviceKey::mint(&mut OsEntropy).unwrap();
    let (fa, fb) = (a.public().fingerprint(), b.public().fingerprint());
    let (a, _) = forged_change_of(&mut f, a, |_| NodeIdentityChange {
        replaces: fb,
        key: fa,
        chain_head: head,
    });
    forged_change_of(&mut f, b, |_| NodeIdentityChange {
        replaces: fa,
        key: fb,
        chain_head: head,
    });

    let err = verify(&f.store, &a.public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        "this node's device key leads back to no genesis"
    );
}

#[test]
fn a_record_whose_writers_closed_their_segments_reports_none_open() {
    let f = fixture();

    let report = verify(&f.store, &f.node.device().public()).unwrap();

    assert!(report.open_segments.is_empty());
}

#[test]
fn a_writer_chain_ended_without_a_closed_segment_is_reported_and_verifies() {
    let mut f = fixture();
    f.store
        .append_open(f.ingest.writer, &[new_event(4), new_event(5)])
        .unwrap();

    let report = verify(&f.store, &f.node.device().public()).unwrap();

    let open: Vec<(WriterId, u64, u64)> = report
        .open_segments
        .iter()
        .map(|s| (s.writer, s.first.get(), s.last.get()))
        .collect();
    assert_eq!(open, [(f.ingest.writer, 4, 5)]);
}

#[test]
fn a_segment_of_a_newer_format_version_is_refused_and_named() {
    let f = fixture();
    f.sql(&format!(
        "UPDATE segments SET format_version = 2 WHERE writer = '{}'",
        f.ingest.writer
    ));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert!(
        matches!(&err, VerifyError::NewerSegment(s) if s.writer == f.ingest.writer && s.format_version == 2),
        "{err}"
    );
    assert_eq!(err.class(), "newer_segment");
    assert_eq!(err.address(), None);
    assert_eq!(
        err.to_string(),
        format!(
            "the segment of writer {} from seq 1 to 3 is format version 2, newer than this parinn's 1",
            f.ingest.writer
        )
    );
}

#[test]
fn segments_that_do_not_tile_a_writer_s_log_are_named() {
    for (statement, why) in [
        (
            "DELETE FROM segments WHERE writer = '{w}'",
            "no segment holds seq 1",
        ),
        (
            "UPDATE segments SET last_seq = 2 WHERE writer = '{w}'",
            "no segment holds seq 3",
        ),
        (
            "UPDATE segments SET last_seq = 4 WHERE writer = '{w}'",
            "a segment runs past its newest event, seq 3",
        ),
        (
            "INSERT INTO segments VALUES ('{w}', 2, 3, 1, 1)",
            "the segment from seq 2 overlaps the one before it",
        ),
        (
            "UPDATE segments SET last_seq = 1 WHERE writer = '{w}';
             INSERT INTO segments VALUES ('{w}', 3, 3, 1, 1)",
            "no segment holds seq 2",
        ),
    ] {
        let f = fixture();
        f.sql(&statement.replace("{w}", &f.ingest.writer.to_string()));

        let err = verify(&f.store, &f.node.device().public()).unwrap_err();

        assert_eq!(err.class(), "segment", "{statement}: {err}");
        assert_eq!(
            err.to_string(),
            format!("writer {}: {why}", f.ingest.writer),
            "{statement}"
        );
    }
}

#[test]
fn only_a_writer_s_newest_segment_may_be_open() {
    let mut f = fixture();
    f.store.append(f.ingest.writer, &[new_event(4)]).unwrap();
    f.sql(&format!(
        "UPDATE segments SET closed = 0 WHERE writer = '{}' AND first_seq = 1",
        f.ingest.writer
    ));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "writer {}: the segment from seq 1 is open, though a later one follows it",
            f.ingest.writer
        )
    );
}

#[test]
fn a_payload_that_is_gone_is_named_by_its_event() {
    let mut f = fixture();
    let mut big = new_event(4);
    big.content = json!({"line": 4, "text": "x".repeat(9000)});
    f.store.append(f.ingest.writer, &[big]).unwrap();
    let name = f
        .store
        .event(f.at(4))
        .unwrap()
        .unwrap()
        .payload
        .unwrap()
        .name;
    std::fs::remove_file(f.home.root().join("payloads").join(name)).unwrap();

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(err.class(), "payload");
    let full = f.at(4).full(&f.node.personal_room());
    assert_eq!(err.address(), Some(full.as_str()));
    assert!(
        err.to_string()
            .starts_with(&format!("the payload of {full} cannot be read: ")),
        "{err}"
    );
}

#[test]
fn no_kept_origins_is_consistent_and_rebuilt_ones_are() {
    let mut f = fixture();
    assert!(verify(&f.store, &f.node.device().public()).is_ok());

    let derived = crate::origin::derive(&f.store, &f.node.device().public()).unwrap();
    f.store
        .replace_origins(
            &derived
                .iter()
                .map(|(w, o)| (*w, o.as_str().to_owned()))
                .collect(),
        )
        .unwrap();

    assert!(verify(&f.store, &f.node.device().public()).is_ok());
}

#[test]
fn a_kept_origin_that_differs_from_the_derived_one_is_named_by_its_writer() {
    let mut f = fixture();
    let mut kept: BTreeMap<_, _> = crate::origin::derive(&f.store, &f.node.device().public())
        .unwrap()
        .iter()
        .map(|(w, o)| (*w, o.as_str().to_owned()))
        .collect();
    kept.insert(f.ingest.writer, "witnessed".to_owned());
    f.store.replace_origins(&kept).unwrap();

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(err.class(), "derived_artifact_consistency");
    assert_eq!(err.address(), None);
    assert_eq!(
        err.to_string(),
        format!(
            "the origin kept for writer {} differs from the record's: run parinn rebuild",
            f.ingest.writer
        )
    );
}

#[test]
fn a_kept_origin_for_a_writer_the_record_does_not_hold_is_named_too() {
    let mut f = fixture();
    let stray = WriterId::derive(&[9; 32]);
    let kept: BTreeMap<_, _> = [(stray, "peer".to_owned())].into();
    f.store.replace_origins(&kept).unwrap();

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(err.class(), "derived_artifact_consistency");
    assert!(err.to_string().contains(&stray.to_string()));
}

/// Records that the node bound the worktree `digest` to a commit.
fn bind_worktree(f: &mut Fixture, digest: &str) {
    let event = NewEvent {
        kind: EventKind::RepositoryBind,
        provenance: Provenance::Structural,
        content: json!({"repository": "a".repeat(40), "worktree_digest": digest}),
        commitment_key: CommitmentKey::from_bytes([7; 32]),
        transcript_timestamp: None,
    };
    f.store.append(f.node.device_writer(), &[event]).unwrap();
}

#[test]
fn a_node_with_no_worktree_bound_keeps_none_and_agrees() {
    let f = fixture();

    assert!(kept_worktrees_agree(&f.store, &f.node).is_ok());
}

#[test]
fn a_worktree_the_record_binds_and_the_store_does_not_keep_is_named_by_its_digest() {
    let mut f = fixture();
    bind_worktree(&mut f, "w1");

    let err = kept_worktrees_agree(&f.store, &f.node).unwrap_err();

    assert_eq!(err.class(), "derived_artifact_consistency");
    assert_eq!(err.address(), None);
    assert_eq!(
        err.to_string(),
        "the identity kept for worktree w1 differs from the record's: run parinn rebuild"
    );
}

#[test]
fn the_worktrees_rebuilt_from_the_record_agree_and_a_changed_or_stray_one_does_not() {
    let mut f = fixture();
    bind_worktree(&mut f, "w1");
    let mut kept = crate::repository::derive_worktrees(&f.store, &f.node).unwrap();
    f.store.replace_worktree_identities(&kept).unwrap();
    assert!(kept_worktrees_agree(&f.store, &f.node).is_ok());

    kept.insert("w1".to_owned(), "other".to_owned());
    f.store.replace_worktree_identities(&kept).unwrap();
    let changed = kept_worktrees_agree(&f.store, &f.node).unwrap_err();
    kept.remove("w1");
    kept.insert("stray".to_owned(), "x".to_owned());
    f.store.replace_worktree_identities(&kept).unwrap();
    let stray = kept_worktrees_agree(&f.store, &f.node).unwrap_err();

    assert!(matches!(changed, VerifyError::DerivedWorktree { worktree } if worktree == "w1"));
    assert!(matches!(stray, VerifyError::DerivedWorktree { worktree } if worktree == "stray"));
}

#[test]
fn a_report_marker_is_no_worktree_and_a_derive_failure_keeps_its_kind() {
    let mut f = fixture();
    f.store.set_worktree_mark("reported:w1", "x").unwrap();

    assert!(kept_worktrees_agree(&f.store, &f.node).is_ok());
    assert!(matches!(
        derive_failed(AdminError::Store(StoreError::NoIndex)),
        VerifyError::Store(StoreError::NoIndex)
    ));
    let other = derive_failed(AdminError::Node("x"));
    assert!(matches!(other, VerifyError::Corrupt(_)), "{other:?}");
}

fn purge(f: &mut Fixture, first: u64, last: u64) {
    use crate::purge::{Planned, Reason, carry_out, plan};
    let planned = plan(
        &f.store,
        f.ingest.writer,
        Seq::new(first).unwrap(),
        Seq::new(last).unwrap(),
        &Reason::Purged,
    )
    .unwrap();
    let Planned::Ready(planned) = planned else {
        panic!("refused");
    };
    carry_out(&mut f.store, &f.node, &planned, None, &mut OsEntropy).unwrap();
}

#[test]
fn a_purged_range_under_its_tombstone_verifies() {
    let mut f = fixture();
    purge(&mut f, 2, 3);

    let report = verify(&f.store, &f.node.device().public()).unwrap();

    assert_eq!(report.events, 8, "the tombstone is an event of the record");
}

#[test]
fn removed_content_a_tombstone_does_not_cover_is_still_a_loss() {
    let mut f = fixture();
    purge(&mut f, 2, 2);
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
fn an_event_a_tombstone_names_that_keeps_its_key_is_named() {
    let mut f = fixture();
    purge(&mut f, 2, 3);
    f.sql(&format!(
        "UPDATE events SET commitment_key = '{}' WHERE seq = 2 AND writer = '{}'",
        "ab".repeat(32),
        f.ingest.writer
    ));

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(err.class(), "tombstone");
    assert!(
        err.to_string()
            .ends_with("does not check: an event it names still keeps its content or its key"),
        "{err}"
    );
    assert!(
        err.address()
            .unwrap()
            .contains(&f.node.device_writer().to_string())
    );
}

#[test]
fn a_tombstone_naming_events_the_record_does_not_hold_is_named() {
    let mut f = fixture();
    purge(&mut f, 1, 1);
    let lie = crate::purge::Tombstone {
        first: Seq::new(1).unwrap(),
        last: Seq::new(2).unwrap(),
        ..tombstone_of(&f)
    };
    f.store
        .append(f.node.device_writer(), &[tombstone_event(&lie)])
        .unwrap();

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert_eq!(err.class(), "tombstone");
    assert!(
        err.to_string()
            .ends_with("it names events the record does not hold"),
        "{err}"
    );
}

fn tombstone_of(f: &Fixture) -> crate::purge::Tombstone {
    f.store
        .log(f.node.device_writer())
        .unwrap()
        .iter()
        .rev()
        .find(|s| s.link.header.kind == EventKind::Tombstone)
        .and_then(|s| s.content.as_ref())
        .and_then(crate::purge::Tombstone::from_value)
        .unwrap()
}

fn tombstone_event(t: &crate::purge::Tombstone) -> NewEvent {
    NewEvent {
        kind: EventKind::Tombstone,
        provenance: Provenance::Structural,
        content: t.to_value(),
        commitment_key: CommitmentKey::from_bytes([99; 32]),
        transcript_timestamp: None,
    }
}

#[test]
fn a_tombstone_that_names_other_commitments_or_holds_no_range_is_named() {
    let mut f = fixture();
    purge(&mut f, 1, 2);
    let honest = tombstone_of(&f);
    let wrong = crate::purge::Tombstone {
        commitments: Digest::of(&[b"other"]),
        ..honest
    };
    f.store
        .append(f.node.device_writer(), &[tombstone_event(&wrong)])
        .unwrap();

    let err = verify(&f.store, &f.node.device().public()).unwrap_err();

    assert!(
        err.to_string()
            .ends_with("the commitments of the events it names differ from its digest"),
        "{err}"
    );

    let mut g = fixture();
    g.store
        .append(
            g.node.device_writer(),
            &[NewEvent {
                content: json!({"first": 1}),
                ..tombstone_event(&wrong)
            }],
        )
        .unwrap();
    let err = verify(&g.store, &g.node.device().public()).unwrap_err();
    assert!(
        err.to_string()
            .ends_with("it holds no range, count and digest"),
        "{err}"
    );
}

#[test]
fn a_store_made_by_this_build_has_the_shape_it_ships() {
    let f = fixture();

    assert!(matches!(
        verify_shape(&f.store, &|| true),
        Ok(Shape::Checked)
    ));
}

#[test]
fn a_schema_object_this_build_does_not_ship_fails_the_shape_naming_it() {
    let f = fixture();
    let conn = rusqlite::Connection::open(f.home.store()).unwrap();
    conn.execute_batch("CREATE TABLE exfil (x TEXT)").unwrap();

    let err = verify_shape(&f.store, &|| true).unwrap_err();

    assert_eq!(
        err.to_string(),
        "the store holds schema objects this parinn does not ship: main.exfil"
    );
    assert_eq!(err.class(), "derived_artifact_consistency");
    assert_eq!(err.check(), "derived artifact consistency");
    assert_eq!(err.address(), None);
}

#[test]
fn an_altered_ranking_or_a_changed_entry_fails_the_shape_as_a_search_index_fault() {
    let f = fixture();
    let conn = rusqlite::Connection::open(f.home.store()).unwrap();
    conn.execute(
        "ATTACH DATABASE ?1 AS idx",
        [f.home
            .root()
            .join(store::search_index::FILE)
            .to_str()
            .unwrap()],
    )
    .unwrap();
    let row = store::search_index::row_id_of(Address::new(f.ingest.writer, Seq::new(1).unwrap()))
        .unwrap();
    conn.execute("DELETE FROM idx.events_fts WHERE rowid = ?1", [row])
        .unwrap();
    conn.execute(
        "INSERT INTO idx.events_fts (rowid, text) VALUES (?1, 'zebra')",
        [row],
    )
    .unwrap();

    let err = verify_shape(&f.store, &|| true).unwrap_err();

    assert_eq!(
        err.to_string(),
        "the search index differs from what the record derives: run parinn rebuild"
    );
    conn.execute(
        "INSERT INTO idx.events_fts (events_fts, rank) VALUES ('rank', 'bm25(0.0)')",
        [],
    )
    .unwrap();
    let err = verify_shape(&f.store, &|| true).unwrap_err();
    assert_eq!(
        err.to_string(),
        "the search index ranks in a way this parinn does not ship: run parinn rebuild"
    );
}

#[test]
fn a_derivation_out_of_time_leaves_the_index_unchecked_neither_passed_nor_failed() {
    let f = fixture();

    assert!(matches!(
        verify_shape(&f.store, &|| false),
        Ok(Shape::IndexUnfinished)
    ));
}

#[test]
fn a_store_without_a_search_index_passes_the_shape() {
    let f = fixture();
    drop(f.store);
    std::fs::remove_file(f.home.root().join(store::search_index::FILE)).unwrap();
    let fresh = Store::open_read_only_made(&f.home.store())
        .unwrap()
        .unwrap();

    assert!(matches!(verify_shape(&fresh, &|| true), Ok(Shape::Checked)));
}

#[test]
fn each_failure_belongs_to_one_of_the_checks_of_adm_09() {
    let address = || "a".to_owned();
    let hash_chain = [
        VerifyError::Broken {
            address: address(),
            reason: String::new(),
        },
        VerifyError::Tombstone {
            address: address(),
            reason: String::new(),
        },
        VerifyError::KeyEvent {
            address: address(),
            reason: String::new(),
        },
        VerifyError::Record { address: address() },
        VerifyError::Provenance {
            address: address(),
            provenance: String::new(),
        },
        VerifyError::Genesis("g"),
    ];
    for err in hash_chain {
        assert_eq!(err.check(), "hash chain", "{err}");
    }
    let payload = [
        VerifyError::Content {
            address: address(),
            removed: false,
        },
        VerifyError::Payload {
            address: address(),
            reason: String::new(),
        },
    ];
    for err in payload {
        assert_eq!(err.check(), "payload integrity", "{err}");
    }
    assert_eq!(VerifyError::Corrupt(StoreError::NoIndex).check(), "store");
    assert_eq!(VerifyError::Store(StoreError::NoIndex).check(), "store");
}
