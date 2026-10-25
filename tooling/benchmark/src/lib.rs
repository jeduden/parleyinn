//! The benchmark suite and its gate (ENG-15). ENG-15 asks for a
//! benchmark suite covering NFR-01..05 and a CI gate that fails on a
//! statistically significant regression greater than 10%.
//!
//! - [`suite`] registers each benchmark against the NFR id it measures
//!   toward, and declares each NFR id ENG-15 names that has no
//!   benchmark of real code yet, with what it waits on.
//! - [`fixture`] makes what a benchmark's core runs over: a fresh home
//!   holding a transcript, for ingestion.
//! - [`measure`] times one sample of a benchmark.
//! - [`samples`] is the file format the samples travel in, one sample
//!   per line.
//! - [`stats`] and [`gate`] decide, from the base's samples and the
//!   head's, whether the head regressed.
//! - [`cli`] is the `benchmark` executable the CI job runs: `run`
//!   samples the suite, `gate` decides.
//!
//! It is repository tooling and never links into a shipped executable.

pub mod accepted;
pub mod cli;
pub mod fixture;
pub mod gate;
pub mod measure;
pub mod samples;
pub mod stats;
pub mod suite;
