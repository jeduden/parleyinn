//! `review-run`: the review of a pull request run by code (ENG-28). It
//! reads the dispatch the review bundle wrote, runs every specialist
//! it names as an agent of its own, in parallel up to a bound and each
//! with its own deadline and cost cap, assembles each one's report from
//! its run log into a file, and runs each generalist once its own
//! specialists have finished, told those files; it starts no run once
//! the review's cost cap is reached, and no generalist once a mass
//! failure of the specialists tripped its breaker, and tells what is
//! running as it goes. From what the runs produced it writes
//! the review outcome, the agents consulted, their answers and their
//! usage, the files the review gate reads, with every agent left
//! without an accepted report or answer, never as clean, and what the
//! review ran on. Each agent has an answer key over what its answer is
//! a function of; given an answer store, an agent whose key holds an
//! accepted answer there is not run again, and an accepted answer is
//! kept there. Resumed from an earlier run, it runs again only what that
//! run left without one or whose key changed, and decides as a full run
//! would. No model decides whether an agent is called or
//! waited for; the generalists do the weighing.
//!
//! It starts only the `claude` command line, through the evaluation's
//! host, and runs in CI alone. It is repository tooling and never
//! links into a shipped executable.

pub mod answers;
pub mod backoff;
pub mod breaker;
pub mod cli;
pub mod controls;
pub mod findings;
pub mod gaps;
pub mod key;
pub mod leak;
pub mod live;
pub(crate) mod memo;
pub mod outcome;
pub mod parts;
pub mod pipeline;
pub mod plan;
pub mod previous;
pub mod requirements;
pub mod resume;
pub mod retry;
pub mod run;
pub mod run_id;
pub mod spend;
pub mod stall;
pub mod store;
pub(crate) mod tasks;
pub mod usage;

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "a host double that cannot write its files fails the test"
)]
mod fake;

/// Where, under the work directory, the review keeps what it runs on
/// and what it writes.
pub const REVIEW: &str = review_eval::world::REVIEW;
/// The specialists' reports, one `<name>.md` each, assembled from the
/// run logs.
pub const REPORTS: &str = review_eval::world::REPORTS;
/// Each run's stream-json log, `<name>.jsonl`, and its errors,
/// `<name>.stderr`; a first attempt's that did not answer as
/// `<name>.first.jsonl` and `<name>.first.stderr`.
pub const LOGS: &str = "review/logs";
/// The review outcome, in the shape the review gate reads.
pub const OUTCOME: &str = "review/outcome.json";
/// The agents whose run answered without an error.
pub const CONSULTED: &str = "review/consulted.json";
/// Each answering agent's final answer, by name.
pub const ANSWERS: &str = "review/answers.json";
/// Each agent's usage and cost, from its run log.
pub const USAGE: &str = "review/usage.json";
/// The review class and the touched ids the review bundle printed,
/// `key=value` per line, as the workflow saves its output.
pub const CLASS: &str = "class.txt";
