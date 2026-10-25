//! The store's append transaction (REC-06, §8.2), through its public
//! API and a real file: a writer's events land a hundred to a
//! transaction, each committed hundred visible to a fresh connection,
//! so a new event that fails costs only the transaction it falls in.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use record::chain;
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::WriterId;
use serde_json::json;
use store::{NewEvent, Store, StoreError};
use testkit::TempDir;

fn new_event(n: usize) -> NewEvent {
    NewEvent {
        kind: EventKind::UnparsedLine,
        provenance: Provenance::Unparsed,
        content: json!({"text": format!("line {n}")}),
        commitment_key: CommitmentKey::from_bytes([1; 32]),
        transcript_timestamp: None,
    }
}

#[test]
fn a_failed_new_event_rolls_back_its_own_hundred_and_keeps_the_hundred_before_it() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    let writer = WriterId::derive(&[1; 32]);
    let mut batch: Vec<NewEvent> = (1..=250).map(new_event).collect();
    // NewEvent 160 fails: a hundred to a transaction keeps 100, where fifty
    // would keep 150 and two hundred none.
    batch[159].content = json!({"x": 0.5});

    let err = Store::open(&path)
        .unwrap()
        .append(writer, &batch)
        .unwrap_err();

    assert!(matches!(err, StoreError::Encoding(_)), "{err}");
    let log = Store::open(&path).unwrap().log(writer).unwrap();
    assert_eq!(log.len(), 100);
    let links: Vec<_> = log.into_iter().map(|s| s.link).collect();
    assert_eq!(chain::check_writer(writer, &links), Ok(()));
}

#[test]
fn a_second_connection_sees_each_committed_transaction() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    let writer = WriterId::derive(&[1; 32]);
    let mut appender = Store::open(&path).unwrap();
    let reader = Store::open(&path).unwrap();

    appender
        .append(writer, &(1..=150).map(new_event).collect::<Vec<_>>())
        .unwrap();

    assert_eq!(reader.log(writer).unwrap().len(), 150);
}
