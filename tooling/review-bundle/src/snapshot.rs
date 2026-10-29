//! One tree's records, as the analysis reads them: the requirement
//! rows, the scenarios, CODEOWNERS, the invariants and the decision
//! records. Paths are relative to the tree's root, so the two trees'
//! records compare and findings name the file a reviewer opens.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::analysis::INVARIANTS;

/// One tree's records.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub requirements: Vec<srs::Requirement>,
    pub scenarios: Vec<scenario::Scenario>,
    pub codeowners: String,
    pub invariants: String,
    /// Each decision record's text, keyed by its path.
    pub decisions: BTreeMap<String, String>,
}

/// What reading a tree can fail with.
#[derive(Debug)]
pub enum Error {
    /// The requirement tables do not read.
    Srs(srs::Error),
    /// The scenarios do not read.
    Scenarios(scenario::Error),
    /// A file or directory could not be read.
    Read { path: PathBuf, source: io::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Srs(e) => write!(f, "review-bundle: {e}"),
            Self::Scenarios(e) => write!(f, "review-bundle: {e}"),
            Self::Read { path, source } => {
                write!(f, "review-bundle: read {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Srs(e) => Some(e),
            Self::Scenarios(e) => Some(e),
            Self::Read { source, .. } => Some(source),
        }
    }
}

impl Snapshot {
    /// Reads the tree at `root`. A tree without CODEOWNERS or decision
    /// records reads them as empty.
    ///
    /// # Errors
    ///
    /// Fails when the SRS, the scenarios or the invariants do not read.
    pub fn read(root: &Path) -> Result<Self, Error> {
        let mut requirements = srs::load(&root.join("docs/srs")).map_err(Error::Srs)?;
        for r in &mut requirements {
            r.path = relative(root, &r.path);
        }
        let mut scenarios =
            scenario::scenarios(&root.join("features")).map_err(Error::Scenarios)?;
        for s in &mut scenarios {
            s.path = relative(root, &s.path);
        }

        Ok(Self {
            requirements,
            scenarios,
            codeowners: optional(&root.join(".github/CODEOWNERS"))?,
            invariants: required(&root.join(INVARIANTS))?,
            decisions: decisions(root)?,
        })
    }

    /// `self` as the tree reads with the file at `rel`, relative to its
    /// root, holding `text`, or gone when `text` is none: the records
    /// that file holds are read again from `text` alone and every other
    /// record is kept, so a changed tree is read without being written.
    /// A file that holds no record leaves the snapshot as it is.
    ///
    /// # Errors
    ///
    /// Fails when `text` does not read as the records its path holds.
    pub fn with_file(&self, rel: &str, text: Option<&str>) -> Result<Self, Error> {
        let mut out = self.clone();
        let path = Path::new(rel);
        let under = |dir: &str, ext: &str| {
            path.parent() == Some(Path::new(dir)) && path.extension().is_some_and(|e| e == ext)
        };
        if rel == INVARIANTS {
            out.invariants = text.unwrap_or_default().to_owned();
        }
        if under("docs/srs", "md") {
            out.requirements.retain(|r| r.path != path);
            if let Some(text) = text {
                out.requirements
                    .extend(srs::parse(path, text).map_err(Error::Srs)?);
            }
            out.requirements
                .sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
        } else if under("features", "feature") {
            out.scenarios.retain(|s| s.path != path);
            if let Some(text) = text {
                out.scenarios
                    .extend(scenario::scenarios_in(path, text).map_err(Error::Scenarios)?);
            }
            out.scenarios
                .sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
        } else if rel == ".github/CODEOWNERS" {
            out.codeowners = text.unwrap_or_default().to_owned();
        } else if under("docs/adr", "md") {
            match text {
                Some(text) => out.decisions.insert(rel.to_owned(), text.to_owned()),
                None => out.decisions.remove(rel),
            };
        }

        Ok(out)
    }

    /// Every requirement and scenario id the tree defines.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        let reqs = self.requirements.iter().map(|r| r.id.as_str());
        reqs.chain(self.scenarios.iter().map(|s| s.id.as_str()))
    }
}

/// `path` relative to `root`, or `path` itself when it lies elsewhere.
fn relative(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

/// The text of `path`.
fn required(path: &Path) -> Result<String, Error> {
    fs::read_to_string(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })
}

/// The text of `path`, empty when there is no such file.
fn optional(path: &Path) -> Result<String, Error> {
    match fs::read_to_string(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        read => read.map_err(|source| Error::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Every Markdown file directly under `docs/adr`, keyed by its path
/// relative to `root`.
fn decisions(root: &Path) -> Result<BTreeMap<String, String>, Error> {
    let dir = root.join("docs/adr");
    let list = |source: io::Error| Error::Read {
        path: dir.clone(),
        source,
    };
    let entries = match fs::read_dir(&dir) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        listed => listed.map_err(list)?,
    };
    let paths = entries
        .map(|e| e.map(|e| e.path()).map_err(list))
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = BTreeMap::new();
    for path in paths
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
    {
        let key = relative(root, &path).display().to_string();
        out.insert(key, required(&path)?);
    }

    Ok(out)
}

#[cfg(test)]
mod tests;
