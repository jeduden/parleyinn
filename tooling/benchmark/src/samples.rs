//! The file the samples travel in: plain text, one sample per line, a
//! benchmark's name and the sample in nanoseconds, separated by a
//! space. A line starting with `#` is a comment and a blank line is
//! skipped, so the output of several `benchmark run`s concatenates
//! into one file, which is how the CI job interleaves its rounds.
//!
//! ```text
//! # benchmark nanoseconds
//! core_start_up 812345
//! core_start_up 798001
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Each benchmark's samples, in nanoseconds, by benchmark name.
pub type Samples = BTreeMap<String, Vec<u64>>;

/// Each stratum's samples, by stratum name. A stratum is one runner's
/// worth of samples: the CI job pools its shards' files, writing a
/// [`STRATUM`] line before each, so the gate ranks a sample only against
/// samples taken on the same runner. A file with no such line is one
/// stratum, named by the empty string.
pub type Strata = BTreeMap<String, Samples>;

/// The comment line `run` writes first.
pub const HEADER: &str = "# benchmark nanoseconds";

/// The comment line that starts a stratum's samples, followed by its
/// name.
pub const STRATUM: &str = "# stratum ";

/// Whether `name` can name a benchmark: lowercase ASCII letters,
/// digits and underscores, so it never holds the separator.
#[must_use]
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// Reads a samples file as one pool, whatever its strata.
///
/// # Errors
///
/// As [`parse_strata`].
pub fn parse(text: &str) -> Result<Samples, String> {
    parse_strata(text).map(|strata| pooled(&strata))
}

/// Reads a samples file, keeping each [`STRATUM`]'s samples apart.
///
/// # Errors
///
/// Fails on a line that is not a valid name, one space and a whole
/// number of nanoseconds, or a stratum line with no name, naming the
/// line.
pub fn parse_strata(text: &str) -> Result<Strata, String> {
    let mut strata = Strata::new();
    let mut current = String::new();
    for (i, line) in text.lines().enumerate() {
        if let Some(name) = line.strip_prefix(STRATUM) {
            if name.trim().is_empty() {
                return Err(format!("line {}: a stratum line names its stratum", i + 1));
            }
            current = name.trim().to_owned();
            continue;
        }
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, nanos) = sample(line).ok_or_else(|| {
            format!(
                "line {}: {line:?} is not a benchmark name and a sample in nanoseconds",
                i + 1
            )
        })?;
        strata
            .entry(current.clone())
            .or_default()
            .entry(name.to_owned())
            .or_default()
            .push(nanos);
    }

    Ok(strata)
}

/// Every stratum's samples together, per benchmark, stratum by stratum
/// in name order.
#[must_use]
pub fn pooled(strata: &Strata) -> Samples {
    let mut all = Samples::new();
    for samples in strata.values() {
        for (name, taken) in samples {
            all.entry(name.clone()).or_default().extend(taken);
        }
    }

    all
}

/// One line's benchmark name and sample.
fn sample(line: &str) -> Option<(&str, u64)> {
    let (name, nanos) = line.split_once(' ')?;
    let nanos = nanos.parse().ok()?;

    valid_name(name).then_some((name, nanos))
}

/// The lines for `nanos`, each one sample of benchmark `name`.
#[must_use]
pub fn render(name: &str, nanos: &[u64]) -> String {
    nanos.iter().fold(String::new(), |mut out, n| {
        let _ = writeln!(out, "{name} {n}");
        out
    })
}

#[cfg(test)]
mod tests;
