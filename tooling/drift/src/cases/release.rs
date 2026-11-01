//! The drifts of the release pipeline (ENG-20) and of the repository's
//! protection (ENG-21).

use super::*;

/// A case for ENG-20 caught by the gate test that reads the same
/// workflows as the scenario.
pub(super) fn release_case(
    name: &'static str,
    injection: Injection,
    test: &'static str,
    want: &'static str,
) -> Case {
    case(
        name,
        test,
        injection,
        cargo_test("engineering", "checkout", test),
        want,
    )
}

/// The gate test of ENG-20's keyless checksum signature.
pub(super) const CHECKSUMS_GATE: &str = "the_checkout_s_release_signs_its_checksums_keyless";

/// The gate test of ENG-20's build provenance.
pub(super) const PROVENANCE_GATE: &str =
    "the_checkout_s_release_attests_provenance_only_in_the_release_build_workflow";

/// The gate test of ENG-20's SBOM.
pub(super) const SBOM_GATE: &str = "the_checkout_s_release_attaches_and_attests_an_sbom";

/// The drifts ENG-20 exists to catch: provenance signed anywhere but
/// the release-only reusable workflow, a signing permission granted to
/// the build CI runs, a verification that accepts a signer at any ref,
/// a missing check of the published release, and an SBOM that is not
/// attested.
pub(super) fn release_cases() -> Vec<Case> {
    [
        release_signer_cases(),
        release_build_cases(),
        release_verify_cases(),
    ]
    .concat()
}

/// The ENG-20 drifts in who signs the build provenance, and with what.
/// The first is caught by the scenario itself, the rest by the gate
/// tests that read the same workflows.
pub(super) fn release_signer_cases() -> Vec<Case> {
    vec![
        case(
            "ci.yml granting attestations to its build call",
            "ENG-20",
            Injection::replace(
                CI_WORKFLOW,
                "    uses: ./.github/workflows/build.yml\n    permissions:\n      contents: read\n",
                "    uses: ./.github/workflows/build.yml\n    permissions:\n      contents: read\n      attestations: write\n",
            ),
            scenario("ENG-20"),
            "ci.yml job build grants attestations: write to build.yml",
        ),
        release_case(
            "build provenance attested outside the release build workflow",
            Injection::replace_line(
                RELEASE_WORKFLOW,
                ATTEST_SBOM,
                concat!(
                    "uses: actions/attest-build-provenance@0000000000000000000000000000000000000000\n",
                    "        with:\n          subject-path: \"dist/parinn-*\"\n",
                    "      - name: Attest the SBOM again\n",
                    "        uses: actions/attest-sbom@0000000000000000000000000000000000000000",
                ),
            ),
            PROVENANCE_GATE,
            "release.yml attests build provenance outside release-build.yml, in job publish",
        ),
        release_case(
            "the build workflow CI runs attesting build provenance",
            Injection::replace(
                BUILD_WORKFLOW,
                "        run: scripts/check-stamp.sh dist/parinn-windows-amd64.exe \"$VERSION\" \"$COMMIT\"\n",
                "        run: scripts/check-stamp.sh dist/parinn-windows-amd64.exe \"$VERSION\" \"$COMMIT\"\n\n  attest:\n    needs: windows\n    runs-on: ubuntu-latest\n    permissions:\n      id-token: write\n      attestations: write\n    steps:\n      - uses: actions/attest-build-provenance@0000000000000000000000000000000000000000\n        with:\n          subject-path: \"dist/parinn-*\"\n",
            ),
            PROVENANCE_GATE,
            "build.yml attests build provenance in job attest; only release-build.yml may",
        ),
        release_case(
            "ci.yml granting id-token to its build call",
            Injection::replace(
                CI_WORKFLOW,
                "    uses: ./.github/workflows/build.yml\n    permissions:\n      contents: read\n",
                "    uses: ./.github/workflows/build.yml\n    permissions:\n      contents: read\n      id-token: write\n",
            ),
            PROVENANCE_GATE,
            "ci.yml job build grants id-token: write to build.yml",
        ),
    ]
}

/// The ENG-20 drifts in the release build workflow itself: it no longer
/// attesting provenance, its attest job running the repository's code,
/// the release going around it, and a release dispatched from any ref.
pub(super) fn release_build_cases() -> Vec<Case> {
    vec![
        release_case(
            "the release build workflow no longer attesting provenance",
            Injection::replace_line(
                RELEASE_BUILD_WORKFLOW,
                ATTEST_PROVENANCE,
                "uses: actions/upload-artifact@0000000000000000000000000000000000000000",
            ),
            PROVENANCE_GATE,
            "release-build.yml does not attest build provenance",
        ),
        release_case(
            "the attest job checking out the repository's code",
            Injection::replace(
                RELEASE_BUILD_WORKFLOW,
                "    steps:\n      - uses: actions/download-artifact@",
                "    steps:\n      - uses: actions/checkout@0000000000000000000000000000000000000000\n      - uses: actions/download-artifact@",
            ),
            PROVENANCE_GATE,
            "release-build.yml job attest checks out the repository's code",
        ),
        release_case(
            "the release building through build.yml directly",
            Injection::replace(
                RELEASE_WORKFLOW,
                "uses: ./.github/workflows/release-build.yml",
                "uses: ./.github/workflows/build.yml",
            ),
            PROVENANCE_GATE,
            "release.yml does not call release-build.yml",
        ),
        release_case(
            "a release dispatched from any ref",
            Injection::replace(
                RELEASE_WORKFLOW,
                "if [ \"$GITHUB_REF\" != refs/heads/main ]; then",
                "if false; then",
            ),
            PROVENANCE_GATE,
            "release.yml builds without refusing a ref other than refs/heads/main",
        ),
    ]
}

/// Changing the SBOM format must fail the ENG-20 format gate.
pub(super) fn sbom_format_case() -> Case {
    release_case(
        "the release generating a non-SPDX SBOM",
        Injection::replace(
            RELEASE_WORKFLOW,
            "syft . --output spdx-json=dist/parinn.spdx.json",
            "syft . --output cyclonedx-json=dist/parinn.spdx.json",
        ),
        SBOM_GATE,
        "job publish does not generate an SPDX SBOM",
    )
}

/// The ENG-20 drifts in what the release publishes and how its verify
/// job checks it.
pub(super) fn release_verify_cases() -> Vec<Case> {
    vec![
        sbom_format_case(),
        release_case(
            "no job verifying the published release",
            Injection::replace(
                RELEASE_WORKFLOW,
                "  verify:\n    needs: publish\n",
                "  verify:\n    needs: build\n",
            ),
            PROVENANCE_GATE,
            "no job verifies the published release after job publish",
        ),
        release_case(
            "an unpinned verify: --signer-workflow without a ref",
            Injection::replace(
                RELEASE_WORKFLOW,
                "              --cert-identity https://github.com/jeduden/parleyinn/.github/workflows/release-build.yml@refs/heads/main \\\n              --source-ref refs/heads/main --source-digest \"$GITHUB_SHA\" \\\n",
                "              --signer-workflow jeduden/parleyinn/.github/workflows/release-build.yml \\\n",
            ),
            PROVENANCE_GATE,
            "job verify verifies an attestation with --signer-workflow, which pins no ref",
        ),
        release_case(
            "the provenance verified against an identity at another ref",
            Injection::replace(
                RELEASE_WORKFLOW,
                "release-build.yml@refs/heads/main",
                "release-build.yml@refs/heads/feature",
            ),
            PROVENANCE_GATE,
            "job verify does not verify build provenance with --repo jeduden/parleyinn --cert-identity https://github.com/jeduden/parleyinn/.github/workflows/release-build.yml@refs/heads/main --source-ref refs/heads/main --deny-self-hosted-runners",
        ),
        release_case(
            "the checksum bundle no longer verified",
            Injection::replace(RELEASE_WORKFLOW, "cosign verify-blob", "cosign version"),
            CHECKSUMS_GATE,
            "does not run cosign verify-blob",
        ),
        release_case(
            "the checksum bundle verified at any ref",
            Injection::replace(
                RELEASE_WORKFLOW,
                "--certificate-identity https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/main",
                "--certificate-identity-regexp '^https://github.com/jeduden/parleyinn/.github/workflows/release.yml@'",
            ),
            CHECKSUMS_GATE,
            "job verify does not pin the certificate identity to https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/main",
        ),
        release_case(
            "the SBOM generated but not attested",
            Injection::replace_line(
                RELEASE_WORKFLOW,
                ATTEST_SBOM,
                "uses: actions/checkout@0000000000000000000000000000000000000000",
            ),
            SBOM_GATE,
            "release.yml does not attest the SBOM",
        ),
        release_case(
            "the SBOM left off the release",
            Injection::replace(
                RELEASE_WORKFLOW,
                "parinn-* parinn.spdx.json checksums.txt checksums.txt.sigstore.json",
                "parinn-* checksums.txt checksums.txt.sigstore.json",
            ),
            SBOM_GATE,
            "the release does not attach the SBOM parinn.spdx.json",
        ),
        release_case(
            "the SBOM attestation no longer verified",
            Injection::replace(
                RELEASE_WORKFLOW,
                " --predicate-type https://spdx.dev/Document/v2.3",
                "",
            ),
            SBOM_GATE,
            "does not verify the SBOM attestation",
        ),
    ]
}

/// A case for ENG-21: each drift is caught by the gate test that reads
/// the same records as the scenario's steps.
pub(super) fn protection_case(
    name: &'static str,
    injection: Injection,
    test: &'static str,
    want: &'static str,
) -> Case {
    case(
        name,
        test,
        injection,
        cargo_test("engineering", "checkout", test),
        want,
    )
}

/// The CI and release tool pins must remain owned by the stakeholder.
pub(super) fn mise_owner_case() -> Case {
    protection_case(
        "the mise toolchain pin without the stakeholder",
        Injection::replace(CODEOWNERS, "/mise.toml                    @jeduden\n", ""),
        "the_checkout_s_codeowners_name_the_stakeholder_on_the_enforcing_paths",
        "mise.toml (CI,) is owned by [], want @jeduden",
    )
}

/// The developer tool pins, which CI's install step reads, must remain
/// owned by the stakeholder.
pub(super) fn mise_dev_owner_case() -> Case {
    protection_case(
        "the developer tool pin without the stakeholder",
        Injection::replace(CODEOWNERS, "/mise.dev.toml                @jeduden\n", ""),
        "the_checkout_s_codeowners_name_the_stakeholder_on_the_enforcing_paths",
        "mise.dev.toml (CI,) is owned by [], want @jeduden",
    )
}

/// The scenarios must remain owned by the stakeholder.
pub(super) fn scenarios_owner_case() -> Case {
    protection_case(
        "the scenarios without the stakeholder",
        Injection::replace(CODEOWNERS, "/features/                    @jeduden\n", ""),
        "the_checkout_s_codeowners_name_the_stakeholder_on_the_enforcing_paths",
        "features/ (the scenarios (`features/`)) is owned by [], want @jeduden",
    )
}

/// No Rust source may get a code owner, the stakeholder included.
pub(super) fn rust_source_owner_case() -> Case {
    protection_case(
        "an owner on Rust source",
        Injection::replace(
            CODEOWNERS,
            "/SECURITY.md                  @jeduden\n",
            "/SECURITY.md                  @jeduden\n*.rs                          @jeduden\n",
        ),
        "the_checkout_s_codeowners_name_no_owner_on_rust_source",
        "crates/parinn/src/main.rs is Rust source owned by [@jeduden]",
    )
}

pub(super) fn signed_commits_case() -> Case {
    protection_case(
        "commits no longer required to be signed",
        Injection::replace(RULESET, "    { \"type\": \"required_signatures\" },\n", ""),
        "the_checkout_s_ruleset_requires_signed_commits",
        "the ruleset has no required_signatures rule",
    )
}

pub(super) fn review_gate_check_case() -> Case {
    protection_case(
        "the review gate's check taken from any app",
        Injection::replace(
            RULESET,
            "{ \"context\": \"review-gate\", \"integration_id\": 5140085 }",
            "{ \"context\": \"review-gate\" }",
        ),
        "the_checkout_s_ruleset_needs_the_review_gate_check",
        "the ruleset takes the review-gate check from any app, want app 5140085",
    )
}

/// The drifts ENG-21 exists to catch: main's committed ruleset or
/// CODEOWNERS weaker than the requirement says.
pub(super) fn protection_cases() -> Vec<Case> {
    vec![
        signed_commits_case(),
        protection_case(
            "an actor allowed to bypass the ruleset",
            Injection::replace(
                RULESET,
                "\"bypass_actors\": [],",
                "\"bypass_actors\": [{ \"actor_id\": 5, \"actor_type\": \"RepositoryRole\", \"bypass_mode\": \"always\" }],",
            ),
            "the_checkout_s_ruleset_rejects_direct_pushes",
            "the ruleset lets a bypass actor bypass it",
        ),
        protection_case(
            "the ruleset no longer enforced",
            Injection::replace(
                RULESET,
                "\"enforcement\": \"active\"",
                "\"enforcement\": \"evaluate\"",
            ),
            "the_checkout_s_ruleset_rejects_direct_pushes",
            "the ruleset's enforcement is not active",
        ),
        protection_case(
            "no approval required",
            Injection::replace(
                RULESET,
                "\"required_approving_review_count\": 1",
                "\"required_approving_review_count\": 0",
            ),
            "the_checkout_s_ruleset_needs_an_approval_from_another_reviewer",
            "the pull_request rule requires no approving review",
        ),
        review_gate_check_case(),
        protection_case(
            "a code owner's approval no longer required",
            Injection::replace(
                RULESET,
                "\"require_code_owner_review\": true",
                "\"require_code_owner_review\": false",
            ),
            "the_checkout_s_ruleset_needs_an_approval_from_another_reviewer",
            "the pull_request rule does not set require_code_owner_review",
        ),
        protection_case(
            "the requirement text without the stakeholder",
            Injection::replace(CODEOWNERS, "/docs/srs/                    @jeduden\n", ""),
            "the_checkout_s_codeowners_name_the_stakeholder_on_the_enforcing_paths",
            "docs/srs/ (`docs/srs/`) is owned by [], want @jeduden",
        ),
        protection_case(
            "the agent instructions without the stakeholder",
            Injection::replace(CODEOWNERS, "/CLAUDE.md                    @jeduden\n", ""),
            "the_checkout_s_codeowners_name_the_stakeholder_on_the_enforcing_paths",
            "CLAUDE.md (the agent instructions) is owned by [], want @jeduden",
        ),
        mise_owner_case(),
        mise_dev_owner_case(),
        protection_case(
            "an owner on a path outside the allowed classes",
            Injection::replace(
                CODEOWNERS,
                "/SECURITY.md                  @jeduden\n",
                "/SECURITY.md                  @jeduden\n/crates/parinn/src/            @jeduden\n",
            ),
            "the_checkout_s_codeowners_name_no_owner_on_any_other_path",
            "/crates/parinn/src/ names an owner on a path outside the allowed classes",
        ),
        scenarios_owner_case(),
        rust_source_owner_case(),
        protection_case(
            "the nightly comparing main's ruleset alone",
            Injection::replace(
                ".github/workflows/nightly.yml",
                "for committed in .github/rulesets/*.json; do",
                "for committed in .github/rulesets/main.json; do",
            ),
            "the_checkout_s_nightly_compares_every_committed_ruleset",
            "nightly.yml does not loop over every committed ruleset",
        ),
        protection_case(
            "the nightly no longer comparing the live rulesets",
            Injection::replace(
                ".github/workflows/nightly.yml",
                "protection-check -- --committed",
                "protection-check -- --help",
            ),
            "the_checkout_s_nightly_compares_every_committed_ruleset",
            "nightly.yml does not compare each one",
        ),
        protection_case(
            "a tag ruleset that includes no ref",
            Injection::replace(
                ".github/rulesets/no-other-tags-allowed.json",
                "\"include\": [\"~ALL\"]",
                "\"include\": []",
            ),
            "the_checkout_s_rulesets_each_name_themselves_their_target_and_refs",
            "no-other-tags-allowed.json includes no ref, so it applies to nothing",
        ),
        protection_case(
            "two committed rulesets sharing a name",
            Injection::replace(
                ".github/rulesets/no-other-tags-allowed.json",
                "\"name\": \"no-other-tags-allowed\",",
                "\"name\": \"releases-semver\",",
            ),
            "the_checkout_s_rulesets_each_name_themselves_their_target_and_refs",
            "repeats the ruleset name releases-semver",
        ),
    ]
}
