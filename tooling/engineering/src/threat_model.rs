//! SEC-17: the threat model kept in the repository covers every row of
//! the boundary register with the threat sources acting across it,
//! names the sources §6.1 lists, compares ParleyInn with Zed Delta
//! control by control, and records a review naming the current minor
//! release.

use crate::Checkout;
use crate::boundary::{self, Row};

/// The threat model, relative to the checkout.
pub const MODEL: &str = "docs/security/threat-model.md";

/// Where §6.1 lists the threat sources.
const SECURITY: &str = "docs/srs/06-security.md";

/// The manifest whose version names the current release.
const VERSION_MANIFEST: &str = "crates/parinn/Cargo.toml";

/// The controls the comparison with Zed Delta goes through.
pub const CONTROLS: [&str; 4] = [
    "Record signing",
    "Trust in other principals",
    "Central-service dependence",
    "Key custody",
];

/// The cells of the table row `line`, trimmed; none for any other line.
fn cells(line: &str) -> Vec<String> {
    match line.trim().strip_prefix('|') {
        Some(rest) => rest.split('|').map(|c| c.trim().to_owned()).collect(),
        None => Vec::new(),
    }
}

/// Every table row of `body` as its cells.
fn table_rows(body: &str) -> Vec<Vec<String>> {
    body.lines().map(cells).filter(|c| !c.is_empty()).collect()
}

/// The threat sources §6.1 lists: the first cell of each row of the
/// table under "### Threat sources".
#[must_use]
pub fn listed_sources(security: &str) -> Vec<String> {
    let table: Vec<&str> = security
        .lines()
        .skip_while(|l| l.trim() != "### Threat sources")
        .skip(1)
        .skip_while(|l| !l.starts_with('|'))
        .take_while(|l| l.starts_with('|'))
        .collect();

    table
        .iter()
        .skip(2)
        .filter_map(|l| cells(l).into_iter().next())
        .collect()
}

/// The sources the model's legend names, by id.
fn legend(body: &str) -> Vec<(String, String)> {
    table_rows(body)
        .into_iter()
        .filter(|c| c.len() >= 2 && is_source_id(&c[0]))
        .map(|c| (c[0].clone(), c[1].clone()))
        .collect()
}

fn is_source_id(cell: &str) -> bool {
    cell.strip_prefix('S')
        .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// Requires the legend to name the sources §6.1 lists, in its order.
#[must_use]
pub fn legend_matches(body: &str, security: &str) -> Vec<String> {
    let names: Vec<String> = legend(body).into_iter().map(|(_, n)| n).collect();
    let listed = listed_sources(security);
    if names == listed {
        return Vec::new();
    }

    vec![format!(
        "the threat sources of {MODEL} differ from those of {SECURITY}: {names:?} against {listed:?}"
    )]
}

/// Requires a row of the model for each register row, naming at least
/// one source of the legend and at least one control.
#[must_use]
pub fn rows_covered(body: &str, register: &[Row]) -> Vec<String> {
    let known: Vec<String> = legend(body).into_iter().map(|(id, _)| id).collect();
    let table = table_rows(body);
    let mut problems = Vec::new();
    for row in register {
        let Some(found) = table.iter().find(|c| c.len() >= 5 && c[0] == row.id) else {
            problems.push(format!("{MODEL} has no row for register row {}", row.id));
            continue;
        };
        let named = found[3]
            .split(',')
            .map(str::trim)
            .any(|s| known.iter().any(|k| k == s));
        if !named {
            problems.push(format!(
                "register row {} names no known threat source",
                row.id
            ));
        }
        if found[4].is_empty() {
            problems.push(format!("register row {} names no control", row.id));
        }
    }

    problems
}

/// Requires the comparison with Zed Delta to go through each control,
/// with a cell for ParleyInn and one for Zed Delta.
#[must_use]
pub fn compares(body: &str) -> Vec<String> {
    let table = table_rows(body);
    let mut problems = Vec::new();
    if !body.contains("## Comparison with Zed Delta") {
        problems.push(format!("{MODEL} has no comparison with Zed Delta"));
    }
    for control in CONTROLS {
        match table.iter().find(|c| c.len() >= 3 && c[0] == control) {
            None => problems.push(format!(
                "the comparison with Zed Delta leaves out {control}"
            )),
            Some(c) if c[1].is_empty() || c[2].is_empty() => {
                problems.push(format!("the comparison on {control} leaves a side empty"))
            }
            Some(_) => {}
        }
    }

    problems
}

/// The minor release `manifest` names: `major.minor` of its version.
#[must_use]
pub fn minor_release(manifest: &str) -> Option<String> {
    let version = manifest
        .lines()
        .find_map(|l| l.strip_prefix("version = \""))?
        .split('"')
        .next()?;
    let mut parts = version.split('.');

    Some(format!("{}.{}", parts.next()?, parts.next()?))
}

/// Requires a review row naming `release` and who reviewed it.
#[must_use]
pub fn reviewed(body: &str, release: &str) -> Vec<String> {
    let row = table_rows(body)
        .into_iter()
        .find(|c| c.len() >= 3 && c[0] == release);
    match row {
        Some(c) if !c[1].is_empty() && !c[2].is_empty() => Vec::new(),
        Some(_) => vec![format!(
            "the review of release {release} names no reviewer or outcome"
        )],
        None => vec![format!("{MODEL} records no review of release {release}")],
    }
}

/// Every problem of the threat model in the checkout (SEC-17).
#[must_use]
pub fn check(c: &Checkout) -> Vec<String> {
    let (body, security, manifest) =
        match (c.read(MODEL), c.read(SECURITY), c.read(VERSION_MANIFEST)) {
            (Ok(b), Ok(s), Ok(m)) => (b, s, m),
            (b, s, m) => return [b, s, m].into_iter().filter_map(Result::err).collect(),
        };
    let register = match boundary::register(c) {
        Ok(r) => r,
        Err(e) => return vec![e],
    };
    let mut problems = legend_matches(&body, &security);
    problems.extend(rows_covered(&body, &register));
    problems.extend(compares(&body));
    problems.extend(match minor_release(&manifest) {
        Some(release) => reviewed(&body, &release),
        None => vec![format!("{VERSION_MANIFEST} names no version")],
    });

    problems
}

#[cfg(test)]
mod tests;
