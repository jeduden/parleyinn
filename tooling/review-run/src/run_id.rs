//! What a review ran on, written beside its outcome so a later run can
//! tell whether an answer it would reuse came from the same inputs: the
//! head commit reviewed, a hash of the inputs every agent reads (the
//! review class, the dispatch, the bundle, the diff against main and the
//! pull request's agent instructions), the split limit, and per agent a
//! SHA-256 of its definition file and, for a specialist, of its own
//! diff. Nothing is reused across a changed head, input, definition or
//! diff.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::plan::Plan;
use crate::{CLASS, REVIEW};

/// Its file under the work directory.
pub const RUN_ID: &str = "review/run-id.json";

/// The files every agent of the review reads, relative to the work
/// directory: a change to any changes every answer.
pub const INPUTS: [&str; 5] = [
    CLASS,
    "review/dispatch.json",
    "review/bundle.md",
    "pr.diff",
    "review/pr-instructions.txt",
];

/// What one review ran on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunId {
    /// The head commit reviewed; none when it could not be read.
    pub head: Option<String>,
    /// The hash of [`INPUTS`].
    pub inputs: String,
    /// The size past which a specialist's diff ran in parts.
    pub split: usize,
    /// Each dispatched agent's, by name.
    pub agents: BTreeMap<String, AgentId>,
    /// The earlier run's directory this one resumed from; absent for a
    /// full run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from: Option<String>,
}

/// What one agent ran on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentId {
    /// The SHA-256 of its definition file; none when it does not read.
    pub definition: Option<String>,
    /// For a specialist, the SHA-256 of its diff; none when it has
    /// none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
}

/// The SHA-256 of `bytes`, in hex.
#[must_use]
pub fn sha256(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// `bytes` in lowercase hex.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The SHA-256 of the file at `path`; none when it does not read.
fn file_hash(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| sha256(&bytes))
}

/// What the review of `plan` in `workdir` runs on, its head `head` and
/// its split limit `split`.
#[must_use]
pub fn compute(workdir: &Path, plan: &Plan, head: Option<String>, split: usize) -> RunId {
    let mut inputs = Sha256::new();
    for rel in INPUTS {
        inputs.update(rel.as_bytes());
        match fs::read(workdir.join(rel)) {
            Ok(bytes) => {
                inputs.update(format!("\0{}\0", bytes.len()).as_bytes());
                inputs.update(&bytes);
            }
            Err(_) => inputs.update(b"\0-\0"),
        }
    }
    let specialists = plan.specialists();
    let agents = plan
        .agents()
        .into_iter()
        .map(|name| {
            let definition = file_hash(&workdir.join(review_eval::component::file_of(&name)));
            let diff = specialists
                .contains(&name)
                .then(|| file_hash(&workdir.join(format!("{REVIEW}/diffs/{name}.diff"))))
                .flatten();
            (name, AgentId { definition, diff })
        })
        .collect();

    RunId {
        head,
        inputs: hex(&inputs.finalize()),
        split,
        agents,
        resumed_from: None,
    }
}

/// Reads a run's identity.
///
/// # Errors
///
/// Fails when it does not read as its shape.
pub fn parse(text: &str) -> Result<RunId, String> {
    serde_json::from_str(text).map_err(|e| format!("malformed run id: {e}"))
}

/// The commit checked out in the git tree at `tree`, its `.git` a
/// directory or a worktree's `gitdir:` file: a detached head's, or the
/// one its branch names, loose or packed; none when it cannot be read.
#[must_use]
pub fn git_head(tree: &Path) -> Option<String> {
    let dotgit = tree.join(".git");
    let gitdir = if dotgit.is_dir() {
        dotgit
    } else {
        let text = fs::read_to_string(&dotgit).ok()?;
        tree.join(text.trim().strip_prefix("gitdir:")?.trim())
    };
    let head = fs::read_to_string(gitdir.join("HEAD")).ok()?;
    let head = head.trim();
    let Some(name) = head.strip_prefix("ref:").map(str::trim) else {
        return is_commit(head).then(|| head.to_owned());
    };
    let common = fs::read_to_string(gitdir.join("commondir"))
        .map_or_else(|_| gitdir.clone(), |c| gitdir.join(c.trim()));
    for dir in [&gitdir, &common] {
        if let Ok(text) = fs::read_to_string(dir.join(name)) {
            let id = text.trim();
            if is_commit(id) {
                return Some(id.to_owned());
            }
        }
    }
    let packed = fs::read_to_string(common.join("packed-refs")).ok()?;
    packed
        .lines()
        .filter_map(|l| l.split_once(' '))
        .find(|(id, r)| *r == name && is_commit(id))
        .map(|(id, _)| id.to_owned())
}

/// Whether `id` reads as a commit id: 40 or 64 lowercase hex digits.
#[must_use]
pub fn is_commit(id: &str) -> bool {
    matches!(id.len(), 40 | 64) && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

#[cfg(test)]
mod tests;
