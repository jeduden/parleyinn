//! The near forms of a finding line (ENG-28). A specialist that names
//! its place and its ids, yet writes the line off its form, stated a
//! finding all the same: the reader takes it as the finding it states,
//! and the assembly writes it in its form beside a note
//! ([`super::OFF_FORM`]) that counts the slip (I6). The near forms:
//!
//! - a linked finding whose last part starts with its ids and goes on,
//!   `… — ENG-11 (provisional)` or `… — ENG-04 and I9`: what follows the
//!   ids joins its consequence;
//! - a linked finding with no part of ids, whose sentence ends with
//!   them, `… the test sleeps, breaking ENG-03.`: the sentence is its
//!   consequence, whole;
//! - a request whose motivation follows the first of [`REASONS`] where
//!   its dash belongs, or that gives none, which reads with
//!   [`NO_MOTIVATION`].
//!
//! A line that names no id, or says nothing beside its ids, is in no
//! form, and so is one missing the dash after its place.

use super::{DASHES, id_shaped};

/// The motivation of a request that gives none.
pub const NO_MOTIVATION: &str = "no motivation given";

/// The words that may stand between two ids.
const AND: [&str; 2] = ["and", "&"];

/// The consequence and ids of a linked finding off its form: `said`,
/// what follows `if not addressed:` up to the first dash, and `rest`,
/// the parts after it. None when it names no id where a near form does,
/// or says nothing beside its ids.
pub(super) fn linked(said: &str, rest: &[&str]) -> Option<(String, Vec<String>)> {
    let parts: Vec<&str> = std::iter::once(said)
        .chain(rest.iter().copied())
        .map(str::trim)
        .collect();
    let (last, before) = parts.split_last()?;
    let words: Vec<&str> = last.split_whitespace().collect();
    let (ids, taken) = run(words.iter().copied());
    let (ids, beside, consequence) = if taken > 0 && !before.is_empty() {
        let remark = words[taken..].join(" ");
        let consequence = joined(before.iter().copied().chain([remark.as_str()]));
        (ids, consequence.clone(), consequence)
    } else {
        let (mut ids, taken) = run(words.iter().rev().copied());
        ids.reverse();
        let head = words[..words.len() - taken].join(" ");
        let beside = joined(before.iter().copied().chain([head.as_str()]));
        (ids, beside, joined(parts.iter().copied()))
    };

    (!ids.is_empty() && !beside.is_empty()).then_some((consequence, ids))
}

/// The ids `words` start with, and how many words they take: one of
/// [`AND`] between two ids is passed over.
fn run<'a>(words: impl Iterator<Item = &'a str>) -> (Vec<String>, usize) {
    let (mut ids, mut taken, mut passed) = (Vec::new(), 0, 0);
    for word in words {
        if let Some(id) = id_of(word) {
            ids.push(id);
            taken += passed + 1;
            passed = 0;
        } else if !ids.is_empty() && passed == 0 && AND.contains(&word) {
            passed = 1;
        } else {
            break;
        }
    }

    (ids, taken)
}

/// `word` as an id, the markup, brackets and punctuation around it
/// aside.
fn id_of(word: &str) -> Option<String> {
    let id = word.trim_matches(|c: char| c.is_ascii_punctuation());

    id_shaped(id).then(|| id.to_owned())
}

/// The parts of `parts` that hold text, joined with a spaced dash.
fn joined<'a>(parts: impl Iterator<Item = &'a str>) -> String {
    parts
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" — ")
}

/// What may stand between a request's requirement and its motivation
/// where the dash is left out.
const REASONS: [&str; 7] = [
    "; ",
    ": ",
    ". ",
    ", since ",
    ", because ",
    " since ",
    " because ",
];

/// `asked` cut at the first of [`REASONS`] in it: the requirement and
/// the motivation, which is empty when it holds none.
fn reasoned(asked: &str) -> (&str, &str) {
    REASONS
        .iter()
        .filter_map(|sep| asked.find(sep).map(|at| (at, sep.len())))
        .min()
        .map_or((asked, ""), |(at, len)| (&asked[..at], &asked[at + len..]))
}

/// `asked`, a request with no dash before its motivation, as its
/// requirement and motivation: cut at the first of [`REASONS`], or
/// whole, with [`NO_MOTIVATION`], when it holds none. A dash it ends
/// with, nothing after it, is left out.
pub(super) fn motivated(asked: &str) -> (&str, &str) {
    match reasoned(asked.trim_end_matches(DASHES)) {
        (requirement, "") => (requirement, NO_MOTIVATION),
        cut => cut,
    }
}

#[cfg(test)]
mod tests;
