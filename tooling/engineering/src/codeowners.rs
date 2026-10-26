//! ENG-21: `CODEOWNERS` names the stakeholder on the specification, the
//! scenarios and the process that builds and checks the code, on
//! nothing else but the supply-chain policy, and on no Rust source.
//!
//! The enforcing paths come from the ENG-21 row of the SRS: each class
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

/// What the ENG-21 row names as enforcing the requirements, as the
/// words it uses and the paths that hold each class. A path with a `*`
/// stands for each directory under its parent.
const ENFORCING: &[(&str, &[&str])] = &[
    ("`docs/srs/`", &["docs/srs/"]),
    ("the decision records", &["docs/adr/"]),
    ("the scenarios (`features/`)", &["features/"]),
    // Every mise configuration and lockfile: CI's install step reads
    // them all to refuse a tool that would build from source, and the
    // developer pins install the tools agents run.
    (
        "CI,",
        &[
            ".github/",
            "scripts/",
            "mise.toml",
            "mise.lock",
            "mise.dev.toml",
            "mise.dev.lock",
        ],
    ),
    (
        "the build, lint and coverage configuration",
        &[
            "Cargo.toml",
            "rust-toolchain.toml",
            "clippy.toml",
            "deny.toml",
            "codecov.yml",
            ".mdsmith.yml",
            "crates/*/clippy.toml",
            "crates/*/Cargo.toml",
            "tooling/*/Cargo.toml",
            "tooling/benchmark/accepted.txt",
            "tooling/eval/recall-v1.txt",
        ],
    ),
    (
        "the agent instructions",
        &["CLAUDE.md", "AGENTS.md", ".claude/", "docs/agents/"],
    ),
    ("`CODEOWNERS` itself", &[".github/CODEOWNERS"]),
];

/// The paths besides the enforcing ones that CODEOWNERS may name: the
/// supply-chain and disclosure policy (ENG-18, ENG-24).
const OTHER_ALLOWED: &[&str] = &["SECURITY.md", "DEPENDENCIES.md"];

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

/// The enforcing paths of the checkout that exist, with the class each
/// belongs to. A path with a `*` stands for each crate directory.
fn existing_enforcing(checkout: &Checkout) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    for (class, paths) in ENFORCING {
        for path in *paths {
            out.extend(expand(checkout, path).into_iter().map(|p| (*class, p)));
        }
    }

    out
}

/// `path`, or each path it stands for, when it exists in the checkout.
fn expand(checkout: &Checkout, path: &str) -> Vec<String> {
    let paths = match path.split_once("/*/") {
        None => vec![path.to_owned()],
        Some((parent, rest)) => fs::read_dir(checkout.root.join(parent))
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .map(|dir| format!("{parent}/{dir}/{rest}"))
            .collect(),
    };

    paths
        .into_iter()
        .filter(|p| checkout.root.join(p).exists())
        .collect()
}

/// Problems with the stakeholder's ownership: the ENG-21 row no longer
/// names a class this check maps, or an enforcing path or the
/// requirement text that exists is not owned by the stakeholder alone.
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

/// Problems with entries that name a path outside the allowed classes:
/// the enforcing paths and the supply-chain policy.
#[must_use]
pub fn no_owner_on_any_other_path(checkout: &Checkout) -> Vec<String> {
    let entries = match load(checkout) {
        Ok(e) => e,
        Err(e) => return vec![e],
    };
    let allowed: Vec<String> = ENFORCING
        .iter()
        .flat_map(|(_, paths)| paths.iter())
        .chain(OTHER_ALLOWED)
        .map(|p| (*p).to_owned())
        .collect();

    let mut problems = unsupported_patterns(&entries);
    problems.extend(
        entries
            .iter()
            .filter(|e| !e.owners.is_empty() && !within(&allowed, &e.pattern))
            .map(|e| {
                format!(
                    "{} names an owner on a path outside the allowed classes",
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
/// checkout, outside build output, that an entry gives an owner.
#[must_use]
pub fn no_owner_on_rust_source(checkout: &Checkout) -> Vec<String> {
    let entries = match load(checkout) {
        Ok(e) => e,
        Err(e) => return vec![e],
    };

    let found = rust_sources(checkout, "");
    let mut problems = found.unread;
    problems.extend(found.sources.into_iter().filter_map(|path| {
        let owners = owners_of(&entries, &path);
        (!owners.is_empty())
            .then(|| format!("{path} is Rust source owned by {}", crate::list(owners)))
    }));

    problems
}

/// The `.rs` files a walk found, and the directories it could not read.
#[derive(Debug, Default)]
struct Sources {
    sources: Vec<String>,
    unread: Vec<String>,
}

/// The `.rs` files under `dir` of the checkout, relative to its root,
/// in name order, leaving out [`NOT_SOURCE`] and [`OTHER_CHECKOUTS`].
/// A symlink is never followed, so a loop cannot recurse; a directory
/// that cannot be read is named, so the check never passes without
/// having looked.
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
    let mut names: Vec<(String, bool)> = entries
        .iter()
        .filter_map(|e| {
            let is_dir = e.file_type().is_ok_and(|t| t.is_dir());
            Some((e.file_name().into_string().ok()?, is_dir))
        })
        .collect();
    names.sort();
    for (name, is_dir) in names {
        let path = if dir.is_empty() {
            name.clone()
        } else {
            format!("{dir}/{name}")
        };
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
