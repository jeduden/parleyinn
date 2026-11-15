//! Whether a report line names a file of the case: by the path as the
//! case gives it, or by a tail of it, as the review takes a place
//! (`review-run`'s weighing): an agent that writes `x.md:3` for
//! `docs/x.md:3` still names the file.

use crate::report::{bare, dropped, split_dashes};

/// Whether `line` names `path`: it holds the path as written, or its
/// place is a tail of the path, or of a file of `files` under it, at a
/// `/` that no other file of `files`, the files the case changes, ends
/// in. A tail two of the case's files share names neither.
pub(super) fn names(line: &str, path: &str, files: &[&str]) -> bool {
    line.contains(path) || lines(line, path, files).is_some()
}

/// The lines the place of `line` that names `path` covers, its first
/// and its last: one line twice, or the two ends of a range,
/// `path:12-20`; a place under a directory more, `pr-head/docs/x.md:3`,
/// names the path too, and so does one that names a file of `files`
/// under `path`, where a case wants a finding anywhere in a module.
/// `None` when no place of the line names the path, though its sentence
/// may.
pub(super) fn lines(line: &str, path: &str, files: &[&str]) -> Option<(usize, usize)> {
    let named = |place: &str, path: &str| {
        ends_in(place, path)
            || (tail_of(path, place) && !files.iter().any(|f| *f != path && ends_in(f, place)))
    };
    places(line)
        .find(|(place, _, _)| {
            named(place, path) || files.iter().any(|f| f.starts_with(path) && named(place, f))
        })
        .map(|(_, first, last)| (first, last))
}

/// The slips of a `report` whose lines name a file of `files` by a tail
/// of its path, one a file: the grader takes the tail, and the review
/// weighs it, but a `withdrawn:` or `revised:` line reaches a finding
/// only by the path as it was written.
pub(super) fn tail_slips(report: &str, files: &[&str]) -> Vec<String> {
    let mut slips: Vec<String> = report
        .lines()
        .flat_map(places)
        .filter_map(|(place, _, _)| {
            let mut ending = files.iter().filter(|f| tail_of(f, &place));
            let file = ending.next()?;
            ending
                .next()
                .is_none()
                .then(|| format!("names {file} by its tail {place}"))
        })
        .collect();
    slips.sort();
    slips.dedup();
    slips
}

/// Each place `line` may be at, its path and the first and last line
/// it covers: its first part, and its second when it is a dropped line,
/// `dropped — path:line — why`, each read from its last word, so the
/// word of a `revised:` or `withdrawn:` line before it is left out. A
/// place in the prose is none.
fn places(line: &str) -> impl Iterator<Item = (String, usize, usize)> + '_ {
    let parts = if dropped(line) { 2 } else { 1 };
    split_dashes(bare(line))
        .into_iter()
        .take(parts)
        .filter_map(|part| part.split_whitespace().next_back())
        .filter_map(|word| {
            let (path, first) = review_gate::report::place(word)?;
            Some((path, first, last_of(word).unwrap_or(first).max(first)))
        })
}

/// The last line of the range a place's `word` ends in, `path:12-20`,
/// when it ends in one.
fn last_of(word: &str) -> Option<usize> {
    let (_, range) = word.rsplit_once(':')?;
    let (_, last) = range.rsplit_once(['-', '–'])?;
    last.trim_matches(|c: char| !c.is_ascii_digit())
        .parse()
        .ok()
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
