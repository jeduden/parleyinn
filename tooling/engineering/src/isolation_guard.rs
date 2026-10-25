//! ENG-14: a guard aborts any test that touches real state, and nothing
//! resolves the real `HOME`, a ParleyInn home or Claude Code's configuration
//! but the guard.
//!
//! The guard is `testkit::guard`: a test asks it for the user's `HOME`,
//! a ParleyInn home or Claude Code's configuration, and it aborts the
//! process when the answer is not under the temporary directory. This
//! module holds the two checks that make the guard binding. The probe
//! runs a test executable that resolves each place, once with a `HOME`
//! the guard must refuse and once with one it must allow. The scan reads
//! the sources for a read of real state that bypasses the guard, so the
//! guard covers every test and not only those that opt in.

use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::Checkout;

/// The variable that makes the scenario runner resolve a place and
/// print it instead of running scenarios; its value names the place.
pub const PROBE: &str = "PARINN_GUARD_PROBE";

/// How long a probe may run: it resolves one path and exits.
pub const PROBE_LIMIT: Duration = Duration::from_secs(60);

/// How often a running probe is polled.
const POLL: Duration = Duration::from_millis(5);

/// What the guard's refusal ends with, so a probe tells it from any
/// other failure.
const REFUSAL: &str = "(ENG-14)";

/// The sources that may read the user's `HOME` themselves: the guard,
/// the scenario runner's own decision to start isolated, and the core
/// executable's entry point, which finds the home the person runs it
/// in. No test reaches that read: a unit test hands `parinn::run` its
/// home, and every start of the executable goes through the testkit's
/// isolation.
pub const RESOLVERS: [&str; 3] = [
    "tooling/testkit/src/guard.rs",
    "tooling/scenario/tests/bdd/main.rs",
    "crates/parinn/src/main.rs",
];

/// The directories whose Rust sources the scan reads.
const SOURCES: [&str; 2] = ["crates", "tooling"];

/// What reads an environment variable, as it ends before the name.
const READS: [&str; 4] = ["var(", "var_os(", "env!(", "option_env!("];

/// The variables that lead to real state, quoted.
const NAMES: [&str; 4] = [
    "\"HOME\"",
    "\"PARINN_HOME\"",
    "\"CLAUDE_CONFIG_DIR\"",
    "\"USERPROFILE\"",
];

/// What a probe printed and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Runs `exe` as a probe for `place`: with nothing but `HOME` set to
/// `home`, the temporary directory set to `temp`, and [`PROBE`] naming
/// the place. A coverage run's profile file is kept, so the probe's
/// profile lands with the others and not where it runs.
///
/// # Errors
///
/// Fails when `exe` cannot start or outlives `limit`.
pub fn run_probe(
    exe: &Path,
    place: &str,
    home: &Path,
    temp: &Path,
    limit: Duration,
) -> Result<Probe, String> {
    let mut child = Command::new(exe)
        .env_clear()
        .env("HOME", home)
        .env("TMPDIR", temp)
        .env(PROBE, place)
        .envs(env::var_os("LLVM_PROFILE_FILE").map(|file| ("LLVM_PROFILE_FILE", file)))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("start probe {}: {e}", exe.display()))?;
    let status = wait_within(&mut child, limit)
        .map_err(|e| format!("probe {place} of {}: {e}", exe.display()))?;

    Ok(Probe {
        success: status.success(),
        stdout: child.stdout.take().map(drain).unwrap_or_default(),
        stderr: child.stderr.take().map(drain).unwrap_or_default(),
    })
}

/// Waits for `child` to end, and kills it when `limit` passes first.
fn wait_within(child: &mut Child, limit: Duration) -> Result<ExitStatus, String> {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait().map_err(|e| format!("wait: {e}"))? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            // A child that cannot be killed has already ended.
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("still running after {limit:?}"));
        }
        thread::sleep(POLL);
    }
}

/// Everything `pipe` holds. A pipe that fails mid-read leaves the text
/// so far, which is all a probe needs.
fn drain(mut pipe: impl Read) -> String {
    let mut text = String::new();
    let _ = pipe.read_to_string(&mut text);

    text
}

/// What is wrong with `probe`, run for `place` with a `HOME` the guard
/// must refuse: it must end without success, print no path, and say the
/// guard refused.
#[must_use]
pub fn not_refused(place: &str, probe: &Probe) -> Option<String> {
    let problem = if probe.success {
        "resolved real state instead of aborting"
    } else if !probe.stdout.trim().is_empty() {
        "printed a path before aborting"
    } else if !probe.stderr.contains(REFUSAL) {
        "ended without the guard's refusal"
    } else {
        return None;
    };

    Some(format!(
        "place {place}: {problem}: stdout {:?}, stderr {:?}",
        probe.stdout.trim(),
        probe.stderr.trim()
    ))
}

/// What is wrong with `probe`, run for `place` with an isolated `HOME`:
/// it must succeed and print a path under `home`.
#[must_use]
pub fn not_allowed(place: &str, probe: &Probe, home: &Path) -> Option<String> {
    let printed = PathBuf::from(probe.stdout.trim());
    if probe.success && printed.starts_with(home) {
        return None;
    }

    Some(format!(
        "place {place}: refused an isolated place: stdout {:?}, stderr {:?}",
        probe.stdout.trim(),
        probe.stderr.trim()
    ))
}

/// Runs `exe` as a probe for each of `places`, with a real `HOME` it must
/// abort on and an isolated one it must resolve, both under `scratch`.
/// The real `HOME` is a stand-in: any `HOME` outside the temporary
/// directory the probe is given is real to the guard.
#[must_use]
pub fn guard_holds(exe: &Path, scratch: &Path, places: &[&str]) -> Vec<String> {
    let (real, temp) = (scratch.join("real-home"), scratch.join("tmp"));
    let isolated = temp.join("home");
    if let Err(e) = fs::create_dir_all(&real).and_then(|()| fs::create_dir_all(&isolated)) {
        return vec![format!("create the probe's directories: {e}")];
    }

    let mut problems = Vec::new();
    for place in places {
        let run = |home: &Path| run_probe(exe, place, home, &temp, PROBE_LIMIT);
        match run(&real) {
            Ok(probe) => problems.extend(not_refused(place, &probe)),
            Err(e) => problems.push(e),
        }
        match run(&isolated) {
            Ok(probe) => problems.extend(not_allowed(place, &probe, &isolated)),
            Err(e) => problems.push(e),
        }
    }

    problems
}

/// The reads of real state in `body`, the text of the Rust source `file`
/// that is not a resolver: a read of `HOME`, `PARINN_HOME`, Claude Code's
/// configuration directory or the profile directory by name, or a call
/// to the standard library's `HOME` lookup. Each is a line: `file:line: ...`.
#[must_use]
pub fn unguarded_reads(file: &str, body: &str) -> Vec<String> {
    let lookup = concat!("home", "_dir(");
    let mut lines: Vec<usize> = NAMES
        .iter()
        .flat_map(|name| body.match_indices(name))
        .filter(|(i, _)| READS.iter().any(|r| body[..*i].trim_end().ends_with(r)))
        .map(|(i, _)| line_of(body, i))
        .chain(body.match_indices(lookup).map(|(i, _)| line_of(body, i)))
        .collect();
    lines.sort_unstable();
    lines.dedup();

    lines
        .into_iter()
        .map(|n| {
            let text = body.lines().nth(n - 1).unwrap_or_default().trim();
            format!("{file}:{n}: reads real state outside testkit::guard: {text}")
        })
        .collect()
}

/// The 1-based line of `body` that byte `at` falls on.
fn line_of(body: &str, at: usize) -> usize {
    body[..at].matches('\n').count() + 1
}

/// Requires every Rust source under `crates/` and `tooling/`, but the
/// [`RESOLVERS`], to leave real state to the guard.
#[must_use]
pub fn no_unguarded_reads(c: &Checkout) -> Vec<String> {
    let mut files = Vec::new();
    let mut problems = Vec::new();
    for dir in SOURCES {
        if let Err(e) = rust_files(&c.root.join(dir), &mut files) {
            problems.push(e);
        }
    }
    for path in files {
        let rel = path.strip_prefix(&c.root).unwrap_or(&path);
        let rel = rel.to_string_lossy().replace('\\', "/");
        if RESOLVERS.contains(&rel.as_str()) {
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(body) => problems.extend(unguarded_reads(&rel, &body)),
            Err(e) => problems.push(format!("read {rel}: {e}")),
        }
    }

    problems
}

/// Collects every `.rs` file under `dir`, build output left out, sorted.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let list = |e: std::io::Error| format!("list {}: {e}", dir.display());
    let mut paths = fs::read_dir(dir)
        .map_err(list)?
        .map(|e| e.map(|e| e.path()).map_err(list))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths {
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "target") {
                rust_files(&path, out)?;
            }
        } else if path.extension().is_some_and(|x| x == "rs") {
            out.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
