//! ENG-28: every language the repository's source and configuration
//! are written in has its specialist. The walk reads every file of the
//! checkout that git would track, leaving out build output, other
//! checkouts and what each `.gitignore` names, and asks the review
//! bundle's dispatch which language specialist reads it. A file that
//! needs a language and has none is named, as is a directory the walk
//! cannot read or a name that is not UTF-8, so the check never passes
//! without having looked.

use std::fs;

use review_bundle::dispatch::{DATA_ROOTS, language_specialist, needs_a_language};

use crate::Checkout;

/// Git's own entry, which no walk reads: a directory in a clone, a
/// file in a worktree.
const GIT: &str = ".git";
/// The build output, a directory no walk enters.
const BUILD_OUTPUT: &str = "target";
/// The agents' worktrees: other checkouts, each with its own files.
const OTHER_CHECKOUTS: &str = ".claude/worktrees";

/// Problems with the languages of the checkout's files: each file that
/// needs a language no specialist reads, and each part of the tree the
/// walk could not read.
#[must_use]
pub fn every_language_has_its_specialist(checkout: &Checkout) -> Vec<String> {
    let mut problems = Vec::new();
    let mut files = Vec::new();
    walk(checkout, "", &[], &mut files, &mut problems);
    problems.extend(
        files
            .into_iter()
            .filter(|f| needs_a_language(f) && language_specialist(f).is_none())
            .map(|f| format!("{f} is in no language a review specialist reads")),
    );

    problems
}

/// One `.gitignore` line, with the directory it sits in.
#[derive(Debug, Clone)]
struct Ignore {
    base: String,
    pattern: String,
    anchored: bool,
    dir_only: bool,
}

/// The ignore patterns of the `.gitignore` text in directory `base`:
/// comments, blank lines and negations (`!`) left out.
fn ignores(base: &str, text: &str) -> Vec<Ignore> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('!'))
        .map(|l| {
            let dir_only = l.ends_with('/');
            let l = l.trim_end_matches('/');
            let anchored = l.contains('/');
            Ignore {
                base: base.to_owned(),
                pattern: l.trim_start_matches('/').to_owned(),
                anchored,
                dir_only,
            }
        })
        .collect()
}

/// Whether `path`, a directory when `is_dir`, is ignored by `ignore`.
fn ignored(ignore: &Ignore, path: &str, is_dir: bool) -> bool {
    if ignore.dir_only && !is_dir {
        return false;
    }
    let Some(rest) = (if ignore.base.is_empty() {
        Some(path)
    } else {
        path.strip_prefix(&format!("{}/", ignore.base))
    }) else {
        return false;
    };
    if ignore.anchored {
        glob(&ignore.pattern, rest)
    } else {
        glob(&ignore.pattern, rest.rsplit('/').next().unwrap_or(rest))
    }
}

/// Whether `text` matches `pattern`, where `*` stands for any run of
/// characters other than `/`.
fn glob(pattern: &str, text: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == text,
        Some((head, tail)) => {
            let Some(rest) = text.strip_prefix(head) else {
                return false;
            };
            (0..=rest.len())
                .filter(|&i| rest.is_char_boundary(i) && !rest[..i].contains('/'))
                .any(|i| glob(tail, &rest[i..]))
        }
    }
}

/// Collects the files under `dir` git would track into `files`, in
/// name order, and each part it cannot read into `problems`. A symlink
/// is never followed; data under [`DATA_ROOTS`] is not entered.
fn walk(
    checkout: &Checkout,
    dir: &str,
    inherited: &[Ignore],
    files: &mut Vec<String>,
    problems: &mut Vec<String>,
) {
    let mut rules = inherited.to_vec();
    let gitignore = if dir.is_empty() {
        ".gitignore".to_owned()
    } else {
        format!("{dir}/.gitignore")
    };
    if let Ok(text) = checkout.read(&gitignore) {
        rules.extend(ignores(dir, &text));
    }
    // One read of the directory and all its entries, so an error in
    // either is the same named problem.
    let entries = fs::read_dir(checkout.root.join(dir))
        .and_then(|entries| entries.collect::<Result<Vec<_>, _>>());
    let entries = match entries {
        Ok(entries) => entries,
        Err(e) => {
            problems.push(format!("read {dir}: {e}"));
            return;
        }
    };
    let mut names = Vec::new();
    for e in entries {
        match e.file_name().into_string() {
            Ok(name) => names.push((name, e.file_type().is_ok_and(|t| t.is_dir()))),
            Err(name) => problems.push(format!("{dir}/{} is not UTF-8", name.to_string_lossy())),
        }
    }
    names.sort();
    for (name, is_dir) in names {
        let path = if dir.is_empty() {
            name.clone()
        } else {
            format!("{dir}/{name}")
        };
        let skipped = name == GIT
            || (is_dir && (name == BUILD_OUTPUT || path == OTHER_CHECKOUTS))
            || rules.iter().any(|r| ignored(r, &path, is_dir))
            || DATA_ROOTS.iter().any(|d| format!("{path}/").starts_with(d));
        if skipped {
            continue;
        }
        if is_dir {
            walk(checkout, &path, &rules, files, problems);
        } else {
            files.push(path);
        }
    }
}

#[cfg(test)]
mod tests;
