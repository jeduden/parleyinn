//! ENG-27: the gates of CI are defined once, in `scripts/ci-gates.txt`.
//! `ci.yml` runs each through `scripts/ci-gate.sh NAME` and the local
//! runner `scripts/ci-local.sh` reads the same list, so a command is
//! written in one place. This check fails when `ci.yml` runs the
//! command of a gate inline, runs a gate the list does not define, or
//! when a gate has no job. Reading `ci.yml` through a [`Checkout`]
//! returns it with each call replaced by the gate's command, so the
//! checks of what CI runs see the commands.
//!
//! [`Checkout`]: crate::Checkout

use crate::Checkout;

/// The CI workflow, relative to the checkout.
pub const CI: &str = ".github/workflows/ci.yml";

/// The list of gates, relative to the checkout.
pub const GATES: &str = "scripts/ci-gates.txt";

/// How `ci.yml` calls a gate: this, then the gate's name.
pub const CALL: &str = "scripts/ci-gate.sh ";

/// The fits a gate may declare: runs by default, runs with `--full`, or
/// only on a CI runner.
const FITS: [&str; 3] = ["fast", "full", "no"];

/// The scopes a gate may declare.
const SCOPES: [&str; 6] = ["ws", "pkgs", "units", "crate", "tags", "files"];

/// One gate: a line of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub name: String,
    pub fit: String,
    pub needs: String,
    pub scope: String,
    /// The command CI runs, or `@job:NAME` or `@workflow:NAME`.
    pub ci: String,
    /// The command here: `=` for the CI command, `-` for none.
    pub local: String,
}

impl Gate {
    /// The command CI runs, with `{pkgs}` as `--workspace`; none when
    /// the gate is defined by a job or a workflow.
    #[must_use]
    pub fn command(&self) -> Option<String> {
        (!self.ci.starts_with('@')).then(|| self.ci.replace("{pkgs}", "--workspace"))
    }
}

/// Reads the list: the gates, and a problem for each line that is not
/// a gate.
#[must_use]
pub fn parse(text: &str) -> (Vec<Gate>, Vec<String>) {
    let mut gates: Vec<Gate> = Vec::new();
    let mut problems = Vec::new();
    for (at, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = at + 1;
        let fields: Vec<&str> = line.splitn(6, '|').collect();
        let [name, fit, needs, scope, ci, local] = fields[..] else {
            problems.push(format!("{GATES}:{at} is not name|fit|needs|scope|ci|local"));
            continue;
        };
        if !FITS.contains(&fit) || !SCOPES.contains(&scope) || [name, ci, local].contains(&"") {
            problems.push(format!(
                "{GATES}:{at} gate {name:?} has a fit, scope or empty field it may not have"
            ));
        } else if gates.iter().any(|g| g.name == name) {
            problems.push(format!("{GATES}:{at} gate {name} is defined twice"));
        } else {
            gates.push(Gate {
                name: name.to_owned(),
                fit: fit.to_owned(),
                needs: needs.to_owned(),
                scope: scope.to_owned(),
                ci: ci.to_owned(),
                local: local.to_owned(),
            });
        }
    }
    (gates, problems)
}

/// The gate a line of `ci.yml` calls, by name.
fn called(line: &str) -> Option<&str> {
    let rest = &line[line.find(CALL)? + CALL.len()..];
    Some(rest.split_whitespace().next().unwrap_or(""))
}

/// `ci` with each call of a gate replaced by its command, so the text
/// reads as the commands CI runs. A call of a gate the list does not
/// define, or that a job defines, stays as written.
#[must_use]
pub fn expand(ci: &str, gates: &[Gate]) -> String {
    let mut out: Vec<String> = ci
        .lines()
        .map(|line| {
            let command = called(line)
                .and_then(|name| gates.iter().find(|g| g.name == name))
                .and_then(Gate::command);
            match (command, line.find(CALL)) {
                (Some(command), Some(at)) => {
                    let after = &line[at + CALL.len()..];
                    let end = after.find(char::is_whitespace).unwrap_or(after.len());
                    format!("{}{command}{}", &line[..at], &after[end..])
                }
                _ => line.to_owned(),
            }
        })
        .collect();
    if ci.ends_with('\n') {
        out.push(String::new());
    }
    out.join("\n")
}

/// The command a `run:` line of `ci.yml` holds on one line.
fn inline_command(line: &str) -> Option<&str> {
    line.trim().trim_start_matches("- ").strip_prefix("run: ")
}

/// Problems with `ci` (the workflow's text) against the list `gates`
/// (its text): a bad line, a gate with no job, a call of no gate, and a
/// gate's command written inline. `workflow_exists` says whether
/// `.github/workflows/NAME.yml` exists.
#[must_use]
pub fn check(ci: &str, gates: &str, workflow_exists: &dyn Fn(&str) -> bool) -> Vec<String> {
    let (gates, mut problems) = parse(gates);
    let calls: Vec<&str> = ci.lines().filter_map(called).collect();
    for gate in &gates {
        problems.extend(job_problem(ci, gate, &calls, workflow_exists));
    }
    for name in calls {
        if !gates
            .iter()
            .any(|g| g.name == name && g.command().is_some())
        {
            problems.push(format!(
                "{CI} calls {CALL}{name}, which {GATES} does not define as a command"
            ));
        }
    }
    for (at, line) in ci.lines().enumerate() {
        let Some(inline) = inline_command(line) else {
            continue;
        };
        for gate in gates
            .iter()
            .filter(|g| g.command().as_deref() == Some(inline))
        {
            problems.push(format!(
                "{CI}:{} runs the command of gate {} inline: run {CALL}{} instead",
                at + 1,
                gate.name,
                gate.name
            ));
        }
    }
    problems
}

/// The problem a gate has with its job or workflow, if any.
fn job_problem(
    ci: &str,
    gate: &Gate,
    calls: &[&str],
    workflow_exists: &dyn Fn(&str) -> bool,
) -> Option<String> {
    let name = &gate.name;
    if let Some(job) = gate.ci.strip_prefix("@job:") {
        let found = ci.lines().any(|l| l.trim_end() == format!("  {job}:"));
        (!found).then(|| format!("gate {name} names job {job}, which {CI} does not have"))
    } else if let Some(workflow) = gate.ci.strip_prefix("@workflow:") {
        let found = workflow_exists(&format!(".github/workflows/{workflow}.yml"));
        (!found).then(|| format!("gate {name} names workflow {workflow}, which does not exist"))
    } else {
        (!calls.contains(&name.as_str()))
            .then(|| format!("gate {name} has no job: run {CALL}{name} in {CI}"))
    }
}

/// Problems with the checkout: [`check`] over its `ci.yml` as written
/// and its list of gates.
#[must_use]
pub fn check_repository(c: &Checkout) -> Vec<String> {
    match (c.read_raw(CI), c.read_raw(GATES)) {
        (Ok(ci), Ok(gates)) => check(&ci, &gates, &|path| c.root.join(path).is_file()),
        (ci, gates) => [ci, gates].into_iter().filter_map(Result::err).collect(),
    }
}

#[cfg(test)]
mod tests;
