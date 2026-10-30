//! `review-eval`: the evaluation of the review's own reviewers
//! (ENG-32). Every reviewer component of the layered review (ENG-28),
//! the orchestrator's skill, each generalist with its deep instance and
//! each specialist the review bundle can dispatch, has cases: a change
//! to main's tree with the report the component must give. A case runs
//! through the review's own harness, `claude --agent` with the review's
//! tools, write rule and turn budget, on the model the component names
//! or one a comparison names, and a grader reads the report as data.
//! The scores fold into a scorecard per component and model, which a
//! pull request's review weighs as deterministic findings and a
//! scheduled run compares with the scorecard the repository publishes.
//!
//! It is repository tooling and never links into a shipped executable.

pub mod case;
pub mod check;
pub mod cli;
pub mod compare;
pub mod component;
pub mod derived;
pub mod diff;
pub mod grade;
pub mod host;
pub mod log;
pub mod plan;
pub mod process;
pub mod published;
pub mod report;
pub mod runner;
pub mod score;
pub mod world;

/// Where the written cases live, relative to the checkout.
pub const CASES: &str = "tooling/review-eval/cases";
/// The scorecard the repository publishes, relative to the checkout.
pub const PUBLISHED: &str = "tooling/review-eval/scorecard-v1.txt";
/// The directory the reviewer components are read from, relative to
/// the tree under evaluation.
pub const INSTRUCTIONS: &str = ".claude";
/// How often a case runs at most.
pub const REPETITIONS: usize = 3;
/// How many of its runs a case must pass.
pub const PASSES: usize = 2;
/// How long one run may take before it is killed and counted as a run
/// that did not answer.
pub const DEADLINE_SECONDS: u64 = 900;
