//! The registry: every drift a check that keeps the records in step
//! exists to catch.

mod answers;
mod architecture;
mod boundary;
mod ci;
mod ci_gates;
mod context;
mod file_sizes;
mod previous;
mod records;
mod release;
mod review_agents;
mod review_findings;
mod review_logs;
mod review_reports;
mod review_stability;
mod spend;

use architecture::*;
use boundary::*;
use ci::*;
use file_sizes::*;
use records::*;
use release::*;
use review_agents::*;
use review_findings::*;
use review_reports::*;
use review_stability::*;

use crate::{
    Case, Check, Injection, cargo_lib_test, cargo_test, clippy, mdsmith, scenario, setup_tools,
    taplo_fmt, taplo_lint,
};

/// The dependency decision the ENG-18 and ENG-26 cases drift.
const TEST_STACK: &str = "docs/adr/ADR-2610142221-json-serde-json.md";

/// The decision the fuzz workspace's dependency rests on.
const FUZZING: &str = "docs/adr/ADR-2610171200-fuzzing-libfuzzer-sys.md";

/// The decision [`TEST_STACK`] superseded.
const OLD_TEST_STACK: &str = "docs/adr/ADR-2610101913-test-stack-after-go.md";

/// The release workflow the ENG-20 cases drift.
const RELEASE_WORKFLOW: &str = ".github/workflows/release.yml";

/// The release-only reusable workflow the ENG-20 cases drift.
const RELEASE_BUILD_WORKFLOW: &str = ".github/workflows/release-build.yml";

/// The reusable build workflow the ENG-20 cases drift.
const BUILD_WORKFLOW: &str = ".github/workflows/build.yml";

/// The CI workflow the ENG-20 cases drift.
const CI_WORKFLOW: &str = ".github/workflows/ci.yml";

/// Where the line using the action that attests build provenance starts.
/// The sha and version after it are the bots' to move, so no case
/// quotes them.
const ATTEST_PROVENANCE: &str = "uses: actions/attest-build-provenance@";

/// Where the line using the action that attests the release's SBOM starts.
const ATTEST_SBOM: &str = "uses: actions/attest-sbom@";

/// The ruleset for main the ENG-21 cases drift.
const RULESET: &str = ".github/rulesets/main.json";

/// The code owners the ENG-21 cases drift.
const CODEOWNERS: &str = ".github/CODEOWNERS";

/// The workflow the ENG-28 cases drift.
const REVIEW_WORKFLOW: &str = ".github/workflows/review.yml";

/// The script the review workflow's post job decides the review with.
const REVIEW_DECIDE_SCRIPT: &str = "scripts/review-decide.sh";

/// The scheduled reviewer evaluation the ENG-32 case drifts.
const REVIEW_EVAL_WORKFLOW: &str = ".github/workflows/review-eval.yml";

/// Lists every registered drift. A check that keeps the records in step
/// gets a case here in the change that adds it (ENG-27); the ENG-27
/// scenario fails while a repository-inspecting scenario or a gate test
/// has none.
#[must_use]
pub fn cases() -> Vec<Case> {
    [
        dependency_cases(),
        decision_cases(),
        repository_cases(),
        review_cases(),
        review_gate_cases(),
        vec![review_report_case()],
        review_agent_cases(),
        review_evaluation_cases(),
        gate_cases(),
        parse_cases(),
        persona_cases(),
        agent_cases(),
        domain_model_cases(),
        ledger_cases(),
        reach_cases(),
        product_crate_cases(),
        deadline_cases(),
        core_wait_cases(),
        confinement_cases(),
        ci_gate_cases(),
        static_analysis_cases(),
        boundary_cases(),
        telemetry_cases(),
        threat_model_cases(),
        privacy_cases(),
        complexity_cases(),
        trusted_text_cases(),
        toml_cases(),
        setup_cases(),
        release_cases(),
        protection_cases(),
        architecture_cases(),
        benchmark_cases(),
        fuzz_cases(),
        coverage_floor_cases(),
        isolation_cases(),
        instruction_cases(),
        transcript_contract_cases(),
        // Late, so adding it moves no earlier case to another shard.
        vec![consent_case(), test_shards_case()],
        review_late_cases(),
        review_report_form_cases(),
        review_run_report_cases(),
        review_report_grading_cases(),
        workflow_cases(),
        review_log_report_cases(),
        review_run_parts_cases(),
        review_finding_shape_cases(),
        review_limit_cases(),
        review_withdrawal_cases(),
        review_place_form_cases(),
        review_retry_cases(),
        vec![review_unlinked_case()],
        vec![review_unknown_id_post_case(), review_case_severity_case()],
        review_undashed_cases(),
        review_resume_cases(),
        vec![review_moved_revision_case()],
        review_control_cases(),
        answers::review_answer_cases(),
        review_stability_cases(),
        review_token_cases(),
        stability_workflow_cases(),
        context::review_context_cases(),
        context::review_context_file_cases(),
        context::review_identity_cases(),
        ci_gates::cases(),
        file_size_cases(),
        previous::review_previous_cases(),
        vec![review_cap_retry_case(), review_generalist_turns_case()],
        review_weighing_cases(),
        spend::review_spend_cases(),
        review_logs::review_log_cases(),
        // Last: its cases edit source the later cases would run stale.
        license_text_cases(),
    ]
    .concat()
}

fn case(
    name: &'static str,
    guards: &'static str,
    injection: Injection,
    check: Check,
    want: &'static str,
) -> Case {
    Case {
        name,
        guards,
        injection,
        check,
        want,
        known_gap: false,
    }
}

#[cfg(test)]
mod tests;
