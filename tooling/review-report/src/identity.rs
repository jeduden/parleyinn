//! A finding's identity across reviews of one pull-request snapshot: its
//! path, the requirement ids and invariants it links, and the SHA-256
//! of its consequence normalised, never its line number, which moves
//! with any edit and says nothing about whether two reviews found the same
//! thing. A specialist's findings are read from its report, linked
//! findings and requirement requests alike; a generalist's from its
//! answer, each finding line it kept.

use std::collections::{BTreeMap, BTreeSet};

use review_gate::report::{self, Line};
use sha2::{Digest, Sha256};

/// One finding's identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fid {
    pub path: String,
    /// Its ids, sorted.
    pub ids: Vec<String>,
    /// The SHA-256 of its normalised consequence, in hex.
    pub consequence: String,
}

impl Fid {
    /// The identity of a finding at `path` linking `ids`, its
    /// consequence `consequence`.
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

/// The ids in `text` that have the shape of a requirement id, `ENG-28`,
/// or of an invariant, `I2`.
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

/// The identities of a specialist's findings in its report `text`.
#[must_use]
pub fn of_report(text: &str) -> BTreeSet<Fid> {
    text.lines()
        .filter_map(|l| of_line(&report::line(l)))
        .collect()
}

/// The identity of the finding a report line holds.
fn of_line(line: &Line) -> Option<Fid> {
    match line {
        Line::Linked(l) => Some(Fid::new(&l.path, l.ids.clone(), &l.consequence)),
        Line::Request(r) => Some(Fid::new(
            &r.path,
            Vec::new(),
            &format!("request: {} {}", r.change, r.requirement),
        )),
        Line::Revised(r) => of_line(&report::line(&r.finding)),
        _ => None,
    }
}

/// The identities of a generalist's kept findings in its answer
/// `text`: what each breaks, its first part after the severity, and
/// every id the line cites.
#[must_use]
pub fn of_answer(text: &str) -> BTreeSet<Fid> {
    text.lines()
        .filter_map(review_eval::report::finding)
        .map(|f| {
            let breaks = f.text.split(" — ").next().unwrap_or_default();
            Fid::new(&f.path, ids(&f.text), breaks)
        })
        .collect()
}

/// Each agent's finding identities in one review: each specialist's
/// from its report in `reports`, each generalist's from its answer in
/// `answers`, both by name.
#[must_use]
pub fn of_review(
    reports: &BTreeMap<String, String>,
    answers: &BTreeMap<String, String>,
) -> BTreeMap<String, BTreeSet<Fid>> {
    let mut out: BTreeMap<String, BTreeSet<Fid>> = reports
        .iter()
        .map(|(name, text)| (name.clone(), of_report(text)))
        .collect();
    for (name, answer) in answers {
        if !reports.contains_key(name) {
            out.insert(name.clone(), of_answer(answer));
        }
    }

    out
}

#[cfg(test)]
mod tests;
