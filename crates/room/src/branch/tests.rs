use record::id::{Seq, WriterId};

use super::*;

pub(crate) fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

pub(crate) fn room(n: u8) -> RoomId {
    RoomId::derive(&[n; 32], &[n; 16])
}

pub(crate) fn branch(name: &str) -> BranchIdentity {
    BranchIdentity::new("repo-1", Some("https://example.test/r.git"), name)
}

/// A link to `main` from room `room_n`, act (writer `w`, seq `seq`),
/// with the commitment whose first byte is `commit`.
pub(crate) fn link(room_n: u8, w: u8, seq: u64, commit: u8) -> BranchLink {
    BranchLink {
        branch: branch("main"),
        room: room(room_n),
        act: Address::new(writer(w), Seq::new(seq).unwrap()),
        commitment: Digest::from_bytes([commit; 32]),
        supersedes: None,
        foreign: false,
    }
}

/// Causal order within one writer: an earlier seq is before a later.
pub(crate) fn chain(a: &BranchLink, b: &BranchLink) -> bool {
    a.act.writer == b.act.writer && a.act.seq < b.act.seq
}

#[test]
fn a_name_is_sanitized_at_construction() {
    let kept = BranchIdentity::new("r", None, "feature/x-1");
    assert_eq!(kept.name(), "feature/x-1");
    let replaced = BranchIdentity::new("r", None, "feat ure`x`");
    assert!(replaced.name().starts_with("<field:") && replaced.name().ends_with('>'));
    assert_eq!(
        replaced.name(),
        provenance::structural_field::sanitize("feat ure`x`", 80).as_str()
    );
}

#[test]
fn a_branch_without_a_remote_is_provisional() {
    let local = BranchIdentity::new("r", None, "main");
    assert!(local.is_provisional());
    assert_eq!(local.remote_url(), None);
    let pushed = BranchIdentity::new("r", Some("u"), "main");
    assert!(!pushed.is_provisional());
    assert_eq!(pushed.remote_url(), Some("u"));
    assert_eq!(pushed.repository_identity(), "r");
}

#[test]
fn no_links_resolve_to_nothing() {
    let r = standing_link(&[], &chain);
    assert_eq!(r.standing, None);
    assert!(r.void.is_empty() && r.conflicts.is_empty());
}

#[test]
fn one_link_stands_alone() {
    let links = [link(1, 1, 1, 9)];
    let r = standing_link(&links, &chain);
    assert_eq!(r.standing, Some(&links[0]));
    assert!(r.void.is_empty() && r.conflicts.is_empty());
}

#[test]
fn the_first_in_causal_order_stands_whatever_the_commitment() {
    let first = link(1, 1, 1, 200);
    let second = link(2, 1, 2, 1);
    let links = [second.clone(), first.clone()];
    let r = standing_link(&links, &chain);
    assert_eq!(r.standing, Some(&links[1]));
    assert_eq!(r.void, vec![&links[0]]);
    assert_eq!(
        r.conflicts,
        vec![Conflict {
            standing: &links[1],
            void: &links[0],
            loss: Loss::Ordered
        }]
    );
}

#[test]
fn of_concurrent_links_the_lower_commitment_stands() {
    let a = link(1, 1, 1, 7);
    let b = link(2, 2, 1, 3);
    let links = [a, b];
    let r = standing_link(&links, &chain);
    assert_eq!(r.standing, Some(&links[1]));
    assert_eq!(r.void, vec![&links[0]]);
    assert_eq!(r.conflicts[0].loss, Loss::Concurrent);
    assert_eq!(r.conflicts[0].standing, &links[1]);
    assert_eq!(r.conflicts[0].void, &links[0]);
}

#[test]
fn a_link_before_one_concurrent_pair_member_does_not_beat_the_lower_commitment() {
    // a1 < a2 (one writer); b concurrent with both. Minimal: a1 and b.
    let a1 = link(1, 1, 1, 50);
    let a2 = link(1, 1, 2, 1);
    let b = link(2, 2, 1, 20);
    let links = [a2, b, a1];
    let r = standing_link(&links, &chain);
    assert_eq!(r.standing, Some(&links[1]));
    assert_eq!(r.void.len(), 2);
    // a2 loses to b only by being concurrent with it; a1 likewise.
    assert!(r.conflicts.iter().all(|c| c.standing == &links[1]));
}

#[test]
fn equal_commitments_fall_back_to_the_lower_act_address() {
    let a = link(1, 1, 1, 5);
    let b = link(2, 2, 1, 5);
    let links = [b.clone(), a.clone()];
    let r = standing_link(&links, &chain);
    let expected = if a.act < b.act { &a } else { &b };
    assert_eq!(r.standing, Some(expected));
}

#[test]
fn a_foreign_link_voids_none_and_is_never_void() {
    let mut foreign = link(3, 3, 1, 0);
    foreign.foreign = true;
    let mine = link(1, 1, 1, 200);
    let links = [foreign, mine];
    let r = standing_link(&links, &chain);
    assert_eq!(r.standing, Some(&links[1]));
    assert!(r.void.is_empty() && r.conflicts.is_empty());
}

#[test]
fn only_foreign_links_leave_no_standing_link() {
    let mut foreign = link(3, 3, 1, 0);
    foreign.foreign = true;
    let links = [foreign];
    let r = standing_link(&links, &chain);
    assert_eq!(r.standing, None);
    assert!(r.void.is_empty());
}

#[test]
fn a_duplicate_act_counts_once() {
    let a = link(1, 1, 1, 5);
    let links = [a.clone(), a.clone(), a];
    let r = standing_link(&links, &chain);
    assert!(r.standing.is_some());
    assert!(r.void.is_empty() && r.conflicts.is_empty());
}

#[test]
fn one_address_with_two_contents_resolves_by_the_lower_one_whatever_the_order() {
    let a = link(1, 1, 1, 5);
    let mut b = a.clone();
    b.commitment = Digest::from_bytes([4; 32]);
    let forward = [a.clone(), b.clone()];
    let backward = [b.clone(), a];
    assert_eq!(standing_link(&forward, &chain).standing, Some(&forward[1]));
    assert_eq!(
        standing_link(&backward, &chain).standing,
        Some(&backward[0])
    );
}

#[test]
fn a_relation_with_no_first_link_still_resolves() {
    let a = link(1, 1, 1, 9);
    let b = link(2, 2, 1, 2);
    let links = [a, b];
    let cycle = |_: &BranchLink, _: &BranchLink| true;
    let r = standing_link(&links, &cycle);
    assert_eq!(r.standing, Some(&links[1]));
    assert_eq!(r.void, vec![&links[0]]);
}

#[test]
fn void_links_are_listed_by_act_address() {
    let links = [link(1, 1, 3, 9), link(1, 1, 1, 1), link(1, 1, 2, 9)];
    let r = standing_link(&links, &chain);
    assert_eq!(r.standing, Some(&links[1]));
    assert_eq!(r.void, vec![&links[2], &links[0]]);
    assert_eq!(r.conflicts.len(), 2);
}

#[test]
fn only_links_naming_the_branch_are_considered() {
    let mut other = link(2, 2, 1, 0);
    other.branch = branch("other");
    let mine = link(1, 1, 1, 200);
    let links = [other, mine];
    let r = standing_link_for(&links, &branch("main"), &chain);
    assert_eq!(r.standing, Some(&links[1]));
    assert!(r.void.is_empty());
    let none = standing_link_for(&links, &branch("absent"), &chain);
    assert_eq!(none.standing, None);
}
