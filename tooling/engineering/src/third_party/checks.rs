//! The ENG-31 checks the scenario binds, each over text it is handed so
//! a test can give it a file that is wrong: what a rendered file holds,
//! what it leaves out, which committed files must differ, where the core
//! selects its file and what the workflows run.

use std::collections::BTreeMap;
use std::path::Path;

use super::{committed_path, crate_entry};
use crate::license_texts::{self, Package};

/// The release target a host builds for, as the core embeds its file:
/// the release target on the host's operating system and architecture.
#[must_use]
pub fn host_target(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("linux", "x86_64") => Some("x86_64-unknown-linux-musl"),
        ("linux", "aarch64") => Some("aarch64-unknown-linux-musl"),
        ("windows", _) => Some("x86_64-pc-windows-gnu"),
        ("macos", _) => Some("aarch64-apple-darwin"),
        _ => None,
    }
}

/// Problems with a file that does not hold what `linked` links: a crate
/// missing, a crate's copyright line missing, a crate's license text
/// missing. Each crate's files are read through `read`.
#[must_use]
pub fn holds_every_crate(
    rendered: &str,
    linked: &[Package],
    read: impl Fn(&Path) -> Result<Vec<(String, String)>, String>,
) -> Vec<String> {
    let text = license_texts::normalize(rendered);
    let mut problems = Vec::new();
    for package in linked {
        let who = format!("{} {}", package.name, package.version);
        let files = match read(&package.dir) {
            Ok(files) => files,
            Err(e) => {
                problems.push(e);
                continue;
            }
        };
        if !rendered.contains(&format!("{who} (")) {
            problems.push(format!("{who} is missing"));
        }
        for line in crate_entry(package, &files).copyrights {
            if !rendered.contains(&line) {
                problems.push(format!("{who}: the copyright line {line:?} is missing"));
            }
        }
        for (name, body) in &files {
            if !text.contains(&license_texts::normalize(body)) {
                problems.push(format!("{who}: the text of {name} is missing"));
            }
        }
    }

    problems
}

/// Problems with a file that names a crate `metadata` lists that the
/// target does not link: a build-only, proc-macro, test, fuzz or tooling
/// crate.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON.
pub fn unlinked_present(
    rendered: &str,
    metadata: &str,
    linked: &[Package],
) -> Result<Vec<String>, String> {
    Ok(license_texts::packages(metadata)?
        .into_iter()
        .filter(|p| {
            !linked
                .iter()
                .any(|l| (&l.name, &l.version) == (&p.name, &p.version))
        })
        .filter(|p| rendered.contains(&format!("{} {} (", p.name, p.version)))
        .map(|p| format!("{} {} is not linked into the executable", p.name, p.version))
        .collect())
}

/// What a target's file must name beyond its crates, fixed here and
/// not taken from the generator's own list: SQLite and the Rust standard
/// library everywhere, the C library on a musl target, and on the Windows
/// target the mingw-w64 runtime and libgcc its GCC links statically.
fn required_names(target: &str) -> Vec<&'static str> {
    let mut names = vec!["SQLite", "Rust standard library"];
    if target.ends_with("-linux-musl") {
        names.push("musl C library");
    }
    if target.ends_with("-windows-gnu") {
        names.extend(["mingw-w64 runtime", "libgcc"]);
    }

    names
}

/// Problems with a file that does not hold, whole, the text of the code
/// its target compiles in without being a crate: for a text kept as
/// published, every line of it; for any other, its words, whatever the
/// copyright lines.
#[must_use]
pub fn holds_compiled_in_texts(target: &str, rendered: &str) -> Vec<String> {
    let lines: Vec<&str> = rendered.lines().map(str::trim_end).collect();
    let words = license_texts::normalize(rendered);
    let mut problems = Vec::new();
    for entry in super::non_crate(target) {
        for (label, text) in &entry.texts {
            let whole = if entry.verbatim {
                in_order(text, &lines)
            } else {
                words.contains(&license_texts::normalize(text))
            };
            if !whole {
                problems.push(format!("{target}: the text of {label} is incomplete"));
            }
        }
    }

    problems
}

/// Whether every non-empty line of `text` stands in `lines`, in the order
/// the text gives them.
fn in_order(text: &str, lines: &[&str]) -> bool {
    let mut from = 0;
    text.lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty())
        .all(|wanted| {
            lines[from..]
                .iter()
                .position(|l| *l == wanted)
                .is_some_and(|at| {
                    from += at + 1;
                    true
                })
        })
}

/// Problems with a file that does not name the code its target compiles
/// in without being a crate.
#[must_use]
pub fn names_compiled_in(target: &str, rendered: &str) -> Vec<String> {
    required_names(target)
        .into_iter()
        .filter(|name| !rendered.contains(name))
        .map(|name| format!("{target}: {name} is not named"))
        .collect()
}

/// Problems with the committed files of `files`, by target: two targets
/// whose linked crates differ must differ in bytes, and two that link the
/// same crates must be identical.
#[must_use]
pub fn files_follow_crates(
    linked: &BTreeMap<String, Vec<Package>>,
    files: &BTreeMap<String, String>,
) -> Vec<String> {
    let key = |t: &str| -> Vec<(String, String)> {
        linked[t]
            .iter()
            .map(|p| (p.name.clone(), p.version.clone()))
            .collect()
    };
    let targets: Vec<&String> = linked.keys().collect();
    let mut problems = Vec::new();
    for (i, a) in targets.iter().enumerate() {
        for b in &targets[i + 1..] {
            let same_crates = key(a) == key(b);
            let same_bytes = files.get(*a) == files.get(*b);
            if same_crates != same_bytes {
                problems.push(format!(
                    "{a} and {b} link {} crates but their files {} in bytes",
                    if same_crates { "the same" } else { "different" },
                    if same_bytes {
                        "do not differ"
                    } else {
                        "differ"
                    },
                ));
            }
        }
    }

    problems
}

/// The operating system and architecture a release target's `cfg` names.
fn target_cfg(target: &str) -> (&'static str, &str) {
    let arch = target.split('-').next().unwrap_or_default();
    let os = if target.contains("linux") {
        "linux"
    } else if target.contains("windows") {
        "windows"
    } else {
        "macos"
    };

    (os, arch)
}

/// Problems with the core's source that embeds the files: a release
/// target whose file no `include_str!` names, whose `include_str!` sits
/// under a `#[cfg]` for another operating system or architecture (two
/// arms with their paths swapped), or whose file the test that compares
/// the embedded text does not name.
#[must_use]
pub fn embedding_selects(source: &str, targets: &[String]) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut problems = Vec::new();
    for target in targets {
        let path = committed_path(target);
        for (what, marker, missing) in [
            ("selects", "include_str!", "does not select"),
            (
                "names",
                "EMBEDDED_FILE",
                "does not name for the test that compares it",
            ),
        ] {
            let found: Vec<usize> = (0..lines.len())
                .filter(|i| lines[*i].contains(&path))
                .filter(|i| {
                    lines[*i].contains(marker) || (*i > 0 && lines[*i - 1].contains(marker))
                })
                .collect();
            if found.is_empty() {
                problems.push(format!("the core {missing} {path}"));
            }
            for at in found {
                problems.extend(wrong_cfg(&lines, at, target, what, &path));
            }
        }
    }

    problems
}

/// The problem with the `#[cfg]` above line `at`, which `what` `path`:
/// it names another operating system or architecture than `target`'s.
fn wrong_cfg(lines: &[&str], at: usize, target: &str, what: &str, path: &str) -> Option<String> {
    let (os, arch) = target_cfg(target);
    let cfg = lines[..at]
        .iter()
        .rev()
        .find(|l| l.trim_start().starts_with("#[cfg("))
        .copied()
        .unwrap_or_default();
    let arch_named: Vec<&str> = cfg.split("target_arch = \"").skip(1).collect();
    let wrong_arch = arch_named
        .iter()
        .any(|rest| !rest.starts_with(&format!("{arch}\"")));
    let missing_arch = os == "linux" && arch_named.is_empty();
    let wrong = !cfg.contains(&format!("target_os = \"{os}\"")) || wrong_arch || missing_arch;

    wrong.then(|| format!("the core {what} {path} under {}", cfg.trim()))
}

/// Problems with the executables the repository ships: any binary but
/// the core, which would need the same texts for its crates (OQ-32).
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON.
pub fn other_executables(metadata: &str) -> Result<Vec<String>, String> {
    let value: serde_json::Value =
        serde_json::from_str(metadata).map_err(|e| format!("read cargo metadata: {e}"))?;
    let root = value["workspace_root"].as_str().unwrap_or_default();
    let product = Path::new(root).join(crate::architecture::PRODUCT);

    Ok(value["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| {
            p["manifest_path"]
                .as_str()
                .is_some_and(|m| Path::new(m).starts_with(&product))
        })
        .flat_map(|p| p["targets"].as_array().into_iter().flatten())
        .filter(|t| {
            t["kind"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|k| k == "bin")
        })
        .filter_map(|t| t["name"].as_str())
        .filter(|name| *name != "parinn")
        .map(|name| format!("the executable {name} ships without the texts of its crates"))
        .collect())
}

/// Problems with the build workflow: the release build must regenerate
/// the files and compare each platform's executable with them.
#[must_use]
pub fn release_compares_output(build_workflow: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let regenerates = build_workflow.lines().any(|l| {
        l.contains("--bin license-check") && l.contains("--out") && l.contains("--locked")
    });
    if !regenerates {
        problems.push(
            "the build workflow does not regenerate the license files with license-check --out"
                .to_owned(),
        );
    }
    // The Linux and Darwin build, the linux/arm64 job and the Windows job
    // each run an executable on its own platform.
    if build_workflow.matches("scripts/check-licenses.sh").count() < 3 {
        problems.push(
            "the build workflow does not compare every platform's licenses output".to_owned(),
        );
    }

    problems
}

#[cfg(test)]
mod tests;
