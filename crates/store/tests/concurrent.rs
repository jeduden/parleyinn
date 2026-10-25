//! Writers appending at once through separate connections, as several
//! agents' hook handlers and MCP servers will (REC-06): each append
//! transaction takes the store's write lock, so seqs stay gap-free and
//! never reused, and every chain and cited head checks.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::BTreeMap;

use record::chain;
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::WriterId;
use serde_json::json;
use store::{NewEvent, Store};
use testkit::TempDir;

fn new_events(n: usize) -> Vec<NewEvent> {
    (0..n)
        .map(|i| NewEvent {
            kind: EventKind::UnparsedLine,
            provenance: Provenance::Unparsed,
            content: json!({"text": format!("line {i}")}),
            commitment_key: CommitmentKey::from_bytes([1; 32]),
            transcript_timestamp: None,
        })
        .collect()
}

/// Has `threads` connections each append `rounds` batches of `per` to
/// the writer `writer_of` names for it.
fn append_at_once(
    path: &std::path::Path,
    threads: u8,
    rounds: usize,
    per: usize,
    writer_of: fn(u8) -> WriterId,
) {
    std::thread::scope(|scope| {
        for t in 0..threads {
            scope.spawn(move || {
                let mut store = Store::open(path).unwrap();
                for _ in 0..rounds {
                    store.append(writer_of(t), &new_events(per)).unwrap();
                }
            });
        }
    });
}

#[test]
fn one_writer_appended_to_from_several_connections_keeps_a_gap_free_chain() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    Store::open(&path).unwrap();

    append_at_once(&path, 4, 5, 30, |_| WriterId::derive(&[1; 32]));

    let writer = WriterId::derive(&[1; 32]);
    let log = Store::open(&path).unwrap().log(writer).unwrap();
    let seqs: Vec<u64> = log
        .iter()
        .map(|s| s.link.header.address.seq.get())
        .collect();
    assert_eq!(seqs, (1..=600).collect::<Vec<_>>());
    let links: Vec<_> = log.into_iter().map(|s| s.link).collect();
    assert_eq!(chain::check_writer(writer, &links), Ok(()));
}

#[test]
fn writers_appending_at_once_cite_heads_that_all_check() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    Store::open(&path).unwrap();

    append_at_once(&path, 4, 10, 5, |t| WriterId::derive(&[t; 32]));

    let store = Store::open(&path).unwrap();
    let record: BTreeMap<_, _> = store
        .writers()
        .unwrap()
        .into_iter()
        .map(|w| {
            (
                w,
                store
                    .log(w)
                    .unwrap()
                    .into_iter()
                    .map(|s| s.link)
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(record.len(), 4);
    for (writer, links) in &record {
        assert_eq!(links.len(), 50);
        assert_eq!(chain::check_writer(*writer, links), Ok(()));
    }
    assert_eq!(chain::check_heads(&record), Ok(()));
    assert!(chain::causal_order(&record).is_ok());
}
