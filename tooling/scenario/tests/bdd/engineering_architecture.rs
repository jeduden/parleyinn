//! The ENG-02 steps: the workspace crates keep the dependency
//! directions their manifests declare, and each library crate declares
//! one responsibility.

use cucumber::{given, then, when};
use engineering::architecture::{self, WorkspaceCrate};
use engineering::{dependencies, outcome};

use crate::World;

/// What the ENG-02 scenario tracks: the workspace crates as the
/// manifests declare them, and the crates one forbidden dependency
/// joined.
#[derive(Debug, Default)]
pub struct Architecture {
    members: Vec<WorkspaceCrate>,
    forbidden: Option<Forbidden>,
}

/// The workspace with one dependency no direction allows, and the two
/// crates it joins.
#[derive(Debug)]
struct Forbidden {
    members: Vec<WorkspaceCrate>,
    from: String,
    on: String,
}

#[given(regex = r"^the declared dependency directions for the workspace crates$")]
fn the_declared_directions(w: &mut World) -> Result<(), String> {
    let root = testkit::repo_root();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let metadata = dependencies::cargo_metadata(&cargo, &root.join("Cargo.toml"), &w.parinn_home)?;
    let members = architecture::workspace(&metadata)?;
    outcome(architecture::directions(&members))?;
    w.section::<Architecture>().members = members;
    Ok(())
}

#[when(regex = r"^a crate depends on a crate its direction forbids$")]
fn a_crate_depends_against_its_direction(w: &mut World) -> Result<(), String> {
    let a = w.section::<Architecture>();
    let (members, from, on) = architecture::with_forbidden_dependency(&a.members)?;
    a.forbidden = Some(Forbidden { members, from, on });
    Ok(())
}

#[then(regex = r"^the architecture check fails naming both crates$")]
fn the_check_fails_naming_both(w: &mut World) -> Result<(), String> {
    let a = w.section::<Architecture>();
    let f = a
        .forbidden
        .as_ref()
        .ok_or("no step made a crate depend against its direction")?;

    architecture::names_both(&architecture::directions(&f.members), &f.from, &f.on)
}

#[then(regex = r"^every library crate has exactly one declared responsibility$")]
fn every_library_crate_has_one_responsibility(w: &mut World) -> Result<(), String> {
    outcome(architecture::responsibilities(
        &w.section::<Architecture>().members,
    ))
}
