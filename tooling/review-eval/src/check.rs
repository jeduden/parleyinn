//! The deterministic check of the cases (ENG-32), run without a model:
//! every component has its defect, clean and injection case; every
//! case parses, applies to the tree and makes the review bundle
//! dispatch its component. The gate test and the `check` command run
//! it; a run starts with it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use review_bundle::dispatch::{DEEP_GENERALISTS, GENERALISTS, agents};
use review_bundle::{Snapshot, analyse};

use crate::case::{self, Case};
use crate::component::{self, Component, Kind, suite_of};
use crate::derived;
use crate::diff::{self, Change};

/// The kinds of case every component must have.
pub const REQUIRED_KINDS: [&str; 3] = ["defect", "clean", "injection"];

/// What the review bundle reads of a tree, whose change makes the
/// head's snapshot differ from main's.
const SNAPSHOT_PATHS: [&str; 4] = ["docs/srs/", "features/", "docs/adr/", ".github/CODEOWNERS"];

/// Every case of the tree at `root`: those written under `cases_dir`
/// and those derived from the tree.
///
/// # Errors
///
/// Fails when a case does not read.
pub fn all_cases(root: &Path, cases_dir: &Path) -> Result<Vec<Case>, String> {
    let mut cases = case::load_dir(cases_dir)?;
    cases.extend(derived::cases(root)?);
    cases.sort_by_key(Case::id);

    Ok(cases)
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
    let components = match catalog(root, tree, selection) {
        Ok(components) => components,
        Err(e) => return vec![e],
    };
    let selected = |name: &str| {
        selection.is_none_or(|names| names.iter().any(|n| suite_of(n) == suite_of(name)))
    };
    let cases = match all_cases(root, cases_dir) {
        Ok(cases) => cases
            .into_iter()
            .filter(|c| selected(&c.component))
            .collect::<Vec<_>>(),
        Err(e) => return vec![e],
    };
    let base = match Snapshot::read(root) {
        Ok(base) => base,
        Err(e) => return vec![format!("read the tree: {e}")],
    };
    let mut problems = Vec::new();
    let by_name: BTreeMap<&str, &Component> =
        components.iter().map(|c| (c.name.as_str(), c)).collect();
    let mut kinds: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for case in &cases {
        let id = case.id();
        let Some(component) = by_name.get(case.component.as_str()) else {
            problems.push(format!("case {id} names no component"));
            continue;
        };
        if suite_of(&case.component) != case.component {
            problems.push(format!(
                "case {id} names a deep instance; cases belong to {}",
                suite_of(&case.component)
            ));
            continue;
        }
        kinds
            .entry(case.component.as_str())
            .or_default()
            .insert(case.kind.as_str());
        if component.kind == Kind::Generalist && !case.expect.outcome.is_empty() {
            problems.push(format!(
                "case {id} expects an outcome of a generalist, which gives none"
            ));
        }
        match dispatches(root, &base, case, component) {
            Ok(None) => {}
            Ok(Some(dispatched)) => problems.push(format!(
                "case {id} does not dispatch {}; the bundle dispatches {}",
                component.name,
                if dispatched.is_empty() {
                    "no agent".to_owned()
                } else {
                    dispatched.into_iter().collect::<Vec<_>>().join(", ")
                }
            )),
            Err(e) => problems.push(e),
        }
    }
    for component in components
        .iter()
        .filter(|c| suite_of(&c.name) == c.name && selected(&c.name))
    {
        let has = kinds.get(component.name.as_str());
        for kind in REQUIRED_KINDS {
            if !has.is_some_and(|kinds| kinds.contains(kind)) {
                problems.push(format!("component {} has no {kind} case", component.name));
            }
        }
    }
    problems.sort();

    problems
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
/// when it does, else the agents it dispatches instead.
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
    if component.kind == Kind::Orchestrator {
        return Ok(None);
    }
    let changes = case.changes(&|file| fs::read_to_string(root.join(file)).ok())?;
    let head =
        head_snapshot(root, base, &changes).map_err(|e| format!("case {}: {e}", case.id()))?;
    let analysis = analyse(base, &head, &diff::all(&changes), case.consent.is_some());
    let dispatched = agents(&analysis.dispatch);
    let family: Vec<&str> = match component.kind {
        Kind::Generalist => GENERALISTS
            .iter()
            .zip(DEEP_GENERALISTS)
            .find(|(normal, deep)| **normal == component.name || *deep == component.name)
            .map_or_else(|| vec![component.name.as_str()], |(n, d)| vec![*n, d]),
        Kind::Specialist | Kind::Orchestrator => vec![component.name.as_str()],
    };
    if family.iter().any(|name| dispatched.contains(*name)) {
        Ok(None)
    } else {
        Ok(Some(dispatched))
    }
}

/// The snapshot of the tree with `changes` made: `base` itself when no
/// change touches what the bundle reads, else read from a scratch copy
/// of those paths with the changes applied.
fn head_snapshot(root: &Path, base: &Snapshot, changes: &[Change]) -> Result<Snapshot, String> {
    let touches = |c: &Change| SNAPSHOT_PATHS.iter().any(|p| c.path.starts_with(p));
    if !changes.iter().any(touches) {
        return Ok(base.clone());
    }
    let scratch = scratch_dir()?;
    let result = (|| {
        for path in SNAPSHOT_PATHS {
            let path = path.trim_end_matches('/');
            copy_path(&root.join(path), &scratch.join(path))?;
        }
        for change in changes.iter().filter(|c| touches(c)) {
            let path = scratch.join(&change.path);
            match &change.new {
                Some(text) => path
                    .parent()
                    .map_or(Ok(()), fs::create_dir_all)
                    .and_then(|()| fs::write(&path, text))
                    .map_err(|e| format!("write {}: {e}", change.path))?,
                None => {
                    fs::remove_file(&path).map_err(|e| format!("remove {}: {e}", change.path))?
                }
            }
        }
        Snapshot::read(&scratch).map_err(|e| format!("read the changed tree: {e}"))
    })();
    let _ = fs::remove_dir_all(&scratch);

    result
}

/// A fresh directory under the system's temporary directory.
fn scratch_dir() -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("review-eval-{}-{stamp}", std::process::id()));
    fs::create_dir(&path).map_err(|e| format!("make {}: {e}", path.display()))?;

    Ok(path)
}

/// Copies `from`, a file or a directory, to `to`; nothing when it does
/// not exist.
fn copy_path(from: &Path, to: &Path) -> Result<(), String> {
    let at = |e: std::io::Error| format!("copy {}: {e}", from.display());
    if from.is_dir() {
        fs::create_dir_all(to).map_err(at)?;
        for entry in fs::read_dir(from).map_err(at)? {
            let entry = entry.map_err(at)?;
            copy_path(&entry.path(), &to.join(entry.file_name()))?;
        }
    } else if from.is_file() {
        to.parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|()| fs::copy(from, to).map(drop))
            .map_err(at)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
