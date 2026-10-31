//! `review-run`: the review of a pull request run by code (ENG-28). It
//! reads the dispatch the review bundle wrote, runs every specialist
//! it names as an agent of its own, in parallel up to a bound and each
//! with its own deadline, then, once every specialist has finished,
//! every generalist, told its specialists' report files. From what the
//! runs produced it writes the review outcome, the agents consulted,
//! their answers and their usage, the files the review gate reads. No
//! model decides whether an agent is called or waited for; the
//! generalists do the weighing.
//!
//! It starts only the `claude` command line, through the evaluation's
//! host, and runs in CI alone. It is repository tooling and never
//! links into a shipped executable.

pub mod cli;
pub mod outcome;
pub mod plan;
pub mod run;
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
/// The specialists' reports, one `<name>.md` each.
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
