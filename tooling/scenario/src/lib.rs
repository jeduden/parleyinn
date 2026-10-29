//! Keeps the executable Gherkin scenarios under `features/` in
//! bijection with the requirement ids of the SRS: every requirement has
//! exactly one tagged scenario, every scenario names a real
//! requirement, and a scenario's priority and invariant tags agree with
//! its requirement's row. The crate's `bdd` test target runs the
//! scenarios through cucumber-rs. It is repository tooling and never
//! links into a shipped executable.

mod gate;
pub mod runner;

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use cucumber::gherkin;

pub use gate::check;

/// The tag, written without its `@` as gherkin stores it, that marks a
/// scenario declared but not yet written: the runner skips it rather
/// than counting a pass that proves nothing.
pub const PENDING: &str = "pending";

/// One scenario under `features/`, as both the gate and the runner see
/// it. A Scenario Outline is one scenario however many Examples rows it
/// has.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scenario {
    pub path: PathBuf,
    pub line: usize,
    pub name: String,
    pub id: String,
    pub priority: String,
    /// The invariant tags, sorted and each once.
    pub invariants: Vec<String>,
    pub pending: bool,
    /// The step texts in order, keywords left out; an outline's keep
    /// their `<placeholders>`.
    pub steps: Vec<String>,
}

/// What reading the scenarios can fail with.
#[derive(Debug)]
pub enum Error {
    /// The features directory could not be walked.
    Walk { path: PathBuf, source: io::Error },
    /// A feature file could not be read or parsed.
    Parse(Box<gherkin::ParseFileError>),
    /// A scenario's tags do not name exactly one requirement and at
    /// most one priority.
    Tags(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Walk { path, source } => {
                write!(f, "scenario: read features: {}: {source}", path.display())
            }
            Self::Parse(err) => write!(f, "scenario: read features: {err}"),
            Self::Tags(msg) => write!(f, "scenario: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Walk { source, .. } => Some(source),
            Self::Parse(err) => Some(err),
            Self::Tags(_) => None,
        }
    }
}

/// Every `.feature` file under `dir`, recursively, sorted by path: the
/// files cucumber-rs's runner walks.
///
/// # Errors
///
/// Fails when a directory cannot be read.
pub fn feature_files(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let walk = |source| Error::Walk {
        path: dir.to_path_buf(),
        source,
    };
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).map_err(walk)? {
        let path = entry.map_err(walk)?.path();
        if path.is_dir() {
            out.extend(feature_files(&path)?);
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("feature"))
        {
            out.push(path);
        }
    }
    out.sort();

    Ok(out)
}

/// Lists every scenario under `dir`, in the order the runner walks
/// them and read with the parser it uses, so the gate and the runner
/// never disagree about which scenarios exist. A scenario must carry
/// exactly one requirement tag and at most one priority tag; anything
/// else is reported with its place.
///
/// # Errors
///
/// Fails when a feature cannot be read or parsed, or a scenario's tags
/// are malformed.
pub fn scenarios(dir: &Path) -> Result<Vec<Scenario>, Error> {
    let mut out = Vec::new();
    for path in feature_files(dir)? {
        let feature = gherkin::Feature::parse_path(&path, gherkin::GherkinEnv::default())
            .map_err(|e| Error::Parse(Box::new(e)))?;
        out.extend(feature_scenarios(&path, &feature)?);
    }

    Ok(out)
}

/// The scenarios of one feature whose text is `text`, read as
/// [`scenarios`] reads the file at `path`, so a changed feature is read
/// without writing a tree.
///
/// # Errors
///
/// Fails as [`scenarios`] does, naming `path`.
pub fn scenarios_in(path: &Path, text: &str) -> Result<Vec<Scenario>, Error> {
    let feature =
        gherkin::Feature::parse(text, gherkin::GherkinEnv::default()).map_err(|source| {
            Error::Parse(Box::new(gherkin::ParseFileError::Parsing {
                path: path.to_path_buf(),
                error: None,
                source,
            }))
        })?;

    feature_scenarios(path, &feature)
}

/// Every scenario of `feature`, read from `path`: the top-level ones,
/// then each rule's, with the feature's, the rule's and their own tags.
fn feature_scenarios(path: &Path, feature: &gherkin::Feature) -> Result<Vec<Scenario>, Error> {
    let none = Vec::new();
    let top = feature.scenarios.iter().map(|s| (&none, s));
    let ruled = feature
        .rules
        .iter()
        .flat_map(|r| r.scenarios.iter().map(move |s| (&r.tags, s)));
    let mut out = Vec::new();
    for (rule_tags, sc) in top.chain(ruled) {
        let tags = feature.tags.iter().chain(rule_tags).chain(&sc.tags);
        let tags = tags.chain(sc.examples.iter().flat_map(|e| &e.tags));
        out.push(scenario_of(path, sc, tags)?);
    }

    Ok(out)
}

/// Reads one scenario's tags: its requirement id, priority, invariants,
/// and whether it is pending.
fn scenario_of<'a>(
    path: &Path,
    sc: &gherkin::Scenario,
    tags: impl Iterator<Item = &'a String>,
) -> Result<Scenario, Error> {
    let mut out = Scenario {
        path: path.to_path_buf(),
        line: sc.position.line,
        name: sc.name.clone(),
        steps: sc.steps.iter().map(|s| s.value.clone()).collect(),
        ..Scenario::default()
    };
    let (mut ids, mut priorities) = (Vec::new(), Vec::new());
    for tag in tags {
        match tag.as_str() {
            PENDING => out.pending = true,
            t if srs::is_requirement_id(t) => ids.push(t.to_owned()),
            t if ["P0", "P1", "P2"].contains(&t) => priorities.push(t.to_owned()),
            t if srs::is_invariant(t) => out.invariants.push(t.to_owned()),
            _ => {}
        }
    }
    let at = format!("{}:{}: {:?}", path.display(), out.line, out.name);
    if ids.len() != 1 {
        return Err(Error::Tags(format!(
            "{at} carries {} requirement tags, want exactly one",
            ids.len()
        )));
    }
    if priorities.len() > 1 {
        return Err(Error::Tags(format!(
            "{at} carries {} priority tags, want at most one",
            priorities.len()
        )));
    }
    out.id = ids.remove(0);
    out.priority = priorities.pop().unwrap_or_default();
    out.invariants.sort();
    out.invariants.dedup();

    Ok(out)
}

#[cfg(test)]
mod tests;
