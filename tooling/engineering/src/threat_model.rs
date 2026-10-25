//! SEC-17: the threat model kept in the repository covers every row of
//! the boundary register with the threat sources acting across it,
//! names the sources §6.1 lists, says of each row whether its controls
//! are enforced in M1, and by which tests, designed, or pending with
//! their milestone, compares ParleyInn with Zed Delta control by
//! control, and records a review naming the current minor release, by
//! a reviewer other than its author once it is reviewed.

use crate::boundary::{self, Row};
use crate::{Checkout, privacy};

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

/// The tests a status may cite: each test function by name with whether
/// it runs, and each scenario by id with whether it is pending.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Proofs {
    pub tests: Vec<(String, bool)>,
    pub scenarios: Vec<(String, bool)>,
}

/// What a row's status says, when it says none of them.
pub const STATUSES: &str =
    "enforced in M1 with the tests that prove it, designed, or pending with its milestone";

/// Requires the status of register row `row` to be one of
/// [`STATUSES`]: a control enforced in M1 cites tests that exist and
/// run, after `test:` or `tests:`; a pending one names its milestone.
#[must_use]
pub fn status_holds(row: &str, status: &str, proofs: &Proofs) -> Vec<String> {
    if status.starts_with("enforced in M1") {
        let cited = privacy::cites(status);
        if cited.is_empty() {
            return vec![format!(
                "register row {row} is enforced in M1 and cites no test"
            )];
        }
        return cited
            .iter()
            .filter(|c| !privacy::resolves(&proofs.tests, &proofs.scenarios, c))
            .map(|c| format!("register row {row} cites {c}, which is no test that exists and runs"))
            .collect();
    }
    if status.starts_with("pending") {
        let milestone = status.split(|c: char| !c.is_ascii_alphanumeric()).any(|w| {
            w.strip_prefix('M')
                .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
        });
        if milestone {
            return Vec::new();
        }
        return vec![format!(
            "register row {row} is pending and names no milestone"
        )];
    }
    if status.starts_with("designed") {
        return Vec::new();
    }

    vec![format!("register row {row} names no status: {STATUSES}")]
}

/// Requires a row of the model for each register row, naming at least
/// one source of the legend, at least one control, and its status.
#[must_use]
pub fn rows_covered(body: &str, register: &[Row], proofs: &Proofs) -> Vec<String> {
    let known: Vec<String> = legend(body).into_iter().map(|(id, _)| id).collect();
    let table = table_rows(body);
    let mut problems = Vec::new();
    for row in register {
        let Some(found) = table.iter().find(|c| c.len() >= 6 && c[0] == row.id) else {
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
        problems.extend(status_holds(&row.id, &found[5], proofs));
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

/// Requires a review row naming `release`, who wrote the model for it
/// and the outcome: reviewed, by someone named other than its author,
/// or awaiting a reviewer. An outcome still awaiting passes here; the
/// release gate, not this check, refuses to ship it.
#[must_use]
pub fn reviewed(body: &str, release: &str) -> Vec<String> {
    let Some(c) = table_rows(body)
        .into_iter()
        .find(|c| c.len() >= 4 && c[0] == release)
    else {
        return vec![format!("{MODEL} records no review of release {release}")];
    };
    let (author, reviewer, outcome) = (&c[1], &c[2], &c[3]);
    let problem = if author.is_empty() || outcome.is_empty() {
        Some(format!(
            "the review of release {release} names no author or outcome"
        ))
    } else if outcome.starts_with("Reviewed") && reviewer.is_empty() {
        Some(format!("release {release} is reviewed by no one named"))
    } else if outcome.starts_with("Reviewed") && reviewer == author {
        Some(format!(
            "release {release} is reviewed by the one who wrote its threat model"
        ))
    } else if !outcome.starts_with("Reviewed") && !outcome.contains("awaiting") {
        Some(format!(
            "the outcome for release {release} is neither reviewed nor awaiting a reviewer"
        ))
    } else {
        None
    };

    problem.into_iter().collect()
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
    let mut problems = Vec::new();
    let proofs = match (
        privacy::test_functions(c),
        privacy::scenario_states(&c.root.join("features")),
    ) {
        (Ok(tests), Ok(scenarios)) => Proofs { tests, scenarios },
        (Err(e), _) | (_, Err(e)) => {
            problems.push(e);
            Proofs::default()
        }
    };
    problems.extend(legend_matches(&body, &security));
    problems.extend(rows_covered(&body, &register, &proofs));
    problems.extend(compares(&body));
    problems.extend(match minor_release(&manifest) {
        Some(release) => reviewed(&body, &release),
        None => vec![format!("{VERSION_MANIFEST} names no version")],
    });

    problems
}

#[cfg(test)]
mod tests;
