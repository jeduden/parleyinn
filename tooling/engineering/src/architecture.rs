//! ENG-02: the workspace's crates keep the dependency directions each
//! manifest declares, and every library crate declares one
//! responsibility.
//!
//! The declaration sits in each crate's own manifest, beside what it
//! governs, under `[package.metadata.parinn]`:
//!
//! - `responsibility`: the one thing a library crate does.
//! - `may-depend-on`: the workspace crates it may depend on, as normal
//!   or build dependencies. Every crate declares it, an empty list
//!   included.
//! - `may-dev-depend-on`: the workspace crates its tests may depend on.
//!
//! A product crate (under `crates/`) never depends on a tooling crate
//! (under `tooling/`) as a normal or build dependency, declared or not:
//! the product's executables must not link repository tooling. The
//! tooling crates fall under the same declarations. The manifests are
//! the stakeholder's paths in CODEOWNERS, so a change to a direction
//! waits for the stakeholder's approval.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// Where the product's crates live: the one place the architecture and
/// ENG-18 checks read it from.
pub(crate) const PRODUCT: &str = "crates/";

/// Where the repository tooling lives.
const TOOLING: &str = "tooling/";

/// One workspace crate as `cargo metadata` and its manifest's
/// `[package.metadata.parinn]` table describe it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceCrate {
    pub name: String,
    /// The crate's directory, relative to the workspace root.
    pub dir: String,
    pub library: bool,
    pub responsibility: Vec<String>,
    pub may_depend_on: Option<Vec<String>>,
    pub may_dev_depend_on: Vec<String>,
    pub links: Vec<Link>,
}

/// One dependency of a crate on a crate reached by path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub on: String,
    pub dev: bool,
}

impl WorkspaceCrate {
    fn is_product(&self) -> bool {
        self.dir.starts_with(PRODUCT)
    }

    fn is_tooling(&self) -> bool {
        self.dir.starts_with(TOOLING)
    }
}

/// Reads the workspace crates out of `cargo metadata --no-deps`, sorted
/// by name.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON or a declaration is not
/// of the shape the module documents.
pub fn workspace(metadata: &str) -> Result<Vec<WorkspaceCrate>, String> {
    let value: Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let root = value["workspace_root"]
        .as_str()
        .ok_or("cargo metadata names no workspace root")?;
    let packages = value["packages"]
        .as_array()
        .ok_or("cargo metadata lists no packages")?;
    let mut members = packages
        .iter()
        .map(|p| member(root, p))
        .collect::<Result<Vec<_>, _>>()?;
    members.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(members)
}

/// One package of the metadata.
fn member(root: &str, package: &Value) -> Result<WorkspaceCrate, String> {
    let name = package["name"].as_str().unwrap_or_default().to_owned();
    let manifest = package["manifest_path"].as_str().unwrap_or_default();
    let dir = manifest
        .strip_prefix(root)
        .and_then(|rest| rest.strip_suffix("Cargo.toml"))
        .map(|rest| rest.trim_matches('/').to_owned())
        .ok_or_else(|| format!("crate {name}: manifest {manifest} is outside the workspace"))?;
    let declared = &package["metadata"]["parinn"];
    let lists = |key| list_of_names(declared, key).map_err(|e| format!("crate {name}: {e}"));

    Ok(WorkspaceCrate {
        dir,
        library: is_library(package),
        responsibility: responsibility(declared).map_err(|e| format!("crate {name}: {e}"))?,
        may_depend_on: lists("may-depend-on")?,
        may_dev_depend_on: lists("may-dev-depend-on")?.unwrap_or_default(),
        links: links(package),
        name,
    })
}

/// Whether the package builds a library.
fn is_library(package: &Value) -> bool {
    package["targets"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|t| t["kind"].as_array().into_iter().flatten())
        .any(|kind| matches!(kind.as_str(), Some("lib" | "rlib" | "proc-macro")))
}

/// The declared responsibilities: one string, or a list of them, which
/// the check then refuses.
fn responsibility(declared: &Value) -> Result<Vec<String>, String> {
    const NOT_TEXT: &str = "responsibility must be a string";
    match &declared["responsibility"] {
        Value::Null => Ok(Vec::new()),
        Value::String(one) => Ok(vec![one.clone()]),
        Value::Array(many) => many
            .iter()
            .map(|r| r.as_str().map(str::to_owned).ok_or(NOT_TEXT.to_owned()))
            .collect(),
        _ => Err(NOT_TEXT.to_owned()),
    }
}

/// The list of crate names under `key`, `None` when it is not declared.
fn list_of_names(declared: &Value, key: &str) -> Result<Option<Vec<String>>, String> {
    match &declared[key] {
        Value::Null => Ok(None),
        Value::Array(names) => names
            .iter()
            .map(|n| n.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()
            .map(Some)
            .ok_or_else(|| format!("{key} must be a list of crate names")),
        _ => Err(format!("{key} must be a list of crate names")),
    }
}

/// The package's dependencies reached by path, which are the workspace's
/// own crates; a build dependency counts as a normal one.
fn links(package: &Value) -> Vec<Link> {
    package["dependencies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|d| d["path"].is_string())
        .filter_map(|d| {
            Some(Link {
                on: d["name"].as_str()?.to_owned(),
                dev: d["kind"].as_str() == Some("dev"),
            })
        })
        .collect()
}

/// Every problem the architecture check finds in the workspace that
/// `metadata` describes: broken directions, then responsibilities not
/// declared once. A `metadata` it cannot read is the one problem.
#[must_use]
pub fn problems(metadata: &str) -> Vec<String> {
    match workspace(metadata) {
        Ok(members) => [directions(&members), responsibilities(&members)].concat(),
        Err(e) => vec![e],
    }
}

/// `members` with one dependency no direction allows added: the first
/// product crate depending on the first tooling crate. Returns the
/// two crates' names, product first, so a scenario can show the check
/// refuses it.
///
/// # Errors
///
/// Fails when the workspace has no product crate or no tooling crate.
pub fn with_forbidden_dependency(
    members: &[WorkspaceCrate],
) -> Result<(Vec<WorkspaceCrate>, String, String), String> {
    let product = members.iter().find(|m| m.is_product());
    let tooling = members.iter().find(|m| m.is_tooling());
    let (Some(product), Some(tooling)) = (product, tooling) else {
        return Err("the workspace needs a product crate and a tooling crate".to_owned());
    };
    let (from, on) = (product.name.clone(), tooling.name.clone());
    let mut out = members.to_vec();
    for m in out.iter_mut().filter(|m| m.name == from) {
        m.links.push(Link {
            on: on.clone(),
            dev: false,
        });
    }

    Ok((out, from, on))
}

/// Requires a problem that names both crates `a` and `b`.
///
/// # Errors
///
/// Fails when no line of `problems` names both.
pub fn names_both(problems: &[String], a: &str, b: &str) -> Result<(), String> {
    let names = |p: &String, n: &str| p.split_whitespace().any(|w| w.trim_end_matches(',') == n);
    if problems.iter().any(|p| names(p, a) && names(p, b)) {
        Ok(())
    } else {
        Err(format!(
            "no problem names both crate {a} and crate {b}: {problems:?}"
        ))
    }
}

/// The dependencies that break the declared directions, one line each,
/// naming both crates.
#[must_use]
pub fn directions(members: &[WorkspaceCrate]) -> Vec<String> {
    let known: BTreeSet<&str> = members.iter().map(|m| m.name.as_str()).collect();
    let by_name: BTreeMap<&str, &WorkspaceCrate> =
        members.iter().map(|m| (m.name.as_str(), m)).collect();
    let mut problems = BTreeSet::new();
    for m in members {
        problems.extend(declaration_problems(m, &known, &by_name));
        problems.extend(m.links.iter().filter_map(|l| link_problem(m, l, &by_name)));
    }

    problems.into_iter().collect()
}

/// What is wrong with the directions `m` declares.
fn declaration_problems(
    m: &WorkspaceCrate,
    known: &BTreeSet<&str>,
    by_name: &BTreeMap<&str, &WorkspaceCrate>,
) -> Vec<String> {
    let Some(may) = &m.may_depend_on else {
        return vec![format!(
            "crate {} declares no may-depend-on directions",
            m.name
        )];
    };
    let named = may
        .iter()
        .map(|n| ("may-depend-on", n))
        .chain(m.may_dev_depend_on.iter().map(|n| ("may-dev-depend-on", n)));
    let mut out = Vec::new();
    for (key, n) in named {
        if !known.contains(n.as_str()) {
            out.push(format!(
                "crate {} names crate {n} in {key}, which is no workspace crate",
                m.name
            ));
        } else if key == "may-depend-on" && m.is_product() && by_name[n.as_str()].is_tooling() {
            out.push(format!(
                "product crate {} may depend on tooling crate {n}, which a product crate never does",
                m.name
            ));
        }
    }

    out
}

/// What is wrong with the dependency `l` of `m`, if anything.
fn link_problem(
    m: &WorkspaceCrate,
    l: &Link,
    by_name: &BTreeMap<&str, &WorkspaceCrate>,
) -> Option<String> {
    let on_tooling = by_name.get(l.on.as_str()).is_some_and(|c| c.is_tooling());
    if m.is_product() && on_tooling && !l.dev {
        return Some(format!(
            "product crate {} depends on tooling crate {}, which a product crate never does",
            m.name, l.on
        ));
    }
    let allowed = if l.dev {
        Some(&m.may_dev_depend_on)
    } else {
        m.may_depend_on.as_ref()
    };
    // A crate that declares no directions is reported once, apart.
    let allowed = allowed?;
    (!allowed.contains(&l.on)).then(|| {
        let kind = if l.dev { " (dev)" } else { "" };
        format!(
            "crate {} depends on crate {}{kind}, which its declared directions forbid",
            m.name, l.on
        )
    })
}

/// The library crates that do not declare exactly one responsibility,
/// and the crates that declare the same one.
#[must_use]
pub fn responsibilities(members: &[WorkspaceCrate]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut by_text: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for m in members.iter().filter(|m| m.library) {
        match m.responsibility.as_slice() {
            [] => problems.push(format!(
                "library crate {} declares no responsibility",
                m.name
            )),
            [one] if one.trim().is_empty() => problems.push(format!(
                "library crate {} declares an empty responsibility",
                m.name
            )),
            [one] => by_text.entry(one.trim()).or_default().push(&m.name),
            many => problems.push(format!(
                "library crate {} declares {} responsibilities, want exactly one",
                m.name,
                many.len()
            )),
        }
    }
    for (text, names) in by_text.iter().filter(|(_, names)| names.len() > 1) {
        problems.push(format!(
            "crates {} declare the same responsibility: {text}",
            names.join(" and ")
        ));
    }
    problems.sort();

    problems
}

#[cfg(test)]
mod tests;
