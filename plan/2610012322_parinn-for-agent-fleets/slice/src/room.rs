//! Room state from acts: branch links, merged by LANE-31's rule with no
//! clock. Of two branch links naming one branch, the first in causal
//! order stands; of concurrent ones, the one with the lower commitment.
//! Causal order comes only from the chain heads each header names as
//! seen (REC-10), so it is the same on every node holding the same
//! writers.

use std::collections::{BTreeMap, HashMap};

use crate::commit::Commitment;
use crate::node::Node;
use crate::record::{Address, Event};

/// A standing branch link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchLink {
    pub room: String,
    pub repository: String,
    pub branch: String,
    pub act: Address,
    pub commitment: Commitment,
}

/// A resolved conflict, recorded with both acts (LANE-31).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub stands: Address,
    pub void: Address,
    pub rule: &'static str,
}

pub const FIRST_IN_CAUSAL_ORDER: &str = "first in causal order";
pub const LOWER_COMMITMENT: &str = "lower commitment";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoomState {
    pub links: Vec<BranchLink>,
    pub conflicts: Vec<Conflict>,
}

/// Each held event's causal past: for every writer, the highest seq of
/// it this event follows. No clock is read.
pub struct CausalOrder {
    writers: Vec<String>,
    pasts: HashMap<Address, Vec<u64>>,
}

impl CausalOrder {
    pub fn of(node: &Node) -> CausalOrder {
        let mut writers: Vec<String> = node
            .events()
            .iter()
            .map(|e| e.header.writer.clone())
            .collect();
        writers.dedup();
        let at = |w: &str| writers.iter().position(|x| x == w);
        let mut pasts: HashMap<Address, Vec<u64>> = HashMap::new();
        let mut pending: Vec<Vec<&Event>> = writers
            .iter()
            .map(|w| {
                node.events()
                    .into_iter()
                    .filter(|e| &e.header.writer == w)
                    .rev()
                    .collect()
            })
            .collect();
        // Advance each writer while every chain head its next event names
        // as seen is computed or not held; those name only earlier events
        // that ended a closed segment, so this always progresses.
        loop {
            let mut progressed = false;
            for (wi, queue) in pending.iter_mut().enumerate() {
                while let Some(e) = queue.last() {
                    let ready = e.header.chain_heads_seen.iter().all(|(w, h)| {
                        let a = Address {
                            writer: w.clone(),
                            seq: h.seq,
                        };
                        pasts.contains_key(&a) || node.event(&a).is_none()
                    });
                    if !ready {
                        break;
                    }
                    let mut vc = queue_prev(&pasts, e).unwrap_or_else(|| vec![0; writers.len()]);
                    for (w, h) in &e.header.chain_heads_seen {
                        let a = Address {
                            writer: w.clone(),
                            seq: h.seq,
                        };
                        if let Some(other) = pasts.get(&a) {
                            vc.iter_mut()
                                .zip(other)
                                .for_each(|(x, y)| *x = (*x).max(*y));
                        } else if let Some(i) = at(w) {
                            vc[i] = vc[i].max(h.seq);
                        }
                    }
                    vc[wi] = e.header.seq;
                    pasts.insert(e.address(), vc);
                    queue.pop();
                    progressed = true;
                }
            }
            if !progressed {
                break;
            }
        }
        CausalOrder { writers, pasts }
    }

    /// Does `a` come before `b` in causal order?
    pub fn before(&self, a: &Address, b: &Address) -> bool {
        if a.writer == b.writer {
            return a.seq < b.seq;
        }
        let i = self.writers.iter().position(|w| *w == a.writer);
        match (i, self.pasts.get(b)) {
            (Some(i), Some(vc)) => vc[i] >= a.seq,
            _ => false,
        }
    }
}

/// The causal past of the writer's previous held event.
fn queue_prev(pasts: &HashMap<Address, Vec<u64>>, e: &Event) -> Option<Vec<u64>> {
    let prev = Address {
        writer: e.header.writer.clone(),
        seq: e.header.seq.checked_sub(1)?,
    };
    pasts.get(&prev).cloned()
}

/// The (room, repository, branch) a link act adding a branch link
/// names, if `e` is one.
fn branch_link(e: &Event) -> Option<(String, String, String)> {
    let c = &e.body.as_ref()?.content;
    if e.header.kind != "act" || c.get("act")? != "link" || c.get("link")? != "branch" {
        return None;
    }
    let s = |k: &str| c.get(k).and_then(|v| v.as_str()).map(str::to_string);
    Some((s("room")?, s("repository")?, s("branch")?))
}

/// Derives room state from the acts the node holds.
pub fn derive(node: &Node) -> RoomState {
    let mut groups: BTreeMap<(String, String, String), Vec<&Event>> = BTreeMap::new();
    for e in node.events() {
        if let Some(key) = branch_link(e) {
            groups.entry(key).or_default().push(e);
        }
    }
    let order = CausalOrder::of(node);
    let mut state = RoomState::default();
    for ((room, repository, branch), acts) in groups {
        let first: Vec<&Event> = acts
            .iter()
            .copied()
            .filter(|e| {
                !acts
                    .iter()
                    .any(|o| order.before(&o.address(), &e.address()))
            })
            .collect();
        let Some(stands) = first.iter().min_by_key(|e| e.header.commitment) else {
            continue;
        };
        for void in acts.iter().filter(|e| e.hash != stands.hash) {
            let rule = if order.before(&stands.address(), &void.address()) {
                FIRST_IN_CAUSAL_ORDER
            } else {
                LOWER_COMMITMENT
            };
            state.conflicts.push(Conflict {
                stands: stands.address(),
                void: void.address(),
                rule,
            });
        }
        state.links.push(BranchLink {
            room,
            repository,
            branch,
            act: stands.address(),
            commitment: stands.header.commitment,
        });
    }
    state.conflicts.sort_by(|x, y| x.void.cmp(&y.void));
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Segment, Writer};

    #[test]
    fn chain_heads_seen_order_events_across_writers() {
        let mut a = Writer::open(b"a", "interactive").unwrap();
        let mut b = Writer::open(b"b", "interactive").unwrap();
        let sa: Segment = a.close_segment().unwrap();
        let b_before = b
            .append("message", "user", serde_json::json!({"text": "x"}))
            .unwrap();
        b.see(&a);
        let b_after = b
            .append("message", "user", serde_json::json!({"text": "y"}))
            .unwrap();
        let sb = b.close_segment().unwrap();
        let dir = std::env::temp_dir().join("s12-room-tests");
        let _ = std::fs::remove_dir_all(&dir);
        let mut n = Node::new(&dir, &[]).unwrap();
        n.receive(&sb).unwrap();
        n.receive(&sa).unwrap();
        let order = CausalOrder::of(&n);
        let a1 = Address {
            writer: a.id().to_string(),
            seq: 1,
        };
        assert!(order.before(&a1, &b_after));
        assert!(!order.before(&a1, &b_before));
        assert!(!order.before(&b_after, &a1));
        assert!(order.before(&b_before, &b_after));
    }
}
