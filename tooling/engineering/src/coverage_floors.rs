//! ENG-11: line coverage of at least 90% for the security-sensitive
//! crates and 80% overall. The `coverage` crate measures every test
//! layer and fails CI when a crate falls below its floor in
//! `[workspace.metadata.coverage]`. These checks hold the profile CI
//! takes to the row's words: each security-sensitive crate the
//! workspace builds is held to at least the row's floor over every test
//! layer, every crate to at least the overall one (so the whole
//! workspace, a sum of its crates, is too), and the CI job that merges
//! the layers is one the `CI` check needs.

use coverage::plan::Plan;

use crate::benchmarks::needs;
use crate::review_workflow::Workflow;

/// The CI job that merges the layers and holds the floors.
pub const JOB: &str = "coverage";

/// The run that merges the layers' profiles and holds the floors.
pub const MERGE: &str = "-p coverage -- --merge";

/// The job every other job joins.
const AGGREGATOR: &str = "ci";

/// What the ENG-11 row says.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The security-sensitive crates, in the row's order.
    pub sensitive: Vec<String>,
    /// Their floor, in percent of lines.
    pub sensitive_floor: f64,
    /// The floor for the workspace as a whole.
    pub overall_floor: f64,
}

/// The words that introduce the list of security-sensitive crates.
const SENSITIVE: &str = "for security-sensitive crates";

/// The word the overall floor ends with.
const OVERALL: &str = "overall";

impl Row {
    /// Reads the row's text.
    ///
    /// # Errors
    ///
    /// Fails when the text names no crate list or states a floor
    /// neither before the list nor before "overall".
    pub fn parse(text: &str) -> Result<Self, String> {
        let (before, rest) = text.split_once(SENSITIVE).unwrap_or((text, ""));
        let list = rest
            .split_once('(')
            .and_then(|(_, l)| l.split_once(')'))
            .map(|(l, after)| (l, after));
        let Some((list, after)) = list else {
            return Err(
                "the ENG-11 row names no security-sensitive crates in parentheses".to_owned(),
            );
        };
        let sensitive = list
            .split(',')
            .map(|c| c.trim().trim_matches('`').to_owned())
            .filter(|c| !c.is_empty())
            .collect();
        let sensitive_floor = last_percent(before)
            .ok_or_else(|| format!("the ENG-11 row states no floor before {SENSITIVE:?}"))?;
        let overall_floor = after
            .split_once(OVERALL)
            .and_then(|(b, _)| last_percent(b))
            .ok_or_else(|| format!("the ENG-11 row states no floor before {OVERALL:?}"))?;

        Ok(Self {
            sensitive,
            sensitive_floor,
            overall_floor,
        })
    }
}

/// The last `N%` in `text`.
fn last_percent(text: &str) -> Option<f64> {
    let (number, _) = text.rsplit_once('%')?;
    let digits = number
        .rsplit(|c: char| !(c.is_ascii_digit() || c == '.'))
        .next()?;
    digits.parse().ok()
}

/// The security-sensitive crates the workspace builds, and those it does
/// not build yet, each in the row's order.
#[must_use]
pub fn built(plan: &Plan, row: &Row) -> (Vec<String>, Vec<String>) {
    row.sensitive
        .iter()
        .cloned()
        .partition(|name| plan.crates.iter().any(|c| &c.name == name))
}

/// What keeps `name`'s floor over every test layer from reaching
/// `floor`; `overall` marks the overall floor's words.
fn short_of(plan: &Plan, name: &str, floor: f64, overall: &str) -> Option<String> {
    match plan.floors_of(name).all {
        None => Some(format!("{name} is held to no floor over every test layer")),
        Some(held) if held < floor => Some(format!(
            "{name} is held to {held}% of its lines, below ENG-11's {floor}%{overall}"
        )),
        Some(_) => None,
    }
}

/// The security-sensitive crates the workspace builds that are held to
/// less than the row's floor.
#[must_use]
pub fn sensitive_problems(plan: &Plan, row: &Row) -> Vec<String> {
    built(plan, row)
        .0
        .iter()
        .filter_map(|name| short_of(plan, name, row.sensitive_floor, ""))
        .collect()
}

/// The crates held to less than the overall floor: while there is none,
/// the workspace, whose lines are its crates' lines, is held to it too.
#[must_use]
pub fn overall_problems(plan: &Plan, row: &Row) -> Vec<String> {
    plan.crates
        .iter()
        .filter_map(|c| short_of(plan, &c.name, row.overall_floor, " overall"))
        .collect()
}

/// What keeps CI from holding the merged profile to the floors in a job
/// the `CI` check needs.
#[must_use]
pub fn gated(workflow: &Workflow) -> Vec<String> {
    let job = |name: &str| workflow.jobs.iter().find(|j| j.name == name);
    let Some(merge) = job(JOB) else {
        return vec![format!("the CI workflow has no job {JOB}")];
    };
    let mut problems = Vec::new();
    if !merge.text.contains(MERGE) {
        problems.push(format!("job {JOB} does not run {MERGE:?}"));
    }
    if !job(AGGREGATOR).is_some_and(|ci| needs(ci).iter().any(|n| n == JOB)) {
        problems.push(format!("job {AGGREGATOR} does not need {JOB}"));
    }

    problems
}

#[cfg(test)]
mod tests;
