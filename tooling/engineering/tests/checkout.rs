//! The checks' public API against the repository checkout itself: its
//! files, its SRS and its review workflow.

use engineering::{Checkout, branch_protection, review_workflow, toolchain};

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn a_checkout_reads_its_files_and_its_srs() {
    let c = checkout();

    assert_eq!(c.contains("Cargo.toml", "[workspace]"), Ok(()));
    assert_eq!(
        c.requirement("ENG-18").map(|r| r.id),
        Ok("ENG-18".to_owned())
    );
}

#[test]
fn a_checkout_names_what_it_cannot_find() {
    let c = checkout();

    assert!(c.read("no-such-file").is_err());
    assert_eq!(
        c.contains("Cargo.toml", "no such text"),
        Err(r#"Cargo.toml does not contain "no such text""#.to_owned())
    );
    assert_eq!(
        c.requirement("ENG-0").map(|r| r.id),
        Err("no requirement ENG-0 in the SRS".to_owned())
    );
}

#[test]
fn the_checkout_pins_its_rust_toolchain() {
    let c = checkout();

    assert_eq!(
        c.read("rust-toolchain.toml")
            .and_then(|t| toolchain::rust_toolchain_pinned(&t)),
        Ok(())
    );
}

#[test]
fn the_checkout_s_release_build_runs_through_the_build_workflow() {
    let build = toolchain::release_build(&checkout(), ".github/workflows/release.yml");

    let build = build.as_deref().map_err(String::as_str);
    assert_eq!(
        build
            .and_then(|b| toolchain::builds_with(b, "--remap-path-prefix").map_err(|_| "no remap")),
        Ok(())
    );
}

#[test]
fn the_checkout_s_review_workflow_parses_into_its_trigger_and_jobs() {
    let flow = checkout()
        .read(".github/workflows/review.yml")
        .map(|body| review_workflow::parse(&body));

    assert_eq!(
        flow.as_ref().map(|w| w.triggers.clone()),
        Ok(vec!["workflow_run".to_owned(), "issue_comment".to_owned()])
    );
    assert!(flow.is_ok_and(|w| w.jobs.len() >= 2));
}

/// Every product crate's clippy.toml refuses the core's reach (I4) and
/// the clock and random order derived artifacts may not read (I10).
#[test]
fn the_product_crates_keep_their_bans() {
    let problems = engineering::product_crates::crates_keep_their_bans(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Every product crate's clippy.toml refuses each blocking call that
/// waits with no deadline (ENG-03).
#[test]
fn the_product_crates_ban_every_wait_with_no_deadline() {
    let wanted = engineering::product_crates::deadline_bans();
    let problems = engineering::product_crates::crates_keep_bans(&checkout(), &wanted);

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Every workspace crate keeps the dependency directions its manifest
/// declares, and every library crate declares one responsibility
/// (ENG-02).
#[test]
fn the_workspace_keeps_its_declared_directions() {
    let home = testkit::TempDir::new().unwrap();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest = testkit::repo_root().join("Cargo.toml");
    let metadata = engineering::dependencies::cargo_metadata(&cargo, &manifest, home.path());

    let problems = engineering::architecture::problems(&metadata.unwrap());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// No product crate keeps mutable global state (ENG-03).
#[test]
fn no_product_crate_keeps_mutable_global_state() {
    let problems = engineering::product_crates::no_mutable_globals(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The workflows that publish a release sign its checksums keyless
/// (ENG-20).
#[test]
fn the_checkout_s_release_signs_its_checksums_keyless() {
    let release = engineering::release_provenance::load(&checkout()).unwrap_or_default();

    assert_eq!(release.checksums_signed_keyless(), Vec::<String>::new());
}

/// Only the release-only `release-build.yml` attests the binaries'
/// provenance, `build.yml` and CI's call of it hold nothing to sign
/// with, and the release workflow verifies what it published against
/// the signer's exact identity at main (ENG-20).
#[test]
fn the_checkout_s_release_attests_provenance_only_in_the_release_build_workflow() {
    let release = engineering::release_provenance::load(&checkout()).unwrap_or_default();

    assert_eq!(
        release.provenance_from_the_release_build_workflow(),
        Vec::<String>::new()
    );
}

/// The release attaches and attests an SPDX SBOM (ENG-20).
#[test]
fn the_checkout_s_release_attaches_and_attests_an_sbom() {
    let release = engineering::release_provenance::load(&checkout()).unwrap_or_default();

    assert_eq!(release.sbom_attached_and_attested(), Vec::<String>::new());
}

fn ruleset() -> serde_json::Value {
    engineering::branch_protection::load(&checkout()).unwrap_or_default()
}

/// The committed ruleset for main rejects direct pushes (ENG-21).
#[test]
fn the_checkout_s_ruleset_rejects_direct_pushes() {
    let problems = engineering::branch_protection::direct_pushes_rejected(&ruleset());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The committed ruleset requires signed commits (ENG-21).
#[test]
fn the_checkout_s_ruleset_requires_signed_commits() {
    let problems = engineering::branch_protection::commits_signed(&ruleset());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The committed ruleset needs an approval from another reviewer
/// (ENG-21).
#[test]
fn the_checkout_s_ruleset_needs_an_approval_from_another_reviewer() {
    let problems = engineering::branch_protection::approval_from_another_reviewer(&ruleset());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// CODEOWNERS names the stakeholder on the requirement text and on the
/// paths that enforce it (ENG-21).
#[test]
fn the_checkout_s_codeowners_name_the_stakeholder_on_the_enforcing_paths() {
    let problems = engineering::codeowners::stakeholder_owns_the_enforcing_paths(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// CODEOWNERS names no owner on any other path (ENG-21).
#[test]
fn the_checkout_s_codeowners_name_no_owner_on_any_other_path() {
    let problems = engineering::codeowners::no_owner_on_any_other_path(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Every agent the review bundle can dispatch is defined, at its
/// effort (ENG-28).
#[test]
fn the_checkout_s_review_agents_are_all_defined() {
    let problems = engineering::review_agents::every_dispatched_agent_is_defined(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// CODEOWNERS names no owner on Rust source (ENG-21).
#[test]
fn the_checkout_s_codeowners_name_no_owner_on_rust_source() {
    let problems = engineering::codeowners::no_owner_on_rust_source(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The nightly workflow compares every committed ruleset with the live
/// one of the same name (ENG-21).
#[test]
fn the_checkout_s_nightly_compares_every_committed_ruleset() {
    let nightly = checkout()
        .read(".github/workflows/nightly.yml")
        .unwrap_or_default();

    assert_eq!(
        branch_protection::nightly_compares_every_ruleset(&nightly),
        Vec::<String>::new()
    );
}

/// Every committed ruleset names itself, its target and the refs it
/// includes, once, and the tag rulesets are committed beside main's
/// (ENG-21).
#[test]
fn the_checkout_s_rulesets_each_name_themselves_their_target_and_refs() {
    let rulesets = match branch_protection::committed(&checkout()) {
        Ok(rulesets) => rulesets,
        Err(e) => panic!("{e}"),
    };
    let names: Vec<&str> = rulesets
        .iter()
        .filter_map(|(_, r)| r.get("name").and_then(serde_json::Value::as_str))
        .collect();

    assert_eq!(
        branch_protection::rulesets_named(&rulesets),
        Vec::<String>::new()
    );
    assert_eq!(names, ["main", "no-other-tags-allowed", "releases-semver"]);
}
