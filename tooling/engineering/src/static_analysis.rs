//! ENG-16's core row: CI gates on the static checks (clippy with
//! warnings denied, rustfmt, cargo-deny, cargo-audit and the custom
//! checks), every crate forbids unsafe code, a build-time check refuses
//! code that computes derived artifacts a clock or a random order, and
//! the core's build-time reach evidence is present and gated. That evidence is the core's
//! clippy bans on sockets and program starts (its own code) and its
//! reach test, which refuses any crate linked into it (dependencies and
//! the code they run as the process starts). Unsafe code being
//! forbidden, the core's own code cannot add start-up code either.
//!
//! "Gated" means a job the `ci` job needs runs the check and is not
//! allowed to fail, and the `ci` job fails unless every job it needs
//! succeeded: the rulesets require that one job.

use std::fs;

use crate::Checkout;
use crate::product_crates::{
    DETERMINISM_METHODS, DETERMINISM_TYPES, REACH_BANS, TYPES, banned, clippy_config, crates,
    missing_bans,
};
use crate::review_workflow::{Job, Workflow};

/// The CI workflow.
pub const CI_WORKFLOW: &str = ".github/workflows/ci.yml";

/// The job the rulesets require, which passes only when every job it
/// needs passed.
pub const AGGREGATOR: &str = "ci";

/// Each static check ENG-16 names, with the command CI runs it with.
/// The custom checks are the gate tests `cargo test` runs, the core's
/// reach test among them.
pub const STATIC_CHECKS: [(&str, &str); 5] = [
    (
        "cargo clippy",
        "cargo clippy --workspace --all-targets --locked -- -D warnings",
    ),
    ("cargo fmt", "cargo fmt --all --check"),
    ("cargo-deny", "cargo deny check licenses bans sources"),
    ("cargo-audit", "cargo audit"),
    ("the custom checks", "cargo test --workspace --locked"),
];

/// The TOML checks, with the command CI runs each with: Taplo holds
/// every TOML file to the layout `.taplo.toml` sets, and each to the
/// schema `.taplo.toml` names for it.
pub const TOML_CHECKS: [(&str, &str); 2] = [
    ("taplo fmt", "taplo fmt --check"),
    ("taplo lint", "taplo lint"),
];

/// The check that enforces the clippy bans.
const CLIPPY: (&str, &str) = STATIC_CHECKS[0];

/// The check that runs the reach test.
const CUSTOM: (&str, &str) = STATIC_CHECKS[4];

/// The core's crate, by its directory under `crates/`.
const CORE: &str = "parinn";

/// The core's reach test, and the test in it.
const REACH_TEST: (&str, &str) = (
    "crates/parinn/tests/reach.rs",
    "the_core_executable_links_only_crates_its_reach_allows",
);

/// How a problem with the core's reach evidence starts.
const MISSING: &str = "the core's reach evidence is missing: ";

/// The attribute a crate root carries to forbid unsafe code itself.
const FORBID: &str = "#![forbid(unsafe_code)]";

/// The one crate that holds unsafe code: the shim through which Windows'
/// key store calls DPAPI (ADR-2610171300). It denies unsafe code in its
/// own lints instead of inheriting the workspace's forbid, allows it in
/// one module, and Miri runs its tests (ENG-09).
pub const UNSAFE_SHIM: &str = "crates/platform";

/// The jobs `job` needs: an inline list or a single job.
#[must_use]
pub fn needs(job: &Job) -> Vec<String> {
    let Some(rest) = job
        .text
        .lines()
        .find_map(|l| l.trim().strip_prefix("needs:"))
    else {
        return Vec::new();
    };

    rest.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whether `job` runs `command`: a line of it, a `run:` key and any
/// environment assignments before the command left out, that is the
/// command, or the command followed by more arguments.
#[must_use]
pub fn runs(job: &Job, command: &str) -> bool {
    job.text.lines().any(|line| {
        let l = line.trim();
        let l = l.strip_prefix("- ").unwrap_or(l);
        let l = without_assignments(l.strip_prefix("run:").unwrap_or(l).trim());
        l.strip_prefix(command)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
    })
}

/// A shell command line without the `NAME=value` assignments before
/// its command, such as `HOME="$decoy" cargo test`. A value may be
/// quoted, spaces and all.
#[must_use]
pub fn without_assignments(line: &str) -> &str {
    let mut rest = line;
    while let Some((name, value)) = rest.split_once('=') {
        let is_name = name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !is_name {
            break;
        }
        let end = match value.chars().next() {
            Some(q @ ('"' | '\'')) => value[1..].find(q).map(|i| i + 2),
            _ => Some(value.find(' ').unwrap_or(value.len())),
        };
        match end {
            Some(end) => rest = value[end..].trim_start(),
            None => break,
        }
    }
    rest
}

/// Whether `job` may fail without failing the workflow.
fn may_fail(job: &Job) -> bool {
    job.text.lines().any(|l| {
        l.trim()
            .strip_prefix("continue-on-error:")
            .is_some_and(|v| v.trim() != "false")
    })
}

/// Requires a job the `ci` job needs, and that may not fail, to run
/// `check` with its `command`.
fn gates_on(flow: &Workflow, (check, command): (&str, &str)) -> Vec<String> {
    let needed = flow
        .jobs
        .iter()
        .find(|j| j.name == AGGREGATOR)
        .map(needs)
        .unwrap_or_default();
    let jobs: Vec<&Job> = flow
        .jobs
        .iter()
        .filter(|j| j.name != AGGREGATOR && runs(j, command))
        .collect();
    if jobs.is_empty() {
        return vec![format!("no CI job runs {check}: {command}")];
    }
    let mut problems = Vec::new();
    for j in jobs {
        let gated = needed.contains(&j.name);
        if gated && !may_fail(j) {
            return Vec::new();
        }
        if !gated {
            problems.push(format!(
                "job {} runs {check}, but the {AGGREGATOR} job does not need it",
                j.name
            ));
        } else {
            problems.push(format!(
                "job {} runs {check}, but may fail without failing CI: continue-on-error",
                j.name
            ));
        }
    }

    problems
}

/// Requires the `ci` job to run after a failed job and to fail unless
/// every job it needs succeeded.
fn aggregates(flow: &Workflow) -> Vec<String> {
    let Some(ci) = flow.jobs.iter().find(|j| j.name == AGGREGATOR) else {
        return vec![format!("the CI workflow has no {AGGREGATOR} job")];
    };
    let wants = ["if: ${{ always() }}", "needs.*.result", "\"success\""];
    if wants.iter().all(|w| ci.text.contains(w)) {
        Vec::new()
    } else {
        vec![format!(
            "the {AGGREGATOR} job does not run after a failed job and fail unless every job it needs succeeded"
        )]
    }
}

/// Requires CI to gate on every static check of [`STATIC_CHECKS`].
#[must_use]
pub fn ci_gates(flow: &Workflow) -> Vec<String> {
    let mut problems: Vec<String> = STATIC_CHECKS
        .iter()
        .flat_map(|check| gates_on(flow, *check))
        .collect();
    problems.extend(aggregates(flow));

    problems
}

/// Requires CI to gate on every TOML check of [`TOML_CHECKS`].
#[must_use]
pub fn toml_gates(flow: &Workflow) -> Vec<String> {
    TOML_CHECKS
        .iter()
        .flat_map(|check| gates_on(flow, *check))
        .collect()
}

/// The lines of the TOML table `[header]`, comments left out.
pub(crate) fn table(toml: &str, header: &str) -> Vec<String> {
    let want = format!("[{header}]");
    let mut inside = false;
    let mut out = Vec::new();
    for line in toml.lines() {
        let line = line.split_once('#').map_or(line, |(code, _)| code).trim();
        if line.starts_with('[') {
            inside = line == want;
        } else if inside && !line.is_empty() {
            out.push(line.to_owned());
        }
    }

    out
}

/// The value `key` holds in a table's `lines`.
pub(crate) fn value<'a>(lines: &'a [String], key: &str) -> Option<&'a str> {
    lines.iter().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        (k.trim() == key).then_some(v.trim())
    })
}

/// Requires the workspace lints to forbid unsafe code.
///
/// # Errors
///
/// Fails when `[workspace.lints.rust]` does not set `unsafe_code` to
/// `forbid`.
pub fn workspace_forbids_unsafe(root_manifest: &str) -> Result<(), String> {
    let lints = table(root_manifest, "workspace.lints.rust");
    let level = value(&lints, "unsafe_code").map(|v| v.replace(' ', ""));
    let forbids = level.is_some_and(|v| v == "\"forbid\"" || v.contains("level=\"forbid\""));
    if forbids {
        Ok(())
    } else {
        Err("the workspace lints do not forbid unsafe_code".to_owned())
    }
}

/// Whether a crate's manifest denies unsafe code in its own lints.
fn denies_unsafe(manifest: &str) -> bool {
    value(&table(manifest, "lints.rust"), "unsafe_code").map(|v| v.replace(' ', ""))
        == Some("\"deny\"".to_owned())
}

/// Whether a crate's manifest inherits the workspace lints.
#[must_use]
pub fn inherits_workspace_lints(manifest: &str) -> bool {
    value(&table(manifest, "lints"), "workspace") == Some("true")
}

/// The workspace's member crates, by their directory: each entry of
/// `members` in the root manifest, a `dir/*` glob read as every
/// directory under `dir` holding a `Cargo.toml`. Sorted.
///
/// # Errors
///
/// Fails when a glob's directory cannot be listed.
pub fn members(c: &Checkout, root_manifest: &str) -> Result<Vec<String>, String> {
    let workspace = table(root_manifest, "workspace").join("\n");
    let list = workspace
        .split_once("members")
        .and_then(|(_, rest)| rest.split_once('['))
        .and_then(|(_, rest)| rest.split_once(']'))
        .map_or("", |(inside, _)| inside);
    let mut out = Vec::new();
    for entry in list.split('"').skip(1).step_by(2) {
        let Some(dir) = entry.strip_suffix("/*") else {
            out.push(entry.to_owned());
            continue;
        };
        let entries = fs::read_dir(c.root.join(dir)).map_err(|e| format!("list {dir}: {e}"))?;
        for e in entries.flatten() {
            if e.path().join("Cargo.toml").is_file() {
                out.push(format!("{dir}/{}", e.file_name().to_string_lossy()));
            }
        }
    }
    out.sort();

    Ok(out)
}

/// Requires a crate with its own lints, `member`, to forbid unsafe
/// code in each of its crate roots.
fn forbids_in_roots(c: &Checkout, member: &str) -> Vec<String> {
    let roots: Vec<&str> = ["src/lib.rs", "src/main.rs"]
        .into_iter()
        .filter(|r| c.root.join(member).join(r).is_file())
        .collect();
    let lead =
        format!("crate {member} neither inherits the workspace lints nor forbids unsafe code");
    if roots.is_empty() {
        return vec![format!("{lead}: it has no src/lib.rs or src/main.rs")];
    }

    roots
        .into_iter()
        .filter(|r| {
            !fs::read_to_string(c.root.join(member).join(r)).is_ok_and(|b| b.contains(FORBID))
        })
        .map(|r| format!("{lead} in {r}"))
        .collect()
}

/// Requires the workspace lints to forbid unsafe code, and every
/// member crate to inherit them or forbid unsafe code in its own crate
/// roots, so no crate can switch the ban off; the one unsafe shim,
/// [`UNSAFE_SHIM`], denies it in its own lints instead.
#[must_use]
pub fn crates_forbid_unsafe(c: &Checkout) -> Vec<String> {
    let root = match c.read("Cargo.toml") {
        Ok(body) => body,
        Err(e) => return vec![e],
    };
    let mut problems: Vec<String> = workspace_forbids_unsafe(&root).err().into_iter().collect();
    let members = match members(c, &root) {
        Ok(members) => members,
        Err(e) => {
            problems.push(e);
            return problems;
        }
    };
    for member in members {
        match c.read(&format!("{member}/Cargo.toml")) {
            Err(e) => problems.push(e),
            Ok(manifest) if inherits_workspace_lints(&manifest) => {}
            Ok(manifest) if member == UNSAFE_SHIM => {
                if !denies_unsafe(&manifest) {
                    problems.push(format!(
                        "crate {member}, the one unsafe shim, does not deny unsafe code in its own lints"
                    ));
                }
            }
            Ok(_) => problems.extend(forbids_in_roots(c, &member)),
        }
    }

    problems
}

/// Requires the core's build-time reach evidence: its clippy
/// configuration banning every socket and program start, its reach test
/// in place, and CI gating on both, clippy and the custom checks.
#[must_use]
pub fn core_reach_evidence(c: &Checkout, flow: &Workflow) -> Vec<String> {
    let mut problems = Vec::new();
    let file = clippy_config(c, CORE);
    match c.read(&file) {
        Err(e) => problems.push(format!("{MISSING}{e}")),
        Ok(body) => {
            let have = banned(&body);
            let held = |b: &&str| have.get(TYPES).is_some_and(|set| set.contains(*b));
            problems.extend(
                REACH_BANS
                    .iter()
                    .filter(|b| !held(b))
                    .map(|b| format!("{MISSING}{file} does not ban {b}")),
            );
        }
    }
    let (reach, test) = REACH_TEST;
    match c.read(reach) {
        Err(e) => problems.push(format!("{MISSING}{e}")),
        Ok(body) if !body.contains(&format!("fn {test}(")) => {
            problems.push(format!("{MISSING}{reach} has no test {test}"));
        }
        Ok(_) => {}
    }
    problems.extend(gates_on(flow, CLIPPY));
    problems.extend(gates_on(flow, CUSTOM));

    problems
}

/// Requires a build-time check that refuses code that computes derived
/// artifacts a clock or a random order: every product crate's clippy configuration bans them,
/// and CI gates on clippy.
#[must_use]
pub fn derived_code_gated(c: &Checkout, flow: &Workflow) -> Vec<String> {
    let names = match crates(c) {
        Ok(names) => names,
        Err(e) => return vec![e],
    };
    let determinism = |b: &str| DETERMINISM_TYPES.contains(&b) || DETERMINISM_METHODS.contains(&b);
    let mut problems = Vec::new();
    for name in names {
        let file = clippy_config(c, &name);
        match c.read(&file) {
            Err(e) => problems.push(e),
            Ok(body) => problems.extend(
                missing_bans(&body)
                    .into_iter()
                    .filter(|(_, b)| determinism(b))
                    .map(|(table, b)| {
                        format!(
                            "code that computes derived artifacts may read a clock or randomness: crate {name}: {file} does not ban {b} in {table}"
                        )
                    }),
            ),
        }
    }
    problems.extend(gates_on(flow, CLIPPY));

    problems
}

#[cfg(test)]
mod tests;
