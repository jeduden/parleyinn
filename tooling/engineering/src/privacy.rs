//! ENG-22: every privacy or data-flow statement in the documentation
//! cites the test that proves it. A statement is a line of prose or of a
//! table that says something leaves the machine, or that nothing does:
//! it holds one of [`PHRASES`]. It cites a test after `test:` or
//! `tests:`, in backticks: the name of a test function that exists and
//! is not ignored, or the `@ID` of a scenario that is not pending. A
//! test in the workspace passes when CI is green, which the CI gates
//! hold; a cited name that does not exist fails here.

use std::fs;
use std::path::Path;

use crate::Checkout;

/// The phrases that make a line a privacy or data-flow statement, in
/// lower case.
pub const PHRASES: [&str; 8] = [
    "telemetry",
    "opens no socket",
    "no outbound",
    "no network",
    "off the machine",
    "never leaves",
    "sends no",
    "leaves the machine",
];

/// The documentation the statements are read in: these files and the
/// Markdown files directly in these directories.
const FILES: [&str; 1] = ["SECURITY.md"];
const DIRECTORIES: [&str; 4] = ["docs", "docs/security", "docs/testing", "docs/personas"];

/// One statement and the tests it cites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub file: String,
    pub line: usize,
    pub cites: Vec<String>,
}

/// The tests a block of prose or one table row cites: after `test:` or
/// `tests:`, backtick spans joined by commas or `and`; each is a test
/// function's name or a scenario's `@ID`.
fn cites(block: &str) -> Vec<String> {
    let mut out = Vec::new();
    let flat = block.replace('\n', " ");
    let mut rest = flat.as_str();
    while let Some(at) = ["test: ", "tests: "]
        .iter()
        .filter_map(|m| rest.find(m))
        .min()
    {
        let (_, after) = rest.split_at(at);
        let mut text = after.split_once(": ").map_or("", |(_, t)| t.trim_start());
        while let Some(span) = text.strip_prefix('`').and_then(|t| t.split_once('`')) {
            out.push(span.0.to_owned());
            text = span.1;
            let joined = [", and ", ", ", " and "]
                .iter()
                .find_map(|j| text.strip_prefix(j));
            match joined {
                Some(t) => text = t.trim_start(),
                None => break,
            }
        }
        rest = text;
    }

    out
}

/// The statements of `body`, a file named `file`, code fences left out.
/// A table row is a block of its own; elsewhere a block runs to the next
/// blank line, so a citation may sit on a later line of the paragraph.
#[must_use]
pub fn statements(file: &str, body: &str) -> Vec<Statement> {
    let mut fenced = false;
    let mut blocks: Vec<(usize, String)> = Vec::new();
    let mut open = false;
    for (i, line) in body.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            open = false;
            continue;
        }
        if fenced {
            continue;
        }
        let table = line.trim_start().starts_with('|');
        if line.trim().is_empty() {
            open = false;
        } else if open && !table {
            if let Some((_, text)) = blocks.last_mut() {
                text.push('\n');
                text.push_str(line);
            }
        } else {
            blocks.push((i + 1, line.to_owned()));
            open = !table;
        }
    }

    blocks
        .into_iter()
        .filter(|(_, text)| {
            let lower = text.to_lowercase();
            PHRASES.iter().any(|p| lower.contains(p))
        })
        .map(|(line, text)| Statement {
            file: file.to_owned(),
            line,
            cites: cites(&text),
        })
        .collect()
}

/// Every test function in the workspace's Rust files, by name, with
/// whether it runs: `#[test]` before it and no `#[ignore]`.
///
/// # Errors
///
/// Fails when a directory cannot be listed or a file read.
pub fn test_functions(c: &Checkout) -> Result<Vec<(String, bool)>, String> {
    let mut files = Vec::new();
    for dir in ["crates", "tooling"] {
        rust_files(&c.root.join(dir), &mut files)?;
    }
    let mut out = Vec::new();
    for path in files {
        let body =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let lines: Vec<&str> = body.lines().collect();
        for (at, line) in lines.iter().enumerate() {
            let Some(name) = line
                .trim_start()
                .strip_prefix("fn ")
                .and_then(|r| r.split_once('(').map(|(n, _)| n))
            else {
                continue;
            };
            let before = &lines[at.saturating_sub(3)..at];
            if before.iter().any(|l| l.contains("#[test]")) {
                let ignored = before.iter().any(|l| l.contains("#[ignore"));
                out.push((name.to_owned(), !ignored));
            }
        }
    }

    Ok(out)
}

/// Collects the `.rs` files under `dir`.
fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let list = |e: std::io::Error| format!("list {}: {e}", dir.display());
    for entry in fs::read_dir(dir).map_err(list)? {
        let path = entry.map_err(list)?.path();
        if path.is_dir() {
            rust_files(&path, out)?;
        } else if path.extension().is_some_and(|x| x == "rs") {
            out.push(path);
        }
    }

    Ok(())
}

/// Whether `cite` names a test that exists and runs, or a scenario that
/// is not pending.
#[must_use]
pub fn resolves(tests: &[(String, bool)], scenarios: &[(String, bool)], cite: &str) -> bool {
    match cite.strip_prefix('@') {
        Some(id) => scenarios.iter().any(|(s, pending)| s == id && !pending),
        None => tests.iter().any(|(name, runs)| name == cite && *runs),
    }
}

/// The Markdown files the statements are read in.
fn documentation(c: &Checkout) -> Vec<String> {
    let mut found: Vec<String> = FILES.iter().map(|f| (*f).to_owned()).collect();
    for dir in DIRECTORIES {
        let Ok(entries) = fs::read_dir(c.root.join(dir)) else {
            continue;
        };
        let mut names: Vec<String> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "md") && p.is_file())
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        names.sort();
        found.extend(names.into_iter().map(|n| format!("{dir}/{n}")));
    }

    found
}

/// Every statement in the documentation, with the file it is in.
///
/// # Errors
///
/// Fails when a documentation file cannot be read.
pub fn all(c: &Checkout) -> Result<Vec<Statement>, String> {
    let mut out = Vec::new();
    for file in documentation(c) {
        out.extend(statements(&file, &c.read(&file)?));
    }

    Ok(out)
}

/// Requires each statement to cite at least one test, and every test it
/// cites to exist.
#[must_use]
pub fn proven(c: &Checkout, scenarios: &[(String, bool)]) -> Vec<String> {
    let (found, tests) = match (all(c), test_functions(c)) {
        (Ok(found), Ok(tests)) => (found, tests),
        (Err(e), _) | (_, Err(e)) => return vec![e],
    };
    let mut problems = Vec::new();
    for s in found {
        if s.cites.is_empty() {
            problems.push(format!(
                "{}:{}: a privacy statement cites no test",
                s.file, s.line
            ));
        }
        for cite in &s.cites {
            if !resolves(&tests, scenarios, cite) {
                problems.push(format!(
                    "{}:{}: cites {cite}, which is no test that exists and runs",
                    s.file, s.line
                ));
            }
        }
    }

    problems
}

/// The scenarios of the checkout, each by id and whether it is pending.
///
/// # Errors
///
/// Fails when the features cannot be read.
pub fn scenario_states(features: &Path) -> Result<Vec<(String, bool)>, String> {
    scenario::scenarios(features)
        .map(|all| all.into_iter().map(|s| (s.id, s.pending)).collect())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
