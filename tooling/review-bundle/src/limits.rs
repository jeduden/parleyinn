//! ENG-34: the findings a changed file's size and type make on their
//! own, with no agent. A source, test, feature or configuration file
//! past its kind's line limit is a nit, and so is a changed file no
//! review specialist reads, which reaches only a generalist: the review
//! is honest about what it did not read. Neither restricts what the
//! review reads; the generalists weigh both, and only the ENG-34 row
//! sets a limit.

use std::path::Path;

use crate::{Finding, NIT, dispatch};

/// The most lines a Rust file may hold, a module's `tests.rs` among
/// them.
pub const RUST_LINES: usize = 1000;
/// The most lines a feature file may hold.
pub const FEATURE_LINES: usize = 500;
/// The most lines a YAML file, a workflow among them, may hold.
pub const YAML_LINES: usize = 500;
/// The most lines a TOML file may hold.
pub const TOML_LINES: usize = 200;
/// The most lines a shell script may hold.
pub const SHELL_LINES: usize = 200;

/// Each kind of file with a limit: its name as ENG-34 states it, its
/// extensions and its limit. Markdown has none here, since `mdsmith
/// check .` holds it to its own caps, and JSON, mostly data, has none.
pub const LIMITS: [(&str, &[&str], usize); 5] = [
    ("Rust", &["rs"], RUST_LINES),
    ("Gherkin", &["feature"], FEATURE_LINES),
    ("YAML", &["yml", "yaml"], YAML_LINES),
    ("TOML", &["toml"], TOML_LINES),
    ("shell", &["sh"], SHELL_LINES),
];

/// The lock files, by name, which tools write.
const LOCK_FILES: [&str; 1] = ["Cargo.lock"];
/// The build output, which no change should hold.
const BUILD_OUTPUT: &str = "target/";
/// The coverage reports' extension.
const COVERAGE: &str = ".lcov";

/// The kind and limit of the file at `path`: none for a kind without a
/// limit, for data under [`dispatch::DATA_ROOTS`] and for a file
/// [`left_out`].
#[must_use]
pub fn limit_of(path: &str) -> Option<(&'static str, usize)> {
    if left_out(path) || dispatch::DATA_ROOTS.iter().any(|d| path.starts_with(d)) {
        return None;
    }
    let ext = Path::new(path).extension()?.to_str()?;
    LIMITS
        .iter()
        .find(|(_, exts, _)| exts.contains(&ext))
        .map(|(kind, _, limit)| (*kind, *limit))
}

/// Whether the repository leaves the file at `path` out of both checks
/// on purpose: a lock file, build output or a coverage report, which
/// tools write.
#[must_use]
pub fn left_out(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    LOCK_FILES.contains(&name) || path.starts_with(BUILD_OUTPUT) || path.ends_with(COVERAGE)
}

/// The nit for the file at `path` when its `lines` are past its kind's
/// limit.
#[must_use]
pub fn oversize(path: &str, lines: usize) -> Option<Finding> {
    let (kind, limit) = limit_of(path).filter(|(_, limit)| lines > *limit)?;
    Some(nit(
        path,
        format!(
            "{path} — if not addressed: a file past the limit exceeds what a reviewer \
             reads whole and what maintainers can hold in view ({lines} lines, past the \
             limit of {limit} for {kind}) — ENG-34"
        ),
    ))
}

/// The nit for the changed file at `path` when no specialist of
/// `scopes` reads it, so only a generalist does.
#[must_use]
pub fn unread(path: &str, scopes: &dispatch::Scopes) -> Option<Finding> {
    if left_out(path) || scopes.values().any(|files| files.contains(path)) {
        return None;
    }
    Some(nit(
        path,
        format!("{path} — if not addressed: {path} reached the merge unread — ENG-28"),
    ))
}

/// Every ENG-34 finding for the changed `paths`, which `scopes` hands
/// the specialists: each file past its limit, by the `lines` its text
/// holds in the pull request's tree, none for a file gone from it, and
/// each file no specialist reads.
#[must_use]
pub fn findings(
    paths: &[String],
    scopes: &dispatch::Scopes,
    lines: &dyn Fn(&str) -> Option<usize>,
) -> Vec<Finding> {
    paths
        .iter()
        .flat_map(|path| {
            let size = lines(path).and_then(|n| oversize(path, n));
            size.into_iter().chain(unread(path, scopes))
        })
        .collect()
}

/// A nit on the file at `path` as a whole.
fn nit(path: &str, body: String) -> Finding {
    Finding {
        path: path.to_owned(),
        line: 0,
        severity: NIT,
        body,
    }
}

#[cfg(test)]
mod tests;
