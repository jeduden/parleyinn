//! Steps of `features/engineering.feature` shared across §10: the
//! checkout a scenario inspects, ENG-01's toolchain pins and release flags,
//! and ENG-24's disclosure policy.

use adr::Adr;
use cucumber::{given, then};
use drift::Case;
use engineering::review_workflow::Workflow;
use engineering::{Checkout, dependencies};

use crate::World;

/// What the §10 scenarios track beside the shared world: the checkout
/// they inspect and what a step read out of it.
#[derive(Debug, Default)]
pub struct Engineering {
    pub checkout: Option<Checkout>,
    pub deps: dependencies::DirectDeps,
    pub adr_dir: String,
    pub adrs: Vec<Adr>,
    pub drifts: Vec<Case>,
    pub flow: Workflow,
    /// The reviewer app's key, as a step last named it.
    pub key: String,
}

impl Engineering {
    /// The reviewer app's key the scenario named.
    pub fn key(&self) -> Result<&str, String> {
        if self.key.is_empty() {
            Err("no step named the reviewer app's key".to_owned())
        } else {
            Ok(&self.key)
        }
    }

    /// The checkout the scenario opened on.
    pub fn checkout(&self) -> Result<&Checkout, String> {
        self.checkout
            .as_ref()
            .ok_or_else(|| "the scenario must open on the repository checkout".to_owned())
    }
}

pub fn eng(w: &mut World) -> &mut Engineering {
    w.section::<Engineering>()
}

const RELEASE: &str = ".github/workflows/release.yml";

#[given(regex = r"^the repository checkout$")]
fn the_repository_checkout(w: &mut World) {
    eng(w).checkout = Some(Checkout::new(&testkit::repo_root()));
}

#[then(regex = r#"^"([^"]+)" pins the Rust toolchain to an exact release$"#)]
fn pins_the_rust_toolchain(w: &mut World, file: String) -> Result<(), String> {
    let toml = eng(w).checkout()?.read(&file)?;
    engineering::toolchain::rust_toolchain_pinned(&toml)
}

#[then(regex = r#"^the release workflow builds with "([^"]+)"$"#)]
fn release_builds_with(w: &mut World, flag: String) -> Result<(), String> {
    let build = engineering::toolchain::release_build(eng(w).checkout()?, RELEASE)?;
    engineering::toolchain::builds_with(&build, &flag)
}

#[then(regex = r#"^"SECURITY.md" links the private vulnerability reporting channel$"#)]
fn security_links_the_channel(w: &mut World) -> Result<(), String> {
    eng(w)
        .checkout()?
        .contains("SECURITY.md", "/security/advisories/new")
}

#[then(regex = r#"^"SECURITY.md" states a 90-day coordinated disclosure policy$"#)]
fn security_states_the_policy(w: &mut World) -> Result<(), String> {
    eng(w)
        .checkout()?
        .contains("SECURITY.md", "90-day coordinated disclosure")
}

#[then(
    regex = r#"^"SECURITY.md" states that security fixes are backported to the latest minor release$"#
)]
fn security_states_backports(w: &mut World) -> Result<(), String> {
    eng(w)
        .checkout()?
        .contains("SECURITY.md", "backported to the latest minor release")
}
