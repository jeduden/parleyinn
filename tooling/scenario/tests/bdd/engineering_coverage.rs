//! The ENG-11 steps. They read the floors the coverage profile is held
//! to, not the profile itself: `[workspace.metadata.coverage]` through
//! `cargo metadata`, and the CI job that runs `coverage --merge`. That
//! job measures the merged profile of every test layer and fails a crate
//! below its floor, so a floor at or above ENG-11's holds the crate to
//! ENG-11 whenever CI passes. Every security-sensitive crate the
//! workspace builds must be held to at least ENG-11's floor, and every
//! crate to the overall one. The security-sensitive crates the workspace
//! does not build yet (`flag`, `pin`, `landmark` and `restore_block` in
//! M1) are skipped, and named when the step fails.

use std::env;
use std::ffi::OsString;

use coverage::plan::Plan;
use cucumber::{given, then};
use engineering::coverage_floors::{self, Stated};
use engineering::{Checkout, dependencies, outcome, review_workflow};

use crate::World;

/// The profile's floors and what ENG-11 states they are held to.
#[derive(Debug, Default)]
struct Profile {
    taken: Option<(Plan, Stated)>,
}

fn taken(w: &mut World) -> Result<&(Plan, Stated), String> {
    w.section::<Profile>()
        .taken
        .as_ref()
        .ok_or_else(|| "no step took a coverage profile".to_owned())
}

#[given(regex = r"^a coverage profile of the whole suite$")]
fn a_coverage_profile(w: &mut World) -> Result<(), String> {
    let code = Checkout::new(&testkit::repo_root());
    outcome(coverage_floors::gated(&review_workflow::parse(
        &code.read(".github/workflows/ci.yml")?,
    )))?;
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let metadata =
        dependencies::cargo_metadata(&cargo, &code.root.join("Cargo.toml"), &w.parinn_home)?;
    let plan = Plan::from_metadata(&metadata)?;
    let stated = Stated::parse(&code.requirement("ENG-11")?.text)?;
    w.section::<Profile>().taken = Some((plan, stated));
    Ok(())
}

#[then(regex = r"^every security-sensitive crate is at least 90% covered$")]
fn every_sensitive_crate(w: &mut World) -> Result<(), String> {
    let (plan, stated) = taken(w)?;
    if stated.sensitive_floor < 90.0 {
        return Err(format!(
            "the step says 90%, but ENG-11 says {}%",
            stated.sensitive_floor
        ));
    }
    let (built, skipped) = coverage_floors::built(plan, stated);
    if built.is_empty() {
        return Err("the workspace builds no security-sensitive crate".to_owned());
    }

    outcome(coverage_floors::sensitive_problems(plan, stated))
        .map_err(|e| format!("{e}\nnot built yet, so skipped: {}", skipped.join(", ")))
}

#[then(regex = r"^the workspace as a whole is at least 80% covered$")]
fn the_whole_workspace(w: &mut World) -> Result<(), String> {
    let (plan, stated) = taken(w)?;
    if stated.overall_floor < 80.0 {
        return Err(format!(
            "the step says 80%, but ENG-11 says {}%",
            stated.overall_floor
        ));
    }

    outcome(coverage_floors::overall_problems(plan, stated))
}
