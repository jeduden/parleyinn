//! Leaves out of a specialist's diff the hunks that change only text an
//! `<?include?>` copies verbatim from its source. A change to a shared
//! section reaches every file that includes it, so without this one
//! edit reads as dozens, and a specialist spends its turns on copies
//! instead of the change (ENG-28). A copy is left out only when the
//! pull request's tree shows it equal to its source there; any other
//! copy, and any hunk that reaches past one, stays. `mdsmith check .`
//! holds every copy to its source, and the source's own change stays.

use std::fmt::Write as _;

/// The first line of a diff that left hunks out, before any header.
pub const NOTE: &str = "Left out: each hunk that changes only text an `<?include?>` copies \
                        verbatim from its source, as the pull request's tree shows it; \
                        read the change at the source, which `mdsmith check .` holds each copy to.";
/// The start of the line that names a file every hunk of which is left out.
pub const LEFT_OUT: &str = "Left out whole: ";

/// The header every file's section of a diff starts with.
const HEADER: &str = "diff --git ";

/// Each copy in `text`, the file at `path` in the pull request's tree, as
/// the line of its directive's `?>` and the line of its `<?/include?>`,
/// 1-based, in the order they open. `read` reads a file of that tree by
/// its path. A copy counts only when its body is its source's text,
/// without front matter, with heading levels and blank lines at either
/// end set aside.
pub fn copied(
    path: &str,
    text: &str,
    read: &dyn Fn(&str) -> Option<String>,
) -> Vec<(usize, usize)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        if !line.trim_start().starts_with("<?include") {
            continue;
        }
        let Some(open) = (at..lines.len()).find(|&i| lines[i].trim_end().ends_with("?>")) else {
            continue;
        };
        let Some(close) = close_of(&lines, open + 1) else {
            continue;
        };
        let source = lines[at..=open]
            .iter()
            .find_map(|l| l.trim().strip_prefix("file:"))
            .and_then(|f| resolve(path, f.trim()))
            .and_then(|f| read(&f));
        if source.is_some_and(|s| same(&lines[open + 1..close], &body(&s))) {
            out.push((open + 1, close + 1));
        }
    }

    out
}

/// The index of the `<?/include?>` that closes the copy whose body starts
/// at index `from`, past any copy nested in it.
fn close_of(lines: &[&str], from: usize) -> Option<usize> {
    let mut depth = 1usize;
    for (i, line) in lines.iter().enumerate().skip(from) {
        let line = line.trim();
        if line.starts_with("<?include") {
            depth += 1;
        } else if line == "<?/include?>" {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }

    None
}

/// The text of `source` after its front matter.
fn body(source: &str) -> Vec<&str> {
    let lines: Vec<&str> = source.lines().collect();
    let start = match lines.first() {
        Some(&"---") => lines
            .iter()
            .skip(1)
            .position(|l| *l == "---")
            .map_or(0, |end| end + 2),
        _ => 0,
    };

    lines[start..].to_vec()
}

/// Whether `copy` and `source` read the same, heading levels and blank
/// lines at either end set aside.
fn same(copy: &[&str], source: &[&str]) -> bool {
    let norm = |lines: &[&str]| -> Vec<String> {
        let start = lines.iter().position(|l| !l.trim().is_empty());
        let end = lines.iter().rposition(|l| !l.trim().is_empty());
        match (start, end) {
            (Some(s), Some(e)) => lines[s..=e]
                .iter()
                .map(|l| l.trim_start_matches('#').to_owned())
                .collect(),
            _ => Vec::new(),
        }
    };

    norm(copy) == norm(source)
}

/// The tree path `file`, as an `<?include?>` in the file at `path` names
/// it, relative to that file's directory, or `None` when it is absolute
/// or leaves the tree.
pub fn resolve(path: &str, file: &str) -> Option<String> {
    let file = file.trim_matches('"');
    if file.starts_with('/') {
        return None;
    }
    let mut parts: Vec<&str> = path.split('/').collect();
    parts.pop();
    for part in file.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }

    Some(parts.join("/"))
}

/// `diff` without the hunks that change only text inside a copy, with
/// [`NOTE`] and a [`LEFT_OUT`] line for each file left out whole after
/// what precedes its first header, when any is left out. Only a Markdown
/// file's sections are read; a section whose header or hunk does not
/// read stays whole.
pub fn prune(diff: &str, read: &dyn Fn(&str) -> Option<String>) -> String {
    let mut preamble = String::new();
    let mut sections: Vec<String> = Vec::new();
    for line in diff.split_inclusive('\n') {
        if line.starts_with(HEADER) {
            sections.push(String::new());
        }
        sections.last_mut().unwrap_or(&mut preamble).push_str(line);
    }
    let (mut kept, mut whole, mut pruned) = (String::new(), Vec::new(), false);
    for section in &sections {
        let (left, any) = section_without_copies(section, read);
        pruned |= any;
        match left {
            Some(left) => kept.push_str(&left),
            None => whole.push(section_path(section).unwrap_or_default()),
        }
    }
    if !pruned {
        return diff.to_owned();
    }
    let mut out = format!("{preamble}{NOTE}\n");
    for path in whole {
        let _ = writeln!(out, "{LEFT_OUT}`{path}`");
    }

    out + &kept
}

/// The new-side path of a section's header.
fn section_path(section: &str) -> Option<String> {
    let header = section.lines().next()?;
    crate::diff::header(header).map(|(_, b)| b)
}

/// `section` without its hunks inside a copy, or `None` when every hunk
/// was, and whether any was.
fn section_without_copies(
    section: &str,
    read: &dyn Fn(&str) -> Option<String>,
) -> (Option<String>, bool) {
    let copies = section_path(section)
        .filter(|p| p.ends_with(".md"))
        .and_then(|p| read(&p).map(|t| copied(&p, &t, read)))
        .unwrap_or_default();
    if copies.is_empty() {
        return (Some(section.to_owned()), false);
    }
    let mut parts: Vec<String> = vec![String::new()];
    for line in section.split_inclusive('\n') {
        if line.starts_with("@@") {
            parts.push(String::new());
        }
        if let Some(part) = parts.last_mut() {
            part.push_str(line);
        }
    }
    let head = parts.remove(0);
    let total = parts.len();
    let left: Vec<String> = parts
        .into_iter()
        .filter(|hunk| !inside(hunk, &copies))
        .collect();
    match (left.len(), total) {
        (0, t) if t > 0 => (None, true),
        (n, t) => (Some(head + &left.concat()), n < t),
    }
}

/// Whether every changed line of `hunk` lies inside one of `copies`.
fn inside(hunk: &str, copies: &[(usize, usize)]) -> bool {
    let mut lines = hunk.lines();
    let Some(mut next) = lines.next().and_then(new_start) else {
        return false;
    };
    let within = |n: usize, removed: bool| {
        copies
            .iter()
            .any(|&(open, close)| open < n && (n < close || (removed && n == close)))
    };
    let mut changed = false;
    for line in lines {
        match line.as_bytes().first() {
            Some(b'+') => {
                if !within(next, false) {
                    return false;
                }
                changed = true;
                next += 1;
            }
            Some(b'-') => {
                if !within(next, true) {
                    return false;
                }
                changed = true;
            }
            Some(b'\\') => {}
            _ => next += 1,
        }
    }

    changed
}

/// The first new-side line a hunk header `@@ -a,b +c,d @@` names.
pub fn new_start(header: &str) -> Option<usize> {
    let plus = header.strip_prefix("@@ -")?.split(' ').nth(1)?;
    let start = plus.strip_prefix('+')?.split(',').next()?;
    start.parse().ok()
}

#[cfg(test)]
mod tests;
