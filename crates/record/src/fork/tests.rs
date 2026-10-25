use super::*;
use crate::commitment::Commitment;
use crate::event::{EventKind, Provenance};
use crate::header::EventHeader;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

/// The chained events `from` to `to` of `w` on top of `out`, their
/// commitments depending on `tag`, so two tags give two histories.
fn extend(mut out: Vec<Link>, w: WriterId, from: u64, to: u64, tag: u8) -> Vec<Link> {
    for n in from..=to {
        let header = EventHeader {
            address: Address::new(w, seq(n)),
            kind: EventKind::UnparsedLine,
            provenance: Provenance::Unparsed,
            prev_hash: out.last().map_or(Digest::ZERO, |l| l.hash),
            heads: Vec::new(),
            commitment: Commitment::from_digest(Digest::of(&[&[tag, n as u8]])),
        };
        let hash = header.hash();
        out.push(Link { header, hash });
    }
    out
}

fn log(w: WriterId, count: u64, tag: u8) -> Vec<Link> {
    extend(Vec::new(), w, 1, count, tag)
}

/// `count` events of `w` that agree with the log tagged `0` up to
/// `shared`, and differ after.
fn diverging(w: WriterId, count: u64, shared: u64) -> Vec<Link> {
    extend(log(w, shared, 0), w, shared + 1, count, 1)
}

#[test]
fn the_same_history_twice_holds_no_fork() {
    let held = log(writer(1), 4, 0);

    assert_eq!(find(&held, &held), vec![]);
}

#[test]
fn a_longer_copy_of_the_same_history_holds_no_fork() {
    let held = log(writer(1), 2, 0);
    let offered = log(writer(1), 5, 0);

    assert_eq!(find(&held, &offered), vec![]);
    assert_eq!(find(&offered, &held), vec![]);
}

#[test]
fn two_histories_that_part_at_one_seq_fork_from_there_to_the_shorter_end() {
    let held = log(writer(1), 5, 0);
    let offered = diverging(writer(1), 4, 2);

    let forks = find(&held, &offered);

    assert_eq!(
        forks,
        vec![Fork {
            writer: writer(1),
            first: seq(3),
            last: seq(4),
            held: held[2].hash,
            offered: offered[2].hash,
        }]
    );
    assert_ne!(forks[0].held, forks[0].offered);
}

#[test]
fn a_fork_found_newest_first_still_starts_at_its_first_seq() {
    let held = log(writer(1), 5, 0);
    let mut offered = diverging(writer(1), 4, 2);
    offered.reverse();

    let forks = find(&held, &offered);

    assert_eq!(forks.len(), 1);
    assert_eq!((forks[0].first, forks[0].last), (seq(3), seq(4)));
    assert_eq!(forks[0].held, held[2].hash);
    assert_eq!(forks[0].offered, offered[1].hash);
}

#[test]
fn a_fork_is_reported_per_writer_in_writer_order() {
    let (a, b, c) = (writer(1), writer(2), writer(3));
    let held: Vec<Link> = [log(c, 2, 0), log(a, 2, 0), log(b, 2, 0)].concat();
    let offered: Vec<Link> = [log(b, 2, 7), log(a, 2, 0), log(c, 2, 7)].concat();

    let forks = find(&held, &offered);

    let writers: Vec<WriterId> = forks.iter().map(|f| f.writer).collect();
    let mut sorted = vec![b, c];
    sorted.sort();
    assert_eq!(writers, sorted);
    assert!(forks.iter().all(|f| f.first == seq(1) && f.last == seq(2)));
}

#[test]
fn an_event_the_node_does_not_hold_is_no_fork() {
    let offered = log(writer(1), 3, 0);

    assert_eq!(find(&[], &offered), vec![]);
}

#[test]
fn a_fork_names_the_writer_range_and_digests_never_content() {
    let fork = find(&log(writer(1), 2, 0), &log(writer(1), 2, 7)).remove(0);

    assert_eq!(
        fork.to_string(),
        format!(
            "writer {} holds two events at seq 1 to 2: {} held, {} offered",
            writer(1),
            fork.held,
            fork.offered
        )
    );
    let single = find(&log(writer(1), 1, 0), &log(writer(1), 1, 7)).remove(0);
    assert!(single.to_string().contains("at seq 1:"));
}
