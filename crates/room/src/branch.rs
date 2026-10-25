//! Branch links (LANE-01, LANE-31). A branch is identified by its
//! repository's identity, its remote URL and its name; a branch with no
//! remote has a provisional, node-local identity that a push rebinds
//! without rewriting the record. A room names a branch by a branch link,
//! an act. When links from this node's principal's rooms name one
//! branch, the first in causal order stands, concurrent ones by the
//! lower commitment; every other is void and shown, and each resolved
//! conflict is recorded with both acts (I6). A link from a foreign room
//! voids none and is never void by this rule.
//!
//! Causal order is the caller's: a relation over two links, taken from
//! the acts' `supersedes` and the writers' chains, never a clock.

use std::collections::BTreeMap;

use provenance::structural_field::{DEFAULT_LIMIT, sanitize};
use record::digest::Digest;
use record::id::{Address, RoomId};

/// A branch: its repository's identity, its remote URL, if it has one,
/// and its name, sanitized per LMK-03.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BranchIdentity {
    repository_identity: String,
    remote_url: Option<String>,
    name: String,
}

impl BranchIdentity {
    /// The branch `name` of the repository `repository_identity`, at
    /// `remote_url` if it has a remote. The name is sanitized here, the
    /// one place untrusted input becomes a branch's name.
    #[must_use]
    pub fn new(repository_identity: &str, remote_url: Option<&str>, name: &str) -> Self {
        Self {
            repository_identity: repository_identity.to_owned(),
            remote_url: remote_url.map(str::to_owned),
            name: sanitize(name, DEFAULT_LIMIT).as_str().to_owned(),
        }
    }

    /// Its repository's identity.
    #[must_use]
    pub fn repository_identity(&self) -> &str {
        &self.repository_identity
    }

    /// Its remote URL, if it has a remote.
    #[must_use]
    pub fn remote_url(&self) -> Option<&str> {
        self.remote_url.as_deref()
    }

    /// Its sanitized name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether it has no remote, so its identity is node-local until a
    /// push rebinds it.
    #[must_use]
    pub fn is_provisional(&self) -> bool {
        self.remote_url.is_none()
    }
}

/// A room's act naming a branch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BranchLink {
    /// The branch it names.
    pub branch: BranchIdentity,
    /// The room that names it.
    pub room: RoomId,
    /// The act's address.
    pub act: Address,
    /// The commitment of the act's event.
    pub commitment: Digest,
    /// The act it supersedes, if any.
    pub supersedes: Option<Address>,
    /// Whether its room is not one of this node's principal's rooms.
    pub foreign: bool,
}

/// How a void link lost to the standing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loss {
    /// The standing link is before it in causal order.
    Ordered,
    /// Concurrent with it, and the standing link has the lower
    /// commitment.
    Concurrent,
}

/// A resolved conflict, with both acts (I6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conflict<'a> {
    /// The link that stands.
    pub standing: &'a BranchLink,
    /// The link made void.
    pub void: &'a BranchLink,
    /// Why it lost.
    pub loss: Loss,
}

/// Which link names a branch, and which are void.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution<'a> {
    /// The link that stands, if any link of this node's principal's
    /// rooms names the branch.
    pub standing: Option<&'a BranchLink>,
    /// Every other link of those rooms, by act address.
    pub void: Vec<&'a BranchLink>,
    /// One conflict per void link, in the same order.
    pub conflicts: Vec<Conflict<'a>>,
}

/// The link that stands among `links`, all of which name one branch:
/// the first in causal order (`causal(a, b)` says `a` is before `b`),
/// concurrent ones by the lower commitment. Each other link of a room
/// of this node's principal is void and listed; a foreign link is
/// neither standing nor void. A duplicate (one act address) counts
/// once. The result is a function of the set of links, whatever order
/// they are given in.
#[must_use]
pub fn standing_link<'a>(
    links: &'a [BranchLink],
    causal: &dyn Fn(&BranchLink, &BranchLink) -> bool,
) -> Resolution<'a> {
    resolve(links.iter(), causal)
}

/// [`standing_link`] over those of `links` that name `branch`.
#[must_use]
pub fn standing_link_for<'a>(
    links: &'a [BranchLink],
    branch: &BranchIdentity,
    causal: &dyn Fn(&BranchLink, &BranchLink) -> bool,
) -> Resolution<'a> {
    resolve(links.iter().filter(|l| &l.branch == branch), causal)
}

/// The resolution of `links`, which all name one branch.
pub(crate) fn resolve<'a>(
    links: impl Iterator<Item = &'a BranchLink>,
    causal: &dyn Fn(&BranchLink, &BranchLink) -> bool,
) -> Resolution<'a> {
    let candidates = candidates(links);
    let Some(standing) = first(&candidates, causal) else {
        return Resolution {
            standing: None,
            void: Vec::new(),
            conflicts: Vec::new(),
        };
    };
    let void: Vec<&BranchLink> = candidates
        .iter()
        .copied()
        .filter(|l| l.act != standing.act)
        .collect();
    let conflicts = void
        .iter()
        .map(|&lost| Conflict {
            standing,
            void: lost,
            loss: if causal(standing, lost) {
                Loss::Ordered
            } else {
                Loss::Concurrent
            },
        })
        .collect();
    Resolution {
        standing: Some(standing),
        void,
        conflicts,
    }
}

/// The links of this node's principal's rooms, one per act address, by
/// address. Two links at one address are a fork (PEER-10); the lesser
/// of them stands in for it, so the choice does not follow arrival.
fn candidates<'a>(links: impl Iterator<Item = &'a BranchLink>) -> Vec<&'a BranchLink> {
    let mut by_act: BTreeMap<Address, &'a BranchLink> = BTreeMap::new();
    for link in links {
        let kept = by_act.entry(link.act).or_insert(link);
        if link < *kept {
            *kept = link;
        }
    }
    by_act.into_values().filter(|l| !l.foreign).collect()
}

/// The link that stands among `candidates`: of those no other is
/// before, the lowest commitment, then the lowest act address. A
/// relation with no such link (a cycle) leaves all of them to choose
/// from, so the result is always defined.
fn first<'a>(
    candidates: &[&'a BranchLink],
    causal: &dyn Fn(&BranchLink, &BranchLink) -> bool,
) -> Option<&'a BranchLink> {
    let unpreceded =
        |c: &&&'a BranchLink| !candidates.iter().any(|d| d.act != c.act && causal(d, c));
    let mut firsts: Vec<&'a BranchLink> = candidates.iter().filter(unpreceded).copied().collect();
    if firsts.is_empty() {
        firsts = candidates.to_vec();
    }
    firsts.into_iter().min_by_key(|l| (l.commitment, l.act))
}

#[cfg(test)]
pub(crate) mod tests;
