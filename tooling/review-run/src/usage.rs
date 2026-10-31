//! What each agent's run cost, from its log, in the shape the review
//! workflow's cost step writes: one entry per agent with its model
//! calls, fresh input, cache writes and reads and output, summed over
//! the models it ran on, the most cached first; with the cost its
//! result reported.

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
    /// How many times it ran.
    pub attempts: u32,
    /// The cost the run's result reported, in US dollars.
    pub cost_usd: f64,
}

/// Each run's usage, the most cached first, then by name.
#[must_use]
pub fn usage(runs: &[AgentRun]) -> Vec<AgentUsage> {
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
                cost_usd: run.log.cost,
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
