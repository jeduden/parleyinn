//! ENG-34: the review bundle flags, with no agent, each changed file
//! past its kind's line limit and each changed file no review
//! specialist reads. The check holds the bundle's limits to those the
//! ENG-34 row states, then runs the review bundle on a change in a
//! fixture tree, once with a Rust file past its limit and a file no
//! specialist reads and once with files within their limits that
//! specialists read, and reads the findings it wrote.

use std::fs;
use std::path::Path;

use review_bundle::cli;
use review_bundle::limits::{LIMITS, RUST_LINES};
use serde_json::Value;

use crate::Checkout;

/// The requirement that sets the limits.
pub const ROW: &str = "ENG-34";
/// The requirement a file no specialist reads leaves unmet.
pub const UNREAD_ID: &str = "ENG-28";
/// The Rust file the change past the limit adds.
pub const OVERSIZE: &str = "crates/a/src/lib.rs";
/// The file in no language a specialist reads that it adds.
pub const UNREAD: &str = "LICENSE";

/// What separates a finding's path from its consequence.
const FORM: &str = " — if not addressed: ";

/// The fixture tree's requirement table, scenarios and invariants: the
/// least the review bundle reads.
const TREE: [(&str, &str); 3] = [
    (
        "docs/srs/05.md",
        "# 5\n\n| ID | Pri | Requirement | Ver | Traces |\n|---|---|---|---|---|\n| REC-01 | P0 | Ingest. | T | I1 |\n",
    ),
    (
        "features/record.feature",
        "Feature: Record\n\n  @REC-01 @P0 @I1\n  Scenario: ingest\n    Given a transcript\n",
    ),
    ("docs/srs/invariants.md", "I1 holds.\n"),
];

/// One finding the review bundle wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nit {
    pub path: String,
    pub severity: String,
    pub body: String,
}

/// What one run of the review bundle wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Built {
    pub checked: Vec<Nit>,
    pub bundle: String,
}

/// Problems with the bundle's limits against the ENG-34 row of the
/// checkout's SRS.
#[must_use]
pub fn limits_match_the_row(checkout: &Checkout) -> Vec<String> {
    match srs::load(&checkout.root.join("docs/srs")) {
        Err(e) => vec![format!("read docs/srs: {e}")],
        Ok(rows) => match rows.iter().find(|r| r.id == ROW) {
            None => vec![format!("no {ROW} row in docs/srs")],
            Some(row) => limits_match(&row.text),
        },
    }
}

/// Problems with the bundle's limits against `text`, the ENG-34 row's:
/// each limit one holds and the other does not.
#[must_use]
pub fn limits_match(text: &str) -> Vec<String> {
    let stated = stated_limits(text);
    let held: Vec<(String, usize)> = LIMITS
        .iter()
        .map(|(kind, _, limit)| ((*kind).to_owned(), *limit))
        .collect();
    let missing = held.iter().filter(|l| !stated.contains(l)).map(|(k, n)| {
        format!("the {ROW} row states no limit of {n} for {k}, which the review bundle holds")
    });
    let extra = stated.iter().filter(|l| !held.contains(l)).map(|(k, n)| {
        format!("the review bundle holds no limit of {n} for {k}, which the {ROW} row states")
    });

    missing.chain(extra).collect()
}

/// The limits `text` states, each as `<lines> for <kind> (`…: the kind
/// and its lines.
fn stated_limits(text: &str) -> Vec<(String, usize)> {
    text.match_indices(" for ")
        .filter_map(|(at, sep)| {
            let limit = text[..at].rsplit(' ').next()?.parse().ok()?;
            let (kind, rest) = text[at + sep.len()..].split_once(' ')?;
            rest.starts_with("(`").then(|| (kind.to_owned(), limit))
        })
        .collect()
}

/// The change past the limit: a Rust file one line past it, and a file
/// no specialist reads.
#[must_use]
pub fn past_the_limit() -> Vec<(&'static str, String)> {
    vec![
        (OVERSIZE, "// x\n".repeat(RUST_LINES + 1)),
        (UNREAD, "text\n".to_owned()),
    ]
}

/// The change within the limits: files at most at their limits, each
/// one a specialist reads.
#[must_use]
pub fn within_the_limits() -> Vec<(&'static str, String)> {
    vec![
        (OVERSIZE, "// x\n".repeat(RUST_LINES)),
        ("scripts/a.sh", "true\n".to_owned()),
    ]
}

/// Runs the review bundle under `work` on a change that adds `files` to
/// the fixture tree, and reads what it wrote.
///
/// # Errors
///
/// Fails when the trees cannot be written or the review bundle fails.
pub fn build(work: &Path, files: &[(&str, String)]) -> Result<Built, String> {
    let (base, head, out) = (work.join("base"), work.join("head"), work.join("out"));
    let added = files.iter().map(|(p, b)| (*p, b.as_str()));
    write_tree(&base, TREE.into_iter())?;
    write_tree(&head, TREE.into_iter().chain(added))?;
    let diff: String = files
        .iter()
        .map(|(p, _)| format!("diff --git a/{p} b/{p}\n+x\n"))
        .collect();
    write_tree(
        work,
        [("pr.diff", diff.as_str()), ("consent.json", "[]")].into_iter(),
    )?;
    fs::create_dir_all(&out).map_err(|e| format!("make {}: {e}", out.display()))?;
    let at = |p: &Path| p.display().to_string();
    let args = [
        "--base".to_owned(),
        at(&base),
        "--head".to_owned(),
        at(&head),
        "--diff".to_owned(),
        at(&work.join("pr.diff")),
        "--out".to_owned(),
        at(&out),
        "--consent".to_owned(),
        at(&work.join("consent.json")),
    ];
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    if cli::run(args, &mut stdout, &mut stderr) != cli::EXIT_OK {
        return Err(String::from_utf8_lossy(&stderr).into_owned());
    }
    let read =
        |name: &str| fs::read_to_string(out.join(name)).map_err(|e| format!("read {name}: {e}"));

    Ok(Built {
        checked: read_checked(&read(cli::CHECKED)?)?,
        bundle: read(cli::BUNDLE)?,
    })
}

/// Writes each of `files`, a path under `root` with its text.
fn write_tree<'a>(
    root: &Path,
    files: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<(), String> {
    for (rel, body) in files {
        let path = root.join(rel);
        let failed = |e: std::io::Error| format!("write {}: {e}", path.display());
        fs::create_dir_all(path.parent().unwrap_or(root)).map_err(failed)?;
        fs::write(&path, body).map_err(failed)?;
    }

    Ok(())
}

/// The findings of `json`, the review bundle's `checked.json`.
fn read_checked(json: &str) -> Result<Vec<Nit>, String> {
    let value: Value =
        serde_json::from_str(json).map_err(|e| format!("read {}: {e}", cli::CHECKED))?;
    let field = |f: &Value, name: &str| f[name].as_str().unwrap_or_default().to_owned();

    let findings = value
        .as_array()
        .ok_or_else(|| format!("read {}: not an array of findings", cli::CHECKED))?;

    Ok(findings
        .iter()
        .map(|f| Nit {
            path: field(f, "path"),
            severity: field(f, "severity"),
            body: field(f, "body"),
        })
        .collect())
}

/// Whether `nit` is a nit at `path` in the finding form, citing `id`.
fn cites(nit: &Nit, path: &str, id: &str) -> bool {
    nit.path == path
        && nit.severity == "nit"
        && nit.body.starts_with(&format!("{path}{FORM}"))
        && nit.body.ends_with(&format!(" — {id}"))
}

/// Whether `nit` cites ENG-34 or the ENG-28 of a file no specialist
/// reads.
fn ours(nit: &Nit) -> bool {
    [ROW, UNREAD_ID].iter().any(|id| cites(nit, &nit.path, id))
}

/// The problem when `built` holds no nit at `path` in the finding form
/// citing `id`.
#[must_use]
pub fn nit_at(built: &Built, path: &str, id: &str) -> Vec<String> {
    if built.checked.iter().any(|n| cites(n, path, id)) {
        return Vec::new();
    }
    let bodies: Vec<&str> = built.checked.iter().map(|n| n.body.as_str()).collect();
    vec![format!(
        "no nit at {path} in the finding form citing {id}, among: {}",
        bodies.join("; ")
    )]
}

/// The problems when the review bundle of `built` does not list each of
/// its ENG-34 and ENG-28 nits, or holds none.
#[must_use]
pub fn listed(built: &Built) -> Vec<String> {
    let nits: Vec<&Nit> = built.checked.iter().filter(|n| ours(n)).collect();
    if nits.is_empty() {
        return vec![format!(
            "the review bundle lists no {ROW} or {UNREAD_ID} nit"
        )];
    }
    nits.iter()
        .filter(|n| {
            !built
                .bundle
                .contains(&format!("- nit `{}:0`: {}", n.path, n.body))
        })
        .map(|n| format!("the review bundle does not list the nit at {}", n.path))
        .collect()
}

/// A problem for each nit of `built` that cites ENG-34 or the ENG-28 of
/// a file no specialist reads.
#[must_use]
pub fn no_nit(built: &Built) -> Vec<String> {
    built
        .checked
        .iter()
        .filter(|n| ours(n))
        .map(|n| format!("a nit at {}: {}", n.path, n.body))
        .collect()
}

#[cfg(test)]
mod tests;
