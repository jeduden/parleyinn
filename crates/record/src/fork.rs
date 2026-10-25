//! Two different events at one (writer, seq) (PEER-10, REC-21): found
//! by comparing the events a node holds with a history offered to it.
//! The comparison is a function of the two histories alone (I10); what
//! holds the first head, and what is refused, is the caller's.

use std::collections::BTreeMap;
use std::fmt;

use crate::chain::Link;
use crate::digest::Digest;
use crate::id::{Address, Seq, WriterId};

/// One writer's conflict: from `first` to `last`, the offered history
/// differs from the held one, and at `first` the held event has the
/// digest `held`, the offered one `offered`. Never any content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fork {
    pub writer: WriterId,
    pub first: Seq,
    pub last: Seq,
    pub held: Digest,
    pub offered: Digest,
}

impl fmt::Display for Fork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let range = if self.first == self.last {
            format!("seq {}", self.first)
        } else {
            format!("seq {} to {}", self.first, self.last)
        };

        write!(
            f,
            "writer {} holds two events at {range}: {} held, {} offered",
            self.writer, self.held, self.offered
        )
    }
}

/// Each writer of `offered` whose events differ from the ones `held`
/// holds at the same addresses, by writer id. An offered event the
/// node does not hold yet is no conflict.
#[must_use]
pub fn find(held: &[Link], offered: &[Link]) -> Vec<Fork> {
    let held: BTreeMap<Address, Digest> = held.iter().map(|l| (l.header.address, l.hash)).collect();
    let mut forks: BTreeMap<WriterId, Fork> = BTreeMap::new();
    for link in offered {
        let address = link.header.address;
        let Some(&had) = held.get(&address).filter(|h| **h != link.hash) else {
            continue;
        };
        let fork = forks.entry(address.writer).or_insert(Fork {
            writer: address.writer,
            first: address.seq,
            last: address.seq,
            held: had,
            offered: link.hash,
        });
        if address.seq < fork.first {
            fork.first = address.seq;
            fork.held = had;
            fork.offered = link.hash;
        }
        fork.last = fork.last.max(address.seq);
    }

    forks.into_values().collect()
}

#[cfg(test)]
mod tests;
