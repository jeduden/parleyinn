//! Measures line coverage of the workspace per test layer (unit,
//! integration and end-to-end) and per crate, and holds each crate to
//! the floors in `[workspace.metadata.coverage]` (ENG-11). It runs
//! cargo-llvm-cov once per test layer over one instrumented build, so
//! each layer's number counts only what that layer's tests ran, and
//! adds the layers up for the whole. It is repository tooling and never
//! links into a shipped executable.

pub mod cli;
mod host;
pub mod lcov;
pub mod measure;
pub mod plan;
pub mod report;
pub mod shape;

use std::path::{Path, PathBuf};

pub use host::Cargo;

/// The variable every cargo a measurement starts carries. A `coverage`
/// run that finds it set is running inside a measurement, a test
/// started by one, and refuses: measuring again would recurse.
pub const MEASURING: &str = "PARINN_COVERAGE_MEASURING";

/// What a measurement does to the machine: run cargo and write files.
/// The tests put a fake in its place.
pub trait Host {
    /// Runs cargo with `args`, its output passed through.
    ///
    /// # Errors
    ///
    /// Fails when cargo cannot start or exits unsuccessfully.
    fn cargo(&self, args: &[String]) -> Result<(), String>;

    /// Runs cargo with `args` and returns what it prints on stdout.
    ///
    /// # Errors
    ///
    /// Fails when cargo cannot start or exits unsuccessfully.
    fn cargo_output(&self, args: &[String]) -> Result<String, String>;

    /// Writes `text` to `path`, creating its directory.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be written.
    fn write(&self, path: &Path, text: &str) -> Result<(), String>;

    /// Appends `text` to `path`, creating it if need be.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be written.
    fn append(&self, path: &Path, text: &str) -> Result<(), String>;

    /// Every Rust source file under `dir`, with its text, skipping
    /// build output.
    ///
    /// # Errors
    ///
    /// Fails when a directory or file cannot be read.
    fn sources(&self, dir: &Path) -> Result<Vec<(PathBuf, String)>, String>;

    /// How many scenarios under `features` are bound, not pending.
    ///
    /// # Errors
    ///
    /// Fails when the features cannot be read.
    fn bound_scenarios(&self, features: &Path) -> Result<usize, String>;
}

#[cfg(test)]
mod tests;
