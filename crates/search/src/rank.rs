//! Ranking the hits of a search (RCL-02): the best `k` of the newest
//! matches, each already scored by BM25 with length normalization, with
//! each further hit of one writer damped so a writer that matches over and
//! over cannot crowd out the others. The order is a function of the
//! matches alone: ties go to the smaller address, whatever order the
//! matches came in (I10).

use std::collections::BTreeMap;

use record::id::{Address, WriterId};

/// The most hits a search returns, RCL-02's maximum.
pub const MAX_HITS: usize = 50;

/// How much each further hit of one writer is damped: its relevance is
/// divided by one plus this times the hits already taken from the writer.
pub const DAMPING: f64 = 0.5;

/// One of the newest matches, before damping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    pub address: Address,
    /// BM25 relevance, higher the better.
    pub relevance: f64,
}

/// One hit: an event and its damped relevance, higher the better.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub address: Address,
    pub score: f64,
}

/// The `k` best of `candidates`, at most [`MAX_HITS`], each further hit
/// of one writer damped by [`DAMPING`].
#[must_use]
pub fn rank(candidates: &[Candidate], k: usize) -> Vec<Hit> {
    let mut taken: BTreeMap<WriterId, u32> = BTreeMap::new();
    let mut left: Vec<&Candidate> = candidates.iter().collect();
    let mut hits = Vec::new();
    while hits.len() < k.min(MAX_HITS) {
        let damped = |c: &Candidate| {
            let before = f64::from(taken.get(&c.address.writer).copied().unwrap_or(0));
            c.relevance / (1.0 + DAMPING * before)
        };
        let best = left.iter().enumerate().max_by(|(_, a), (_, b)| {
            damped(a)
                .total_cmp(&damped(b))
                .then_with(|| b.address.cmp(&a.address))
        });
        let Some((at, _)) = best else { break };
        let chosen = left.swap_remove(at);
        hits.push(Hit {
            address: chosen.address,
            score: damped(chosen),
        });
        *taken.entry(chosen.address.writer).or_default() += 1;
    }

    hits
}

#[cfg(test)]
mod tests;
