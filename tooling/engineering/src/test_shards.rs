//! The CI `test` job runs every workspace crate's tests but the
//! scenario crate's, which the `scenarios` job runs, divided over
//! shards so each fits its five-minute budget. Each shard runs
//! `cargo test` on a package list of its own, the list taken from the
//! job's place in the matrix (`strategy.job-index`), never from a
//! number the matrix lists. This check holds the lists to every crate
//! exactly once, so the custom checks (ENG-16) and the decoy `HOME`
//! (ENG-14) still cover every test target: a crate added to the
//! workspace and to no shard fails it, and so does one in two shards.

use std::collections::BTreeSet;

use crate::Checkout;
use crate::review_workflow::{Job, Workflow};
use crate::static_analysis::{CI_WORKFLOW, members, runs, table, value};

/// The sharded job.
pub const TEST_JOB: &str = "test";

/// The crate the `scenarios` job runs instead, beside its scenarios.
pub const SCENARIO: &str = "scenario";

/// The command each shard runs its list with, behind the decoy `HOME`.
pub const SHARD_COMMAND: &str = "cargo test --locked \"${packages[@]}\"";

/// The lines that choose a shard's list by its place in the matrix,
/// and fail a job whose matrix has more or fewer shards than lists.
pub const SELECTION: [&str; 4] = [
    "JOB_INDEX: ${{ strategy.job-index }}",
    "JOB_TOTAL: ${{ strategy.job-total }}",
    "if [ \"${#shards[@]}\" -ne \"$JOB_TOTAL\" ]; then",
    "read -ra packages <<< \"${shards[$JOB_INDEX]}\"",
];

/// How a problem with the shards starts.
const LEAD: &str = "the test job's shards";

/// The package lists of the `shards=( ... )` array in `job`, one per
/// shard, in order. A list may span lines through a backslash before
/// each line break, as bash reads it inside double quotes.
///
/// # Errors
///
/// Fails when the job has no such array, or holds anything in it but
/// double-quoted lists.
pub fn lists(job: &Job) -> Result<Vec<String>, String> {
    let lines: Vec<&str> = job.text.lines().collect();
    let open = lines.iter().position(|l| l.trim() == "shards=(");
    let close = open.and_then(|o| {
        lines[o..]
            .iter()
            .position(|l| l.trim() == ")")
            .map(|c| o + c)
    });
    let (Some(open), Some(close)) = (open, close) else {
        return Err(format!(
            "{LEAD}: no shards=( ... ) array in {CI_WORKFLOW} lists their packages"
        ));
    };
    let body = lines[open + 1..close].join("\n").replace("\\\n", " ");
    let parts: Vec<&str> = body.split('"').collect();
    if parts.len().is_multiple_of(2) || parts.iter().step_by(2).any(|p| !p.trim().is_empty()) {
        return Err(format!(
            "{LEAD}: the shards=( ... ) array in {CI_WORKFLOW} holds more than double-quoted package lists"
        ));
    }

    Ok(parts
        .iter()
        .skip(1)
        .step_by(2)
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect())
}

/// The packages one shard's `list` runs, out of the workspace's
/// `packages`: each `-p` name, or with `--workspace` every package but
/// those it excludes. Sorted.
///
/// # Errors
///
/// Fails on a flag other than `-p`, `--workspace` and `--exclude`, on
/// `-p` beside `--workspace`, on `--exclude` without it, and on a name
/// that is no workspace package.
pub fn selected(list: &str, packages: &[String]) -> Result<Vec<String>, String> {
    let mut named = BTreeSet::new();
    let mut excluded = BTreeSet::new();
    let mut workspace = false;
    let mut words = list.split_whitespace();
    while let Some(word) = words.next() {
        let set = match word {
            "--workspace" => {
                workspace = true;
                continue;
            }
            "-p" | "--package" => &mut named,
            "--exclude" => &mut excluded,
            other => {
                return Err(format!(
                    "`{other}`: list only -p, --workspace and --exclude"
                ));
            }
        };
        let name = words.next().ok_or(format!("`{word}` names no package"))?;
        if !packages.iter().any(|p| p == name) {
            return Err(format!("{name} is no workspace crate"));
        }
        set.insert(name.to_owned());
    }
    if workspace == !named.is_empty() || (!workspace && !excluded.is_empty()) {
        return Err("list -p packages, or --workspace with its --exclude packages".to_owned());
    }
    if !workspace {
        return Ok(named.into_iter().collect());
    }

    Ok(packages
        .iter()
        .filter(|p| !excluded.contains(*p))
        .cloned()
        .collect())
}

/// Requires `shards`, the packages each shard runs, to hold every one
/// of `packages` but [`SCENARIO`] exactly once, and [`SCENARIO`] in
/// none.
#[must_use]
pub fn coverage(shards: &[Vec<String>], packages: &[String]) -> Vec<String> {
    let mut problems = Vec::new();
    for package in packages {
        let at: Vec<String> = (1..)
            .zip(shards)
            .filter(|(_, s)| s.contains(package))
            .map(|(n, _)| n.to_string())
            .collect();
        match (package == SCENARIO, at.len()) {
            (true, 0) | (false, 1) => {}
            (true, _) => problems.push(format!(
                "{LEAD}: shard {} runs the {SCENARIO} crate, which the scenarios job runs: exclude it in {CI_WORKFLOW}",
                at.join(" and ")
            )),
            (false, 0) => problems.push(format!(
                "{LEAD}: crate {package} runs in no shard: add -p {package} to one in {CI_WORKFLOW}"
            )),
            (false, _) => problems.push(format!(
                "{LEAD}: crate {package} runs in shards {}: keep it in one in {CI_WORKFLOW}",
                at.join(" and ")
            )),
        }
    }

    problems
}

/// The number of entries the job's `shard:` matrix lists.
fn matrix_size(job: &Job) -> Option<usize> {
    job.text.lines().find_map(|l| {
        let list = l.trim().strip_prefix("shard: [")?.strip_suffix(']')?;
        Some(list.split(',').filter(|e| !e.trim().is_empty()).count())
    })
}

/// The workspace's package names, by each member's manifest. Sorted.
///
/// # Errors
///
/// Fails when the root or a member's manifest cannot be read, or a
/// member's manifest names no package.
pub fn packages(c: &Checkout) -> Result<Vec<String>, String> {
    let root = c.read("Cargo.toml")?;
    let mut out = Vec::new();
    for member in members(c, &root)? {
        let file = format!("{member}/Cargo.toml");
        let manifest = c.read(&file)?;
        let name = value(&table(&manifest, "package"), "name")
            .map(|n| n.trim_matches('"').to_owned())
            .ok_or(format!("{file} names no package"))?;
        out.push(name);
    }
    out.sort();

    Ok(out)
}

/// The problems with how `job` picks its shard and runs its list.
fn selection(job: &Job) -> Vec<String> {
    let mut problems: Vec<String> = SELECTION
        .iter()
        .filter(|line| !job.text.contains(*line))
        .map(|line| {
            format!(
                "{LEAD}: the job does not choose its list by its place in the matrix in {CI_WORKFLOW}: {line}"
            )
        })
        .collect();
    if !runs(job, SHARD_COMMAND) {
        problems.push(format!(
            "{LEAD}: the job does not run {SHARD_COMMAND} in {CI_WORKFLOW}"
        ));
    }

    problems
}

/// Requires the CI `test` job to choose each shard's package list by
/// the shard's place in the matrix, to list one per shard, and the
/// lists to run every workspace crate but [`SCENARIO`] exactly once.
#[must_use]
pub fn shards_cover_the_workspace(c: &Checkout, flow: &Workflow) -> Vec<String> {
    let Some(job) = flow.jobs.iter().find(|j| j.name == TEST_JOB) else {
        return vec![format!("the CI workflow has no {TEST_JOB} job")];
    };
    let mut problems = selection(job);
    let (lists, packages) = match (lists(job), packages(c)) {
        (Ok(l), Ok(p)) => (l, p),
        (l, p) => {
            problems.extend(l.err().into_iter().chain(p.err()));
            return problems;
        }
    };
    if matrix_size(job) != Some(lists.len()) {
        problems.push(format!(
            "{LEAD}: the matrix in {CI_WORKFLOW} does not list one shard per package list ({} lists)",
            lists.len()
        ));
    }
    let mut shards = Vec::new();
    for (n, list) in (1..).zip(&lists) {
        match selected(list, &packages) {
            Ok(s) => shards.push(s),
            Err(e) => problems.push(format!("{LEAD}: shard {n}: {e}")),
        }
    }
    if shards.len() == lists.len() {
        problems.extend(coverage(&shards, &packages));
    }

    problems
}

#[cfg(test)]
mod tests;
