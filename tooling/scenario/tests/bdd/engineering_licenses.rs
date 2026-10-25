//! The ENG-30 steps: every crate a release links carries a license text
//! that fits its declared license, and CI runs the check.

use cucumber::{then, when};
use engineering::license_texts::{self, Package};
use engineering::{dependencies, outcome};

use crate::World;
use crate::engineering::eng;

/// The crates the scenario read: those linked into a shipped executable
/// for a release target.
#[derive(Debug, Default)]
struct Linked {
    packages: Vec<Package>,
}

#[when(
    regex = r#"^every crate linked into a shipped executable for a release target is read through "cargo metadata" and the sources cargo has fetched$"#
)]
fn every_linked_crate_is_read(w: &mut World) -> Result<(), String> {
    let checkout = eng(w).checkout()?.clone();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let targets = license_texts::release_targets(&checkout.read(license_texts::BUILD_WORKFLOW)?)?;
    let home = w.parinn_home.clone();
    let packages = license_texts::linked_for_release(&targets, |target| {
        license_texts::cargo_metadata_for(&cargo, &checkout.root.join("Cargo.toml"), &home, target)
    })?;
    w.section::<Linked>().packages = packages;
    Ok(())
}

/// Problems with the crates read, from their license files.
fn problems(w: &mut World) -> Vec<String> {
    let packages = w.section::<Linked>().packages.clone();

    license_texts::check_packages(&packages, license_texts::license_files)
}

#[then(regex = r"^each crate has a license file in its source$")]
fn each_crate_has_a_license_file(w: &mut World) -> Result<(), String> {
    outcome(license_texts::split_problems(problems(w)).0)
}

#[then(regex = r"^the repository keeps no license text for any crate by name and version$")]
fn the_repository_keeps_no_license_text(w: &mut World) -> Result<(), String> {
    let packages = w.section::<Linked>().packages.clone();

    outcome(license_texts::kept_texts(eng(w).checkout()?, &packages))
}

#[then(
    regex = r"^each license file's text fits the canonical text of a license in the crate's declared expression, from the allow-list in .+$"
)]
fn each_license_file_fits(w: &mut World) -> Result<(), String> {
    let mut found = license_texts::split_problems(problems(w)).1;
    let deny = eng(w).checkout()?.read("deny.toml")?;
    found.extend(license_texts::canon_covers(&deny));

    outcome(found)
}

#[then(
    regex = r"^the CI workflow runs the check on every pull request, with the locked dependency tree$"
)]
fn ci_runs_the_check(w: &mut World) -> Result<(), String> {
    let ci = eng(w).checkout()?.read(".github/workflows/ci.yml")?;

    outcome(license_texts::ci_runs_check(&ci))
}

#[then(regex = r"^the check adds no dependency to any shipped executable$")]
fn the_check_adds_no_dependency(w: &mut World) -> Result<(), String> {
    let root = eng(w).checkout()?.root.clone();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let metadata = dependencies::cargo_metadata(&cargo, &root.join("Cargo.toml"), &w.parinn_home)?;

    outcome(license_texts::product_links_check(&metadata)?)
}
