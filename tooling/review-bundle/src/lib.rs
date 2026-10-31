//! Builds what the review workflow hands the reviewing agent, before it
//! runs (ENG-21, ENG-28): a review bundle of only the requirement rows,
//! scenarios and invariants the pull request touches, so the agent
//! reads a few thousand model tokens instead of browsing the SRS; the
//! review class that sets how deep the review goes; and the findings deterministic
//! checks make on their own, which the review gate weighs beside the
//! agent's. It reads main's tree and the pull request's tree as data
//! and runs nothing from either. It is repository tooling and never
//! links into a shipped executable.

pub mod analysis;
pub mod cli;
pub mod copies;
pub mod diff;
pub mod dispatch;
pub mod owners;
pub mod render;
pub mod snapshot;

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "a fixture that cannot be written fails the test"
)]
mod fixture;

use serde::Serialize;

pub use analysis::{Analysis, Change, Class, analyse};
pub use snapshot::Snapshot;

/// A finding withholds approval.
pub const BLOCKING: &str = "blocking";
/// A finding is reported and withholds nothing.
pub const NIT: &str = "nit";

/// One deterministic finding, in the shape of the review outcome's
/// findings, so the review gate reads both alike. Line 0 means the
/// whole file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Finding {
    pub path: String,
    pub line: usize,
    pub severity: &'static str,
    pub body: String,
}
