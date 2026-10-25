//! A benchmark, and any step that holds the core to a time budget,
//! runs a release build (NFR-01, NFR-02, ENG-15). A development build
//! is an order of magnitude slower and flakes on a loaded runner, and
//! the seams it carries are not the build users run. Two scans hold
//! the repository to it:
//!
//! - a workflow or script line that builds, runs or tests the
//!   `benchmark` package names `--release`, and a line that runs the
//!   benchmark never points it at a `debug` directory;
//! - a Rust function in an integration test or bench that reads a clock
//!   (`Instant::now`, `.elapsed()`) starts the core only through
//!   `release_core_executable` or `run_release_core`.

use std::fs;
use std::path::Path;

use crate::Checkout;

/// Workflow and script directories the first scan reads.
const BUILD_DIRS: [&str; 2] = [".github/workflows", "scripts"];

/// The directories whose crates' integration tests and benches the
/// second scan reads.
const CRATE_DIRS: [&str; 2] = ["crates", "tooling"];

/// What starts the development build of the core in a step.
const DEBUG_CORE: [&str; 6] = [
    "core_executable(",
    "run_core(",
    "run_core_with(",
    "run_core_killed(",
    "run_core_at_terminal(",
    "CARGO_BIN_EXE_parinn",
];

/// What starts the release build of the core in a step.
const RELEASE_CORE: [&str; 2] = ["release_core_executable(", "run_release_core("];

/// What reads a clock to time a step.
const CLOCK: [&str; 2] = ["Instant::now(", ".elapsed("];

/// The cargo verbs that build or start what they name.
const CARGO_VERBS: [&str; 5] = ["build", "run", "test", "bench", "check"];

/// How a command line names the benchmark package.
const SUITE_FLAGS: [&str; 2] = ["-p benchmark", "--package benchmark"];

/// The problems in one workflow or script `text`, named by `file`.
#[must_use]
pub fn debug_benchmark_lines(file: &str, text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let code = line.trim_start();
        if code.starts_with('#') {
            continue;
        }
        let at = format!("{file}:{}", n + 1);
        if builds_suite(code) && !code.contains("--release") {
            found.push(format!(
                "{at} builds the benchmark suite without --release: a benchmark runs a release build"
            ));
        }
        if code.contains("benchmark") && code.contains("debug") && names_debug_dir(code) {
            found.push(format!(
                "{at} points the benchmark at a debug build: a benchmark runs a release build"
            ));
        }
    }

    found
}

/// Whether `line` has cargo build, run, test, bench or check the
/// benchmark package.
fn builds_suite(line: &str) -> bool {
    SUITE_FLAGS.iter().any(|f| line.contains(f))
        && CARGO_VERBS
            .iter()
            .any(|v| line.contains(&format!("cargo {v} ")))
}

/// Whether `line` names a `debug` directory of a target tree.
fn names_debug_dir(line: &str) -> bool {
    line.contains("/debug/") || line.contains("target/debug")
}

/// The problems in one Rust `source`, named by `file`.
#[must_use]
pub fn debug_core_in_timed_fns(file: &str, source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let starts: Vec<usize> = (0..lines.len())
        .filter(|&i| fn_name(lines[i]).is_some())
        .collect();
    starts
        .iter()
        .enumerate()
        .filter_map(|(k, &first)| {
            let end = starts.get(k + 1).copied().unwrap_or(lines.len());
            let body = RELEASE_CORE
                .iter()
                .fold(lines[first..end].join("\n"), |b, call| b.replace(call, ""));
            let called = body_debug_call(&body)?;
            let name = fn_name(lines[first])?;
            Some(format!(
                "{file}:{} fn {name} reads a clock and starts the development core with {called}: a step that times the core runs the release build",
                first + 1
            ))
        })
        .collect()
}

/// The development-core call a timed `body` makes, if it reads a clock
/// and makes one; a release call is already left out of `body`.
fn body_debug_call(body: &str) -> Option<&'static str> {
    if !CLOCK.iter().any(|c| body.contains(c)) {
        return None;
    }

    DEBUG_CORE
        .iter()
        .copied()
        .filter_map(|call| body.find(call).map(|at| (at, call)))
        .min_by_key(|(at, _)| *at)
        .map(|(_, call)| call)
}

/// The name of the function `line` declares, if it declares one.
fn fn_name(line: &str) -> Option<&str> {
    let rest = line.trim_start();
    let rest = rest.strip_prefix("pub(crate) ").unwrap_or(rest);
    let rest = rest.strip_prefix("pub ").unwrap_or(rest);
    let rest = rest.strip_prefix("fn ")?;

    rest.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .next()
}

/// Every problem in the checkout's workflows, scripts, integration
/// tests and benches.
#[must_use]
pub fn problems(checkout: &Checkout) -> Vec<String> {
    let mut found = Vec::new();
    for dir in BUILD_DIRS {
        for (name, text) in files_under(&checkout.root, dir, |_| true) {
            found.extend(debug_benchmark_lines(&name, &text));
        }
    }
    for top in CRATE_DIRS {
        for member in subdirs(&checkout.root.join(top)) {
            for part in ["tests", "benches"] {
                let dir = format!("{top}/{member}/{part}");
                for (name, text) in files_under(&checkout.root, &dir, |p| p.ends_with(".rs")) {
                    found.extend(debug_core_in_timed_fns(&name, &text));
                }
            }
        }
    }

    found
}

/// The names of the directories in `dir`, sorted.
fn subdirs(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();

    names
}

/// Each file under `root/dir`, recursively, whose path `keep` accepts:
/// its path from `root` and its text. A file that is not UTF-8 text is
/// left out.
fn files_under(root: &Path, dir: &str, keep: fn(&str) -> bool) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_owned()];
    while let Some(next) = pending.pop() {
        let mut entries: Vec<_> = fs::read_dir(root.join(&next))
            .into_iter()
            .flatten()
            .flatten()
            .collect();
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            let path = format!("{next}/{}", entry.file_name().to_string_lossy());
            if entry.path().is_dir() {
                pending.push(path);
            } else if keep(&path)
                && let Ok(text) = fs::read_to_string(entry.path())
            {
                files.push((path, text));
            }
        }
    }

    files
}

#[cfg(test)]
mod tests;
