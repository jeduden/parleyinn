//! SEC-15: no telemetry, crash reporting to remote services or update
//! check ships. Three records hold it. No crate a lockfile resolves is
//! a telemetry, crash-reporting, update-check or network-client crate.
//! No product crate's shipped code names an `http://` or `https://`
//! address. The boundary register rejects the price-table fetch, update
//! checks and telemetry in a row of its own. The run-time half is the
//! confinement of every end-to-end test (ENG-12): a connection any of
//! them opens fails it.

use std::fs;
use std::path::Path;

use crate::Checkout;
use crate::boundary::Row;

/// The crates, by name, that report, check for an update or reach a
/// remote host. A crate whose name is one of these, or starts with one
/// followed by `-` or `_`, is refused.
const DENIED: [&str; 24] = [
    "sentry",
    "opentelemetry",
    "tracing-opentelemetry",
    "posthog",
    "mixpanel",
    "amplitude",
    "segment",
    "bugsnag",
    "rollbar",
    "datadog",
    "newrelic",
    "honeycomb",
    "minidumper",
    "crash-handler",
    "breakpad",
    "self_update",
    "self-update",
    "update-informer",
    "updater",
    "reqwest",
    "hyper",
    "ureq",
    "isahc",
    "curl",
];

/// The crate names `Cargo.lock` resolves.
#[must_use]
pub fn locked(lock: &str) -> Vec<String> {
    lock.lines()
        .filter_map(|l| l.strip_prefix("name = \""))
        .filter_map(|l| l.strip_suffix('"'))
        .map(str::to_owned)
        .collect()
}

/// Whether `name` is a denied crate.
fn denied(name: &str) -> bool {
    DENIED.iter().any(|d| {
        name == *d
            || name
                .strip_prefix(d)
                .is_some_and(|rest| rest.starts_with(['-', '_']))
    })
}

/// The lockfiles the repository commits: the workspace's and the fuzz
/// workspace's, which links the product crates too.
pub const LOCKFILES: [&str; 2] = ["Cargo.lock", "fuzz/Cargo.lock"];

/// Each crate the lockfile `file`, whose text is `lock`, resolves that
/// is a telemetry, crash-reporting, update-check or network-client
/// crate.
#[must_use]
pub fn denied_crates(file: &str, lock: &str) -> Vec<String> {
    locked(lock)
        .into_iter()
        .filter(|n| denied(n))
        .map(|n| format!("{file} resolves {n}, a crate that reports, checks for updates or reaches a remote host"))
        .collect()
}

/// Each line of `body` that names a web address, as `file:line`.
#[must_use]
pub fn addresses(file: &str, body: &str) -> Vec<String> {
    body.lines()
        .enumerate()
        .filter(|(_, l)| l.contains("http://") || l.contains("https://"))
        .map(|(i, l)| format!("{file}:{}: names a web address: {}", i + 1, l.trim()))
        .collect()
}

/// The `.rs` files under `dir`, tests left out, sorted.
fn shipped(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let list = |e: std::io::Error| format!("list {}: {e}", dir.display());
    let mut paths = fs::read_dir(dir)
        .map_err(list)?
        .map(|e| e.map(|e| e.path()).map_err(list))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        if name.as_deref() == Some("tests") {
            continue;
        }
        if path.is_dir() {
            shipped(&path, out)?;
        } else if path.extension().is_some_and(|x| x == "rs") && name.as_deref() != Some("tests.rs")
        {
            out.push(path);
        }
    }

    Ok(())
}

/// Each web address in the shipped code of the product crates.
#[must_use]
pub fn shipped_addresses(c: &Checkout) -> Vec<String> {
    let mut files = Vec::new();
    if let Err(e) = shipped(&c.root.join("crates"), &mut files) {
        return vec![e];
    }
    let mut problems = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(&c.root)
            .unwrap_or(&path)
            .display()
            .to_string();
        match fs::read_to_string(&path) {
            Ok(body) => problems.extend(addresses(&rel, &body)),
            Err(e) => problems.push(format!("read {rel}: {e}")),
        }
    }

    problems
}

/// Requires a register row that rejects the price-table fetch, update
/// checks and telemetry, citing SEC-15.
#[must_use]
pub fn register_rejects(rows: &[Row]) -> Vec<String> {
    let rejected = rows.iter().any(|r| {
        r.name.contains("update checks")
            && r.name.contains("telemetry")
            && r.component == "rejected"
            && r.requirements.contains("SEC-15")
    });
    if rejected {
        return Vec::new();
    }

    vec!["no rejected register row names update checks and telemetry under SEC-15".to_owned()]
}

/// Every problem of the three records, in the checkout (SEC-15).
#[must_use]
pub fn check(c: &Checkout) -> Vec<String> {
    let mut problems = Vec::new();
    for file in LOCKFILES {
        match c.read(file) {
            Ok(lock) => problems.extend(denied_crates(file, &lock)),
            Err(e) => problems.push(e),
        }
    }
    problems.extend(shipped_addresses(c));
    match crate::boundary::register(c) {
        Ok(rows) => problems.extend(register_rejects(&rows)),
        Err(e) => problems.push(e),
    }

    problems
}

#[cfg(test)]
mod tests;
