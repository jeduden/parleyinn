//! What the recall tools show of a purged and a quarantined address
//! (RCL-06, SEC-12): a purged one as a tombstone of its reason, a
//! quarantined one as a quarantine marker of its range alone, never
//! its content, and no search hit.

use super::recall_calls::device;
use super::*;
use administration::act::{self, Act};
use administration::purge::{self, Planned, Reason};
use administration::quarantine::Selector;
use record::id::{Address, Seq};

/// A fixture of ten lines, the first line `needle` for a search.
fn ten() -> Fixture {
    let mut lines: Vec<String> = (1..=10).map(|i| format!("line {i} quiet")).collect();
    lines[3] = "line 4 needle".to_owned();
    super::recall_calls::fixture_of(&lines)
}

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

/// Purges events `first` to `last` of the fixture's run.
fn purge(f: &Fixture, first: u64, last: u64) {
    purge_for(f, first, last, &Reason::Purged);
}

/// The same, for `reason`.
fn purge_for(f: &Fixture, first: u64, last: u64, reason: &Reason) {
    let mut store = Store::open(&f.home.store()).unwrap();
    let node = Node::find(&f.home, &store).unwrap();
    let planned = purge::plan(&store, f.ingested.writer, seq(first), seq(last), reason);
    let Planned::Ready(plan) = planned.unwrap() else {
        panic!("refused");
    };
    purge::carry_out(&mut store, &node, &plan, None, &mut OsEntropy).unwrap();
}

/// Quarantines events `first` to `last` of the fixture's run.
fn quarantine(f: &Fixture, first: u64, last: u64) {
    let mut store = Store::open(&f.home.store()).unwrap();
    let node = Node::find(&f.home, &store).unwrap();
    let selector = Selector::Range {
        writer: f.ingested.writer,
        first: seq(first),
        last: seq(last),
    };
    let add = Act::QuarantineAdd {
        selector,
        reason: "suspect".to_owned(),
    };
    act::record(&mut store, &node, &add, &mut OsEntropy).unwrap();
}

fn tombstones(envelope: &Value) -> Vec<(String, String)> {
    envelope["tombstones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            (
                t["range"].as_str().unwrap().to_owned(),
                t["reason"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn markers(envelope: &Value) -> Vec<Value> {
    envelope["quarantine_markers"].as_array().unwrap().clone()
}

fn expand(server: &mut Server, range: &str) -> Value {
    let arguments = json!({"range": range, "scope": "room"});

    envelope_of(&super::recall_calls::tool(
        server,
        "event_expand",
        arguments,
    ))
}

#[test]
fn a_purged_range_shows_as_one_tombstone_between_the_events_around_it() {
    let f = ten();
    purge(&f, 3, 5);
    let mut server = super::recall_calls::recording(&f);

    let envelope = expand(&mut server, "A1:1-8");

    let seqs: Vec<String> = envelope["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["content"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        seqs,
        [
            "line 1 quiet",
            "line 2 quiet",
            "line 6 quiet",
            "line 7 quiet",
            "line 8 quiet"
        ]
    );
    assert_eq!(
        tombstones(&envelope),
        [("A1·3–5".to_owned(), "purged".to_owned())]
    );
    assert!(!envelope.to_string().contains("needle"));
}

#[test]
fn a_single_purged_address_is_a_tombstone_and_a_quarantined_one_a_marker_of_its_range_alone() {
    let f = ten();
    purge(&f, 2, 2);
    quarantine(&f, 4, 4);
    let mut server = super::recall_calls::recording(&f);

    let envelope = expand(&mut server, "A1:1-5");

    assert_eq!(
        tombstones(&envelope),
        [("A1·2".to_owned(), "purged".to_owned())]
    );
    assert_eq!(markers(&envelope), [json!({"range": "A1·4"})]);
    assert_eq!(envelope["items"].as_array().unwrap().len(), 3);
    let get = super::recall_calls::tool(
        &mut server,
        "event_get",
        json!({"address": "A1:4", "scope": "room"}),
    );
    let got = envelope_of(&get);
    assert_eq!(got["items"], json!([]));
    assert_eq!(tombstones(&got), []);
    assert_eq!(markers(&got), [json!({"range": "A1·4"})]);
    assert!(!get.to_string().contains("needle"));
}

#[test]
fn a_quarantine_holds_back_events_recorded_after_it_and_its_release_gives_them_back() {
    let f = ten();
    let mut server = f.server();
    let writer = f.ingested.writer;
    let selector = Selector::Writer(writer);
    {
        let mut store = Store::open(&f.home.store()).unwrap();
        let node = Node::find(&f.home, &store).unwrap();
        let add = Act::QuarantineAdd {
            selector: selector.clone(),
            reason: "the whole writer".to_owned(),
        };
        act::record(&mut store, &node, &add, &mut OsEntropy).unwrap();
    }

    let held = expand(&mut server, &format!("{writer}:9-10"));

    assert_eq!(held["items"], json!([]));
    assert_eq!(tombstones(&held), []);
    assert_eq!(
        markers(&held),
        [json!({"range": format!("parinn:room/{}/w/{writer}/9-10", f.ingested.room)})]
    );
    let mut store = Store::open(&f.home.store()).unwrap();
    let node = Node::find(&f.home, &store).unwrap();
    act::record(
        &mut store,
        &node,
        &Act::QuarantineRelease { selector },
        &mut OsEntropy,
    )
    .unwrap();
    let released = expand(&mut server, &format!("{writer}:9-10"));
    assert_eq!(released["items"].as_array().unwrap().len(), 2);
    assert_eq!(released["tombstones"], json!([]));
    assert_eq!(released["quarantine_markers"], json!([]));
}

#[test]
fn a_retention_purge_shows_retention_and_a_purged_quarantined_address_only_a_tombstone() {
    let f = ten();
    let retention = Reason::Retention {
        policy: "retention.max_age".to_owned(),
    };
    purge_for(&f, 2, 3, &retention);
    quarantine(&f, 3, 4);
    let mut server = super::recall_calls::recording(&f);

    let envelope = expand(&mut server, "A1:1-5");

    assert_eq!(
        tombstones(&envelope),
        [("A1·2–3".to_owned(), "retention".to_owned())]
    );
    assert_eq!(markers(&envelope), [json!({"range": "A1·4"})]);
    let clipped = expand(&mut server, "A1:4-5");
    assert_eq!(tombstones(&clipped), []);
    assert_eq!(markers(&clipped), [json!({"range": "A1·4"})]);
}

#[test]
fn a_search_finds_neither_a_purged_nor_a_quarantined_event() {
    let f = ten();
    let mut server = f.server();
    let search = |server: &mut Server| {
        let found = super::recall_calls::tool(
            server,
            "event_search",
            json!({"query": "needle", "scope": "room"}),
        );
        envelope_of(&found)["items"].as_array().unwrap().len()
    };
    assert_eq!(search(&mut server), 1);

    quarantine(&f, 4, 4);
    assert_eq!(search(&mut server), 0);
    let mut store = Store::open(&f.home.store()).unwrap();
    let node = Node::find(&f.home, &store).unwrap();
    let release = Act::QuarantineRelease {
        selector: Selector::Range {
            writer: f.ingested.writer,
            first: seq(4),
            last: seq(4),
        },
    };
    act::record(&mut store, &node, &release, &mut OsEntropy).unwrap();
    drop(store);
    assert_eq!(search(&mut server), 1);

    purge(&f, 4, 4);
    assert_eq!(search(&mut server), 0);
}

#[test]
fn many_tombstones_stop_a_call_at_the_cap_and_its_cursor_goes_on() {
    let lines: Vec<String> = (1..=400).map(|i| format!("line {i}")).collect();
    let f = super::recall_calls::fixture_of(&lines);
    // Every second event purged: no two share a tombstone.
    let ranges: Vec<store::Erase> = (1..=399)
        .step_by(2)
        .map(|n| store::Erase {
            writer: f.ingested.writer,
            first: seq(n),
            last: seq(n),
        })
        .collect();
    let recorded: Vec<store::NewEvent> = ranges
        .iter()
        .zip(1u8..)
        .map(|(r, n)| store::NewEvent {
            kind: EventKind::Tombstone,
            provenance: record::event::Provenance::Structural,
            content: administration::purge::Tombstone {
                writer: r.writer,
                first: r.first,
                last: r.last,
                count: 1,
                reason: "purged".to_owned(),
                policy: None,
                commitments: record::digest::Digest::ZERO,
            }
            .to_value(),
            commitment_key: record::commitment::CommitmentKey::from_bytes([n; 32]),
            transcript_timestamp: None,
        })
        .collect();
    let mut store = Store::open(&f.home.store()).unwrap();
    store.purge(device(&f), &recorded, &ranges).unwrap();
    drop(store);
    let mut server = super::recall_calls::recording(&f);

    let first = expand(&mut server, "A1:1-400");

    assert_eq!(first["truncated"], true);
    let mut shown = tombstones(&first).len();
    let mut items = first["items"].as_array().unwrap().len();
    let mut cursor = first["next_cursor"].as_str().unwrap().to_owned();
    loop {
        let answer = super::recall_calls::tool(
            &mut server,
            "event_expand",
            json!({"cursor": cursor, "scope": "room"}),
        );
        assert!(envelope::tokens::estimate(super::recall_calls::text_of(&answer)) <= 8_000);
        let page = envelope_of(&answer);
        shown += tombstones(&page).len();
        items += page["items"].as_array().unwrap().len();
        match page["next_cursor"].as_str() {
            Some(next) => cursor = next.to_owned(),
            None => break,
        }
    }
    assert_eq!((shown, items), (200, 200));
}

#[test]
fn a_purged_event_is_no_gap_in_the_record_and_its_address_resolves_with_no_content() {
    let f = ten();
    purge(&f, 10, 10);
    let mut server = f.server();
    let w = f.ingested.writer;

    let envelope = expand(&mut server, &format!("{w}:9-12"));

    assert_eq!(envelope["items"].as_array().unwrap().len(), 1);
    assert_eq!(tombstones(&envelope).len(), 1);
    let store = Store::open(&f.home.store()).unwrap();
    assert!(
        store
            .event(Address::new(w, seq(10)))
            .unwrap()
            .unwrap()
            .content
            .is_none()
    );
}

#[test]
fn a_page_of_tombstones_alone_stops_at_the_cap_and_goes_on() {
    let lines: Vec<String> = (1..=300).map(|i| format!("line {i}")).collect();
    let f = super::recall_calls::fixture_of(&lines);
    // Purged and quarantined seqs alternate: every one a tombstone.
    for n in (2..=300).step_by(2) {
        quarantine(&f, n, n);
    }
    let ranges: Vec<store::Erase> = (1..=299)
        .step_by(2)
        .map(|n| store::Erase {
            writer: f.ingested.writer,
            first: seq(n),
            last: seq(n),
        })
        .collect();
    let recorded: Vec<store::NewEvent> = ranges
        .iter()
        .zip(1u8..)
        .map(|(r, n)| store::NewEvent {
            kind: EventKind::Tombstone,
            provenance: record::event::Provenance::Structural,
            content: administration::purge::Tombstone {
                writer: r.writer,
                first: r.first,
                last: r.last,
                count: 1,
                reason: "purged".to_owned(),
                policy: None,
                commitments: record::digest::Digest::ZERO,
            }
            .to_value(),
            commitment_key: record::commitment::CommitmentKey::from_bytes([n; 32]),
            transcript_timestamp: None,
        })
        .collect();
    let mut store = Store::open(&f.home.store()).unwrap();
    store.purge(device(&f), &recorded, &ranges).unwrap();
    drop(store);
    let mut server = super::recall_calls::recording(&f);

    let first = expand(&mut server, "A1:1-300");

    assert_eq!(first["items"], json!([]));
    assert_eq!(first["truncated"], true);
    assert!(tombstones(&first).len() < 300);
    assert!(first["next_cursor"].is_string());
}
