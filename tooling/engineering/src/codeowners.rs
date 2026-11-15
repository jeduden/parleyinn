//! ENG-21: `CODEOWNERS` names the stakeholder on what guides the
//! review (the agent instructions, the review's code, cases and
//! scorecard, CI and the merge rules, the manifests and tool pins CI
//! builds and checks with, the bans and acceptances it holds the code
//! to, the linter configuration), on
//! nothing else, and on no Rust source outside the review's crates.
//!
//! The guiding paths come from the ENG-21 row of the SRS: each class
//! it names maps to the paths that hold it today, and a class the row
//! stops naming is reported, so the mapping cannot go stale unseen. The
//! stakeholder is read from the SRS as well.

use std::fs;

use crate::Checkout;

/// The file, relative to the checkout.
pub const FILE: &str = ".github/CODEOWNERS";

/// One line of the file: a path pattern and its owners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub pattern: String,
    pub owners: Vec<String>,
}

/// The review's crates: the bundle, the run, the gate, the report and
/// the evaluation with its cases and scorecard.
const REVIEW_CRATES: &str = "tooling/review-*/";

/// What the ENG-21 row names as guiding the review, as the words it
/// uses and the paths that hold each class. A path with a `*` stands
/// for each directory of its parent the name matches.
const ENFORCING: &[(&str, &[&str])] = &[
    (
        "the agent instructions",
        &["CLAUDE.md", "AGENTS.md", ".claude/", "docs/agents/"],
    ),
    ("the review's code, cases and scorecard", &[REVIEW_CRATES]),
    ("CI and the merge rules", &[".github/", "scripts/"]),
    ("`CODEOWNERS` itself", &[".github/CODEOWNERS"]),
    // A manifest's `[lints]` decides whether the bans take effect, and
    // the mise files pin the tools CI's gates run.
    (
        "the manifests and tool pins CI builds and checks with",
        &[
            "Cargo.toml",
            "crates/*/Cargo.toml",
            "tooling/*/Cargo.toml",
            "fuzz/Cargo.toml",
            "mise.toml",
            "mise.lock",
            "mise.dev.toml",
            "mise.dev.lock",
        ],
    ),
    (
        "the bans and acceptances CI holds the code to",
        &[
            "clippy.toml",
            "crates/*/clippy.toml",
            "deny.toml",
            "tooling/benchmark/accepted.txt",
            "tooling/eval/recall-v1.txt",
        ],
    ),
    ("the linter configuration", &[".mdsmith.yml"]),
];

/// The directories the Rust source check does not descend into: build
/// output and git's own store.
const NOT_SOURCE: &[&str] = &["target", ".git"];

/// Other checkouts of the repository, which are no part of this one
/// (see `agent_instructions`), relative to the root.
const OTHER_CHECKOUTS: &str = ".claude/worktrees";

/// Reads the file.
///
/// # Errors
///
/// Fails when it cannot be read.
pub fn load(checkout: &Checkout) -> Result<Vec<Entry>, String> {
    Ok(parse(&checkout.read(FILE)?))
}

/// Parses `text`: comments and blank lines left out, an owner list ends
/// at a `#`.
#[must_use]
pub fn parse(text: &str) -> Vec<Entry> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let pattern = words.next()?.to_owned();
            let owners = words
                .take_while(|w| !w.starts_with('#'))
                .map(str::to_owned)
                .collect();
            Some(Entry { pattern, owners })
        })
        .collect()
}

/// The owners of `path`, a file or a directory (with a trailing `/`):
/// those of the last entry that matches it, none when no entry does.
#[must_use]
pub fn owners_of<'a>(entries: &'a [Entry], path: &str) -> &'a [String] {
    entries
        .iter()
        .rev()
        .find(|e| matches(&e.pattern, path))
        .map_or(&[], |e| e.owners.as_slice())
}

/// Whether `pattern` covers `path`. It reads the syntax the repository's
/// CODEOWNERS uses: a leading `/` anchors the pattern at the root, a
/// trailing `/` names a directory, `*` and `?` stand within one name,
/// and a pattern naming a directory covers everything under it.
fn matches(pattern: &str, path: &str) -> bool {
    let anchored = pattern.starts_with('/') || pattern.trim_end_matches('/').contains('/');
    let pattern: Vec<&str> = pattern
        .trim_start_matches('/')
        .trim_end_matches('/')
        .split('/')
        .collect();
    let path: Vec<&str> = path.trim_end_matches('/').split('/').collect();
    if anchored {
        starts_with_names(&path, &pattern)
    } else {
        (0..path.len()).any(|at| starts_with_names(&path[at..], &pattern))
    }
}

/// Whether the leading names of `path` match `pattern`, one for one.
fn starts_with_names(path: &[&str], pattern: &[&str]) -> bool {
    path.len() >= pattern.len() && pattern.iter().zip(path).all(|(p, n)| glob(p, n))
}

/// Whether `name` matches `pattern`, where `*` stands for any run of
/// characters and `?` for one.
fn glob(pattern: &str, name: &str) -> bool {
    fn go(p: &[char], n: &[char]) -> bool {
        match p.split_first() {
            None => n.is_empty(),
            Some(('*', rest)) => (0..=n.len()).any(|i| go(rest, &n[i..])),
            Some((c, rest)) => n
                .split_first()
                .is_some_and(|(m, tail)| (*c == '?' || c == m) && go(rest, tail)),
        }
    }

    go(
        &pattern.chars().collect::<Vec<_>>(),
        &name.chars().collect::<Vec<_>>(),
    )
}

/// Whether a pattern uses syntax this reader does not.
fn unsupported(pattern: &str) -> bool {
    pattern.contains("**") || pattern.contains(['[', '!', '\\'])
}

/// The stakeholder, as the SRS index's `Stakeholder` row names it.
///
/// # Errors
///
/// Fails when the index cannot be read or has no stakeholder row naming
/// a user.
pub fn stakeholder(checkout: &Checkout) -> Result<String, String> {
    let index = checkout.read("docs/srs/index.md")?;
    let row = index
        .lines()
        .find(|l| l.contains("**Stakeholder**"))
        .ok_or("docs/srs/index.md has no Stakeholder row")?;
    row.split_whitespace()
        .find(|w| w.starts_with('@'))
        .map(|w| w.trim_end_matches([',', '|', '.']).to_owned())
        .ok_or_else(|| "the Stakeholder row names no user".to_owned())
}

/// The guiding paths of the checkout that exist, with the class each
/// belongs to. A path with a `*` stands for each directory it matches.
fn existing_enforcing(checkout: &Checkout) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    for (class, paths) in ENFORCING {
        for path in *paths {
            out.extend(expand(checkout, path).into_iter().map(|p| (*class, p)));
        }
    }

    out
}

/// `path`, or each path it stands for, when it exists in the checkout:
/// a name with a `*` stands for each entry of its parent it matches.
fn expand(checkout: &Checkout, path: &str) -> Vec<String> {
    let paths = match path.split_once('*') {
        None => vec![path.to_owned()],
        Some((before, after)) => {
            let (parent, start) = before.rsplit_once('/').unwrap_or(("", before));
            let (end, rest) = after.split_at(after.find('/').unwrap_or(after.len()));
            let name = format!("{start}*{end}");
            fs::read_dir(checkout.root.join(parent))
                .into_iter()
                .flatten()
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|entry| glob(&name, entry))
                .map(|entry| format!("{}{rest}", under(parent, &entry)))
                .collect()
        }
    };

    let mut paths: Vec<String> = paths
        .into_iter()
        .filter(|p| checkout.root.join(p).exists())
        .collect();
    paths.sort();

    paths
}

/// Problems with the stakeholder's ownership: the ENG-21 row no longer
/// names a class this check maps, or a guiding path that exists is not
/// owned by the stakeholder alone.
#[must_use]
pub fn stakeholder_owns_the_enforcing_paths(checkout: &Checkout) -> Vec<String> {
    let (stakeholder, entries) = match (stakeholder(checkout), load(checkout)) {
        (Ok(s), Ok(e)) => (s, e),
        (r, e) => return [r.err(), e.err()].into_iter().flatten().collect(),
    };
    let row = match checkout.requirement("ENG-21") {
        Ok(row) => row.text,
        Err(e) => return vec![e],
    };
    let mut problems: Vec<String> = ENFORCING
        .iter()
        .filter(|(phrase, _)| !row.contains(phrase))
        .map(|(phrase, _)| format!("the ENG-21 row no longer says {phrase:?}"))
        .collect();
    problems.extend(unsupported_patterns(&entries));
    for (class, path) in existing_enforcing(checkout) {
        let dir = if checkout.root.join(&path).is_dir() {
            format!("{path}/")
        } else {
            path.clone()
        };
        let owners = owners_of(&entries, &dir);
        if owners != [stakeholder.clone()] {
            problems.push(format!(
                "{path} ({class}) is owned by {}, want {stakeholder}",
                crate::list(owners),
            ));
        }
    }

    problems
}

/// Problems with entries that name a path outside the guiding classes.
#[must_use]
pub fn no_owner_on_any_other_path(checkout: &Checkout) -> Vec<String> {
    let entries = match load(checkout) {
        Ok(e) => e,
        Err(e) => return vec![e],
    };
    let allowed: Vec<String> = ENFORCING
        .iter()
        .flat_map(|(_, paths)| paths.iter())
        .map(|p| (*p).to_owned())
        .collect();

    let mut problems = unsupported_patterns(&entries);
    problems.extend(
        entries
            .iter()
            .filter(|e| !e.owners.is_empty() && !within(&allowed, &e.pattern))
            .map(|e| {
                format!(
                    "{} names an owner on a path that does not guide the review",
                    e.pattern
                )
            }),
    );

    problems
}

/// Whether `pattern` is one of `allowed` or lies under a directory of
/// it.
fn within(allowed: &[String], pattern: &str) -> bool {
    let pattern = pattern.trim_start_matches('/');
    allowed
        .iter()
        .any(|a| pattern == a || (a.ends_with('/') && pattern.starts_with(a.as_str())))
}

/// Problems with the entries that use syntax this reader cannot judge.
fn unsupported_patterns(entries: &[Entry]) -> Vec<String> {
    entries
        .iter()
        .filter(|e| unsupported(&e.pattern))
        .map(|e| {
            format!(
                "{} uses CODEOWNERS syntax this check does not read",
                e.pattern
            )
        })
        .collect()
}

/// Problems with the owners of Rust source: each `.rs` file of the
/// checkout, outside build output and the review's crates, that an
/// entry gives an owner.
#[must_use]
pub fn no_owner_on_rust_source(checkout: &Checkout) -> Vec<String> {
    let entries = match load(checkout) {
        Ok(e) => e,
        Err(e) => return vec![e],
    };

    let found = rust_sources(checkout, "");
    let mut problems = found.unread;
    let product = |path: &String| !matches(REVIEW_CRATES, path);
    problems.extend(
        found
            .sources
            .into_iter()
            .filter(product)
            .filter_map(|path| {
                let owners = owners_of(&entries, &path);
                (!owners.is_empty())
                    .then(|| format!("{path} is Rust source owned by {}", crate::list(owners)))
            }),
    );

    problems
}

/// The `.rs` files a walk found, and the directories it could not read.
#[derive(Debug, Default)]
struct Sources {
    sources: Vec<String>,
    unread: Vec<String>,
}

/// The path of `name` in `dir`, relative to the checkout's root, where
/// an empty `dir` is the root itself.
fn under(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_owned()
    } else {
        format!("{dir}/{name}")
    }
}

/// The `.rs` files under `dir` of the checkout, relative to its root,
/// in name order, leaving out [`NOT_SOURCE`] and [`OTHER_CHECKOUTS`].
/// A symlink is never followed, so a loop cannot recurse; a directory
/// that cannot be read, or an entry whose name is not UTF-8, is named,
/// so the check never passes without having looked.
fn rust_sources(checkout: &Checkout, dir: &str) -> Sources {
    let mut found = Sources::default();
    // One read of the directory and all its entries, so an error in
    // either is the same named problem.
    let entries = fs::read_dir(checkout.root.join(dir))
        .and_then(|entries| entries.collect::<Result<Vec<_>, _>>());
    let entries = match entries {
        Ok(entries) => entries,
        Err(e) => {
            found.unread.push(format!("read {dir}: {e}"));
            return found;
        }
    };
    let mut names: Vec<(String, bool)> = Vec::new();
    for e in &entries {
        let is_dir = e.file_type().is_ok_and(|t| t.is_dir());
        match e.file_name().into_string() {
            Ok(name) => names.push((name, is_dir)),
            Err(name) => found.unread.push(format!(
                "{} is not UTF-8",
                under(dir, &name.to_string_lossy())
            )),
        }
    }
    names.sort();
    for (name, is_dir) in names {
        let path = under(dir, &name);
        if is_dir && !NOT_SOURCE.contains(&name.as_str()) && path != OTHER_CHECKOUTS {
            let inner = rust_sources(checkout, &path);
            found.sources.extend(inner.sources);
            found.unread.extend(inner.unread);
        } else if !is_dir && name.ends_with(".rs") {
            found.sources.push(path);
        }
    }

    found
}

#[cfg(test)]
mod tests;
