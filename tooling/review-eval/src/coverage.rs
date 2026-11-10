//! Coverage: which lines of a file a run's reads covered. A reviewer
//! is told to read its own diff file with `Read`, so the review can
//! tell from the log alone which ranges of that file it never read.

use std::path::Path;

use crate::log::ReadCall;

/// The lines a `Read` without a limit reads, as Claude Code's tool
/// does: this many from its offset.
pub const DEFAULT_LIMIT: usize = 2000;

/// The 1-based inclusive line ranges of `rel`, a file of `total_lines`
/// lines under the work directory, that no read covered, sorted and
/// merged: none when the whole file was read. Only a read a result
/// answered, with a good range, covers lines, and only when it names
/// that very file: its path, relative paths taken under the work
/// directory, equals the work directory's path joined with `rel`, empty
/// and `.` components aside, under one of `roots`, the names the work
/// directory goes by (as given, and as resolved). A path that climbs,
/// holding `..`, names nothing, so no other file of the same name, such
/// as one in the pull request's own tree, stands in for it.
#[must_use]
pub fn unread(
    reads: &[ReadCall],
    roots: &[&Path],
    rel: &str,
    total_lines: usize,
) -> Vec<(usize, usize)> {
    let mut spans: Vec<(usize, usize)> = reads
        .iter()
        .filter(|r| {
            r.answered && !r.bad_range && roots.iter().any(|root| names(root, &r.path, rel))
        })
        .map(|r| span(r, total_lines))
        .filter(|(start, end)| start <= end)
        .collect();
    spans.sort_unstable();
    let mut gaps = Vec::new();
    let mut next = 1;
    for (start, end) in spans {
        if start > next {
            gaps.push((next, start - 1));
        }
        next = next.max(end.saturating_add(1));
    }
    if next <= total_lines {
        gaps.push((next, total_lines));
    }
    gaps
}

/// Whether `path`, a read's, names `rel` under the work directory
/// `root`: a relative `path` is taken under `root`, and neither may
/// climb.
fn names(root: &Path, path: &str, rel: &str) -> bool {
    let Some(root) = root.to_str() else {
        return false;
    };
    let under = |p: &str| {
        let full = if p.starts_with('/') {
            p.to_owned()
        } else {
            format!("{root}/{p}")
        };
        components(&full).map(|parts| parts.join("/"))
    };
    let wanted = under(rel);
    wanted.is_some() && under(path) == wanted
}

/// The lines `read` covers, clipped to `total_lines`; a start past the
/// end means none.
fn span(read: &ReadCall, total_lines: usize) -> (usize, usize) {
    let start = read.offset.unwrap_or(1).max(1);
    let end = start
        .saturating_add(read.limit.unwrap_or(DEFAULT_LIMIT))
        .saturating_sub(1)
        .min(total_lines);
    (start, end)
}

/// A path's components, without empty and `.` ones; none when one is
/// `..`, as a path that climbs is never resolved.
fn components(path: &str) -> Option<Vec<&str>> {
    let parts: Vec<&str> = path
        .split('/')
        .filter(|c| !c.is_empty() && *c != ".")
        .collect();
    (!parts.contains(&"..")).then_some(parts)
}

#[cfg(test)]
mod tests;
