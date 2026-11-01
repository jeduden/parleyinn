//! Reads `.github/CODEOWNERS` for the paths it assigns, so the review
//! class follows the same list the stakeholder's consent does (ENG-21).
//!
//! It reads the subset of the format the repository uses, by the
//! gitignore rules CODEOWNERS follows: a pattern that starts with `/` or
//! holds a `/` anywhere but at its end is anchored at the root, any
//! other matches at any depth; a trailing `/` names a directory's
//! contents; and `*` matches within one path segment.

use std::collections::BTreeSet;

/// The patterns `codeowners` assigns, in file order: each line's first
/// field, comments and blank lines left out.
#[must_use]
pub fn patterns(codeowners: &str) -> Vec<String> {
    codeowners
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

/// Whether `path`, relative to the root, is one `pattern` assigns.
#[must_use]
pub fn owns(pattern: &str, path: &str) -> bool {
    let anchored = pattern.trim_end_matches('/').contains('/');
    let dir = pattern.ends_with('/');
    let want: Vec<&str> = pattern
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let have: Vec<&str> = path.split('/').collect();
    if want.is_empty() {
        return false;
    }
    let starts = if anchored {
        0..1
    } else {
        0..have.len().saturating_sub(want.len()) + 1
    };

    starts.into_iter().any(|at| {
        let rest = &have[at..];
        rest.len() >= want.len()
            && !(dir && rest.len() == want.len())
            && want.iter().zip(rest).all(|(w, h)| glob(w, h))
    })
}

/// The characters besides whitespace a consenting comment splits on.
const SEPARATORS: [char; 10] = ['`', ',', ';', ':', '(', ')', '[', ']', '"', '\''];

/// The words of `comment`: split on whitespace and [`SEPARATORS`], each
/// with its trailing dots dropped, empty ones left out.
fn tokens(comment: &str) -> BTreeSet<&str> {
    comment
        .split(|c: char| c.is_whitespace() || SEPARATORS.contains(&c))
        .map(|t| t.trim_end_matches('.'))
        .filter(|t| !t.is_empty())
        .collect()
}

/// Whether `comment` names `path`, an owned path, so it consents to it
/// (ENG-21): a word of it is the path itself; the text of a pattern of
/// `patterns` that owns it, its leading `/` removed; or a directory that
/// holds it, other than the root, with a trailing `/`, or without one
/// when it holds a `/` itself. A bare `docs` is prose and names nothing.
#[must_use]
pub fn names(patterns: &[String], comment: &str, path: &str) -> bool {
    let words = tokens(comment);
    let owning = patterns
        .iter()
        .filter(|pat| owns(pat, path))
        .map(|pat| pat.trim_start_matches('/'));
    let dirs = path
        .match_indices('/')
        .flat_map(|(at, _)| [&path[..=at], &path[..at]])
        .filter(|d| d.ends_with('/') || d.contains('/'));

    words.contains(path) || owning.chain(dirs).any(|name| words.contains(name))
}

/// Whether `pattern`, where `*` matches any run of characters, matches
/// all of `text`.
fn glob(pattern: &str, text: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == text,
        Some((head, tail)) => text.strip_prefix(head).is_some_and(|rest| {
            (0..=rest.len())
                .filter(|&i| rest.is_char_boundary(i))
                .any(|i| glob(tail, &rest[i..]))
        }),
    }
}

#[cfg(test)]
mod tests;
