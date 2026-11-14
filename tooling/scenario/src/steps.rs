//! The step gate: every step definition under `tests/bdd` is used by a
//! row of a scenario that runs, or stands on a list that only shrinks.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use cucumber::codegen::Regex;

use crate::{Error, Scenario};

/// One step definition: the function an attribute binds to a step text.
#[derive(Debug, Clone)]
pub struct StepDef {
    /// The file's name, without its directory.
    pub file: String,
    /// The function's name.
    pub name: String,
    /// `given`, `when` or `then`.
    pub kind: String,
    pattern: Pattern,
}

#[derive(Debug, Clone)]
enum Pattern {
    Regex(Regex),
    Literal(String),
}

impl StepDef {
    /// The entry that names this definition on the list: its file and
    /// function, which no edit above it moves.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{} {}", self.file, self.name)
    }

    fn matches(&self, kind: &str, text: &str) -> bool {
        self.kind == kind
            && match &self.pattern {
                Pattern::Regex(rx) => rx.is_match(text),
                Pattern::Literal(lit) => lit == text,
            }
    }
}

/// Every step definition in the `.rs` files directly under `dir`.
///
/// # Errors
///
/// Fails when a file cannot be read, or an attribute's pattern is not
/// one this reader understands: a step it cannot read must not pass as
/// one it need not check.
pub fn step_definitions(dir: &Path) -> Result<Vec<StepDef>, Error> {
    let walk = |source| Error::Walk {
        path: dir.to_path_buf(),
        source,
    };
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).map_err(walk)? {
        let path = entry.map_err(walk)?.path();
        if path.extension().is_some_and(|e| e == "rs") {
            paths.push(path);
        }
    }
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let text = fs::read_to_string(&path).map_err(walk)?;
        let file = path.file_name().map(|n| n.to_string_lossy().into_owned());
        out.extend(definitions_in(&file.unwrap_or_default(), &text)?);
    }

    Ok(out)
}

/// The step definitions of one file's `text`.
fn definitions_in(file: &str, text: &str) -> Result<Vec<StepDef>, Error> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some((kind, rest)) = attribute(line) else {
            continue;
        };
        let at = format!("{file}:{}", i + 1);
        let body = attribute_body(rest, &lines[i + 1..]);
        let pattern = pattern_of(&body).map_err(|why| Error::Steps(format!("{at}: {why}")))?;
        let name = lines[i + 1..]
            .iter()
            .find_map(|l| fn_name(l))
            .ok_or_else(|| Error::Steps(format!("{at}: no function follows the attribute")))?;
        out.push(StepDef {
            file: file.to_owned(),
            name,
            kind: kind.to_owned(),
            pattern,
        });
    }

    Ok(out)
}

/// The kind and the text after the opening of a step attribute that
/// starts `line`.
fn attribute(line: &str) -> Option<(&'static str, &str)> {
    ["given", "when", "then"].into_iter().find_map(|kind| {
        let open = format!("#[{kind}(");
        line.strip_prefix(open.as_str()).map(|rest| (kind, rest))
    })
}

/// An attribute's text up to its close, which may lie on later lines.
fn attribute_body(first: &str, later: &[&str]) -> String {
    let mut body = first.to_owned();
    for l in later {
        if body.trim_end().ends_with(")]") {
            break;
        }
        body.push('\n');
        body.push_str(l);
    }

    body
}

/// The function name of a declaration line.
fn fn_name(line: &str) -> Option<String> {
    let rest = line.trim_start();
    let rest = rest.strip_prefix("pub ").unwrap_or(rest);
    let rest = rest.strip_prefix("async ").unwrap_or(rest);
    let rest = rest.strip_prefix("fn ")?;
    let end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;

    Some(rest[..end].to_owned())
}

/// Reads the pattern of an attribute body: `regex = <string>` or a bare
/// literal. A literal with a cucumber-expression parameter is refused.
fn pattern_of(body: &str) -> Result<Pattern, String> {
    let body = body.trim_start();
    let (is_regex, body) = match body.strip_prefix("regex") {
        Some(rest) => (true, rest.trim_start().strip_prefix('=').unwrap_or(rest)),
        None => (false, body),
    };
    let text = string_literal(body.trim_start())?;
    if is_regex {
        return Regex::new(&text)
            .map(Pattern::Regex)
            .map_err(|e| format!("pattern {text:?}: {e}"));
    }
    if text.contains(['{', '(', '/']) {
        return Err(format!(
            "expression {text:?} has parameters or alternatives"
        ));
    }

    Ok(Pattern::Literal(text))
}

/// The value of the string literal, plain or raw, that starts `s`.
fn string_literal(s: &str) -> Result<String, String> {
    if let Some(raw) = s.strip_prefix('r') {
        let hashes = raw.chars().take_while(|&c| c == '#').count();
        let open = &raw[hashes..];
        let open = open.strip_prefix('"').ok_or("not a string literal")?;
        let close = format!("\"{}", "#".repeat(hashes));
        let end = open.find(&close).ok_or("unterminated raw string")?;
        return Ok(open[..end].to_owned());
    }
    let inner = s.strip_prefix('"').ok_or("not a string literal")?;
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Ok(out),
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => break,
            },
            c => out.push(c),
        }
    }

    Err("unterminated string".to_owned())
}

/// The step texts of every row of every scenario that runs, each with
/// its kind: an outline's placeholders filled from each Examples row.
fn running_steps(scenarios: &[Scenario]) -> Vec<(&str, String)> {
    let mut out = Vec::new();
    for sc in scenarios.iter().filter(|s| !s.pending) {
        let rows: Vec<_> = if sc.examples.is_empty() {
            vec![None]
        } else {
            sc.examples.iter().map(Some).collect()
        };
        for row in rows {
            for (kind, text) in sc.step_kinds.iter().zip(&sc.steps) {
                let mut text = text.clone();
                for (head, cell) in row.into_iter().flatten() {
                    text = text.replace(&format!("<{head}>"), cell);
                }
                out.push((kind.as_str(), text));
            }
        }
    }

    out
}

/// Compares the step definitions against the scenarios that run and the
/// list `allowed` of definitions no running row uses, and returns every
/// disagreement, sorted, one line each. It is empty exactly when:
///
/// - every definition that no row of a scenario that runs uses stands
///   on the list;
/// - every entry of the list names a definition that exists and that no
///   such row uses, so the list only shrinks.
#[must_use]
pub fn check_steps(defs: &[StepDef], scenarios: &[Scenario], allowed: &[String]) -> Vec<String> {
    let steps = running_steps(scenarios);
    let unused: BTreeSet<String> = defs
        .iter()
        .filter(|d| !steps.iter().any(|(kind, text)| d.matches(kind, text)))
        .map(StepDef::key)
        .collect();
    let known: BTreeSet<String> = defs.iter().map(StepDef::key).collect();
    let listed: BTreeSet<&str> = allowed.iter().map(String::as_str).collect();
    let mut problems = Vec::new();
    for key in unused.iter().filter(|k| !listed.contains(k.as_str())) {
        problems.push(format!(
            "{key}: no row of a scenario that runs uses this step definition; \
             write the row, or delete the definition"
        ));
    }
    for key in &listed {
        if !known.contains(*key) {
            problems.push(format!(
                "{key}: listed, but no step definition has that name; remove it from the list"
            ));
        } else if !unused.contains(*key) {
            problems.push(format!(
                "{key}: listed, but a row that runs now uses it; remove it from the list"
            ));
        }
    }
    problems.sort();

    problems
}

#[cfg(test)]
mod tests;
