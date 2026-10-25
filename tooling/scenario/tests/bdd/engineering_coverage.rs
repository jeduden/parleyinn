//! The ENG-11 steps. They read the floors the coverage profile is held
//! to, not the profile itself: `[workspace.metadata.coverage]` through
//! `cargo metadata`, and the CI job that runs `coverage --merge`. That
//! job measures the merged profile of every test layer and fails a crate
//! below its floor, so a floor at or above the row's holds the crate to
//! the row whenever CI passes. Every security-sensitive crate the
//! workspace builds must be held to at least the row's floor, and every
//! crate to the overall one. The security-sensitive crates the workspace
//! does not build yet (`flag`, `pin`, `landmark` and `restore_block` in
//! M1) are skipped, and named when the step fails.

use std::env;
use std::ffi::OsString;

use coverage::plan::Plan;
use cucumber::{given, then};
use engineering::coverage_floors::{self, Row};
use engineering::{Checkout, dependencies, outcome, review_workflow};

use crate::World;

/// The profile's floors and the row they are held to.
#[derive(Debug, Default)]
struct Profile {
    taken: Option<(Plan, Row)>,
}

fn taken(w: &mut World) -> Result<&(Plan, Row), String> {
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
    let row = Row::parse(&code.requirement("ENG-11")?.text)?;
    w.section::<Profile>().taken = Some((plan, row));
    Ok(())
}

#[then(regex = r"^every security-sensitive crate is at least 90% covered$")]
fn every_sensitive_crate(w: &mut World) -> Result<(), String> {
    let (plan, row) = taken(w)?;
    if row.sensitive_floor < 90.0 {
        return Err(format!(
            "the step says 90%, but ENG-11 says {}%",
            row.sensitive_floor
        ));
    }
    let (built, _) = coverage_floors::built(plan, row);
    if built.is_empty() {
        return Err("the workspace builds no security-sensitive crate".to_owned());
    }

    let skipped = coverage_floors::built(plan, row).1;
    outcome(coverage_floors::sensitive_problems(plan, row))
        .map_err(|e| format!("{e}\nnot built yet, so skipped: {}", skipped.join(", ")))
}

#[then(regex = r"^the workspace as a whole is at least 80% covered$")]
fn the_whole_workspace(w: &mut World) -> Result<(), String> {
    let (plan, row) = taken(w)?;
    if row.overall_floor < 80.0 {
        return Err(format!(
            "the step says 80%, but ENG-11 says {}%",
            row.overall_floor
        ));
    }

    outcome(coverage_floors::overall_problems(plan, row))
}
