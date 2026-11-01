//! The stability measurement's states (ENG-32): the frozen pull-request
//! states it reviews [`SAMPLES`] times each, one `[[state]]` per state
//! in `tooling/review-eval/stability-states.toml`, each with a name,
//! the pull request it was and the base and head commits it compared.
//! A name is lowercase letters, digits and dashes, as it names the
//! measurement's artifacts; a commit is 40 lowercase hex digits.
//!
//! `review-report states FILE` prints the measurement's two job matrices,
//! `states=` and `samples=`, for the workflow's step outputs.

use serde::{Deserialize, Serialize};

/// The states, under the repository.
pub const STATES: &str = "tooling/review-eval/stability-states.toml";
/// How many fresh reviews the measurement runs of each state.
pub const SAMPLES: usize = 3;

/// The states file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    state: Vec<State>,
}

/// One frozen pull-request state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub name: String,
    pub pull: u64,
    pub base: String,
    pub head: String,
}

/// Whether `id` is a full commit id: 40 lowercase hex digits.
fn is_commit(id: &str) -> bool {
    id.len() == 40 && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Whether `name` may name a state: lowercase letters, digits and
/// dashes, not empty.
fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Reads the states `text`.
///
/// # Errors
///
/// Fails when it does not read, holds no state, or a state's name or
/// commits are not in their shape, its name is taken, or its base is
/// its head; naming the state.
pub fn parse(text: &str) -> Result<Vec<State>, String> {
    let file: File = toml::from_str(text).map_err(|e| format!("malformed states: {e}"))?;
    if file.state.is_empty() {
        return Err("the states holds no state".to_owned());
    }
    for (at, s) in file.state.iter().enumerate() {
        if !is_name(&s.name) {
            return Err(format!(
                "state {:?}: a name is lowercase letters, digits and dashes",
                s.name
            ));
        }
        if file.state[..at].iter().any(|t| t.name == s.name) {
            return Err(format!("state {:?} is named twice", s.name));
        }
        if !is_commit(&s.base) || !is_commit(&s.head) {
            return Err(format!(
                "state {:?}: base and head are 40-digit commit ids",
                s.name
            ));
        }
        if s.base == s.head {
            return Err(format!("state {:?}: its base is its head", s.name));
        }
    }

    Ok(file.state)
}

/// One review the measurement runs: a state and its sample, from one.
#[derive(Debug, Serialize)]
struct Sample<'a> {
    #[serde(flatten)]
    state: &'a State,
    sample: usize,
}

/// The measurement's job matrices for `states`, as step outputs: one job per
/// state, and one per state and sample.
#[must_use]
pub fn matrices(states: &[State]) -> String {
    let samples: Vec<Sample> = states
        .iter()
        .flat_map(|state| (1..=SAMPLES).map(move |sample| Sample { state, sample }))
        .collect();

    format!(
        "states={}\nsamples={}\n",
        serde_json::json!({ "include": states }),
        serde_json::json!({ "include": samples })
    )
}

#[cfg(test)]
mod tests;
