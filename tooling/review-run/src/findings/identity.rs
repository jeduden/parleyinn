//! A kept finding's identity across reviews of one pull request: its
//! path, the requirement ids and invariants it cites, sorted, and the
//! SHA-256 of its consequence normalised, never its line, which moves
//! with any edit. It is the identity `review-report`'s `identity`
//! module gives a finding (jeduden/parleyinn#105), kept here while that
//! crate is not on this branch's base; the two must be one function
//! once both are on main.

use review_gate::Finding;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// One finding's identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fid {
    pub path: String,
    /// Its ids, sorted, each once.
    pub ids: Vec<String>,
    /// The SHA-256 of its normalised consequence, in hex.
    pub consequence: String,
}

impl Fid {
    /// The identity of a finding at `path` citing `ids`, its consequence
    /// `consequence`.
    #[must_use]
    pub fn new(path: &str, mut ids: Vec<String>, consequence: &str) -> Self {
        ids.sort();
        ids.dedup();
        let digest = Sha256::digest(normalise(consequence).as_bytes());
        Self {
            path: path.to_owned(),
            ids,
            consequence: digest.iter().map(|b| format!("{b:02x}")).collect(),
        }
    }
}

/// `text` lowercased, its markup and closing punctuation dropped and
/// its whitespace collapsed, so a reworded space or a bold word is the
/// same consequence.
#[must_use]
pub fn normalise(text: &str) -> String {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|w| w.trim_matches(['*', '_', '`']).to_lowercase())
        .filter(|w| !w.is_empty())
        .collect();
    words
        .join(" ")
        .trim_end_matches(['.', ';', ',', ':', '!'])
        .to_owned()
}

/// The ids in `text` shaped as a requirement id, `ENG-28`, or an
/// invariant, `I2`.
#[must_use]
pub fn ids(text: &str) -> Vec<String> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .filter(|w| id_shaped(w))
        .map(str::to_owned)
        .collect()
}

/// Whether `word` is shaped as a requirement id or an invariant.
fn id_shaped(word: &str) -> bool {
    let digits = |n: &str| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit());
    match word.split_once('-') {
        Some((family, n)) => {
            !family.is_empty() && family.bytes().all(|b| b.is_ascii_uppercase()) && digits(n)
        }
        None => word.strip_prefix('I').is_some_and(digits),
    }
}

/// The identity of a finding the review outcome kept: what it breaks,
/// the first part of its body, and every id its body cites.
#[must_use]
pub fn of_finding(f: &Finding) -> Fid {
    let breaks = f.body.split(" — ").next().unwrap_or_default();
    Fid::new(&f.path, ids(&f.body), breaks)
}

#[cfg(test)]
mod tests;
