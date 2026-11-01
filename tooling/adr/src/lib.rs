//! Reads the architecture decision records under `docs/adr` (ENG-26):
//! one file per decision, its identity and status in front matter, its
//! reasoning in sections. A decision about dependencies also carries a
//! Module table, which ENG-18 checks against the direct dependencies.
//! It is repository tooling and never links into a shipped executable.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The decision is put forward, not yet in force.
pub const PROPOSED: &str = "proposed";
/// The decision is in force.
pub const ACCEPTED: &str = "accepted";
/// Another decision replaced this one.
pub const SUPERSEDED: &str = "superseded";

/// One decision record.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Adr {
    pub path: PathBuf,
    pub id: String,
    pub title: String,
    pub status: String,
    pub summary: String,
    pub superseded_by: String,
    /// Each level-2 heading, mapped to the trimmed text under it.
    pub sections: BTreeMap<String, String>,
    /// The decision's Module table, empty when it names no dependency.
    pub modules: Vec<Module>,
}

/// One row of an ADR's Module table: a dependency by the name its
/// manifest gives it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub purpose: String,
    pub license: String,
    pub maintenance: String,
}

/// What reading the records can fail with.
#[derive(Debug)]
pub enum Error {
    /// The directory could not be listed.
    List { dir: PathBuf, source: io::Error },
    /// A record could not be read.
    Read { path: PathBuf, source: io::Error },
    /// A record's front matter is missing or not the flat shape this
    /// reader takes.
    FrontMatter { path: PathBuf, msg: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::List { dir, source } => write!(f, "adr: list {}: {source}", dir.display()),
            Self::Read { path, source } => write!(f, "adr: read {}: {source}", path.display()),
            Self::FrontMatter { path, msg } => write!(f, "adr: {}: {msg}", path.display()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::List { source, .. } | Self::Read { source, .. } => Some(source),
            Self::FrontMatter { .. } => None,
        }
    }
}

/// Reads every record directly in `dir`, in file-name order.
///
/// # Errors
///
/// Fails when `dir` or a record cannot be read or parsed.
pub fn load(dir: &Path) -> Result<Vec<Adr>, Error> {
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

    paths
        .into_iter()
        .map(|path| {
            let body = fs::read_to_string(&path).map_err(|source| Error::Read {
                path: path.clone(),
                source,
            })?;
            parse(&path, &body)
        })
        .collect()
}

/// Reads one record from `body`. CRLF line endings read as LF, so a
/// Windows checkout parses the same as any other.
///
/// # Errors
///
/// Fails when the front matter is missing or malformed.
pub fn parse(path: &Path, body: &str) -> Result<Adr, Error> {
    let body = body.replace("\r\n", "\n");
    let (mut fm, rest) = front_matter(&body).map_err(|msg| Error::FrontMatter {
        path: path.to_path_buf(),
        msg,
    })?;
    let sections = sections(rest);
    let modules = sections
        .get("Decision")
        .map(|d| modules(d))
        .unwrap_or_default();
    let mut take = |key: &str| fm.remove(key).unwrap_or_default();

    Ok(Adr {
        path: path.to_path_buf(),
        id: take("id"),
        title: take("title"),
        status: take("status"),
        summary: take("summary"),
        superseded_by: take("superseded-by"),
        sections,
        modules,
    })
}

/// Splits `body` into its front matter, read as flat scalar keys, and
/// the rest. A folded or literal block scalar (`>-`, `|`) joins its
/// indented lines with spaces. Anything else, such as a list or a
/// nested map, is refused rather than misread. Any Markdown file with
/// such front matter reads through it, an agent definition among them.
///
/// # Errors
///
/// Fails when `body` has no front matter, or a line of it is not a flat
/// scalar key.
pub fn front_matter(body: &str) -> Result<(BTreeMap<String, String>, &str), String> {
    let no_front_matter = || "no front matter".to_owned();
    let (head, rest) = body.split_once("\n---\n").ok_or_else(no_front_matter)?;
    let head = head.strip_prefix("---\n").ok_or_else(no_front_matter)?;

    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let mut block: Option<String> = None;
    for (n, line) in (2..).zip(head.lines()) {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with([' ', '\t']) {
            let key = block
                .as_ref()
                .ok_or_else(|| format!("line {n}: unsupported front matter {line:?}"))?;
            let joined = out.entry(key.clone()).or_default();
            *joined = format!("{joined} {}", line.trim()).trim().to_owned();
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .filter(|(key, _)| !key.is_empty())
            .ok_or_else(|| format!("line {n}: malformed front matter {line:?}"))?;
        let value = value.trim();
        block = [">", ">-", "|", "|-"]
            .contains(&value)
            .then(|| key.to_owned());
        out.insert(
            key.to_owned(),
            if block.is_some() {
                String::new()
            } else {
                unquote(value).to_owned()
            },
        );
    }

    Ok((out, rest))
}

/// Strips one pair of matching single or double quotes.
fn unquote(s: &str) -> &str {
    ['"', '\'']
        .iter()
        .find_map(|q| s.strip_prefix(*q).and_then(|inner| inner.strip_suffix(*q)))
        .unwrap_or(s)
}

/// Maps each level-2 heading in `body` to the trimmed text under it,
/// up to the next level-2 heading. A `## ` line inside a fenced code
/// block is text, not a heading.
fn sections(body: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    let mut fence = None;
    for line in body.lines() {
        fence = srs::fence_after(fence, line);
        if let Some(heading) = line.strip_prefix("## ").filter(|_| fence.is_none()) {
            if let Some((name, text)) = current.take() {
                out.insert(name, text.join("\n").trim().to_owned());
            }
            current = Some((heading.trim().to_owned(), Vec::new()));
            continue;
        }
        if let Some((_, text)) = current.as_mut() {
            text.push(line);
        }
    }
    if let Some((name, text)) = current {
        out.insert(name, text.join("\n").trim().to_owned());
    }

    out
}

/// Reads `body`'s Module table: the table whose header leads with
/// "Module", each name with its backticks trimmed.
fn modules(body: &str) -> Vec<Module> {
    let mut out = Vec::new();
    for table in srs::tables(body)
        .into_iter()
        .filter(|t| t.header[0] == "Module")
    {
        let col = |name: &str| table.header.iter().position(|h| h == name);
        let (purpose, license, maintenance) = (col("Purpose"), col("License"), col("Maintenance"));
        for row in &table.rows {
            let cell = |i: Option<usize>| {
                i.and_then(|i| row.cells.get(i))
                    .cloned()
                    .unwrap_or_default()
            };
            out.push(Module {
                name: row.cells[0].trim_matches('`').to_owned(),
                purpose: cell(purpose),
                license: cell(license),
                maintenance: cell(maintenance),
            });
        }
    }

    out
}

#[cfg(test)]
mod tests;
