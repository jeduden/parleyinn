//! The deterministic check of the cases (ENG-32), run without a model:
//! every component has its defect, clean and injection case; every
//! case parses, applies to the tree and, for a component the review
//! bundle dispatches, makes it dispatch that component. The gate test
//! and the `check` command run it; a run starts with it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use review_bundle::dispatch::{DEEP_GENERALISTS, GENERALISTS, agents};
use review_bundle::{Snapshot, analyse};

use crate::case::{self, Case};
use crate::component::{self, Component, Kind, suite_of};
use crate::derived;
use crate::diff::{self, Change};

/// The kinds of case every component must have.
pub const REQUIRED_KINDS: [&str; 3] = ["defect", "clean", "injection"];

/// Every case of the tree at `root`: those written under `cases_dir`
/// and those derived from the tree.
///
/// # Errors
///
/// Fails when a case does not read.
pub fn all_cases(root: &Path, cases_dir: &Path) -> Result<Vec<Case>, String> {
    all_cases_from(root, cases_dir, None)
}

/// [`all_cases`], with a component's written cases taken from
/// `base_cases` instead when it has a directory there: the cases main
/// already holds judge a component main already has, so a change
/// cannot soften the cases that judge it; a new component is judged by
/// the cases it brings.
///
/// # Errors
///
/// Fails when a case does not read.
pub fn all_cases_from(
    root: &Path,
    cases_dir: &Path,
    base_cases: Option<&Path>,
) -> Result<Vec<Case>, String> {
    let mut cases = case::load_dir(cases_dir)?;
    if let Some(base) = base_cases {
        let judging = case::load_dir(base)?;
        let kept: BTreeSet<&str> = judging.iter().map(|c| c.component.as_str()).collect();
        cases.retain(|c| !kept.contains(c.component.as_str()));
        cases.extend(judging);
    }
    cases.extend(derived::cases(root)?);
    cases.sort_by_key(Case::id);

    Ok(cases)
}

/// The problems of a tree's cases, in two kinds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Problems {
    /// A component without one of its required kinds of case.
    pub kinds: Vec<String>,
    /// A case that names no component, a deep instance or an outcome
    /// of a generalist or a consultant, does not apply, or does not
    /// dispatch its component.
    pub cases: Vec<String>,
}

impl Problems {
    /// Every problem, sorted.
    #[must_use]
    pub fn all(&self) -> Vec<String> {
        let mut out = self.kinds.clone();
        out.extend(self.cases.iter().cloned());
        out.sort();
        out
    }
}

/// Every problem with the cases of the tree at `root`, whose
/// components are read from `tree` and whose written cases lie under
/// `cases_dir`; none when they are sound. With a `selection`, only the
/// suites of those components are checked, so one component's cases
/// can run while another's are still being written.
#[must_use]
pub fn problems(
    root: &Path,
    tree: &Path,
    cases_dir: &Path,
    selection: Option<&[String]>,
) -> Vec<String> {
    problems_from(root, tree, cases_dir, None, selection)
}

/// [`problems`], with the cases read as [`all_cases_from`] reads them.
#[must_use]
pub fn problems_from(
    root: &Path,
    tree: &Path,
    cases_dir: &Path,
    base_cases: Option<&Path>,
    selection: Option<&[String]>,
) -> Vec<String> {
    examine_from(root, tree, cases_dir, base_cases, selection).map_or_else(|e| vec![e], |p| p.all())
}

/// The components without their kinds of case, or what could not be
/// read.
#[must_use]
pub fn kind_problems(root: &Path, tree: &Path, cases_dir: &Path) -> Vec<String> {
    examine(root, tree, cases_dir, None).map_or_else(|e| vec![e], |p| p.kinds)
}

/// The cases that do not read, apply or dispatch their component, or
/// what could not be read.
#[must_use]
pub fn case_problems(root: &Path, tree: &Path, cases_dir: &Path) -> Vec<String> {
    examine(root, tree, cases_dir, None).map_or_else(|e| vec![e], |p| p.cases)
}

/// Examines the cases as [`problems`] does, each problem in its kind.
///
/// # Errors
///
/// Fails when the components, the cases or the tree cannot be read.
pub fn examine(
    root: &Path,
    tree: &Path,
    cases_dir: &Path,
    selection: Option<&[String]>,
) -> Result<Problems, String> {
    examine_from(root, tree, cases_dir, None, selection)
}

/// [`examine`], with the cases read as [`all_cases_from`] reads them.
///
/// # Errors
///
/// Fails when the components, the cases or the tree cannot be read.
pub fn examine_from(
    root: &Path,
    tree: &Path,
    cases_dir: &Path,
    base_cases: Option<&Path>,
    selection: Option<&[String]>,
) -> Result<Problems, String> {
    let components = catalog(root, tree, selection)?;
    let selected = |name: &str| {
        selection.is_none_or(|names| names.iter().any(|n| suite_of(n) == suite_of(name)))
    };
    let cases: Vec<Case> = all_cases_from(root, cases_dir, base_cases)?
        .into_iter()
        .filter(|c| selected(&c.component))
        .collect();
    let base = Snapshot::read(root).map_err(|e| format!("read the tree: {e}"))?;
    let mut problems = Problems::default();
    let by_name: BTreeMap<&str, &Component> =
        components.iter().map(|c| (c.name.as_str(), c)).collect();
    let mut kinds: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for case in &cases {
        let id = case.id();
        let Some(component) = by_name.get(case.component.as_str()) else {
            problems.cases.push(format!("case {id} names no component"));
            continue;
        };
        if suite_of(&case.component) != case.component {
            problems.cases.push(format!(
                "case {id} names a deep instance; cases belong to {}",
                suite_of(&case.component)
            ));
            continue;
        }
        kinds
            .entry(case.component.as_str())
            .or_default()
            .insert(case.kind.as_str());
        if matches!(component.kind, Kind::Generalist | Kind::Consultant)
            && !case.expect.outcome.is_empty()
        {
            problems.cases.push(format!(
                "case {id} expects an outcome of a {}, which gives none",
                component.kind.word()
            ));
        }
        match dispatches(root, &base, case, component) {
            Ok(None) => {}
            Ok(Some(dispatched)) => problems.cases.push(format!(
                "case {id} does not dispatch {}; the bundle dispatches {}",
                component.name,
                if dispatched.is_empty() {
                    "no agent".to_owned()
                } else {
                    dispatched.into_iter().collect::<Vec<_>>().join(", ")
                }
            )),
            Err(e) => problems.cases.push(e),
        }
    }
    for component in components
        .iter()
        .filter(|c| suite_of(&c.name) == c.name && selected(&c.name))
    {
        let has = kinds.get(component.name.as_str());
        for kind in REQUIRED_KINDS {
            if !has.is_some_and(|kinds| kinds.contains(kind)) {
                problems
                    .kinds
                    .push(format!("component {} has no {kind} case", component.name));
            }
        }
    }
    problems.kinds.sort();
    problems.cases.sort();

    Ok(problems)
}

/// The components to check: every one of the tree, or with a
/// `selection` those named and the suites they run.
fn catalog(
    root: &Path,
    tree: &Path,
    selection: Option<&[String]>,
) -> Result<Vec<Component>, String> {
    match selection {
        None => component::catalog(root, tree),
        Some(names) => {
            let mut wanted: Vec<String> = names.iter().map(|n| suite_of(n)).collect();
            wanted.extend(names.iter().cloned());
            wanted.sort();
            wanted.dedup();
            wanted
                .iter()
                .map(|name| component::load(tree, name))
                .collect()
        }
    }
}

/// Whether the review bundle dispatches `component` for `case`: none
/// when it does, or when the bundle never dispatches the component,
/// the orchestrator and a consultant; else the agents it dispatches
/// instead.
///
/// # Errors
///
/// Fails when the case does not apply or the head's snapshot cannot be
/// read.
pub fn dispatches(
    root: &Path,
    base: &Snapshot,
    case: &Case,
    component: &Component,
) -> Result<Option<BTreeSet<String>>, String> {
    let changes = case.changes(&|file| fs::read_to_string(root.join(file)).ok())?;
    if matches!(component.kind, Kind::Orchestrator | Kind::Consultant) {
        return Ok(None);
    }
    let head = head_snapshot(base, &changes).map_err(|e| format!("case {}: {e}", case.id()))?;
    let analysis = analyse(base, &head, &diff::all(&changes), case.consent.is_some());
    let dispatched = agents(&analysis.dispatch);
    let family: Vec<&str> = match component.kind {
        Kind::Generalist => GENERALISTS
            .iter()
            .zip(DEEP_GENERALISTS)
            .find(|(normal, deep)| **normal == component.name || *deep == component.name)
            .map_or_else(|| vec![component.name.as_str()], |(n, d)| vec![*n, d]),
        Kind::Specialist | Kind::Orchestrator | Kind::Consultant => vec![component.name.as_str()],
    };
    if family.iter().any(|name| dispatched.contains(*name)) {
        Ok(None)
    } else {
        Ok(Some(dispatched))
    }
}

/// The snapshot of the tree with `changes` made: `base` with each
/// changed record file read again from its new text, so no tree is
/// written and the records the change leaves alone are not read twice.
fn head_snapshot(base: &Snapshot, changes: &[Change]) -> Result<Snapshot, String> {
    let mut head = base.clone();
    for change in changes {
        head = head
            .with_file(&change.path, change.new.as_deref())
            .map_err(|e| format!("read the changed tree: {e}"))?;
    }

    Ok(head)
}

#[cfg(test)]
mod tests;
