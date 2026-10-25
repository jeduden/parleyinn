//! The isolation guard (ENG-14). Test code resolves the user's `HOME`, a
//! ParleyInn home and Claude Code's configuration through [`resolve`] and
//! nowhere else. Anything that does not lie under the system's temporary
//! directory is real state, and resolving it aborts the process before
//! a path to it exists.

use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::process;

/// Something a test must never reach for real.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// The user's `HOME` directory.
    UserHome,
    /// A ParleyInn home: `PARINN_HOME`, else `.parinn` in the user's `HOME`.
    ParleyInnHome,
    /// Claude Code's configuration directory: `CLAUDE_CONFIG_DIR`, else
    /// `.claude` in the user's `HOME`.
    ClaudeConfig,
    /// Claude Code's global state file: `.claude.json` in the
    /// configuration directory, else in the user's `HOME`.
    ClaudeJson,
}

/// Why the guard refuses to resolve a place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// No `HOME` to resolve the place from.
    Unset,
    /// The place is not under the temporary directory `temp`.
    Outside {
        place: Place,
        path: PathBuf,
        temp: PathBuf,
    },
}

/// The value of an environment variable, as a process reads it.
pub type Var<'a> = &'a dyn Fn(&str) -> Option<OsString>;

impl Place {
    /// Every place the guard keeps tests out of.
    pub const ALL: [Self; 4] = [
        Self::UserHome,
        Self::ParleyInnHome,
        Self::ClaudeConfig,
        Self::ClaudeJson,
    ];

    /// The name a probe takes this place by.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::UserHome => "user-home",
            Self::ParleyInnHome => "parinn-home",
            Self::ClaudeConfig => "claude-config",
            Self::ClaudeJson => "claude-json",
        }
    }

    /// The place a probe names, if any.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }

    /// What the place is, as a refusal says it.
    fn describe(self) -> &'static str {
        match self {
            Self::UserHome => "the user's `HOME` directory",
            Self::ParleyInnHome => "the ParleyInn home",
            Self::ClaudeConfig => "Claude Code's configuration directory",
            Self::ClaudeJson => "Claude Code's state file",
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unset => write!(f, "refusing to resolve a path: HOME is not set (ENG-14)"),
            Self::Outside { place, path, temp } => write!(
                f,
                "refusing to resolve {} at {}: it is not under the temporary directory {}; \
                 point HOME and PARINN_HOME at a testkit::TempDir first (ENG-14)",
                place.describe(),
                path.display(),
                temp.display()
            ),
        }
    }
}

/// Where `place` lies according to `var`, real or not.
fn locate(place: Place, var: Var<'_>) -> Result<PathBuf, Refusal> {
    let home = || var("HOME").map(PathBuf::from).ok_or(Refusal::Unset);
    let config = || var("CLAUDE_CONFIG_DIR").map(PathBuf::from);
    match place {
        Place::UserHome => home(),
        Place::ParleyInnHome => match var("PARINN_HOME") {
            Some(parinn) => Ok(PathBuf::from(parinn)),
            None => home().map(|h| h.join(".parinn")),
        },
        Place::ClaudeConfig => match config() {
            Some(dir) => Ok(dir),
            None => home().map(|h| h.join(".claude")),
        },
        Place::ClaudeJson => match config() {
            Some(dir) => Ok(dir.join(".claude.json")),
            None => home().map(|h| h.join(".claude.json")),
        },
    }
}

/// `path` with every symlink on its way resolved, as far as it exists:
/// the rest follows as written.
fn resolved(path: &Path) -> PathBuf {
    let mut rest = Vec::new();
    let mut base = path;
    loop {
        if let Ok(real) = fs::canonicalize(base) {
            return rest.iter().rev().fold(real, |p, name| p.join(name));
        }
        match (base.parent(), base.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name);
                base = parent;
            }
            _ => return path.to_path_buf(),
        }
    }
}

/// Whether `path` lies strictly under `temp`, by where it really leads:
/// absolute, with no `..`, and no symlink out.
fn under(path: &Path, temp: &Path) -> bool {
    let climbs = path.components().any(|c| c == Component::ParentDir);
    if !path.is_absolute() || climbs {
        return false;
    }
    let (path, temp) = (resolved(path), resolved(temp));

    path != temp && path.starts_with(temp)
}

/// Resolves `place` from `var` and requires it to lie under `temp`.
///
/// # Errors
///
/// Refuses a place that is real state, or has no `HOME` to resolve from.
pub fn check(place: Place, var: Var<'_>, temp: &Path) -> Result<PathBuf, Refusal> {
    let path = locate(place, var)?;
    if under(&path, temp) {
        Ok(path)
    } else {
        Err(Refusal::Outside {
            place,
            path,
            temp: temp.to_path_buf(),
        })
    }
}

/// [`check`], and on a refusal writes it to `report` and calls `abort`.
pub fn check_or_abort(
    place: Place,
    var: Var<'_>,
    temp: &Path,
    report: &mut dyn Write,
    abort: fn() -> !,
) -> PathBuf {
    match check(place, var, temp) {
        Ok(path) => path,
        Err(refusal) => {
            // The process is about to abort; a report that cannot be
            // written changes nothing about that.
            let _ = writeln!(report, "{refusal}");
            abort()
        }
    }
}

/// Resolves `place` for this process, aborting it when the place is
/// real state.
#[must_use]
pub fn resolve(place: Place) -> PathBuf {
    check_or_abort(
        place,
        &|name| env::var_os(name),
        &env::temp_dir(),
        &mut io::stderr(),
        process::abort,
    )
}

#[cfg(test)]
mod tests;
