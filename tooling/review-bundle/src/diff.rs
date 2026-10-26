//! Reads a unified diff, as `git diff` prints it, for the paths it
//! changes, the lines it adds, and the requirement ids its changed lines
//! name.
//!
//! A `diff --git` header names its paths plainly (`a/X b/Y`) or, when a
//! path holds a quote, a control character or a byte past ASCII, as a
//! C-style quoted string (`"a/x\303\251"`). Both forms read. A header
//! that reads as neither is listed, never guessed at, so the analysis
//! can fail closed on it.

use std::collections::BTreeSet;

/// The header every file's section of a diff starts with.
const HEADER: &str = "diff --git ";

/// What the diff's headers name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Paths {
    /// Every path the diff changes, both sides of a rename, sorted and
    /// each once.
    pub paths: Vec<String>,
    /// Every header line whose paths do not read, in diff order.
    pub unparsed: Vec<String>,
}

/// Every path the `diff --git` headers of `diff` name, and the headers
/// that do not read.
#[must_use]
pub fn changed_paths(diff: &str) -> Paths {
    let mut paths = BTreeSet::new();
    let mut unparsed = Vec::new();
    for line in diff.lines().filter(|l| l.starts_with(HEADER)) {
        match header(line) {
            Some((a, b)) => {
                paths.insert(a);
                paths.insert(b);
            }
            None => unparsed.push(line.to_owned()),
        }
    }

    Paths {
        paths: paths.into_iter().collect(),
        unparsed,
    }
}

/// Every added line of `diff`, without its `+`, with the file it is
/// added to: the new side of the header above it, or the header line
/// itself when that does not read. Lines before any header belong to no
/// file and are left out.
#[must_use]
pub fn added_lines(diff: &str) -> Vec<(String, &str)> {
    let mut file: Option<String> = None;
    let mut out = Vec::new();
    for line in diff.lines() {
        if line.starts_with(HEADER) {
            file = Some(header(line).map_or_else(|| line.to_owned(), |(_, b)| b));
        } else if let (Some(added), false, Some(f)) =
            (line.strip_prefix('+'), line.starts_with("+++"), &file)
        {
            out.push((f.clone(), added));
        }
    }

    out
}

/// The old and new path a `diff --git` header names, or `None` when
/// either side does not read.
fn header(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix(HEADER)?;
    let (a, rest) = if rest.starts_with('"') {
        let (a, rest) = quoted(rest)?;
        (a, rest.strip_prefix(' ')?)
    } else if rest.ends_with('"') {
        let at = rest.rfind(" \"")?;
        (rest[..at].to_owned(), &rest[at + 1..])
    } else {
        return plain(rest);
    };
    let b = if rest.starts_with('"') {
        let (b, tail) = quoted(rest)?;
        tail.is_empty().then_some(b)?
    } else {
        rest.to_owned()
    };

    Some((
        a.strip_prefix("a/")?.to_owned(),
        b.strip_prefix("b/")?.to_owned(),
    ))
}

/// The paths of an unquoted header's `rest`, `a/X b/Y`. A path may hold
/// ` b/` itself, so with more than one the split is ambiguous: it reads
/// only as the one split where both sides name the same path, as a
/// header of a file neither renamed nor copied does, and else not at
/// all.
fn plain(rest: &str) -> Option<(String, String)> {
    let splits: Vec<usize> = rest.match_indices(" b/").map(|(at, _)| at).collect();
    let at = match splits[..] {
        [at] => at,
        _ => *splits
            .iter()
            .find(|&&at| rest[..at].get(2..) == rest.get(at + 3..))?,
    };

    Some((
        rest[..at].strip_prefix("a/")?.to_owned(),
        rest[at + 3..].to_owned(),
    ))
}

/// The quoted string `s` starts with, unescaped, and what follows its
/// closing quote.
fn quoted(s: &str) -> Option<(String, &str)> {
    let body = s.strip_prefix('"')?;
    let mut escaped = false;
    let end = body.char_indices().find_map(|(i, c)| {
        let close = c == '"' && !escaped;
        escaped = c == '\\' && !escaped;
        close.then_some(i)
    })?;
    let bytes = unquote(&body[..end])?;

    Some((
        String::from_utf8_lossy(&bytes).into_owned(),
        &body[end + 1..],
    ))
}

/// The bytes the C-style escapes in `s` stand for: `\\`, `\"`, `\t`,
/// `\n`, `\r` and three octal digits. Any other escape does not read.
fn unquote(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len());
    let mut bytes = s.bytes();
    while let Some(b) = bytes.next() {
        if b != b'\\' {
            out.push(b);
            continue;
        }
        let byte = match bytes.next()? {
            b'\\' => b'\\',
            b'"' => b'"',
            b't' => b'\t',
            b'n' => b'\n',
            b'r' => b'\r',
            d @ b'0'..=b'7' => {
                let mut value = u32::from(d - b'0');
                for _ in 0..2 {
                    let d = bytes.next().filter(|d| (b'0'..=b'7').contains(d))?;
                    value = value * 8 + u32::from(d - b'0');
                }
                u8::try_from(value).ok()?
            }
            _ => return None,
        };
        out.push(byte);
    }

    Some(out)
}

/// Every requirement id `text` names, in order, repeats kept.
pub fn ids_in(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .filter(|t| srs::is_requirement_id(t))
}

/// Every requirement id an added or removed line names, sorted and each
/// once. File headers (`+++`, `---`) are not changed lines.
#[must_use]
pub fn mentioned_ids(diff: &str) -> Vec<String> {
    let changed = diff.lines().filter(|l| {
        (l.starts_with('+') || l.starts_with('-')) && !l.starts_with("+++") && !l.starts_with("---")
    });
    let out: BTreeSet<String> = changed.flat_map(ids_in).map(str::to_owned).collect();

    out.into_iter().collect()
}

/// Every requirement id an added or removed line of a file other than
/// Markdown names, sorted and each once: code and configuration that
/// name a row, not prose that quotes one. Lines before any header
/// belong to no file and are left out.
#[must_use]
pub fn ids_in_code(diff: &str) -> Vec<String> {
    let mut file: Option<String> = None;
    let mut out = BTreeSet::new();
    for line in diff.lines() {
        if line.starts_with(HEADER) {
            file = Some(header(line).map_or_else(|| line.to_owned(), |(_, b)| b));
        } else if let Some(f) = &file
            && !f.ends_with(".md")
            && (line.starts_with('+') || line.starts_with('-'))
            && !line.starts_with("+++")
            && !line.starts_with("---")
        {
            out.extend(ids_in(line).map(str::to_owned));
        }
    }

    out.into_iter().collect()
}

#[cfg(test)]
mod tests;
