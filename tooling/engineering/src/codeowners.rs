//! ENG-21: `CODEOWNERS` names the stakeholder on the requirement text
//! and on everything that enforces it, and on nothing else but the
//! decision records, the supply-chain policy and the security-sensitive
//! crates.
//!
//! The enforcing paths come from the ENG-21 row of the SRS: each class
//! it names maps to the paths that hold it today, and a class the row
//! stops naming is reported, so the mapping cannot go stale unseen. The
//! stakeholder and the security-sensitive crates are read from the SRS
//! as well.

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
    ("the requirement–scenario gate", &["tooling/"]),
    // Every mise configuration and lockfile: CI's install step reads
    // them all to refuse a tool that would build from source, and the
    // developer pins install the tools agents run.
    (
        "gate, CI,",
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
        "the lint and coverage configuration",
        &[
            "Cargo.toml",
            "rust-toolchain.toml",
            "clippy.toml",
            "deny.toml",
            "codecov.yml",
            ".mdsmith.yml",
            "crates/*/clippy.toml",
            "crates/*/Cargo.toml",
            "crates/*/tests/reach.rs",
        ],
    ),
    (
        "the agent instructions",
        &["CLAUDE.md", "AGENTS.md", ".claude/"],
    ),
    ("`CODEOWNERS` itself", &[".github/CODEOWNERS"]),
];

/// The paths besides the enforcing ones that CODEOWNERS may name: the
/// decision records (ENG-26) and the supply-chain and disclosure policy
/// (ENG-18, ENG-24).
const OTHER_ALLOWED: &[&str] = &["docs/adr/", "SECURITY.md", "DEPENDENCIES.md"];

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

/// The stakeholder and the security reviewer, as the SRS names them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roles {
    pub stakeholder: String,
    pub security_reviewer: String,
}

/// Reads the roles from the SRS index's `Stakeholder` row, which says
/// who the stakeholder is and whether that person is also the security
/// reviewer.
///
/// # Errors
///
/// Fails when the index cannot be read or has no stakeholder row naming
/// a user.
pub fn roles(checkout: &Checkout) -> Result<Roles, String> {
    let index = checkout.read("docs/srs/index.md")?;
    let row = index
        .lines()
        .find(|l| l.contains("**Stakeholder**"))
        .ok_or("docs/srs/index.md has no Stakeholder row")?;
    let who = row
        .split_whitespace()
        .find(|w| w.starts_with('@'))
        .map(|w| w.trim_end_matches([',', '|', '.']).to_owned())
        .ok_or("the Stakeholder row names no user")?;
    let security_reviewer = if row.contains("the security reviewer") {
        who.clone()
    } else {
        String::new()
    };

    Ok(Roles {
        stakeholder: who,
        security_reviewer,
    })
}

/// The security-sensitive crates, as the ENG-11 row names them.
///
/// # Errors
///
/// Fails when the SRS has no such row or the row lists no crate.
pub fn security_crates(checkout: &Checkout) -> Result<Vec<String>, String> {
    let text = checkout.requirement("ENG-11")?.text;
    let (_, after) = text
        .split_once("security-sensitive crates (")
        .ok_or("ENG-11 names no security-sensitive crates")?;
    let (list, _) = after
        .split_once(')')
        .ok_or("ENG-11's crate list is not closed")?;
    let crates: Vec<String> = list
        .split(',')
        .map(|c| c.trim().trim_matches('`').to_owned())
        .filter(|c| !c.is_empty())
        .collect();
    if crates.is_empty() {
        Err("ENG-11 lists no security-sensitive crate".to_owned())
    } else {
        Ok(crates)
    }
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
    let (roles, entries) = match (roles(checkout), load(checkout)) {
        (Ok(r), Ok(e)) => (r, e),
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
        if owners != [roles.stakeholder.clone()] {
            problems.push(format!(
                "{path} ({class}) is owned by {}, want {}",
                crate::list(owners),
                roles.stakeholder
            ));
        }
    }

    problems
}

/// Problems with entries that name a path outside the allowed classes:
/// the enforcing paths, the decision records, the supply-chain policy
/// and the security-sensitive crates.
#[must_use]
pub fn no_owner_on_any_other_path(checkout: &Checkout) -> Vec<String> {
    let entries = match load(checkout) {
        Ok(e) => e,
        Err(e) => return vec![e],
    };
    let crates = match security_crates(checkout) {
        Ok(c) => c,
        Err(e) => return vec![e],
    };
    let mut allowed: Vec<String> = ENFORCING
        .iter()
        .flat_map(|(_, paths)| paths.iter())
        .chain(OTHER_ALLOWED)
        .map(|p| (*p).to_owned())
        .collect();
    allowed.extend(crates.iter().map(|c| format!("crates/{c}/")));

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

/// Problems with the security-sensitive crates' owner: each crate the
/// SRS names, built or not, is owned by the security reviewer alone.
#[must_use]
pub fn security_crates_owned_by_the_security_reviewer(checkout: &Checkout) -> Vec<String> {
    let (roles, entries, crates) =
        match (roles(checkout), load(checkout), security_crates(checkout)) {
            (Ok(r), Ok(e), Ok(c)) => (r, e, c),
            (r, e, c) => return [r.err(), e.err(), c.err()].into_iter().flatten().collect(),
        };
    if roles.security_reviewer.is_empty() {
        return vec!["the SRS index names no security reviewer".to_owned()];
    }

    crates
        .iter()
        .filter(|c| {
            owners_of(&entries, &format!("crates/{c}/")) != [roles.security_reviewer.clone()]
        })
        .map(|c| format!("crates/{c}/ is not owned by {}", roles.security_reviewer))
        .collect()
}

#[cfg(test)]
mod tests;
