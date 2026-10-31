//! The ids a specialist's linked finding may cite: every requirement id
//! the SRS defines in main's tree or in the pull request's, so a change
//! that adds a requirement can cite it and one that removes a
//! requirement still lets a finding name it, and the invariants. A
//! tree with no `docs/srs` adds no id; one whose tables do not read
//! fails the review before any agent runs.

use std::collections::BTreeSet;
use std::path::Path;

use review_eval::world::PR_HEAD;

/// Where the SRS lives in a tree.
const SRS: &str = "docs/srs";

/// The ids under `workdir`, main's tree with the pull request's in
/// `pr-head/`.
///
/// # Errors
///
/// Fails when either tree's SRS does not read.
pub fn ids(workdir: &Path) -> Result<BTreeSet<String>, String> {
    let mut out = review_gate::report::invariants();
    for dir in [workdir.join(SRS), workdir.join(PR_HEAD).join(SRS)] {
        if !dir.is_dir() {
            continue;
        }
        let requirements = srs::load(&dir).map_err(|e| e.to_string())?;
        out.extend(requirements.into_iter().map(|r| r.id));
    }

    Ok(out)
}

#[cfg(test)]
mod tests;
