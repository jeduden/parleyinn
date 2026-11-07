//! Coverage: which lines of a file a run's reads covered. A reviewer
//! is told to read its own diff file with `Read`, so the review can
//! tell from the log alone which ranges of that file it never read.

use crate::log::ReadCall;

/// The lines a `Read` without a limit reads, as Claude Code's tool
/// does: this many from its offset.
pub const DEFAULT_LIMIT: usize = 2000;

/// The 1-based inclusive line ranges of `path`, of `total_lines`
/// lines, that no read which did not fail covered, sorted and merged:
/// none when the whole file was read. A read names `path` when its
/// path, normalised, ends with `path`'s components.
#[must_use]
pub fn unread(reads: &[ReadCall], path: &str, total_lines: usize) -> Vec<(usize, usize)> {
    let wanted = components(path);
    let mut spans: Vec<(usize, usize)> = reads
        .iter()
        .filter(|r| !r.failed && components(&r.path).ends_with(&wanted))
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

/// Whether the reads covered every line of `path`.
#[must_use]
pub fn read_whole(reads: &[ReadCall], path: &str, total_lines: usize) -> bool {
    unread(reads, path, total_lines).is_empty()
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

/// A path's components, without empty and `.` ones.
fn components(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|c| !c.is_empty() && *c != ".")
        .collect()
}

#[cfg(test)]
mod tests;
