//! The ENG-31 steps: the core executable carries the license texts of
//! what it links for its own target, and CI keeps them in step.

use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

use cucumber::{then, when};
use engineering::license_texts::{self, Package};
use engineering::third_party::{self, checks};
use engineering::{dependencies, outcome};

use crate::World;
use crate::engineering::eng;

/// What the scenario read and regenerated: the release targets, the
/// metadata of each, the crates it links and the file regenerated twice.
#[derive(Debug, Default)]
struct ThirdParty {
    targets: Vec<String>,
    metadata: BTreeMap<String, String>,
    linked: BTreeMap<String, Vec<Package>>,
    first: BTreeMap<String, String>,
    second: BTreeMap<String, String>,
}

fn state(w: &mut World) -> &mut ThirdParty {
    w.section::<ThirdParty>()
}

#[when(regex = r"^the release targets are read from the build workflow$")]
fn the_release_targets_are_read(w: &mut World) -> Result<(), String> {
    let build = eng(w).checkout()?.read(license_texts::BUILD_WORKFLOW)?;
    state(w).targets = license_texts::release_targets(&build)?;
    Ok(())
}

#[then(
    regex = r#"^each target has a committed "licenses/<target-triple>/THIRD_PARTY_LICENSES" and no other target has one$"#
)]
fn each_target_has_a_committed_file(w: &mut World) -> Result<(), String> {
    let checkout = eng(w).checkout()?.clone();
    let targets = state(w).targets.clone();
    let mut problems = Vec::new();
    for target in &targets {
        if checkout.read(&third_party::committed_path(target)).is_err() {
            problems.push(format!("{target} has no committed file"));
        }
    }
    let directory = checkout.root.join(third_party::DIRECTORY);
    for entry in fs::read_dir(&directory)
        .map_err(|e| format!("read licenses: {e}"))?
        .flatten()
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !targets.contains(&name) {
            problems.push(format!("licenses/{name} is no release target's directory"));
        }
    }

    outcome(problems)
}

/// Regenerates every target's file from its `cargo metadata`.
fn regenerate(w: &mut World) -> Result<BTreeMap<String, String>, String> {
    let root = eng(w).checkout()?.root.clone();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let (home, targets) = (w.parinn_home.clone(), state(w).targets.clone());
    let mut metadata = BTreeMap::new();
    for target in &targets {
        metadata.insert(
            target.clone(),
            license_texts::cargo_metadata_for(&cargo, &root.join("Cargo.toml"), &home, target)?,
        );
    }
    let linked = third_party::linked_per_target(&targets, |t| Ok(metadata[t].clone()))?;
    let mut files = BTreeMap::new();
    for (target, packages) in &linked {
        files.insert(
            target.clone(),
            third_party::generate(target, packages, license_texts::license_files)?,
        );
    }
    let s = state(w);
    (s.metadata, s.linked) = (metadata, linked);

    Ok(files)
}

#[when(
    regex = r#"^each target's file is regenerated twice with "cargo metadata --filter-platform <target-triple>"$"#
)]
fn each_target_is_regenerated_twice(w: &mut World) -> Result<(), String> {
    let first = regenerate(w)?;
    let second = regenerate(w)?;
    let s = state(w);
    (s.first, s.second) = (first, second);
    Ok(())
}

#[then(
    regex = r"^both regenerations give the same bytes, with no timestamp, host path or randomness$"
)]
fn both_regenerations_give_the_same_bytes(w: &mut World) -> Result<(), String> {
    let root = eng(w).checkout()?.root.to_string_lossy().into_owned();
    let s = state(w);
    let mut problems = Vec::new();
    if s.first != s.second {
        problems.push("two regenerations differ".to_owned());
    }
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_default();
    for (target, text) in &s.first {
        for (what, found) in [
            ("the checkout's path", text.contains(&root)),
            ("a registry path", text.contains("registry/src")),
            (
                "CARGO_HOME",
                !cargo_home.is_empty() && text.contains(&cargo_home),
            ),
            ("a timestamp", has_timestamp(text)),
        ] {
            if found {
                problems.push(format!("{target} holds {what}"));
            }
        }
    }

    outcome(problems)
}

/// Whether `text` holds an ISO date and time, such as `2026-10-23T08:00`.
fn has_timestamp(text: &str) -> bool {
    text.as_bytes().windows(16).any(|w| {
        let digit = |i: usize| w[i].is_ascii_digit();
        [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15]
            .iter()
            .all(|i| digit(*i))
            && w[4] == b'-'
            && w[7] == b'-'
            && w[10] == b'T'
            && w[13] == b':'
    })
}

#[then(regex = r"^the committed file equals the regeneration$")]
fn the_committed_file_equals_the_regeneration(w: &mut World) -> Result<(), String> {
    let checkout = eng(w).checkout()?.clone();

    outcome(third_party::committed_problems(&checkout, &state(w).first))
}

#[then(
    regex = r#"^it holds the license text and copyright lines of every crate a normal-dependency path from a binary under "crates/" reaches for that target$"#
)]
fn it_holds_every_linked_crate(w: &mut World) -> Result<(), String> {
    let s = state(w);
    let mut problems = Vec::new();
    for (target, text) in &s.first {
        for p in checks::holds_every_crate(text, &s.linked[target], license_texts::license_files) {
            problems.push(format!("{target}: {p}"));
        }
        problems.extend(checks::holds_compiled_in_texts(target, text));
    }

    outcome(problems)
}

#[then(regex = r"^no build-only, proc-macro, test, fuzz or tooling crate appears in it$")]
fn no_unlinked_crate_appears(w: &mut World) -> Result<(), String> {
    let s = state(w);
    let mut problems = Vec::new();
    for (target, text) in &s.first {
        for p in checks::unlinked_present(text, &s.metadata[target], &s.linked[target])? {
            problems.push(format!("{target}: {p}"));
        }
    }

    outcome(problems)
}

#[then(
    regex = r"^it names SQLite, the Rust standard library, on the musl targets the C library, and any other runtime the target links statically$"
)]
fn it_names_what_is_compiled_in(w: &mut World) -> Result<(), String> {
    let s = state(w);

    outcome(
        s.first
            .iter()
            .flat_map(|(target, text)| checks::names_compiled_in(target, text))
            .collect(),
    )
}

#[then(
    regex = r"^the committed files of any two targets whose linked crates differ differ in bytes$"
)]
fn committed_files_differ_with_the_crates(w: &mut World) -> Result<(), String> {
    let checkout = eng(w).checkout()?.clone();
    let s = state(w);
    let mut files = BTreeMap::new();
    for target in &s.targets {
        files.insert(
            target.clone(),
            checkout.read(&third_party::committed_path(target))?,
        );
    }

    outcome(checks::files_follow_crates(&s.linked, &files))
}

#[when(regex = r"^each target's build selects its embedded file$")]
fn each_build_selects_its_file(w: &mut World) -> Result<(), String> {
    eng(w)
        .checkout()?
        .read("crates/parinn/src/licenses.rs")
        .map(|_| ())
}

#[then(regex = r"^the embedded text equals that target's committed file$")]
fn the_embedded_text_equals_the_committed_file(w: &mut World) -> Result<(), String> {
    let source = eng(w).checkout()?.read("crates/parinn/src/licenses.rs")?;

    outcome(checks::embedding_selects(&source, &state(w).targets))
}

#[then(regex = r#"^the built core executable's "parinn licenses" output equals it$"#)]
fn the_built_core_prints_it(w: &mut World) -> Result<(), String> {
    let target = checks::host_target(std::env::consts::OS, std::env::consts::ARCH)
        .ok_or("no release target embeds a file for this host")?;
    let want = eng(w)
        .checkout()?
        .read(&third_party::committed_path(target))?;
    let core = testkit::core_executable()?;
    let home = testkit::TempDir::new().map_err(|e| e.to_string())?;
    let mut command = Command::new(core);
    testkit::isolate(command.arg("licenses"), home.path());
    let out = command
        .output()
        .map_err(|e| format!("run parinn licenses: {e}"))?;

    if out.stdout == want.as_bytes() && out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "parinn licenses does not print {}",
            third_party::committed_path(target)
        ))
    }
}

#[then(
    regex = r"^any other executable a component ships in carries the same for the crates linked into it$"
)]
fn no_other_executable_ships(w: &mut World) -> Result<(), String> {
    let root = eng(w).checkout()?.root.clone();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let metadata = dependencies::cargo_metadata(&cargo, &root.join("Cargo.toml"), &w.parinn_home)?;

    outcome(checks::other_executables(&metadata)?)
}

#[then(
    regex = r"^the CI workflow fails a pull request whose committed file differs from a regeneration$"
)]
fn ci_fails_a_differing_file(w: &mut World) -> Result<(), String> {
    let ci = eng(w).checkout()?.read(".github/workflows/ci.yml")?;

    outcome(license_texts::ci_runs_check(&ci))
}

#[then(
    regex = r"^the release build fails when a built executable's output differs from its target's regenerated file$"
)]
fn the_release_compares_the_output(w: &mut World) -> Result<(), String> {
    let build = eng(w).checkout()?.read(license_texts::BUILD_WORKFLOW)?;

    outcome(checks::release_compares_output(&build))
}

#[then(regex = r"^the core gains no dependency and opens no socket$")]
fn the_core_gains_nothing(w: &mut World) -> Result<(), String> {
    let root = eng(w).checkout()?.root.clone();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let metadata = dependencies::cargo_metadata(&cargo, &root.join("Cargo.toml"), &w.parinn_home)?;
    let mut problems = license_texts::product_links_check(&metadata)?;
    let source = eng(w).checkout()?.read("crates/parinn/src/licenses.rs")?;
    for banned in ["std::net", "std::process", "std::fs"] {
        if source.contains(banned) {
            problems.push(format!("crates/parinn/src/licenses.rs uses {banned}"));
        }
    }

    outcome(problems)
}
