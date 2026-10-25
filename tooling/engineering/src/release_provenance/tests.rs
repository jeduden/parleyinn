use super::*;

const BUILD_YML: &str = "\
name: Build
on:
  workflow_call:
    inputs:
      version:
        type: string
permissions:
  contents: read
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: cargo build
";

const RELEASE_BUILD_YML: &str = "\
name: Release build
on:
  workflow_call:
    inputs:
      version:
        type: string
jobs:
  build:
    uses: ./.github/workflows/build.yml
    permissions:
      contents: read
  attest:
    needs: build
    permissions:
      contents: read
      id-token: write
      attestations: write
    steps:
      - uses: actions/download-artifact@abc
      - uses: actions/attest-build-provenance@abc # v4
";

const CI_YML: &str = "\
name: CI
on:
  push:
jobs:
  coverage:
    permissions:
      id-token: write
  build:
    uses: ./.github/workflows/build.yml
    permissions:
      contents: read
";

const RELEASE_YML: &str = "\
name: Release
on:
  workflow_dispatch:
jobs:
  preflight:
    steps:
      - run: |
          if [ \"$GITHUB_REF\" != refs/heads/main ]; then exit 1; fi
  build:
    needs: preflight
    uses: ./.github/workflows/release-build.yml
    permissions:
      id-token: write
      attestations: write
  publish:
    needs: build
    permissions:
      id-token: write
      attestations: write
    steps:
      - run: syft . --output spdx-json=dist/parinn.spdx.json
      - uses: actions/attest-sbom@abc
        with:
          sbom-path: dist/parinn.spdx.json
      - run: cosign sign-blob --bundle checksums.txt.sigstore.json checksums.txt
      - run: |
          gh release create \"$VERSION\" \\
            --generate-notes \\
            parinn-* parinn.spdx.json checksums.txt checksums.txt.sigstore.json
  verify:
    needs: publish
    steps:
      - run: |
          cosign verify-blob --bundle checksums.txt.sigstore.json \\
            --certificate-identity https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/main \\
            --certificate-oidc-issuer https://token.actions.githubusercontent.com \\
            checksums.txt
          for bin in parinn-*; do
            gh attestation verify \"$bin\" --repo jeduden/parleyinn \\
              --cert-identity https://github.com/jeduden/parleyinn/.github/workflows/release-build.yml@refs/heads/main \\
              --source-ref refs/heads/main --deny-self-hosted-runners
            gh attestation verify \"$bin\" --repo jeduden/parleyinn \\
              --cert-identity https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/main \\
              --source-ref refs/heads/main --predicate-type https://spdx.dev/Document/v2.3
          done
";

/// The verify job's check of the build provenance, as it stands in
/// [`RELEASE_YML`].
const PINNED: &str = "--cert-identity https://github.com/jeduden/parleyinn/.github/workflows/release-build.yml@refs/heads/main \\
              --source-ref refs/heads/main";

/// Which of the four workflows a drift edits.
#[derive(Clone, Copy)]
enum File {
    Release,
    ReleaseBuild,
    Build,
    Ci,
}

fn good() -> Release {
    Release {
        release: parse(RELEASE_YML),
        release_build: parse(RELEASE_BUILD_YML),
        build: parse(BUILD_YML),
        ci: parse(CI_YML),
    }
}

fn edited(file: File, from: &str, to: &str) -> Release {
    let edit = |body: &str| {
        assert!(body.contains(from), "{from}");
        parse(&body.replace(from, to))
    };
    let mut r = good();
    match file {
        File::Release => r.release = edit(RELEASE_YML),
        File::ReleaseBuild => r.release_build = edit(RELEASE_BUILD_YML),
        File::Build => r.build = edit(BUILD_YML),
        File::Ci => r.ci = edit(CI_YML),
    }

    r
}

fn drifted(from: &str, to: &str) -> Release {
    edited(File::Release, from, to)
}

fn has(problems: &[String], want: &str) {
    assert!(
        problems.iter().any(|p| p.contains(want)),
        "{want:?} not in {problems:#?}"
    );
}

fn provenance(file: File, from: &str, to: &str) -> Vec<String> {
    edited(file, from, to).provenance_from_the_release_build_workflow()
}

#[test]
fn a_good_release_has_no_problem() {
    let r = good();

    assert_eq!(r.checksums_signed_keyless(), Vec::<String>::new());
    assert_eq!(
        r.provenance_from_the_release_build_workflow(),
        Vec::<String>::new()
    );
    assert_eq!(r.sbom_attached_and_attested(), Vec::<String>::new());
}

#[test]
fn load_reads_the_four_workflows_or_names_the_missing_one() {
    let all = [
        (RELEASE, RELEASE_YML),
        (RELEASE_BUILD, RELEASE_BUILD_YML),
        (BUILD, BUILD_YML),
        (CI, CI_YML),
    ];
    let (_dir, c) = crate::tests::checkout(&all);
    assert_eq!(load(&c).unwrap(), good());

    for missing in [RELEASE_BUILD, BUILD, CI] {
        let rest: Vec<_> = all.into_iter().filter(|(p, _)| *p != missing).collect();
        let (_dir, c) = crate::tests::checkout(&rest);
        assert!(
            load(&c)
                .unwrap_err()
                .starts_with(&format!("read {missing}"))
        );
    }
}

#[test]
fn identity_names_a_workflow_at_the_release_ref() {
    assert_eq!(
        identity("release.yml"),
        "https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/main"
    );
}

#[test]
fn provenance_attested_in_the_release_workflow_is_outside_the_release_build_workflow() {
    let problems = provenance(
        File::Release,
        "      - uses: actions/attest-sbom@abc\n",
        "      - uses: actions/attest-build-provenance@abc\n",
    );

    has(
        &problems,
        "release.yml attests build provenance outside release-build.yml, in job publish",
    );
}

#[test]
fn the_build_workflow_ci_runs_attests_nothing_and_asks_for_no_signing_permission() {
    let problems = provenance(
        File::Build,
        "      - run: cargo build\n",
        "      - run: cargo build\n  attest:\n    permissions:\n      id-token: write\n    steps:\n      - uses: actions/attest-build-provenance@abc\n",
    );
    has(
        &problems,
        "build.yml attests build provenance in job attest; only release-build.yml may",
    );
    has(&problems, "build.yml job attest asks for id-token: write");

    let problems = provenance(
        File::Build,
        "      version:\n",
        "      attest:\n        type: boolean\n      version:\n",
    );
    has(&problems, "build.yml takes an attest input");
}

#[test]
fn the_release_build_workflow_builds_through_build_yml_and_attests_after_it() {
    let problems = provenance(
        File::ReleaseBuild,
        "uses: ./.github/workflows/build.yml",
        "uses: x/y@z",
    );
    has(&problems, "release-build.yml does not call build.yml");

    let problems = provenance(
        File::ReleaseBuild,
        "actions/attest-build-provenance@",
        "actions/other@",
    );
    has(
        &problems,
        "release-build.yml does not attest build provenance",
    );

    let problems = provenance(File::ReleaseBuild, "    needs: build\n", "");
    has(
        &problems,
        "release-build.yml job attest does not wait for job build",
    );
}

#[test]
fn the_attest_job_holds_both_permissions_and_runs_no_repository_code() {
    let problems = provenance(File::ReleaseBuild, "      attestations: write\n", "");
    has(&problems, "job attest lacks attestations: write");

    let problems = provenance(
        File::ReleaseBuild,
        "      - uses: actions/download-artifact@abc\n",
        "      - uses: actions/checkout@abc\n",
    );
    has(
        &problems,
        "release-build.yml job attest checks out the repository's code",
    );
}

#[test]
fn the_build_call_inside_the_release_build_workflow_is_granted_no_signing_permission() {
    let problems = provenance(
        File::ReleaseBuild,
        "    uses: ./.github/workflows/build.yml\n    permissions:\n      contents: read\n",
        "    uses: ./.github/workflows/build.yml\n    permissions:\n      contents: read\n      attestations: write\n",
    );

    has(
        &problems,
        "release-build.yml job build grants attestations: write to build.yml",
    );
}

#[test]
fn ci_grants_its_build_call_no_signing_permission_and_never_calls_the_release_build() {
    let problems = provenance(
        File::Ci,
        "    uses: ./.github/workflows/build.yml\n    permissions:\n      contents: read\n",
        "    uses: ./.github/workflows/build.yml\n    permissions:\n      contents: read\n      id-token: write\n",
    );
    has(
        &problems,
        "ci.yml job build grants id-token: write to build.yml",
    );

    let problems = provenance(
        File::Ci,
        "uses: ./.github/workflows/build.yml",
        "uses: ./.github/workflows/release-build.yml",
    );
    has(
        &problems,
        "ci.yml job build calls release-build.yml, which only release.yml may call",
    );
}

#[test]
fn the_release_calls_the_release_build_granting_both_permissions_from_main_only() {
    let problems = provenance(
        File::Release,
        "uses: ./.github/workflows/release-build.yml",
        "uses: ./.github/workflows/build.yml",
    );
    has(&problems, "release.yml does not call release-build.yml");

    let problems = provenance(
        File::Release,
        "    uses: ./.github/workflows/release-build.yml\n    permissions:\n      id-token: write\n",
        "    uses: ./.github/workflows/release-build.yml\n    permissions:\n",
    );
    has(&problems, "job build lacks id-token: write");

    let problems = provenance(File::Release, "!= refs/heads/main", "= refs/heads/x");
    has(
        &problems,
        "release.yml builds without refusing a ref other than refs/heads/main",
    );
}

#[test]
fn a_release_without_a_verify_job_is_named_by_every_check() {
    let r = drifted(
        "  verify:\n    needs: publish\n",
        "  verify:\n    needs: build\n",
    );
    let want = "no job verifies the published release after job publish";

    has(&r.provenance_from_the_release_build_workflow(), want);
    has(&r.checksums_signed_keyless(), want);
    has(&r.sbom_attached_and_attested(), want);
}

#[test]
fn a_release_nobody_creates_has_no_publish_job() {
    let r = drifted("gh release create", "gh release view");
    let want = "no job in release.yml creates the release";

    has(&r.checksums_signed_keyless(), want);
    has(&r.provenance_from_the_release_build_workflow(), want);
    has(&r.sbom_attached_and_attested(), want);
}

#[test]
fn the_verify_job_pins_the_release_build_workflow_s_identity_and_ref() {
    let want = "job verify does not verify build provenance with --repo jeduden/parleyinn \
                --cert-identity https://github.com/jeduden/parleyinn/.github/workflows/release-build.yml@refs/heads/main \
                --source-ref refs/heads/main --deny-self-hosted-runners";

    let problems = provenance(
        File::Release,
        PINNED,
        "--signer-workflow jeduden/parleyinn/.github/workflows/release-build.yml",
    );
    has(&problems, want);
    has(
        &problems,
        "job verify verifies an attestation with --signer-workflow, which pins no ref",
    );

    let problems = provenance(
        File::Release,
        "release-build.yml@refs/heads/main",
        "release-build.yml@refs/heads/x",
    );
    has(&problems, want);

    let problems = provenance(
        File::Release,
        "release-build.yml@refs/heads/main",
        "release-build.yml@refs/heads/main-x",
    );
    has(&problems, want);

    let problems = provenance(
        File::Release,
        PINNED,
        "--cert-identity https://github.com/jeduden/parleyinn/.github/workflows/release-build.yml@refs/heads/main",
    );
    has(&problems, want);

    let problems = provenance(File::Release, " --deny-self-hosted-runners", "");
    has(&problems, want);

    let problems = provenance(
        File::Release,
        "for bin in parinn-*; do",
        "for bin in parinn-linux-amd64; do",
    );
    has(&problems, "does not verify every binary (parinn-*)");
}

#[test]
fn a_verify_by_regular_expression_is_named_unpinned() {
    let problems = provenance(
        File::Release,
        "--cert-identity https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/main",
        "--cert-identity-regex '^https://github.com/jeduden/parleyinn/'",
    );

    has(
        &problems,
        "job verify verifies an attestation with --cert-identity-regex, which pins no ref",
    );
}

#[test]
fn the_checksums_are_signed_keyless_and_published_with_their_bundle() {
    let r = drifted(
        "cosign sign-blob --bundle checksums.txt.sigstore.json checksums.txt",
        "cosign sign-blob --key k.pem --bundle checksums.txt.sigstore.json checksums.txt",
    );
    let problems = r.checksums_signed_keyless();
    has(&problems, "does not run");
    has(&problems, "signs with a key, not keyless");

    let r = drifted(
        "    needs: build\n    permissions:\n      id-token: write\n",
        "    needs: build\n    permissions:\n",
    );
    has(
        &r.checksums_signed_keyless(),
        "job publish lacks the id-token write permission",
    );

    let r = drifted(
        "parinn-* parinn.spdx.json checksums.txt checksums.txt.sigstore.json",
        "parinn-* parinn.spdx.json",
    );
    let problems = r.checksums_signed_keyless();
    has(&problems, "the release does not attach checksums.txt");
    has(
        &problems,
        "the release does not attach checksums.txt.sigstore.json",
    );
}

#[test]
fn the_verify_job_checks_the_bundle_against_the_release_workflow_s_identity_at_main() {
    let want = "job verify does not pin the certificate identity to \
                https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/main";

    let r = drifted("cosign verify-blob", "cosign version");
    has(
        &r.checksums_signed_keyless(),
        "job verify does not run cosign verify-blob",
    );

    let r = drifted(
        "--certificate-identity https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/main",
        "--certificate-identity-regexp '^https://github.com/jeduden/parleyinn/.github/workflows/release.yml@'",
    );
    has(&r.checksums_signed_keyless(), want);

    let r = drifted(
        "--certificate-identity https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/main",
        "--certificate-identity=https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/x",
    );
    has(&r.checksums_signed_keyless(), want);

    let r = drifted(
        "--certificate-oidc-issuer https://token.actions.githubusercontent.com",
        "--certificate-oidc-issuer https://example.com",
    );
    has(
        &r.checksums_signed_keyless(),
        "does not pin the OIDC issuer https://token.actions.githubusercontent.com",
    );

    let r = drifted(
        "cosign verify-blob --bundle checksums.txt.sigstore.json",
        "cosign verify-blob",
    );
    has(
        &r.checksums_signed_keyless(),
        "job verify verifies no checksums.txt.sigstore.json",
    );
}

#[test]
fn the_sbom_is_generated_as_spdx_attested_and_attached() {
    let r = drifted("syft . --output", "syft . --other");
    has(&r.sbom_attached_and_attested(), "does not generate an SBOM");

    let r = drifted("--output spdx-json=", "--output cyclonedx-json=");
    has(
        &r.sbom_attached_and_attested(),
        "job publish does not generate an SPDX SBOM",
    );

    let r = drifted(
        "--output spdx-json=dist/parinn.spdx.json",
        "--output spdx-json=",
    );
    has(
        &r.sbom_attached_and_attested(),
        "job publish names no SBOM file",
    );

    let r = drifted("uses: actions/attest-sbom@", "uses: actions/other@");
    has(&r.sbom_attached_and_attested(), "does not attest the SBOM");

    let r = drifted(
        "sbom-path: dist/parinn.spdx.json",
        "sbom-path: dist/other.spdx.json",
    );
    has(
        &r.sbom_attached_and_attested(),
        "does not attest the generated SBOM dist/parinn.spdx.json",
    );

    let r = drifted(
        "parinn-* parinn.spdx.json checksums.txt",
        "parinn-* checksums.txt",
    );
    has(
        &r.sbom_attached_and_attested(),
        "the release does not attach the SBOM parinn.spdx.json",
    );
}

#[test]
fn the_sbom_attestation_is_verified_against_the_release_workflow_at_main() {
    let want = "job verify does not verify the SBOM attestation \
                (--predicate-type https://spdx.dev/Document/v2.3) with \
                --cert-identity https://github.com/jeduden/parleyinn/.github/workflows/release.yml@refs/heads/main \
                --source-ref refs/heads/main";

    let r = drifted(" --predicate-type https://spdx.dev/Document/v2.3", "");
    has(&r.sbom_attached_and_attested(), want);

    let r = drifted(
        "--source-ref refs/heads/main --predicate-type",
        "--predicate-type",
    );
    has(&r.sbom_attached_and_attested(), want);
}

#[test]
fn flag_value_reads_a_separate_or_joined_value_without_quotes() {
    assert_eq!(flag_value("a --x 'v' b", "--x").as_deref(), Some("v"));
    assert_eq!(flag_value("a --x=\"v\" b", "--x").as_deref(), Some("v"));
    assert_eq!(flag_value("a --xy v", "--x"), None);
    assert_eq!(flag_value("a --x", "--x"), None);
}

#[test]
fn needs_reads_a_name_or_a_list() {
    let job = |text: &str| Job {
        name: "j".to_owned(),
        text: text.to_owned(),
    };

    assert_eq!(needs(&job("    needs: a\n")), ["a"]);
    assert_eq!(needs(&job("    needs: [a, b]\n")), ["a", "b"]);
    assert!(needs(&job("    runs-on: x\n")).is_empty());
}

#[test]
fn logical_joins_continued_lines_and_trims_indents() {
    assert_eq!(logical("  a \\\n    b \\\n    c\n  d\n"), "a b c\nd\n");
}
