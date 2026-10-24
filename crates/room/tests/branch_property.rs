//! ENG-08 style properties of LANE-31's branch-link rule and LANE-01's
//! routing, over randomly generated link sets: the result is a function
//! of the set of links, whatever order they arrive in and however many
//! times one repeats (I10), and every other link of the same branch is
//! void and shown with both acts (I6). Every case is seeded, so a
//! failure names a seed that reproduces it; `PARINN_PROPERTY_CASES`
//! runs more.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::BTreeSet;

use record::digest::Digest;
use record::id::{Address, RoomId, Seq, WriterId};
use room::branch::{BranchIdentity, BranchLink, Loss, standing_link_for};
use room::route::{RunSeatState, SeatTarget, seat_for_run_event};

/// How many link sets each property is checked over by default.
const CASES: u64 = 64;

/// A small seeded generator (xorshift64*): no clock, no OS randomness.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    /// A Fisher-Yates shuffle.
    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
}

fn cases() -> u64 {
    std::env::var("PARINN_PROPERTY_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(CASES)
}

fn room(n: u8) -> RoomId {
    RoomId::derive(&[n; 32], &[n; 16])
}

fn branch(name: &str) -> BranchIdentity {
    BranchIdentity::new("repo", Some("https://example.test/r.git"), name)
}

/// Causal order within one writer: an earlier seq is before a later.
fn chain(a: &BranchLink, b: &BranchLink) -> bool {
    a.act.writer == b.act.writer && a.act.seq < b.act.seq
}

/// Links to `main` or `side` from 3 rooms (one of them foreign) at
/// distinct acts of 3 writers, with commitments from a small range so
/// ties happen.
fn links(rng: &mut Rng) -> Vec<BranchLink> {
    let mut out = Vec::new();
    for w in 1..=3u8 {
        for seq in 1..=3u64 {
            if rng.below(2) == 0 {
                continue;
            }
            let room_n = 1 + rng.below(3) as u8;
            out.push(BranchLink {
                branch: branch(if rng.below(4) == 0 { "side" } else { "main" }),
                room: room(room_n),
                act: Address::new(WriterId::derive(&[w; 32]), Seq::new(seq).unwrap()),
                commitment: Digest::from_bytes([rng.below(4) as u8; 32]),
                supersedes: None,
                foreign: room_n == 3,
            });
        }
    }
    out
}

/// `links` shuffled, with some repeated.
fn rearranged(rng: &mut Rng, links: &[BranchLink]) -> Vec<BranchLink> {
    let mut all = links.to_vec();
    for link in links {
        if rng.below(3) == 0 {
            all.push(link.clone());
        }
    }
    rng.shuffle(&mut all);
    all
}

fn acts(links: &[&BranchLink]) -> Vec<Address> {
    links.iter().map(|l| l.act).collect()
}

#[test]
fn the_standing_link_is_a_function_of_the_set_of_links() {
    for seed in 0..cases() {
        let mut rng = Rng::new(seed);
        let links = links(&mut rng);
        let again = rearranged(&mut rng, &links);
        let one = standing_link_for(&links, &branch("main"), &chain);
        let two = standing_link_for(&again, &branch("main"), &chain);
        assert_eq!(
            one.standing.map(|l| l.act),
            two.standing.map(|l| l.act),
            "seed {seed}"
        );
        assert_eq!(acts(&one.void), acts(&two.void), "seed {seed}");
        assert_eq!(one.conflicts.len(), two.conflicts.len(), "seed {seed}");
    }
}

#[test]
fn every_other_link_is_void_and_shown_with_both_acts() {
    for seed in 0..cases() {
        let mut rng = Rng::new(seed);
        let links = links(&mut rng);
        let r = standing_link_for(&links, &branch("main"), &chain);
        let mine: BTreeSet<Address> = links
            .iter()
            .filter(|l| l.branch == branch("main") && !l.foreign)
            .map(|l| l.act)
            .collect();
        let mut seen: BTreeSet<Address> = acts(&r.void).into_iter().collect();
        assert_eq!(seen.len(), r.void.len(), "seed {seed}: void lists a repeat");
        seen.extend(r.standing.map(|l| l.act));
        assert_eq!(seen, mine, "seed {seed}: standing and void cover the links");
        assert_eq!(r.conflicts.len(), r.void.len(), "seed {seed}");
        for (conflict, void) in r.conflicts.iter().zip(&r.void) {
            assert_eq!(conflict.void.act, void.act, "seed {seed}");
            assert_eq!(Some(conflict.standing.act), r.standing.map(|l| l.act));
            match conflict.loss {
                Loss::Ordered => assert!(chain(conflict.standing, conflict.void)),
                Loss::Concurrent => {
                    assert!(!chain(conflict.void, conflict.standing), "seed {seed}");
                }
            }
        }
    }
}

#[test]
fn a_foreign_link_changes_nothing() {
    for seed in 0..cases() {
        let mut rng = Rng::new(seed);
        let links = links(&mut rng);
        let without: Vec<BranchLink> = links.iter().filter(|l| !l.foreign).cloned().collect();
        let a = standing_link_for(&links, &branch("main"), &chain);
        let b = standing_link_for(&without, &branch("main"), &chain);
        assert_eq!(a.standing.map(|l| l.act), b.standing.map(|l| l.act));
        assert_eq!(acts(&a.void), acts(&b.void), "seed {seed}");
    }
}

#[test]
fn routing_is_a_function_of_the_set_of_links() {
    for seed in 0..cases() {
        let mut rng = Rng::new(seed);
        let links = links(&mut rng);
        let again = rearranged(&mut rng, &links);
        let rooms = [1, 2].map(|n| RunSeatState {
            room: room(n),
            member_with_work: rng.below(4) != 0,
        });
        let main = branch("main");
        let one = seat_for_run_event(Some(&main), &rooms, &links, &chain);
        let two = seat_for_run_event(Some(&main), &rooms, &again, &chain);
        assert_eq!(one, two, "seed {seed}");
        if let SeatTarget::RunSeat(r) = one {
            let standing = standing_link_for(&links, &main, &chain).standing;
            assert_eq!(standing.map(|l| l.room), Some(r), "seed {seed}");
            assert!(rooms.iter().any(|s| s.room == r && s.member_with_work));
        }
    }
}
