//! The requirement–scenario gate.

use std::collections::{BTreeMap, BTreeSet};

use crate::Scenario;

/// Compares the requirements against the scenarios and returns every
/// disagreement, sorted, one line each. It is empty exactly when:
///
/// - every requirement has one scenario tagged with its id, and every
///   scenario's id names a requirement;
/// - a scenario's priority tag equals its requirement's Pri field, and
///   is absent when the requirement has none (NFR, ASM);
/// - a scenario's invariant tags equal its requirement's Traces.
///
/// A requirement edited in the SRS without its scenario, or a scenario
/// retagged without the SRS, fails the build here rather than drifting.
#[must_use]
pub fn check(reqs: &[srs::Requirement], scenarios: &[Scenario]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut by_id: BTreeMap<&str, &Scenario> = BTreeMap::new();
    for sc in scenarios {
        if let Some(prev) = by_id.get(sc.id.as_str()) {
            problems.push(format!(
                "{}: tagged on {}:{} and {}:{}",
                sc.id,
                prev.path.display(),
                prev.line,
                sc.path.display(),
                sc.line
            ));
            continue;
        }
        by_id.insert(&sc.id, sc);
    }

    let known: BTreeSet<&str> = reqs.iter().map(|r| r.id.as_str()).collect();
    for r in reqs {
        let Some(sc) = by_id.get(r.id.as_str()) else {
            problems.push(format!("{}: no scenario under features/", r.id));
            continue;
        };
        if sc.priority != r.priority {
            problems.push(format!(
                "{}: scenario priority {:?}, requirement says {:?}",
                r.id, sc.priority, r.priority
            ));
        }
        let mut want = r.traces.clone();
        want.sort();
        if sc.invariants != want {
            problems.push(format!(
                "{}: scenario invariants [{}], requirement traces [{}]",
                r.id,
                sc.invariants.join(" "),
                want.join(" ")
            ));
        }
    }
    for (id, sc) in by_id.iter().filter(|(id, _)| !known.contains(*id)) {
        problems.push(format!(
            "{id}: tagged on {}:{} but no SRS requirement has that id",
            sc.path.display(),
            sc.line
        ));
    }
    problems.sort();

    problems
}

#[cfg(test)]
mod tests;
