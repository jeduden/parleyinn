//! The stability measurement's snapshots (ENG-32): the frozen pull-request
//! snapshots it reviews [`SAMPLES`] times each, one `[[snapshot]]` per snapshot
//! in `tooling/review-eval/stability-snapshots.toml`, each with a name,
//! the pull request it was and the base and head commits it compared.
//! A name is lowercase letters, digits and dashes, as it names the
//! measurement's artifacts; a commit is 40 lowercase hex digits.
//!
//! `review-report snapshots FILE` prints the measurement's two job matrices,
//! `snapshots=` and `samples=`, for the workflow's step outputs.

use serde::{Deserialize, Serialize};

/// The snapshots, under the repository.
pub const SNAPSHOTS: &str = "tooling/review-eval/stability-snapshots.toml";
/// How many fresh reviews the measurement runs of each snapshot.
pub const SAMPLES: usize = 3;

/// The snapshots file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    snapshot: Vec<Snapshot>,
}

/// One frozen pull-request snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub name: String,
    pub pull: u64,
    pub base: String,
    pub head: String,
}

/// Whether `id` is a full commit id: 40 lowercase hex digits.
fn is_commit(id: &str) -> bool {
    id.len() == 40 && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Whether `name` may name a snapshot: lowercase letters, digits and
/// dashes, not empty.
fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Reads the snapshots `text`.
///
/// # Errors
///
/// Fails when it does not read, holds no snapshot, or a snapshot's name or
/// commits are not in their shape, its name is taken, or its base is
/// its head; naming the snapshot.
pub fn parse(text: &str) -> Result<Vec<Snapshot>, String> {
    let file: File = toml::from_str(text).map_err(|e| format!("malformed snapshots: {e}"))?;
    if file.snapshot.is_empty() {
        return Err("the snapshots file holds no snapshot".to_owned());
    }
    for (at, s) in file.snapshot.iter().enumerate() {
        if !is_name(&s.name) {
            return Err(format!(
                "snapshot {:?}: a name is lowercase letters, digits and dashes",
                s.name
            ));
        }
        if file.snapshot[..at].iter().any(|t| t.name == s.name) {
            return Err(format!("snapshot {:?} is named twice", s.name));
        }
        if !is_commit(&s.base) || !is_commit(&s.head) {
            return Err(format!(
                "snapshot {:?}: base and head are 40-digit commit ids",
                s.name
            ));
        }
        if s.base == s.head {
            return Err(format!("snapshot {:?}: its base is its head", s.name));
        }
    }

    Ok(file.snapshot)
}

/// One review the measurement runs: a snapshot and its sample, from one.
#[derive(Debug, Serialize)]
struct Sample<'a> {
    #[serde(flatten)]
    snapshot: &'a Snapshot,
    sample: usize,
}

/// The measurement's job matrices for `snapshots`, as step outputs: one job per
/// snapshot, and one per snapshot and sample.
#[must_use]
pub fn matrices(snapshots: &[Snapshot]) -> String {
    let samples: Vec<Sample> = snapshots
        .iter()
        .flat_map(|snapshot| (1..=SAMPLES).map(move |sample| Sample { snapshot, sample }))
        .collect();

    format!(
        "snapshots={}\nsamples={}\n",
        serde_json::json!({ "include": snapshots }),
        serde_json::json!({ "include": samples })
    )
}

#[cfg(test)]
mod tests;
