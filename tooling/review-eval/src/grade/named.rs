//! Whether a report line names a file of the case: by the path as the
//! case gives it, or by a tail of it, as the review takes a place
//! (`review-run`'s weighing): an agent that writes `x.md:3` for
//! `docs/x.md:3` still names the file.

use crate::report::{bare, dropped, split_dashes};

/// Whether `line` names `path`: it holds the path as written, or its
/// place is a tail of the path at a `/` that no other file of `files`,
/// the files the case changes, ends in. A tail two of the case's files
/// share names neither.
pub(super) fn names(line: &str, path: &str, files: &[&str]) -> bool {
    line.contains(path)
        || places(line).any(|place| {
            tail_of(path, &place) && !files.iter().any(|f| *f != path && ends_in(f, &place))
        })
}

/// The path of each place `line` may be at: its first part, and its
/// second when it is a dropped line, `dropped — path:line — why`, each
/// read from its last word, so the word of a `revised:` or `withdrawn:`
/// line before it is left out. A place in the prose is none.
fn places(line: &str) -> impl Iterator<Item = String> + '_ {
    let parts = if dropped(line) { 2 } else { 1 };
    split_dashes(bare(line))
        .into_iter()
        .take(parts)
        .filter_map(|part| part.split_whitespace().next_back())
        .filter_map(|word| review_gate::report::place(word).map(|(path, _)| path))
}

/// Whether `tail` is `path` with one or more of its directories left
/// off.
fn tail_of(path: &str, tail: &str) -> bool {
    tail != path && ends_in(path, tail)
}

/// Whether `path` is `tail` or ends in it at a `/`.
fn ends_in(path: &str, tail: &str) -> bool {
    path == tail || path.strip_suffix(tail).is_some_and(|d| d.ends_with('/'))
}

#[cfg(test)]
mod tests;
