//! ENG-31: the `THIRD_PARTY_LICENSES` file of each release target. The
//! tool writes it from `cargo metadata --filter-platform <target>` and
//! the crates' fetched sources, and a short fixed list of the code a
//! target compiles in without being a crate. The same lockfile gives the
//! same bytes: the order is sorted, and the text holds no timestamp, host
//! path or randomness. The core executable embeds its own target's file
//! and prints it with `parinn licenses`.

pub mod checks;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::license_texts::{self, Package};
use crate::{Checkout, architecture};

/// The directory holding the committed files, one directory per release
/// target.
pub const DIRECTORY: &str = "licenses";

/// The name of each target's file.
pub const FILE: &str = "THIRD_PARTY_LICENSES";

/// The committed file of `target`, relative to the checkout.
#[must_use]
pub fn committed_path(target: &str) -> String {
    format!("{DIRECTORY}/{target}/{FILE}")
}

/// The blessing SQLite carries in place of a copyright notice.
const SQLITE: &str = "The author disclaims copyright to this source code.  In place of\na legal notice, here is a blessing:\n\n   * May you do good and not evil.\n   * May you find forgiveness for yourself and forgive others.\n   * May you share freely, never taking more than you give.";

/// The notices the mingw-w64 runtime requires a binary that links it to
/// carry, as its project publishes them in `COPYING.MinGW-w64-runtime.txt`
/// (the master branch of its mirror). Each notice states its own
/// copyright line, so the text is kept as published.
const MINGW_W64: &str = include_str!("third_party/runtime/mingw-w64.txt");

/// The GNU General Public License, version 3, which the GCC runtime
/// libraries carry with the exception below.
const GPL_3: &str = include_str!("third_party/runtime/GPL-3.0.txt");

/// The GCC Runtime Library Exception, version 3.1, as the GCC project
/// publishes it in `COPYING.RUNTIME`.
const GCC_EXCEPTION: &str = include_str!("third_party/runtime/gcc-exception.txt");

/// One piece of third-party code in a file: its name, the license it
/// declares, the copyright lines its texts state and those texts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub license: String,
    pub copyrights: Vec<String>,
    /// The texts, as (label, text).
    pub texts: Vec<(String, String)>,
    /// Whether the texts are kept as published, their copyright lines
    /// included, because they state notices the license requires.
    pub verbatim: bool,
}

/// The code `target` compiles in without being a crate: SQLite, the Rust
/// standard library and, on a musl target, the C library.
#[must_use]
pub fn non_crate(target: &str) -> Vec<Entry> {
    let canon = |id: &str| {
        (
            id.to_owned(),
            license_texts::canonical(id).unwrap_or_default().to_owned(),
        )
    };
    let mut entries = vec![
        Entry {
            name: "SQLite".to_owned(),
            license: "public domain".to_owned(),
            copyrights: Vec::new(),
            texts: vec![("SQLite".to_owned(), SQLITE.to_owned())],
            verbatim: false,
        },
        Entry {
            name: "Rust standard library".to_owned(),
            license: "MIT OR Apache-2.0".to_owned(),
            copyrights: vec!["Copyright (c) The Rust Project Developers".to_owned()],
            texts: vec![canon("MIT"), canon("Apache-2.0")],
            verbatim: false,
        },
    ];
    if target.ends_with("-windows-gnu") {
        entries.extend(mingw_runtime());
    }
    if target.ends_with("-linux-musl") {
        entries.push(Entry {
            name: "musl C library".to_owned(),
            license: "MIT".to_owned(),
            copyrights: vec!["Copyright © 2005-2020 Rich Felker, et al.".to_owned()],
            texts: vec![canon("MIT")],
            verbatim: false,
        });
    }

    entries
}

/// The runtime the Windows target links statically through mingw-w64's
/// GCC: the mingw-w64 start-up code and libraries, and libgcc.
fn mingw_runtime() -> [Entry; 2] {
    [
        Entry {
            name: "mingw-w64 runtime (crt, libmingw32, libmingwex)".to_owned(),
            license: "ZPL-2.1 and public domain, per file".to_owned(),
            copyrights: vec![
                "Copyright (c) 2009, 2010, 2011, 2012, 2013 by the mingw-w64 project".to_owned(),
            ],
            texts: vec![("mingw-w64 runtime".to_owned(), MINGW_W64.to_owned())],
            verbatim: true,
        },
        Entry {
            name: "libgcc and libgcc_eh".to_owned(),
            license: "GPL-3.0-or-later WITH GCC-exception-3.1".to_owned(),
            copyrights: vec!["Copyright (C) Free Software Foundation, Inc.".to_owned()],
            texts: vec![
                ("GPL-3.0".to_owned(), GPL_3.to_owned()),
                (
                    "GCC Runtime Library Exception".to_owned(),
                    GCC_EXCEPTION.to_owned(),
                ),
            ],
            verbatim: true,
        },
    ]
}

/// A crate as an [`Entry`]: its declared license and the copyright lines
/// and texts of its license files.
#[must_use]
pub fn crate_entry(package: &Package, files: &[(String, String)]) -> Entry {
    let mut copyrights: Vec<String> = Vec::new();
    for line in files.iter().flat_map(|(_, text)| text.lines()) {
        let line = line.trim();
        if license_texts::copyright_line(line) && !copyrights.iter().any(|c| c == line) {
            copyrights.push(line.to_owned());
        }
    }

    Entry {
        name: format!("{} {}", package.name, package.version),
        license: package.license.clone().unwrap_or_default(),
        copyrights,
        texts: files
            .iter()
            .map(|(name, text)| (label(name, text), text.clone()))
            .collect(),
        verbatim: false,
    }
}

/// What a license text is called: the id of the canonical text it fits,
/// or the name of its file.
fn label(file: &str, text: &str) -> String {
    license_texts::ids()
        .into_iter()
        .find(|id| {
            license_texts::canonical(id).is_some_and(|canon| license_texts::fits(text, canon))
        })
        .map_or_else(|| file.to_owned(), str::to_owned)
}

/// A text without its copyright lines, with `\n` line ends, no run of
/// blank lines and none at either end.
fn body(text: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for line in text.lines().filter(|l| !license_texts::copyright_line(l)) {
        let line = line.trim_end();
        // The copyright lines leave gaps; one blank line is enough.
        if !(line.is_empty() && kept.last().is_some_and(|l| l.is_empty())) {
            kept.push(line);
        }
    }

    kept.join("\n").trim_matches('\n').to_owned()
}

/// A text as published: `\n` line ends, no trailing space, no blank line at
/// either end, its copyright lines kept.
fn published(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim_matches('\n')
        .to_owned()
}

/// The file for the code in `entries`: one section per distinct license text, naming
/// the code that uses it with the copyright lines it states, then the
/// text once. Sections sort by label and the code in them by name.
#[must_use]
pub fn render(entries: &[Entry]) -> String {
    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    // (label, normalized text) -> (text, the entries using it)
    let mut sections: BTreeMap<(String, String), (String, BTreeSet<&str>)> = BTreeMap::new();
    for entry in &sorted {
        for (label, text) in &entry.texts {
            let (key, shown) = if entry.verbatim {
                (published(text), published(text))
            } else {
                (license_texts::normalize(text), body(text))
            };
            let section = sections
                .entry((label.clone(), key))
                .or_insert_with(|| (shown, BTreeSet::new()));
            section.1.insert(&entry.name);
        }
    }

    let mut out = "Third-party licenses of ParleyInn\n\nThis executable includes the code below. Each section gives a license text\nonce, then the code that uses it with the copyright lines it states.\n"
        .to_owned();
    for ((label, _), (text, users)) in &sections {
        out.push_str(&format!("\n{}\nLicense: {label}\n\n", "=".repeat(78)));
        for user in users {
            let entry = sorted.iter().find(|e| e.name == *user);
            out.push_str(&format!(
                "{user} ({})\n",
                entry.map_or("", |e| e.license.as_str())
            ));
            for line in entry.iter().flat_map(|e| &e.copyrights) {
                out.push_str(&format!("    {line}\n"));
            }
        }
        out.push_str(&format!("\n{}\n{text}\n", "-".repeat(78)));
    }

    out
}

/// The file of `target`, generated from its `metadata`, reading each
/// linked crate's license files through `read`.
///
/// # Errors
///
/// Fails when the metadata cannot be read, or a crate's directory can't.
pub fn generate(
    target: &str,
    linked: &[Package],
    read: impl Fn(&Path) -> Result<Vec<(String, String)>, String>,
) -> Result<String, String> {
    let mut entries = non_crate(target);
    for package in linked {
        entries.push(crate_entry(package, &read(&package.dir)?));
    }

    Ok(render(&entries))
}

/// Problems with the committed files against `generated`, the file of
/// each release target: a target with none, a file that differs, and a
/// directory for no target.
#[must_use]
pub fn committed_problems(
    checkout: &Checkout,
    generated: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for (target, want) in generated {
        match checkout.read(&committed_path(target)) {
            Err(_) => problems.push(format!(
                "{target} has no committed {}; run license-check --write",
                committed_path(target)
            )),
            Ok(have) if &have != want => problems.push(format!(
                "{} differs from its regeneration; run license-check --write",
                committed_path(target)
            )),
            Ok(_) => {}
        }
    }
    if let Ok(entries) = fs::read_dir(checkout.root.join(DIRECTORY)) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !generated.contains_key(&name) {
                problems.push(format!(
                    "{DIRECTORY}/{name} is no release target's directory"
                ));
            }
        }
    }
    problems.sort();

    problems
}

/// Writes each target's file under `root`, creating its directory.
///
/// # Errors
///
/// Fails when a directory or file cannot be written.
pub fn write(root: &Path, generated: &BTreeMap<String, String>) -> Result<(), String> {
    for (target, text) in generated {
        let path = root.join(committed_path(target));
        let dir = path.parent().unwrap_or(root);
        fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
        fs::write(&path, text).map_err(|e| format!("write {}: {e}", path.display()))?;
    }

    Ok(())
}

/// What `license-check` is asked to do beyond checking.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Options {
    /// Rewrite the committed files from the regeneration.
    pub write: bool,
    /// Also write the regeneration under this directory, in the layout
    /// of the committed files, leaving the committed files alone.
    pub out: Option<PathBuf>,
}

/// Checks the repository at `root` for ENG-30 and ENG-31, and acts on
/// `options`. It runs `cargo metadata` once per release target the build
/// workflow names, with `PARINN_HOME` pointed at an empty directory of
/// its own (ENG-14): the crates a release links must carry a fitting
/// license text, and each target's committed file must equal its
/// regeneration.
///
/// # Errors
///
/// Fails when the build workflow, cargo, a crate's directory or a file
/// to write cannot be used.
pub fn run_release(cargo: &OsStr, root: &Path, options: &Options) -> Result<Vec<String>, String> {
    let checkout = Checkout::new(root);
    let targets = license_texts::release_targets(&checkout.read(license_texts::BUILD_WORKFLOW)?)?;
    let home = std::env::temp_dir().join(format!("license-check-{}", std::process::id()));
    fs::create_dir_all(&home).map_err(|e| format!("create {}: {e}", home.display()))?;
    let per_target = linked_per_target(&targets, |target| {
        license_texts::cargo_metadata_for(cargo, &root.join("Cargo.toml"), &home, target)
    });
    let _ = fs::remove_dir_all(&home);
    let per_target = per_target?;

    let all: BTreeMap<(String, String), Package> = per_target
        .values()
        .flatten()
        .map(|p| ((p.name.clone(), p.version.clone()), p.clone()))
        .collect();
    let mut problems =
        license_texts::check_repository(&checkout, &all.into_values().collect::<Vec<_>>());

    let mut generated = BTreeMap::new();
    for (target, linked) in &per_target {
        generated.insert(
            target.clone(),
            generate(target, linked, license_texts::license_files)?,
        );
    }
    if options.write {
        write(root, &generated)?;
    } else {
        problems.extend(committed_problems(&checkout, &generated));
    }
    if let Some(out) = &options.out {
        write(out, &generated)?;
    }

    Ok(problems)
}

/// The crates linked into a shipped executable for each of `targets`,
/// from the `metadata` of each.
///
/// # Errors
///
/// Fails when a target's metadata cannot be read.
pub fn linked_per_target(
    targets: &[String],
    metadata: impl Fn(&str) -> Result<String, String>,
) -> Result<BTreeMap<String, Vec<Package>>, String> {
    let mut all = BTreeMap::new();
    for target in targets {
        all.insert(
            target.clone(),
            license_texts::linked(&metadata(target)?, Path::new(architecture::PRODUCT))?,
        );
    }

    Ok(all)
}

#[cfg(test)]
mod tests;
