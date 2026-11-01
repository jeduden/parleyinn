//! ENG-16's core row against the repository itself: CI gates on the
//! static checks, every crate forbids unsafe code, and the core's
//! build-time reach evidence is present and gated. The ENG-16 outline
//! stays pending until its other components and TrustedText land, so
//! these gate tests prove the core row in every `cargo test`.

use engineering::review_workflow::{self, Workflow};
use engineering::{Checkout, static_analysis};

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

fn ci(c: &Checkout) -> Workflow {
    review_workflow::parse(&c.read(static_analysis::CI_WORKFLOW).unwrap_or_default())
}

/// CI gates on clippy, rustfmt, cargo-deny, cargo-audit and the custom
/// checks, each under the `ci` job.
#[test]
fn the_ci_workflow_gates_on_the_static_checks() {
    let problems = static_analysis::ci_gates(&ci(&checkout()));

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The test job's shards, each chosen by its place in the matrix, run
/// every workspace crate's tests but the scenario crate's exactly
/// once, so the custom checks and the decoy `HOME` cover them all.
#[test]
fn the_ci_test_shards_run_every_crate_once() {
    let c = checkout();

    let problems = engineering::test_shards::shards_cover_the_workspace(&c, &ci(&c));

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Every workspace crate keeps the workspace's unsafe ban, or its own.
#[test]
fn every_crate_forbids_unsafe_code() {
    let problems = static_analysis::crates_forbid_unsafe(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The core's reach evidence: its clippy bans and its reach test, each
/// run by a job the `ci` job needs.
#[test]
fn the_core_s_reach_evidence_is_present_and_gated() {
    let c = checkout();

    let problems = static_analysis::core_reach_evidence(&c, &ci(&c));

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Derived code is refused a clock and a random order at build time.
#[test]
fn derived_code_is_refused_a_clock_or_randomness_at_build_time() {
    let c = checkout();

    let problems = static_analysis::derived_code_gated(&c, &ci(&c));

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Restore content is built from trusted text alone: `TrustedText` is
/// constructed only inside the `restore_block` crate, and there only
/// from qualifying pins, sanitized structural fields and fixed text
/// (SEC-07, ENG-16).
#[test]
fn restore_content_is_built_from_trusted_text_alone() {
    let problems = engineering::trusted_text::constructions(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Function complexity is held at build time: the workspace lints deny
/// clippy's complexity lints, and every crate's clippy configuration
/// holds each threshold at or below its ceiling.
#[test]
fn every_crate_holds_the_complexity_limits() {
    let problems = engineering::complexity::crates_hold_limits(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// CI gates on Taplo's TOML checks: every TOML file's layout, and each
/// against the schema `.taplo.toml` names for it.
#[test]
fn the_ci_workflow_gates_on_the_toml_checks() {
    let problems = static_analysis::toml_gates(&ci(&checkout()));

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
