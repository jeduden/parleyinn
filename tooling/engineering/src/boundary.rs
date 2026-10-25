//! SEC-19, and the core's row of SEC-01: every component, process and
//! protocol sits in exactly one boundary of the register in SRS §6.3,
//! every B1, B2 and B3 row is off until turned on, every process the
//! repository builds is one the register lists, and the build-time
//! reach evidence of each component that exists is present. In M1 the
//! core is the one component the repository builds; the rows of the
//! components that follow need their evidence when their crates land.

use crate::review_workflow::Workflow;
use crate::{Checkout, static_analysis};

/// The register, relative to the checkout.
pub const REGISTER: &str = "docs/srs/06b-boundary-register.md";

/// The boundaries a row may name: B0 to B3, or a protocol that stays
/// outside ParleyInn, or none for a rejected row.
const BOUNDARIES: [&str; 6] = ["B0", "B1", "B2", "B3", "outside", "—"];

/// The executables the repository builds, each with the register's
/// component it runs as (SEC-01: one component per process).
pub const PROCESSES: [(&str, &str); 1] = [("parinn", "core")];

/// One row of the register.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub name: String,
    pub component: String,
    pub boundary: String,
    pub reach: String,
    pub default: String,
    pub requirements: String,
}

/// The rows of the register in `body`: each table row whose first cell
/// is a number or an `X` and a number.
#[must_use]
pub fn parse_rows(body: &str) -> Vec<Row> {
    body.lines()
        .filter_map(|line| {
            let cells: Vec<&str> = line.trim().strip_prefix('|')?.split('|').collect();
            let cell = |i: usize| cells.get(i).map_or(String::new(), |c| c.trim().to_owned());
            let id = cell(0);
            let number = id.strip_prefix('X').unwrap_or(&id);
            let numbered = !number.is_empty() && number.chars().all(|c| c.is_ascii_digit());

            numbered.then(|| Row {
                id,
                name: cell(1),
                component: cell(2),
                boundary: cell(3),
                reach: cell(4),
                default: cell(5),
                requirements: cell(8),
            })
        })
        .collect()
}

/// The register kept in the checkout.
///
/// # Errors
///
/// Fails when the file cannot be read or holds no row.
pub fn register(c: &Checkout) -> Result<Vec<Row>, String> {
    let found = parse_rows(&c.read(REGISTER)?);
    if found.is_empty() {
        return Err(format!("{REGISTER} holds no row"));
    }

    Ok(found)
}

/// Requires each row to name exactly one boundary, and each component to
/// sit in one boundary across its rows (SEC-19).
#[must_use]
pub fn assigned_once(rows: &[Row]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen: Vec<&Row> = Vec::new();
    for row in rows {
        if !BOUNDARIES.contains(&row.boundary.as_str()) {
            problems.push(format!(
                "row {} names {:?}, not exactly one of B0, B1, B2, B3, outside or none",
                row.id, row.boundary
            ));
        }
        if let Some(other) = seen.iter().find(|r| {
            r.component == row.component
                && r.boundary != row.boundary
                && row.boundary.starts_with('B')
                && r.boundary.starts_with('B')
        }) {
            problems.push(format!(
                "the {} sits in {} in row {} and in {} in row {}",
                row.component, other.boundary, other.id, row.boundary, row.id
            ));
        }
        if seen.iter().any(|r| r.id == row.id) {
            problems.push(format!("row {} appears twice", row.id));
        }
        seen.push(row);
    }

    problems
}

/// Requires every B1, B2 and B3 row to be off by default (SEC-19, SEC-01).
#[must_use]
pub fn off_until_on(rows: &[Row]) -> Vec<String> {
    rows.iter()
        .filter(|r| ["B1", "B2", "B3"].contains(&r.boundary.as_str()) && r.default != "off")
        .map(|r| {
            format!(
                "row {} is {} and {:?} by default, not off",
                r.id, r.boundary, r.default
            )
        })
        .collect()
}

/// Requires the row of a feature that reaches off the machine through
/// a tool the principal runs outside ParleyInn, its own tunnel, to
/// exist, sit in B1, B2 or B3, be off by default and name the
/// requirements that govern it (SEC-19).
#[must_use]
pub fn tunnel_has_its_own_row(rows: &[Row]) -> Vec<String> {
    let Some(row) = rows
        .iter()
        .find(|r| r.reach.contains("own tunnel, outside ParleyInn"))
    else {
        return vec!["no row names the principal's own tunnel, outside ParleyInn".to_owned()];
    };
    let mut problems = off_until_on(std::slice::from_ref(row));
    if !row.boundary.starts_with('B') {
        problems.push(format!("row {} is in no boundary", row.id));
    }
    if row.requirements.is_empty() {
        problems.push(format!("row {} names no requirement", row.id));
    }

    problems
}

/// The executables the product crates build, by name, sorted: every
/// target of kind `bin` that `cargo metadata` lists for a package under
/// `crates/`, whether a `[[bin]]` table, `src/main.rs` or a file under
/// `src/bin/` declares it.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON.
pub fn executables(metadata: &str) -> Result<Vec<String>, String> {
    let value: serde_json::Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let root = value["workspace_root"].as_str().unwrap_or_default();
    let product = std::path::Path::new(root).join(crate::architecture::PRODUCT);
    let mut found: Vec<String> = value["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| {
            p["manifest_path"]
                .as_str()
                .is_some_and(|m| std::path::Path::new(m).starts_with(&product))
        })
        .flat_map(|p| p["targets"].as_array().into_iter().flatten())
        .filter(|t| {
            t["kind"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|k| k == "bin")
        })
        .filter_map(|t| t["name"].as_str().map(str::to_owned))
        .collect();
    found.sort();
    found.dedup();

    Ok(found)
}

/// Requires each of `found` executables to run as a component the
/// register lists (SEC-19, SEC-01).
#[must_use]
pub fn unlisted_processes(found: &[String], rows: &[Row]) -> Vec<String> {
    found
        .iter()
        .filter_map(|name| {
            let component = PROCESSES.iter().find(|(p, _)| p == name).map(|(_, c)| *c);
            match component {
                None => Some(format!(
                    "the process {name} runs as no component the register lists"
                )),
                Some(c) if !rows.iter().any(|r| r.component == c) => Some(format!(
                    "the process {name} runs as the {c}, which no register row names"
                )),
                Some(_) => None,
            }
        })
        .collect()
}

/// The build-time reach evidence of each component the repository
/// builds, one problem per gap (SEC-01, SEC-19): for the core, its
/// clippy bans, its reach test and the CI jobs that gate on them.
#[must_use]
pub fn evidence(c: &Checkout, flow: &Workflow, found: &[String]) -> Vec<String> {
    found
        .iter()
        .flat_map(|name| match PROCESSES.iter().find(|(p, _)| p == name) {
            Some((_, "core")) => static_analysis::core_reach_evidence(c, flow),
            _ => vec![format!(
                "no reach evidence is defined for the process {name}"
            )],
        })
        .collect()
}

/// The CI boundary check: every problem of the register, the processes
/// `metadata`, the workspace's `cargo metadata`, lists, and the
/// evidence, in the checkout (SEC-19).
#[must_use]
pub fn check(c: &Checkout, flow: &Workflow, metadata: &str) -> Vec<String> {
    let rows = match register(c) {
        Ok(rows) => rows,
        Err(e) => return vec![e],
    };
    let found = match executables(metadata) {
        Ok(found) => found,
        Err(e) => return vec![e],
    };
    let mut problems = assigned_once(&rows);
    problems.extend(off_until_on(&rows));
    problems.extend(tunnel_has_its_own_row(&rows));
    problems.extend(unlisted_processes(&found, &rows));
    problems.extend(evidence(c, flow, &found));

    problems
}

#[cfg(test)]
mod tests;
