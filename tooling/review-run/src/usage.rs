//! What each agent's run cost, from its log, in the shape the review
//! workflow's cost step writes: one entry per agent with its model
//! calls, fresh input, cache writes and reads and output, summed over
//! the models it ran on, the most cached first; with the parts it ran
//! in, a specialist run in parts being one entry, and the cost its
//! results reported, and its answer key. An agent a resumed run reused
//! from an earlier run, or one whose answer the answer store held, cost
//! nothing this time: its entry says so, and names that run or the
//! store.

use serde::Serialize;

use crate::run::AgentRun;

/// One agent's usage.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgentUsage {
    pub agent: String,
    pub calls: u64,
    pub fresh: u64,
    pub cache_write: u64,
    pub cache_read: u64,
    pub output: u64,
    /// How many times it ran: for a specialist run in parts, every
    /// part's attempts.
    pub attempts: u32,
    /// How many parts it ran in: one unless its diff was split.
    pub parts: u32,
    /// The cost the run's result reported, in US dollars; none for an
    /// agent reused.
    pub cost_usd: f64,
    /// Whether its answer was reused from an earlier run; absent when not.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub reused: bool,
    /// The earlier run's directory it was reused from; absent when not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resumed_from: Option<String>,
    /// Its answer key; absent when it has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer_key: Option<String>,
    /// Whether its answer was reused from the answer store; absent when
    /// not.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub answer_store: bool,
}

/// Each run's usage, the most cached first, then by name; a run reused
/// from the earlier run at `resumed_from` costing nothing.
#[must_use]
pub fn usage(runs: &[AgentRun], resumed_from: Option<&str>) -> Vec<AgentUsage> {
    let mut out: Vec<AgentUsage> = runs
        .iter()
        .map(|run| {
            let mut u = AgentUsage {
                agent: run.name.clone(),
                calls: 0,
                fresh: 0,
                cache_write: 0,
                cache_read: 0,
                output: 0,
                attempts: run.attempts,
                parts: run.parts,
                cost_usd: if run.reused { 0.0 } else { run.log.cost },
                reused: run.reused,
                resumed_from: resumed_from
                    .filter(|_| run.reused && !run.from_store)
                    .map(str::to_owned),
                answer_key: run.answer_key.clone(),
                answer_store: run.from_store,
            };
            for m in run.log.usage.values() {
                u.calls += m.calls;
                u.fresh += m.fresh;
                u.cache_write += m.cache_write;
                u.cache_read += m.cache_read;
                u.output += m.output;
            }
            u
        })
        .collect();
    out.sort_by(|a, b| {
        (b.cache_write + b.cache_read)
            .cmp(&(a.cache_write + a.cache_read))
            .then_with(|| a.agent.cmp(&b.agent))
    });

    out
}

/// The total cost of `usage`, in US dollars.
#[must_use]
pub fn total(usage: &[AgentUsage]) -> f64 {
    usage.iter().map(|u| u.cost_usd).sum()
}

#[cfg(test)]
mod tests;
