//! ENG-18: every direct dependency is justified by exactly one accepted
//! ADR, with an allow-listed license; those linked into a shipped
//! executable stay within the stated target.

use std::collections::{BTreeMap, BTreeSet};

use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use adr::Adr;

use crate::{Checkout, architecture, list};

/// The crates the workspace's members depend on directly from a
/// registry, read from `cargo metadata --no-deps`: normal, build and
/// dev dependencies alike, and never a member reached by path.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON.
pub fn cargo_direct_deps(metadata: &str) -> Result<Vec<String>, String> {
    let value: serde_json::Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let deps: BTreeSet<&str> = value["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|p| p["dependencies"].as_array().into_iter().flatten())
        .filter(|d| {
            d["source"]
                .as_str()
                .is_some_and(|s| s.starts_with("registry+"))
        })
        .filter_map(|d| d["name"].as_str())
        .collect();

    Ok(deps.into_iter().map(str::to_owned).collect())
}

/// The crates linked into a shipped executable: the registry crates
/// reached through normal dependencies from each package under
/// `product`, relative to the workspace root cargo reports, with a
/// binary target, following the workspace's own
/// crates by path. Dev and build dependencies link into no shipped
/// executable, and the repository's tooling ships none.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON, names no workspace root
/// to find the product's crates under, or a crate it follows depends by
/// path on a package that is no workspace member, whose crates would
/// go uncounted.
pub fn cargo_shipped_deps(metadata: &str, product: &Path) -> Result<Vec<String>, String> {
    let value: serde_json::Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let packages: BTreeMap<&str, &serde_json::Value> = value["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| Some((p["name"].as_str()?, p)))
        .collect();
    let root = value["workspace_root"]
        .as_str()
        .ok_or("cargo metadata names no workspace_root")?;
    let product = Path::new(root).join(product);
    let mut todo: Vec<&serde_json::Value> = value["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| ships(p, &product))
        .collect();
    let (mut seen, mut deps) = (BTreeSet::new(), BTreeSet::new());
    while let Some(p) = todo.pop() {
        if !seen.insert(p["name"].as_str()) {
            continue;
        }
        for d in normal(p) {
            match (d["source"].as_str(), d["name"].as_str()) {
                (Some(s), Some(name)) if s.starts_with("registry+") => {
                    deps.insert(name);
                }
                (None, Some(name)) => todo.push(packages.get(name).ok_or_else(|| {
                    format!(
                        "{} depends by path on {name}, which is no workspace member",
                        p["name"].as_str().unwrap_or_default()
                    )
                })?),
                _ => {}
            }
        }
    }

    Ok(deps.into_iter().map(str::to_owned).collect())
}

/// Whether `package` builds a shipped executable: a binary target of
/// a crate under `product`.
pub(crate) fn ships(package: &serde_json::Value, product: &Path) -> bool {
    let under = package["manifest_path"]
        .as_str()
        .is_some_and(|m| Path::new(m).starts_with(product));
    let bin = package["targets"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|t| t["kind"].as_array().into_iter().flatten())
        .any(|k| k == "bin");
    under && bin
}

/// The dependencies of `package` that link into what it builds: neither
/// dev nor build.
fn normal(package: &serde_json::Value) -> impl Iterator<Item = &serde_json::Value> {
    package["dependencies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|d| d["kind"].is_null())
}

/// Runs `cargo metadata` on the workspace `manifest` roots, without
/// resolving its dependency graph, with `PARINN_HOME` pointed at
/// `parinn_home` (ENG-14).
///
/// # Errors
///
/// Fails when cargo cannot run, exits unsuccessfully, or prints no
/// UTF-8.
pub fn cargo_metadata(
    cargo: &OsStr,
    manifest: &Path,
    parinn_home: &Path,
) -> Result<String, String> {
    let out = Command::new(cargo)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
            "--manifest-path",
        ])
        .arg(manifest)
        .env("PARINN_HOME", parinn_home)
        .output()
        .map_err(|e| format!("run cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr).trim_end()
        ));
    }

    String::from_utf8(out.stdout).map_err(|e| format!("read cargo metadata: {e}"))
}

/// The direct dependencies a manifest names, and those of them linked
/// into a shipped executable.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DirectDeps {
    /// Every direct dependency: each needs an accepted ADR.
    pub all: Vec<String>,
    /// Those linked into a shipped executable: ENG-18's target counts
    /// these.
    pub shipped: Vec<String>,
}

/// The direct dependencies the manifest `file` names: the registry
/// crates `metadata` lists for the Cargo workspace a `Cargo.toml` roots,
/// with the product's crates where the architecture check finds them.
/// The fuzz workspace's `fuzz/Cargo.toml` is read the same way, but
/// nothing there links into a shipped executable: its crates are not
/// under the product.
///
/// # Errors
///
/// Fails on a manifest with no reader, or when it cannot be read.
pub fn direct_deps(
    checkout: &Checkout,
    file: &str,
    metadata: impl Fn(&Path) -> Result<String, String>,
) -> Result<DirectDeps, String> {
    match file {
        "Cargo.toml" => {
            let json = metadata(&checkout.root.join(file))?;
            Ok(DirectDeps {
                all: cargo_direct_deps(&json)?,
                shipped: cargo_shipped_deps(&json, Path::new(architecture::PRODUCT))?,
            })
        }
        "fuzz/Cargo.toml" => Ok(DirectDeps {
            all: cargo_direct_deps(&metadata(&checkout.root.join(file))?)?,
            shipped: Vec::new(),
        }),
        other => Err(format!("no reader for the dependencies in {other}")),
    }
}

/// The records whose decision is in force.
fn accepted(adrs: &[Adr]) -> impl Iterator<Item = &Adr> {
    adrs.iter().filter(|a| a.status == adr::ACCEPTED)
}

/// Finds, for each direct dependency, the one accepted record that
/// justifies it.
#[must_use]
pub fn deps_named_once(deps: &[String], adrs: &[Adr]) -> Vec<String> {
    let mut by: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for a in accepted(adrs) {
        for m in &a.modules {
            by.entry(&m.name).or_default().push(&a.id);
        }
    }

    deps.iter()
        .filter_map(|dep| {
            let ids = by.get(dep.as_str()).map(Vec::as_slice).unwrap_or_default();
            (ids.len() != 1).then(|| {
                format!(
                    "{dep} is named by {} accepted ADRs {}, want exactly one",
                    ids.len(),
                    list(ids)
                )
            })
        })
        .collect()
}

/// Refuses an accepted record that justifies a module the manifests no
/// longer require directly: a stale decision.
#[must_use]
pub fn modules_are_deps(deps: &[String], adrs: &[Adr]) -> Vec<String> {
    accepted(adrs)
        .flat_map(|a| a.modules.iter().map(move |m| (a, m)))
        .filter(|(_, m)| !deps.contains(&m.name))
        .map(|(a, m)| {
            format!(
                "{} names {}, which is not a direct dependency",
                a.id, m.name
            )
        })
        .collect()
}

/// Checks each accepted module row is filled in, and that its record
/// weighs alternatives.
#[must_use]
pub fn deps_justified(adrs: &[Adr]) -> Vec<String> {
    let mut problems = Vec::new();
    for a in accepted(adrs) {
        if !a.modules.is_empty() && a.sections.get("Alternatives").is_none_or(String::is_empty) {
            problems.push(format!("{} weighs no alternatives", a.id));
        }
        for m in &a.modules {
            for (column, value) in [
                ("Purpose", &m.purpose),
                ("License", &m.license),
                ("Maintenance", &m.maintenance),
            ] {
                if value.is_empty() {
                    problems.push(format!("{} leaves {column:?} empty for {}", a.id, m.name));
                }
            }
        }
    }

    problems
}

/// Checks every accepted module's license against `allowed`, and, for
/// a module `exceptions` names, against the licenses its exception
/// adds.
#[must_use]
pub fn licenses_allowed(
    adrs: &[Adr],
    allowed: &[String],
    exceptions: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    accepted(adrs)
        .flat_map(|a| &a.modules)
        .filter(|m| {
            let extra = exceptions
                .get(&m.name)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let all: Vec<String> = allowed.iter().chain(extra).cloned().collect();
            !license_allowed(&m.license, &all)
        })
        .map(|m| {
            format!(
                "{} is licensed {:?}, not on the allow-list {}",
                m.name,
                m.license,
                list(allowed)
            )
        })
        .collect()
}

/// The per-crate license exceptions `deny_toml` names: each
/// `{ crate = "name", allow = ["License", ...] }` line, comments
/// skipped.
#[must_use]
pub fn license_exceptions(deny_toml: &str) -> BTreeMap<String, Vec<String>> {
    deny_toml
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            let rest = line.strip_prefix("{ crate = \"")?;
            let (name, rest) = rest.split_once("\", allow = [")?;
            let (licenses, _) = rest.split_once(']')?;
            let licenses = licenses
                .split(',')
                .map(|l| l.trim().trim_matches('"').to_owned())
                .filter(|l| !l.is_empty())
                .collect();
            Some((name.to_owned(), licenses))
        })
        .collect()
}

/// Whether `license` is on `allowed`. A family name admits only its
/// named permissive variants, never a bare family or any other id
/// sharing its prefix: "BSD" admits BSD-2-Clause and BSD-3-Clause, not
/// BSD-4-Clause or BSD-Protection. An SPDX `OR` expression, such as
/// "MIT OR Apache-2.0", offers a choice, so one allowed choice admits
/// it. An `AND` expression, such as "(MIT OR Apache-2.0) AND NCSA",
/// binds the crate to every part, so each must be allowed.
#[must_use]
pub fn license_allowed(license: &str, allowed: &[String]) -> bool {
    license.split(" AND ").all(|part| {
        part.trim_matches(['(', ')']).split(" OR ").any(|choice| {
            allowed.iter().any(|a| match a.as_str() {
                "BSD" => ["BSD-2-Clause", "BSD-3-Clause"].contains(&choice),
                a => choice == a,
            })
        })
    })
}

/// The licenses ENG-18's text allows: the list in `allow-list (...)`.
///
/// # Errors
///
/// Fails when the text states no allow-list.
pub fn allow_list(eng18: &str) -> Result<Vec<String>, String> {
    let (_, rest) = eng18
        .split_once("allow-list (")
        .ok_or("ENG-18 does not state its license allow-list")?;
    let (list, _) = rest
        .split_once(')')
        .ok_or("ENG-18 does not state its license allow-list")?;

    Ok(list.split(", ").map(str::to_owned).collect())
}

/// The most direct dependencies linked into a shipped executable that
/// ENG-18's text allows: the number in `≤ N direct dependencies linked
/// into a shipped executable`.
///
/// # Errors
///
/// Fails when the text states no target.
pub fn dependency_target(eng18: &str) -> Result<usize, String> {
    eng18
        .split("≤ ")
        .skip(1)
        .find_map(|rest| {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            let stated = rest[digits.len()..]
                .starts_with(" direct dependencies linked into a shipped executable");
            stated.then(|| digits.parse().ok()).flatten()
        })
        .ok_or_else(|| "ENG-18 does not state its dependency target".to_owned())
}

/// Checks the direct dependencies linked into a shipped executable
/// against the target.
///
/// # Errors
///
/// Fails when there are more than `target`.
pub fn within_target(shipped: &[String], target: usize) -> Result<(), String> {
    if shipped.len() > target {
        return Err(format!(
            "{} direct dependencies linked into a shipped executable, want at most {target}: {}",
            shipped.len(),
            list(shipped)
        ));
    }

    Ok(())
}

/// Checks `body`, the text of `file`, links every record in `adr_dir`
/// that names a module, the way its catalog renders them.
#[must_use]
pub fn lists_dependency_adrs(body: &str, file: &str, adr_dir: &str, adrs: &[Adr]) -> Vec<String> {
    adrs.iter()
        .filter(|a| !a.modules.is_empty())
        .filter_map(|a| {
            let name = a
                .path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            let link = format!("{adr_dir}/{name}");
            (!body.contains(&format!("({link})"))).then(|| format!("{file} does not link {link}"))
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod tests;
