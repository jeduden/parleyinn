//! ENG-28: what the review bundle writes for an agent does not turn on
//! the stakeholder's comments. A review a comment brings takes the
//! agents' outcome of the same head as its own, so a comment may change
//! nothing an agent read: a line that restated a missing consent would
//! outlive the consent. The check runs the checkout's bundle twice on
//! one change that waits on every consent there is, a stakeholder path
//! (ENG-21), a guarded change with its record (ENG-29) and a lengthened
//! size list (ENG-34), once with no comment and once with a comment
//! that consents, and holds the bundle and every context to the same
//! bytes, each with the fixed sentence that says whose check consent is.

use std::fs;
use std::path::Path;

use review_bundle::cli::{BUNDLE, CHECKED, CONTEXTS};
use review_bundle::render::CONSENT;
use review_bundle::size_list::SIZE_LIST;

use crate::Checkout;
use crate::review_runs::{INVARIANTS, RECORD, bundle, one_line_diff};

/// The stakeholder path the change edits.
const OWNED: &str = "docs/agents/specialist-outcome.md";

/// How many findings of the change wait on a comment: ENG-21's,
/// ENG-29's and ENG-34's.
const WAITING: usize = 3;

/// The change: a line added to [`OWNED`], to the invariants and to
/// their record, and a file added to the size list.
fn change() -> String {
    let edits: String = [OWNED, INVARIANTS, RECORD]
        .iter()
        .map(|path| format!("{}+x\n", one_line_diff(path)))
        .collect();
    format!("{edits}{}+tooling/x.rs 1001\n", one_line_diff(SIZE_LIST))
}

/// The stakeholder's comment that consents to all of the change.
fn comment() -> String {
    serde_json::json!([{
        "author": "stakeholder",
        "at": "2026-01-01T00:00:00Z",
        "body": format!("I consent to docs/agents/ and {SIZE_LIST}."),
    }])
    .to_string()
}

/// What is wrong with what the review bundle, run on `checkout` as both
/// trees, writes for an agent of [`change`] under `out`: a file that
/// differs once the stakeholder consents, or one that does not say
/// whose check consent is.
#[must_use]
pub fn consent_problems(checkout: &Checkout, out: &Path) -> Vec<String> {
    let diff = change();
    let run = |name: &str, comments: &str| {
        let dir = out.join(name);
        bundle(checkout, &dir, &diff, comments).map(|_| dir)
    };
    match (run("unconsented", "[]"), run("consented", &comment())) {
        (Ok(without), Ok(with)) => differences(&without, &with),
        (without, with) => [without, with]
            .into_iter()
            .filter_map(Result::err)
            .flatten()
            .collect(),
    }
}

/// What is wrong with the files the review bundle wrote under
/// `without`, with no comment, beside those under `with`, with one that
/// consents: the comment moved fewer than [`WAITING`] blocking
/// findings, so the change proves nothing; no context was written; or
/// a file an agent reads differs or lacks [`CONSENT`].
#[must_use]
pub fn differences(without: &Path, with: &Path) -> Vec<String> {
    let read = |dir: &Path, rel: &str| {
        let file = dir.join(rel);
        fs::read_to_string(&file).map_err(|e| format!("read {}: {e}", file.display()))
    };
    let mut problems = Vec::new();
    let checked = |dir: &Path| {
        let file = dir.join(CHECKED);
        read(dir, CHECKED)
            .and_then(|text| blocking(&text).map_err(|e| format!("parse {}: {e}", file.display())))
    };
    match (checked(without), checked(with)) {
        (Ok(was), Ok(is)) => {
            let waiting = was.iter().filter(|body| !is.contains(body)).count();
            if waiting < WAITING {
                problems.push(format!(
                    "the comment lifts {waiting} blocking finding(s) of the change, not the \
                     {WAITING} that wait on consent (ENG-21, ENG-29, ENG-34)"
                ));
            }
        }
        (was, is) => problems.extend([was, is].into_iter().filter_map(Result::err)),
    }
    // Either run's contexts: one that only a comment brings, or takes
    // away, turns on the comment too.
    let mut contexts = contexts(without);
    contexts.extend(self::contexts(with));
    contexts.sort();
    contexts.dedup();
    if contexts.is_empty() {
        problems.push(format!(
            "the review bundle wrote no context under {}",
            without.join(CONTEXTS).display()
        ));
    }
    for rel in contexts.iter().map(String::as_str).chain([BUNDLE]) {
        match (read(without, rel), read(with, rel)) {
            (Ok(was), Ok(is)) => problems.extend(file_problems(rel, &was, &is)),
            (was, is) => problems.extend([was, is].into_iter().filter_map(Result::err)),
        }
    }

    problems
}

/// The bodies of the blocking findings `checked`, a deterministic
/// findings file, holds. A file that does not parse holds none to
/// count: taken for an empty one, it would count every finding lifted.
fn blocking(checked: &str) -> Result<Vec<String>, serde_json::Error> {
    let findings: Vec<serde_json::Value> = serde_json::from_str(checked)?;

    Ok(findings
        .iter()
        .filter(|f| f["severity"] == review_bundle::BLOCKING)
        .filter_map(|f| f["body"].as_str().map(str::to_owned))
        .collect())
}

/// The contexts the review bundle wrote under `out`, each relative to
/// it, sorted; none when the directory does not read.
fn contexts(out: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(out.join(CONTEXTS))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| format!("{CONTEXTS}/{}", entry.file_name().to_string_lossy()))
        .collect();
    names.sort();

    names
}

/// What is wrong with `rel`, a file an agent reads: `was` its text with
/// no comment, `is` with one that consents.
#[must_use]
pub fn file_problems(rel: &str, was: &str, is: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if was != is {
        problems.push(format!(
            "{rel} turns on the stakeholder's comments, though a comment's review takes the \
             agents' outcome as its own"
        ));
    }
    if !was.contains(CONSENT) {
        problems.push(format!(
            "{rel} does not tell its reader that consent is the review gate's check"
        ));
    }

    problems
}

#[cfg(test)]
mod tests;
