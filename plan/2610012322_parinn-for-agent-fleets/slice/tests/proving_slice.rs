//! Spike S12, plan 2610012322 phase 1's gate: two writers' segments,
//! received in either order, rebuild a byte-identical index file, the
//! search index and one room state; every event keeps its writer and trust, and the other
//! writer's events read untrusted; a purged event leaves no stored or
//! replicated value that confirms a guess at its content.

use std::path::PathBuf;

use s12slice::fixture::{self, CAUSAL_BRANCH, CONCURRENT_BRANCH, TwoWriters};
use s12slice::index;
use s12slice::node::Node;
use s12slice::record::{Address, Event, Segment};
use s12slice::room::{self, RoomState};
use s12slice::scan::{self, Guess};

const PER_WRITER: usize = 2_000;
const PER_SEGMENT: usize = 250;

fn two_writers() -> TwoWriters {
    fixture::two_writers(PER_WRITER, PER_SEGMENT).unwrap()
}

fn fresh(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("s12")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A node with the named own writers that received the segments in
/// order.
fn node<'a>(name: &str, keys: &[&str], segments: impl IntoIterator<Item = &'a Segment>) -> Node {
    let mut n = Node::new(&fresh(name), keys).unwrap();
    for s in segments {
        n.receive(s).unwrap();
    }
    n
}

fn index_bytes(n: &Node) -> Vec<u8> {
    std::fs::read(n.rebuild().unwrap()).unwrap()
}

/// What the node stores confirms the guess, one entry per way.
fn stored_confirmations(n: &Node, guess: &Guess) -> Vec<String> {
    n.rebuild().unwrap();
    scan::confirmations(&scan::read_all(&n.files().unwrap()).unwrap(), guess)
}

fn events(segments: &[Segment]) -> impl Iterator<Item = &Event> {
    segments.iter().flat_map(|s| &s.events)
}

/// B's segments newest first, interleaved with A's, then A's again.
fn reordered(f: &TwoWriters) -> Vec<&Segment> {
    let mut out = Vec::new();
    let mut b = f.b_segments.iter().rev();
    for a in &f.a_segments {
        out.push(a);
        out.extend(b.next());
    }
    out.extend(b);
    out.extend(&f.a_segments);
    out
}

#[test]
fn either_order_rebuilds_a_byte_identical_index_on_each_node() {
    let f = two_writers();
    let mut per_node = Vec::new();
    for (name, key) in [("a", f.a.id()), ("b", f.b.id())] {
        let ab = node(
            &format!("order-{name}-ab"),
            &[key],
            f.a_segments.iter().chain(&f.b_segments),
        );
        let ba = node(
            &format!("order-{name}-ba"),
            &[key],
            f.b_segments.iter().chain(&f.a_segments),
        );
        let mixed = node(&format!("order-{name}-mixed"), &[key], reordered(&f));
        let bytes = index_bytes(&ab);
        assert_eq!(
            bytes,
            index_bytes(&ba),
            "node {name}: A then B against B then A"
        );
        assert_eq!(
            bytes,
            index_bytes(&mixed),
            "node {name}: reordered and duplicated"
        );
        per_node.push(bytes);
    }
    // The comparison is not vacuous: trust derives per node, so the
    // two nodes' index files differ.
    assert_ne!(per_node[0], per_node[1]);
}

#[test]
fn every_event_keeps_its_writer_and_the_other_writers_events_read_untrusted() {
    let f = two_writers();
    let (a, b) = (f.a.id(), f.b.id());
    let all: Vec<&Event> = events(&f.a_segments).chain(events(&f.b_segments)).collect();
    let mut expected: Vec<Address> = all.iter().map(|e| e.address()).collect();
    expected.sort();
    for (own, other) in [(a, b), (b, a)] {
        let n = node(
            &format!("trust-{own}"),
            &[own],
            f.a_segments.iter().chain(&f.b_segments),
        );
        let rows = index::rows(&n.rebuild().unwrap()).unwrap();
        let addresses: Vec<Address> = rows.iter().map(|r| r.address.clone()).collect();
        assert_eq!(
            addresses, expected,
            "every event indexed under its own address"
        );
        let mut own_trusted = 0;
        for r in &rows {
            let e = all.iter().find(|e| e.address() == r.address).unwrap();
            assert_eq!(r.provenance, e.header.provenance);
            if r.address.writer == other {
                assert_eq!(
                    (r.origin.as_str(), r.trust.as_str()),
                    ("peer", "untrusted"),
                    "{:?}",
                    r.address
                );
            } else {
                assert_eq!(r.origin, "witnessed");
                let trusted = matches!(
                    r.provenance.as_str(),
                    "user" | "harness_meta" | "structural" | "operator"
                );
                assert_eq!(
                    r.trust == "trusted",
                    trusted,
                    "{:?} {}",
                    r.address,
                    r.provenance
                );
                own_trusted += usize::from(trusted);
            }
        }
        // The other writer has events of the same classes; only the
        // reading node makes them untrusted.
        assert!(own_trusted > 0);
    }
}

#[test]
fn either_order_derives_one_branch_link_per_branch() {
    let f = two_writers();
    let a = f.a.id();
    let ab = node("room-ab", &[a], f.a_segments.iter().chain(&f.b_segments));
    let ba = node("room-ba", &[a], f.b_segments.iter().chain(&f.a_segments));
    let state: RoomState = room::derive(&ab);
    assert_eq!(state, room::derive(&ba));

    let standing = |branch: &str| -> Vec<Address> {
        state
            .links
            .iter()
            .filter(|l| l.branch == branch)
            .map(|l| l.act.clone())
            .collect()
    };
    // Concurrent branch links: the lower commitment stands (LANE-31).
    let (la, lb) = &f.concurrent_links;
    let commitment = |at: &Address| {
        events(&f.a_segments)
            .chain(events(&f.b_segments))
            .find(|e| &e.address() == at)
            .unwrap()
            .header
            .commitment
    };
    let lower = if commitment(la) < commitment(lb) {
        la
    } else {
        lb
    };
    assert_eq!(standing(CONCURRENT_BRANCH), vec![lower.clone()]);
    // Seat B linked after it had seen A's branch link: the first in
    // causal order stands.
    assert_eq!(standing(CAUSAL_BRANCH), vec![f.causal_links.0.clone()]);

    // Each resolved conflict is recorded with both acts.
    let mut named: Vec<(Address, Address)> = state
        .conflicts
        .iter()
        .map(|c| (c.stands.clone(), c.void.clone()))
        .collect();
    named.sort();
    let higher = if lower == la { lb } else { la };
    let mut expected = vec![(lower.clone(), higher.clone()), f.causal_links.clone()];
    expected.sort();
    assert_eq!(named, expected);

    // The index file holds the same room state.
    let mut in_index = index::branch_links(&ab.rebuild().unwrap()).unwrap();
    in_index.sort();
    let mut derived: Vec<(String, Address)> = state
        .links
        .iter()
        .map(|l| (l.branch.clone(), l.act.clone()))
        .collect();
    derived.sort();
    assert_eq!(in_index, derived);
}

#[test]
fn a_purged_event_leaves_nothing_that_confirms_a_guess() {
    let mut f = two_writers();
    let (a, b) = (f.a.id().to_string(), f.b.id().to_string());
    let originals: Vec<Segment> = f.a_segments.clone();

    // The purged event: one of A's user messages, long enough that its
    // opening phrase matches only itself.
    let victim = events(&originals)
        .find(|e| {
            e.header.provenance == "user"
                && e.body
                    .as_ref()
                    .is_some_and(|b| b.content["text"].as_str().unwrap().len() > 600)
        })
        .unwrap()
        .clone();
    let content = victim.body.clone().unwrap().content;
    let text = content["text"].as_str().unwrap().to_string();
    let phrase: String = text
        .split_whitespace()
        .take(8)
        .collect::<Vec<_>>()
        .join(" ");
    let guess = Guess { text, content };

    // Before the purge the store confirms the guess, so the scan can see.
    let before = node(
        "purge-before",
        &[a.as_str()],
        originals.iter().chain(&f.b_segments),
    );
    assert!(!stored_confirmations(&before, &guess).is_empty());
    assert_eq!(
        index::phrase_matches(&before.rebuild().unwrap(), &phrase).unwrap(),
        vec![victim.address()]
    );
    let room_before = room::derive(&before);

    // B's node holds A's segments as first sent.
    let mut node_b = node(
        "purge-b",
        &[b.as_str()],
        originals.iter().chain(&f.b_segments),
    );

    // A's principal purges it on A's node.
    let tombstone =
        f.a.purge(&[victim.address()], "asked by the principal")
            .unwrap();
    let tail = f.a.close_segment().unwrap();
    assert!(tail.events.iter().any(|e| e.address() == tombstone));
    let served = f.a.segments();

    // What A's node now serves confirms nothing; the chain still verifies.
    let replicated: Vec<(String, Vec<u8>)> = served
        .iter()
        .enumerate()
        .map(|(i, s)| (format!("served segment {i}"), s.encode()))
        .collect();
    assert_eq!(
        scan::confirmations(&replicated, &guess),
        Vec::<String>::new()
    );
    for s in &served {
        assert_eq!(&Segment::decode(&s.encode()).unwrap(), s);
    }

    // A's node, holding the purged writer in either order, or the
    // original content first and the purge after, or the purge first
    // and a stale copy after, stores nothing that confirms the guess.
    let orders: [(&str, Vec<&Segment>); 4] = [
        ("ab", served.iter().chain(&f.b_segments).collect()),
        ("ba", f.b_segments.iter().chain(&served).collect()),
        (
            "late",
            originals
                .iter()
                .chain(&f.b_segments)
                .chain(&served)
                .collect(),
        ),
        (
            "stale",
            served
                .iter()
                .chain(&f.b_segments)
                .chain(&originals)
                .collect(),
        ),
    ];
    let mut indexes = Vec::new();
    for (name, order) in orders {
        let n = node(&format!("purge-a-{name}"), &[a.as_str()], order);
        assert_eq!(
            stored_confirmations(&n, &guess),
            Vec::<String>::new(),
            "order {name}"
        );
        assert_eq!(
            index::phrase_matches(&n.rebuild().unwrap(), &phrase).unwrap(),
            vec![],
            "order {name}"
        );
        assert_eq!(
            room::derive(&n),
            room_before,
            "order {name}: room state as before"
        );
        indexes.push(index_bytes(&n));
    }
    assert!(
        indexes.windows(2).all(|w| w[0] == w[1]),
        "one index whatever the order"
    );

    // B's node keeps its copy until its principal applies the erasure
    // request (PEER-11); after that, a stale copy does not bring it back.
    for s in &served {
        node_b.receive(s).unwrap();
    }
    assert!(!stored_confirmations(&node_b, &guess).is_empty());
    node_b.apply_erasure_request(&tombstone).unwrap();
    assert_eq!(stored_confirmations(&node_b, &guess), Vec::<String>::new());
    for s in &originals {
        node_b.receive(s).unwrap();
    }
    assert_eq!(stored_confirmations(&node_b, &guess), Vec::<String>::new());
    assert_eq!(room::derive(&node_b), room_before);
}
