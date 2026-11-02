//! The checks behind the bound scenarios of SRS §10, Engineering
//! quality. Each scenario's steps, in the `scenario` crate's `bdd`
//! target, read the repository checkout and call these. Every check
//! returns its problems, one line each, so a failing step names all of
//! them at once. It is repository tooling and never links into a
//! shipped executable.

pub mod agent_instructions;
pub mod architecture;
pub mod benchmarks;
pub mod boundary;
pub mod branch_protection;
pub mod calibration;
pub mod ci_gates;
pub mod codeowners;
pub mod complexity;
pub mod confinement;
pub mod coverage_floors;
pub mod decisions;
pub mod dependencies;
pub mod file_sizes;
pub mod fuzzing;
pub mod isolation_guard;
pub mod license_texts;
pub mod privacy;
pub mod product_crates;
pub mod registry;
pub mod release_budgets;
pub mod release_provenance;
pub mod review_agents;
pub mod review_answers;
pub mod review_controls;
pub mod review_evaluation;
pub mod review_languages;
pub mod review_leak;
pub mod review_limits;
pub mod review_parts;
pub mod review_previous;
pub mod review_refusals;
pub mod review_resume;
pub mod review_runs;
pub mod review_workflow;
pub mod stability_measurement;
pub mod static_analysis;
pub mod telemetry;
pub mod test_shards;
pub mod third_party;
pub mod threat_model;
pub mod toolchain;
pub mod trusted_text;
pub mod workflows;

use std::fs;
use std::path::{Path, PathBuf};

/// The repository checkout a scenario inspects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkout {
    pub root: PathBuf,
}

impl Checkout {
    /// The checkout at `root`.
    #[must_use]
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    /// Reads `file`, a path relative to the checkout, as written.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be read.
    pub fn read_raw(&self, file: &str) -> Result<String, String> {
        fs::read_to_string(self.root.join(file)).map_err(|e| format!("read {file}: {e}"))
    }

    /// Reads `file`, a path relative to the checkout. The CI workflow
    /// reads as the commands it runs, each call of a gate replaced by
    /// the gate's command from `scripts/ci-gates.txt` (ENG-27), so a
    /// check of what CI runs does not care where the command is
    /// written; [`Checkout::read_raw`] returns it as written.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be read.
    pub fn read(&self, file: &str) -> Result<String, String> {
        let text = self.read_raw(file)?;
        if file != ci_gates::CI {
            return Ok(text);
        }
        Ok(self
            .read_raw(ci_gates::GATES)
            .map_or(text.clone(), |gates| {
                ci_gates::expand(&text, &ci_gates::parse(&gates).0)
            }))
    }

    /// Checks that `file` contains `want`.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be read or lacks `want`.
    pub fn contains(&self, file: &str, want: &str) -> Result<(), String> {
        if self.read(file)?.contains(want) {
            Ok(())
        } else {
            Err(format!("{file} does not contain {want:?}"))
        }
    }

    /// Reads one requirement row from the checkout's SRS, so a scenario
    /// checks the row's own words rather than a copy of them.
    ///
    /// # Errors
    ///
    /// Fails when the SRS cannot be read or has no such row.
    pub fn requirement(&self, id: &str) -> Result<srs::Requirement, String> {
        srs::load(&self.root.join("docs/srs"))
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| format!("no requirement {id} in the SRS"))
    }
}

/// Turns a check's problems into a step's outcome: none is a pass, and
/// any fails with every problem, one per line.
///
/// # Errors
///
/// Fails when `problems` is not empty.
pub fn outcome(problems: Vec<String>) -> Result<(), String> {
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

/// Formats `items` the way the gates print a list: `[a b c]`.
#[must_use]
pub fn list(items: &[impl AsRef<str>]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join(" ")
    )
}

#[cfg(test)]
mod tests;
