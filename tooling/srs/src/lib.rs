//! Reads the requirements of the Software Requirements Specification
//! under `docs/srs`, each a block under a heading that is its id, so the
//! test suite can hold the specification and the executable scenarios in
//! step, and refuses a requirement written as a table's row. It is
//! repository tooling for the gates in the `scenario` crate and never
//! links into a shipped executable.

mod appendix;
mod block;
mod personas;
mod table;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use appendix::{Counts, invariant_coverage, priority_counts, stated_counts, traced_coverage};
pub use block::{Block, blocks};
pub use personas::{persona_agents, persona_coverage};
pub use table::{Row, Table, fence_after, fence_marker, tables};

/// Every requirement family prefix, so the scenario gate reads ids the
/// same way.
pub const FAMILIES: [&str; 18] = [
    "REC", "PRV", "PIN", "RCL", "LMK", "INJ", "CMP", "ADM", "MEM", "OPS", "LANE", "VIEW", "OWN",
    "PEER", "SEC", "NFR", "ENG", "ASM",
];

/// One requirement: a block under a heading that is its id.
/// `priority` and `traces` are empty when it has no such field: the
/// non-functional requirements and the assumptions carry neither, the
/// engineering requirements no Traces. `line` is its heading's.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Requirement {
    pub id: String,
    pub priority: String,
    pub verify: String,
    pub traces: Vec<String>,
    pub text: String,
    pub path: PathBuf,
    pub line: usize,
}

/// What reading the specification can fail with.
#[derive(Debug)]
pub enum Error {
    /// The directory could not be listed.
    List { dir: PathBuf, source: io::Error },
    /// A file could not be read.
    Read { path: PathBuf, source: io::Error },
    /// A requirement is malformed or written as a table's row, or an
    /// id defined twice.
    Malformed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::List { dir, source } => write!(f, "srs: list {}: {source}", dir.display()),
            Self::Read { path, source } => write!(f, "srs: read {}: {source}", path.display()),
            Self::Malformed(msg) => write!(f, "srs: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::List { source, .. } | Self::Read { source, .. } => Some(source),
            Self::Malformed(_) => None,
        }
    }
}

/// Whether `s` is a requirement id: a family prefix and a two-digit
/// number, such as `REC-01` or `ASM-04`.
#[must_use]
pub fn is_requirement_id(s: &str) -> bool {
    s.split_once('-').is_some_and(|(family, n)| {
        FAMILIES.contains(&family) && n.len() == 2 && n.bytes().all(|b| b.is_ascii_digit())
    })
}

/// Whether `s` is one invariant, `I1` to `I10`.
#[must_use]
pub fn is_invariant(s: &str) -> bool {
    s == "I10" || numbered_once(s, 'I')
}

/// Whether `s` is `prefix` followed by one digit from 1 to 9.
fn numbered_once(s: &str, prefix: char) -> bool {
    let mut chars = s.chars();
    chars.next() == Some(prefix)
        && chars.next().is_some_and(|c| ('1'..='9').contains(&c))
        && chars.next().is_none()
}

/// Reads every requirement from the Markdown files directly in `dir`,
/// in file-name order, and reports an id two requirements share.
///
/// # Errors
///
/// Fails when `dir` or a file in it cannot be read, a requirement is
/// malformed or written as a table's row, or two share an id.
pub fn load(dir: &Path) -> Result<Vec<Requirement>, Error> {
    let mut all = Vec::new();
    let mut seen: BTreeMap<String, (PathBuf, usize)> = BTreeMap::new();
    for path in markdown_files(dir)? {
        let body = fs::read_to_string(&path).map_err(|source| Error::Read {
            path: path.clone(),
            source,
        })?;
        for r in parse(&path, &body)? {
            if let Some((prev, line)) = seen.get(&r.id) {
                return Err(Error::Malformed(format!(
                    "{}:{}: {} already defined at {}:{line}",
                    r.path.display(),
                    r.line,
                    r.id,
                    prev.display()
                )));
            }
            seen.insert(r.id.clone(), (r.path.clone(), r.line));
            all.push(r);
        }
    }

    Ok(all)
}

/// The paths directly in `dir` whose names end in `.md`, sorted.
fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let list = |source| Error::List {
        dir: dir.to_path_buf(),
        source,
    };
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).map_err(list)? {
        let path = entry.map_err(list)?.path();
        if path.extension().is_some_and(|e| e == "md") {
            paths.push(path);
        }
    }
    paths.sort();

    Ok(paths)
}

/// Reads the requirements from one document, in its order. Each is a
/// block, a heading that is a requirement id, which must carry fields
/// of [`FIELDS`], each once, and a text, or it is reported with its
/// line. A requirement table, one whose header leads with "ID" and
/// names a "Requirement" or an "Assumption" column, is refused with its
/// header's line: a requirement is written as a block. Every other
/// table (constraints, open questions, glossaries) is passed by.
///
/// # Errors
///
/// Fails on a requirement table, or on the first malformed block.
pub fn parse(path: &Path, body: &str) -> Result<Vec<Requirement>, Error> {
    if let Some(table) = tables(body)
        .into_iter()
        .find(|t| is_requirement_table(&t.header))
    {
        return Err(Error::Malformed(format!(
            "{}:{}: a requirement table: write each requirement as a block under a heading that is its id",
            path.display(),
            table.line
        )));
    }

    blocks(body)
        .iter()
        .map(|block| requirement_of_block(path, block))
        .collect()
}

/// Whether `header` is a requirement table's: it leads with "ID" and
/// names a "Requirement" or an "Assumption" column.
fn is_requirement_table(header: &[String]) -> bool {
    header.first().is_some_and(|h| h == "ID")
        && header
            .iter()
            .any(|h| h == "Requirement" || h == "Assumption")
}

/// The labels a block's fields may carry beside its id and its text:
/// what the requirement tables held as columns.
pub const FIELDS: [&str; 7] = [
    "Pri",
    "Ver",
    "Traces",
    "Category",
    "Evidence so far",
    "Verified in",
    "Affects",
];

/// Reads one block.
fn requirement_of_block(path: &Path, block: &Block) -> Result<Requirement, Error> {
    let malformed = |what: String| {
        Error::Malformed(format!(
            "{}:{}: {}: {what}",
            path.display(),
            block.line,
            block.id
        ))
    };
    if block.fields.is_empty() {
        return Err(malformed("no fields".to_owned()));
    }
    let mut fields: BTreeMap<&str, &str> = BTreeMap::new();
    for (label, value) in &block.fields {
        if !FIELDS.contains(&label.as_str()) {
            return Err(malformed(format!("unknown field {label:?}")));
        }
        if fields.insert(label, value).is_some() {
            return Err(malformed(format!("field {label:?} twice")));
        }
    }
    if block.text.is_empty() {
        return Err(malformed("no text".to_owned()));
    }
    let field = |name: &str| {
        fields
            .get(name)
            .map(|v| (*v).to_owned())
            .unwrap_or_default()
    };

    Ok(Requirement {
        id: block.id.clone(),
        priority: field("Pri"),
        verify: field("Ver"),
        traces: parse_traces(&field("Traces")).map_err(malformed)?,
        text: block.text.clone(),
        path: path.to_path_buf(),
        line: block.line,
    })
}

/// Reads a Traces field: "—" or empty for none, else a comma-separated
/// list of invariants.
fn parse_traces(cell: &str) -> Result<Vec<String>, String> {
    if cell.is_empty() || cell == "—" {
        return Ok(Vec::new());
    }

    split_list(cell, is_invariant, "trace")
}

/// Reads a comma-separated cell whose every entry, trimmed, is
/// `valid`; the first entry that is not is reported as a malformed
/// `kind`.
fn split_list(cell: &str, valid: impl Fn(&str) -> bool, kind: &str) -> Result<Vec<String>, String> {
    cell.split(',')
        .map(|part| {
            let entry = part.trim();
            if valid(entry) {
                Ok(entry.to_owned())
            } else {
                Err(format!("malformed {kind} {entry:?}"))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
