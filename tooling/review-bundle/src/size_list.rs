//! ENG-34: the size list only shrinks. A change that adds a file to the
//! size list, or raises a file's listed lines, is a blocking finding
//! until a stakeholder comment names the size list; a change that only
//! lowers or removes lines raises none, so it waits for no one.

use std::collections::BTreeMap;

use crate::{BLOCKING, Finding, NIT, diff};

/// The size list, relative to the root: one file a line,
/// `<path> <lines>`.
pub const SIZE_LIST: &str = "tooling/engineering/oversize.txt";

/// The file and the lines `line` of the size list gives it, when it is
/// an entry and no comment.
fn entry(line: &str) -> Option<(&str, usize)> {
    let mut words = line.split_whitespace();
    let (path, lines) = (words.next()?, words.next()?);
    let lines = lines.parse().ok()?;
    (!path.starts_with('#') && words.next().is_none()).then_some((path, lines))
}

/// The most lines `changed`, added or removed lines with their files,
/// give each file the size list lists.
fn listed<'a>(changed: &[(String, &'a str)]) -> BTreeMap<&'a str, usize> {
    let mut out = BTreeMap::new();
    let entries = changed
        .iter()
        .filter(|(file, _)| file == SIZE_LIST)
        .filter_map(|(_, line)| entry(line));
    for (path, lines) in entries {
        let most = out.entry(path).or_insert(lines);
        *most = lines.max(*most);
    }

    out
}

/// The files `diff` lengthens the size list by, sorted: each it adds,
/// and each whose listed lines it raises.
#[must_use]
pub fn lengthened(diff: &str) -> Vec<String> {
    let removed = listed(&diff::removed_lines(diff));

    listed(&diff::added_lines(diff))
        .into_iter()
        .filter(|(path, lines)| removed.get(path).is_none_or(|was| lines > was))
        .map(|(path, _)| path.to_owned())
        .collect()
}

/// ENG-34: the finding for `diff` when it lengthens the size list:
/// blocking until `consent`, a stakeholder comment that names the size
/// list, then a nit that names the files.
#[must_use]
pub fn finding(diff: &str, consent: bool) -> Option<Finding> {
    let files = lengthened(diff);
    if files.is_empty() {
        return None;
    }
    let (severity, comment) = if consent {
        (NIT, "a stakeholder comment names")
    } else {
        (BLOCKING, "no stakeholder comment names")
    };

    Some(Finding {
        path: SIZE_LIST.to_owned(),
        line: 0,
        severity,
        body: format!(
            "this change lengthens the size list (ENG-34): it adds or raises {}, and \
             {comment} {SIZE_LIST}, consenting to it",
            files.join(", ")
        ),
    })
}

#[cfg(test)]
mod tests;
