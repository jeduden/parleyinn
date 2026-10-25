//! SEC-01's core row, SEC-15, SEC-17, SEC-19 and ENG-22 against the
//! repository itself. The SEC-01 scenario stays pending until the B1
//! components of M7 have their evidence; these gate tests prove its
//! core row in every `cargo test`. SEC-19, SEC-15, SEC-17 and ENG-22
//! are bound scenarios whose steps call the same checks.

use engineering::review_workflow::{self, Workflow};
use engineering::{
    Checkout, boundary, confinement, privacy, product_crates, static_analysis, telemetry,
    threat_model,
};

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

/// The workspace's `cargo metadata`, read with `PARINN_HOME` in a
/// temporary directory (ENG-14).
fn metadata() -> String {
    let home = testkit::TempDir::new().unwrap();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest = testkit::repo_root().join("Cargo.toml");

    engineering::dependencies::cargo_metadata(&cargo, &manifest, home.path()).unwrap()
}

fn ci(c: &Checkout) -> Workflow {
    review_workflow::parse(&c.read(static_analysis::CI_WORKFLOW).unwrap_or_default())
}

/// Every component, process and protocol sits in one boundary; every
/// B1 to B3 row is off by default; every process the repository builds
/// is listed; the evidence of each is present (SEC-19).
#[test]
fn the_boundary_register_assigns_each_component_once_and_the_core_keeps_its_evidence() {
    let c = checkout();

    let problems = boundary::check(&c, &ci(&c), &metadata());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The core opens no socket and starts no program but its own kernel
/// worker: its clippy bans and its reach test are in place, CI gates on
/// them, and every product crate keeps its bans (SEC-01).
#[test]
fn the_core_opens_no_socket_and_starts_no_program_but_its_kernel_worker() {
    let c = checkout();

    let mut problems = static_analysis::core_reach_evidence(&c, &ci(&c));
    problems.extend(product_crates::crates_keep_their_bans(&c));
    problems.extend(confinement::unconfined_starts(&c));

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Every process the repository builds runs as the core, in B0, so no
/// core process runs or starts a component behind B1 to B3 (SEC-01).
#[test]
fn only_the_core_is_built_and_it_sits_in_b0() {
    let c = checkout();
    let rows = boundary::register(&c).unwrap();

    let found = boundary::executables(&metadata()).unwrap();

    assert_eq!(found, ["parinn"]);
    assert!(
        rows.iter()
            .filter(|r| r.component == "core")
            .all(|r| r.boundary == "B0"),
        "a row of the core is outside B0"
    );
}

/// No telemetry, crash report or update check is linked or written
/// (SEC-15).
#[test]
fn no_telemetry_crash_report_or_update_check_ships() {
    let c = checkout();

    let mut problems = telemetry::check(&c);
    problems.extend(confinement::unconfined_starts(&c));

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The threat model covers the register and compares ParleyInn with
/// Zed Delta (SEC-17).
#[test]
fn the_threat_model_covers_the_register_and_compares_with_zed_delta() {
    let problems = threat_model::check(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Every privacy or data-flow statement cites a test that exists
/// (ENG-22).
#[test]
fn every_privacy_statement_cites_a_test_that_exists() {
    let c = checkout();
    let scenarios = privacy::scenario_states(&c.root.join("features")).unwrap();

    let problems = privacy::proven(&c, &scenarios);

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
