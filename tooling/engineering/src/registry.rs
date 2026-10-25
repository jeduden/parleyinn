//! ENG-27: the drift registry covers every check that keeps the records
//! in step. The drift suite injects the cases; these checks hold the
//! registry itself, so they run without mdsmith.

use std::path::Path;

use drift::Case;
use scenario::Scenario;

/// The step a scenario opens with when it inspects the repository
/// itself.
pub const CHECKOUT_STEP: &str = "the repository checkout";

/// The gate tests outside the scenarios that ENG-27 holds to a drift
/// case: the parse gates of the SRS and the ADRs, the
/// requirement–scenario gate, the Appendix B check, the persona gates
/// of Appendix C and §2.5, the finding ledger's check that every closed
/// domain-model finding stays in the text, and the core executable's
/// dependency reach (SEC-01); the three isolation gates of ENG-14: the
/// scan for a read of real state around the guard, the decoy `HOME` CI runs
/// the suite with, and the scan of the agent instruction files; and the
/// gates that prove what a pending scenario cannot yet: the core rows
/// of ENG-12 and ENG-16, ENG-15's benchmark gate, and ENG-21's ruleset
/// and CODEOWNERS; and the gates behind ENG-03, pending until M1 adds
/// injectable I/O traits, ENG-20's three workflow gates, the gate that
/// holds every crate to the complexity limits, and the gate that holds
/// CI to the TOML checks.
pub const GATE_TESTS: [&str; 36] = [
    "specification_parses",
    "decision_records_parse",
    "specification_and_features_agree",
    "appendix_b_matches_the_traces",
    "appendix_c_covers_every_requirement",
    "personas_match_the_agents",
    "finding_ledger_is_carried",
    "the_core_executable_links_only_crates_its_reach_allows",
    "only_the_isolation_guard_reads_the_home",
    "the_ci_test_job_runs_with_a_decoy_home",
    "the_agent_instructions_direct_no_destructive_command_at_real_state",
    "the_core_s_tests_start_it_confined",
    "the_ci_workflow_gates_on_the_static_checks",
    "every_crate_forbids_unsafe_code",
    "the_core_s_reach_evidence_is_present_and_gated",
    "derived_code_is_refused_a_clock_or_randomness_at_build_time",
    "every_crate_holds_the_complexity_limits",
    "the_ci_workflow_gates_on_the_toml_checks",
    "the_benchmark_suite_registers_every_nfr_id_eng_15_names",
    "ci_gates_on_the_benchmark_job",
    "benchmarks_and_timed_steps_run_release_builds",
    "the_calibration_workload_is_frozen",
    "the_checkout_s_ruleset_rejects_direct_pushes",
    "the_checkout_s_ruleset_requires_signed_commits",
    "the_checkout_s_ruleset_needs_an_approval_from_another_reviewer",
    "the_checkout_s_codeowners_name_the_stakeholder_on_the_enforcing_paths",
    "the_checkout_s_codeowners_name_no_owner_on_any_other_path",
    "the_checkout_s_security_crates_belong_to_the_security_reviewer",
    "the_checkout_s_nightly_compares_every_committed_ruleset",
    "the_checkout_s_rulesets_each_name_themselves_their_target_and_refs",
    "the_product_crates_keep_their_bans",
    "the_product_crates_ban_every_wait_with_no_deadline",
    "no_product_crate_keeps_mutable_global_state",
    "the_checkout_s_release_signs_its_checksums_keyless",
    "the_checkout_s_release_attests_provenance_only_in_the_release_build_workflow",
    "the_checkout_s_release_attaches_and_attests_an_sbom",
];

/// Refuses a case whose injection no longer finds its target, so a case
/// cannot rot into injecting nothing.
#[must_use]
pub fn drifts_apply(root: &Path, cases: &[Case]) -> Vec<String> {
    cases
        .iter()
        .filter_map(|c| {
            drift::validate(root, &c.injection)
                .err()
                .map(|e| format!("drift case {:?}: {e}", c.name))
        })
        .collect()
}

/// The ids of the non-pending scenarios that open on the checkout.
#[must_use]
pub fn checkout_scenarios(scenarios: &[Scenario]) -> Vec<String> {
    scenarios
        .iter()
        .filter(|s| !s.pending && s.steps.first().is_some_and(|step| step == CHECKOUT_STEP))
        .map(|s| s.id.clone())
        .collect()
}

/// Requires a drift case guarding each of `names`.
#[must_use]
pub fn guarded(cases: &[Case], names: &[impl AsRef<str>]) -> Vec<String> {
    names
        .iter()
        .map(AsRef::as_ref)
        .filter(|name| !cases.iter().any(|c| c.guards == *name))
        .map(|name| format!("{name} has no drift case"))
        .collect()
}

#[cfg(test)]
mod tests;
