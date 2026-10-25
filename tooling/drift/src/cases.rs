//! The registry: every drift a check that keeps the records in step
//! exists to catch.

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

/// The pinned action that attests build provenance.
const ATTEST_PROVENANCE: &str =
    "uses: actions/attest-build-provenance@4d101475d8b20a2381f78447822ac1eab6504dd8 # v4.2.2";

/// The step the release workflow attests its SBOM with.
const ATTEST_SBOM_STEP: &str =
    "        uses: actions/attest-sbom@c604332985a26aa8cf1bdc465b92731239ec6b9e # v4.1.0\n";

/// The ruleset for main the ENG-21 cases drift.
const RULESET: &str = ".github/rulesets/main.json";

/// The code owners the ENG-21 cases drift.
const CODEOWNERS: &str = ".github/CODEOWNERS";

/// The workflow the ENG-28 cases drift.
const REVIEW_WORKFLOW: &str = ".github/workflows/review.yml";

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
        complexity_cases(),
        toml_cases(),
        setup_cases(),
        release_cases(),
        protection_cases(),
        architecture_cases(),
        benchmark_cases(),
        isolation_cases(),
        instruction_cases(),
        transcript_contract_cases(),
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

/// The drifts ENG-18 exists to catch: the manifests and the dependency
/// ADRs disagreeing.
fn dependency_cases() -> Vec<Case> {
    vec![
        case(
            "a direct dependency no ADR justifies",
            "ENG-18",
            Injection::replace(TEST_STACK, "| `serde_json` |", "| `serde-json` |"),
            scenario("ENG-18"),
            "serde_json is named by 0 accepted ADRs",
        ),
        case(
            "the ADR justifying the test stack removed",
            "ENG-18",
            Injection::remove(TEST_STACK),
            scenario("ENG-18"),
            "cucumber is named by 0 accepted ADRs",
        ),
        case(
            "an ADR still justifying a module the manifests dropped",
            "ENG-18",
            Injection::replace(
                TEST_STACK,
                "| `serde_json`",
                "| `example.com/stale` | p | MIT | m |\n| `serde_json`",
            ),
            scenario("ENG-18"),
            "names example.com/stale, which is not a direct dependency",
        ),
        case(
            "a license off the allow-list",
            "ENG-18",
            Injection::replace(
                TEST_STACK,
                "MIT OR Apache-2.0 | Active; the de-facto Rust",
                "GPL-3.0 | Active; the de-facto Rust",
            ),
            scenario("ENG-18"),
            r#"serde is licensed "GPL-3.0", not on the allow-list"#,
        ),
        case(
            "more shipped dependencies than the target",
            "ENG-18",
            Injection::replace(
                "docs/srs/10-engineering-quality.md",
                "The target is ≤ 10 direct",
                "The target is ≤ 2 direct",
            ),
            scenario("ENG-18"),
            "direct dependencies linked into a shipped executable, want at most 2",
        ),
        case(
            "a fuzz dependency no ADR justifies",
            "ENG-18",
            Injection::replace(FUZZING, "| `libfuzzer-sys` |", "| `libfuzzer_sys` |"),
            scenario("ENG-18"),
            "libfuzzer-sys is named by 0 accepted ADRs",
        ),
        case(
            "a license beyond the allow-list with no exception for its crate",
            "ENG-18",
            Injection::replace(
                "deny.toml",
                r#"  { crate = "libfuzzer-sys", allow = ["NCSA"] },"#,
                "",
            ),
            scenario("ENG-18"),
            r#"libfuzzer-sys is licensed "(MIT OR Apache-2.0) AND NCSA", not on the allow-list"#,
        ),
    ]
}

/// The drifts ENG-26 exists to catch in the decision records.
fn decision_cases() -> Vec<Case> {
    vec![
        case(
            "two ADRs sharing an id",
            "ENG-26",
            Injection::copy(TEST_STACK, "docs/adr/ADR-2610142221-copy.md"),
            scenario("ENG-26"),
            "id ADR-2610142221 already used by",
        ),
        case(
            "an ADR superseded without a successor",
            "ENG-26",
            Injection::replace(TEST_STACK, "status: accepted", "status: superseded"),
            scenario("ENG-26"),
            r#"is superseded by "", which is no other ADR"#,
        ),
        case(
            "a superseded ADR left in force beside its successor",
            "ENG-26",
            Injection::replace(OLD_TEST_STACK, "status: superseded", "status: accepted"),
            scenario("ENG-26"),
            r#"ADR-2610101913 names successor ADR-2610142221 but its status is "accepted", not superseded"#,
        ),
        // Phase 3 of plan 2609292156 moves ADR-01 to ADR-10 into files
        // and makes this drift fail ENG-26.
        Case {
            known_gap: true,
            ..case(
                "the SRS citing an ADR with no file",
                "ENG-26",
                Injection::replace(
                    "docs/srs/01-introduction.md",
                    "## 1.1 Purpose\n",
                    "## 1.1 Purpose\n\nSee ADR-99.\n",
                ),
                scenario("ENG-26"),
                "ADR-99",
            )
        },
    ]
}

/// The drifts ENG-30 exists to catch: a license the tool keeps no text
/// for, a canonical text that no longer fits, and CI that no longer runs
/// the check.
fn license_text_cases() -> Vec<Case> {
    [license_check_cases(), license_edit_cases()].concat()
}

/// The license cases that edit no source the checks are built from.
fn license_check_cases() -> Vec<Case> {
    vec![
        case(
            "a license deny.toml grants with no canonical text",
            "ENG-30",
            Injection::replace(
                "deny.toml",
                r#"{ crate = "libfuzzer-sys", allow = ["NCSA"] },"#,
                r#"{ crate = "libfuzzer-sys", allow = ["NCSA", "WTFPL"] },"#,
            ),
            scenario("ENG-30"),
            "deny.toml grants WTFPL, which the license tool keeps no text for",
        ),
        case(
            "a license text the repository keeps for a linked crate",
            "ENG-30",
            Injection::create("vendor/serde_json/LICENSE-MIT", "MIT"),
            scenario("ENG-30"),
            "the repository keeps a license text at vendor/serde_json/LICENSE-MIT",
        ),
        case(
            "CI no longer runs the license check",
            "ENG-30",
            Injection::replace(CI_WORKFLOW, "--bin license-check", "--bin protection-check"),
            scenario("ENG-30"),
            "the CI workflow does not run license-check with --locked",
        ),
        case(
            "a target's committed license file out of step with the lockfile",
            "ENG-31",
            Injection::replace(
                "licenses/aarch64-unknown-linux-musl/THIRD_PARTY_LICENSES",
                "Third-party licenses of ParleyInn",
                "Third-party licenses of ParleyInn (stale)",
            ),
            scenario("ENG-31"),
            "licenses/aarch64-unknown-linux-musl/THIRD_PARTY_LICENSES differs from its regeneration",
        ),
        case(
            "a release target with no committed license file",
            "ENG-31",
            Injection::remove("licenses/aarch64-apple-darwin/THIRD_PARTY_LICENSES"),
            scenario("ENG-31"),
            "aarch64-apple-darwin has no committed file",
        ),
        case(
            "the release build no longer regenerating the license files",
            "ENG-31",
            Injection::replace(
                BUILD_WORKFLOW,
                "--bin license-check -- --out",
                "--bin license-check -- --write",
            ),
            scenario("ENG-31"),
            "the build workflow does not regenerate the license files",
        ),
        case(
            "the core no longer selecting a target's file",
            "ENG-31",
            Injection::replace(
                "crates/parinn/src/licenses.rs",
                r#"include_str!("../../../licenses/aarch64-apple-darwin/THIRD_PARTY_LICENSES")"#,
                r#"include_str!("../../../licenses/aarch64-unknown-linux-musl/THIRD_PARTY_LICENSES")"#,
            ),
            scenario("ENG-31"),
            "the core does not select licenses/aarch64-apple-darwin/THIRD_PARTY_LICENSES",
        ),
    ]
}

/// The license cases that edit source the checks are built from, which
/// run last in the order they are listed.
fn license_edit_cases() -> Vec<Case> {
    vec![
        // The two cases below are last. The suite restores the checkout
        // with its original file times, so cargo keeps the build of the
        // edited source for the cases that follow: the edit to the core
        // costs the ENG-31 scenario its step that runs the core, and the
        // edit to the canonical text, which is compiled into the check,
        // fails every license scenario at its text step.
        case(
            "the core no longer answering parinn licenses",
            "licenses_is_a_command_that_prints_the_embedded_texts",
            Injection::replace(
                "crates/parinn/src/lib.rs",
                r#"Some("licenses") =>"#,
                r#"Some("licences") =>"#,
            ),
            cargo_lib_test(
                "parinn",
                "tests::licenses_is_a_command_that_prints_the_embedded_texts",
            ),
            "licenses_is_a_command_that_prints_the_embedded_texts",
        ),
        case(
            "a canonical text that no longer fits the license files",
            "ENG-30",
            Injection::replace(
                "tooling/engineering/src/license_texts/canon/MIT.txt",
                "Permission is hereby granted",
                "Permission is not granted",
            ),
            scenario("ENG-30"),
            "declares MIT, but no license file fits MIT",
        ),
    ]
}

/// The drifts the other repository-inspecting scenarios catch.
fn repository_cases() -> Vec<Case> {
    vec![
        case(
            "release builds no longer static",
            "ENG-01",
            Injection::replace(
                ".github/workflows/build.yml",
                "    x86_64-unknown-linux-musl:linux-amd64",
                "    x86_64-unknown-linux-gnu:linux-amd64",
            ),
            scenario("ENG-01"),
            r#"the release build does not contain "x86_64-unknown-linux-musl""#,
        ),
        case(
            "release builds keeping the builder's paths",
            "ENG-01",
            Injection::replace(
                ".github/workflows/build.yml",
                "base=\"--remap-path-prefix=$PWD=. --remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo\"",
                "base=\"\"",
            ),
            scenario("ENG-01"),
            r#"the release build does not contain "--remap-path-prefix""#,
        ),
        case(
            "the Rust toolchain no longer pinned to a release",
            "ENG-01",
            Injection::replace(
                "rust-toolchain.toml",
                r#"channel = "1.99.0""#,
                r#"channel = "stable""#,
            ),
            scenario("ENG-01"),
            r#"rust-toolchain.toml does not pin an exact release: "stable""#,
        ),
        case(
            "the disclosure policy dropped",
            "ENG-24",
            Injection::replace(
                "SECURITY.md",
                "We follow a 90-day coordinated disclosure policy.",
                "We follow a disclosure policy.",
            ),
            scenario("ENG-24"),
            r#"does not contain "90-day coordinated disclosure""#,
        ),
        case(
            "S6's finding dropped from OQ-01",
            "ASM-09",
            Injection::replace(
                "docs/srs/13-open-questions-and-risks.md",
                "every automatic one (ASM-09)",
                "every automatic one",
            ),
            scenario("ASM-09"),
            "the OQ-01 row does not record that exit status 2 blocks every compaction (ASM-09)",
        ),
        case(
            "the drift suite dropped from CI",
            "ENG-27",
            Injection::replace(
                ".github/workflows/ci.yml",
                "cargo test --locked -p drift --test suite -- --ignored",
                "true",
            ),
            scenario("ENG-27"),
            r#"does not contain "cargo test --locked -p drift --test suite -- --ignored""#,
        ),
    ]
}

/// The drifts ENG-28 exists to catch in the reviewing agent's reach: the
/// agent reaching the approval. [`review_gate_cases`] holds the approval
/// skipping the review gate.
fn review_cases() -> Vec<Case> {
    vec![
        case(
            "the reviewer run as the pull request defines it",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "  workflow_run:\n",
                "  pull_request_target:\n  workflow_run:\n",
            ),
            scenario("ENG-28"),
            "triggers on [pull_request_target workflow_run issue_comment], want workflow_run and issue_comment",
        ),
        case(
            "the reviewing agent granted a write permission",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "      pull-requests: read\n",
                "      pull-requests: write\n",
            ),
            scenario("ENG-28"),
            "agent job review is granted a write permission",
        ),
        case(
            "the reviewing agent handed the reviewer app's key",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "          github_token: ${{ github.token }}\n",
                "          github_token: ${{ secrets.JEDUDEN_REVIEW_AGENT_KEY }}\n",
            ),
            scenario("ENG-28"),
            "agent job review reads JEDUDEN_REVIEW_AGENT_KEY",
        ),
        case(
            "the review posted after the agent failed",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "    if: needs.review.outputs.pull != ''\n",
                "    if: always() && needs.review.outputs.pull != ''\n",
            ),
            scenario("ENG-28"),
            "job post runs on always(), even after a failed review",
        ),
    ]
}

/// The drifts ENG-28 exists to catch in what the approval rests on: the
/// review gate, the deterministic findings and the review bundle.
fn review_gate_cases() -> Vec<Case> {
    vec![
        case(
            "the review posted without the review gate",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "cargo run --locked --quiet -p review-gate --offline -- --outcome",
                "cp outcome.json review.json; true --outcome",
            ),
            scenario("ENG-28"),
            r#"job post does not run "cargo run --locked --quiet -p review-gate""#,
        ),
        case(
            "the review posted without the deterministic findings",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "            --checked checked.json > review.json",
                "            > review.json",
            ),
            scenario("ENG-28"),
            r#"job post does not run "--checked checked.json""#,
        ),
        // The test asserts with its own message, the findings, which
        // lack the PRV-11 finding once it drifts; only the failing
        // test's banner names it.
        case(
            "ENG-29 no longer naming a requirement marked for I2 review",
            "ENG-29",
            Injection::replace(
                "docs/srs/10-engineering-quality.md",
                "(PRV-10, PRV-11, PRV-12, PRV-13)",
                "(PRV-10, PRV-12, PRV-13)",
            ),
            cargo_test(
                "review-bundle",
                "bundle",
                "the_requirements_eng_29_marks_follow_its_row",
            ),
            "---- the_requirements_eng_29_marks_follow_its_row stdout ----",
        ),
        // The SRS no longer a stakeholder path, a change to it routes
        // light; the failure prints the class the bundle chose.
        case(
            "CODEOWNERS no longer holding the SRS",
            "ENG-28",
            Injection::replace(
                ".github/CODEOWNERS",
                "/docs/srs/                    @jeduden",
                "/docs/srsx/                   @jeduden",
            ),
            cargo_test(
                "review-bundle",
                "e2e_review_bundle",
                "the_executable_bundles_this_repository_against_itself",
            ),
            "class=light",
        ),
    ]
}

/// The drifts the gate tests and mdsmith catch between the SRS, the
/// scenarios and the generated sections.
fn gate_cases() -> Vec<Case> {
    vec![
        case(
            "a requirement left without its scenario",
            "specification_and_features_agree",
            Injection::replace(
                "features/engineering.feature",
                "@ENG-25 @P1 @pending",
                "@ENG-99 @P1 @pending",
            ),
            cargo_test("scenario", "gate", "specification_and_features_agree"),
            "ENG-25: no scenario under features/",
        ),
        case(
            "a scenario's priority off its requirement's",
            "specification_and_features_agree",
            Injection::replace(
                "features/engineering.feature",
                "@ENG-25 @P1 @pending",
                "@ENG-25 @P0 @pending",
            ),
            cargo_test("scenario", "gate", "specification_and_features_agree"),
            r#"ENG-25: scenario priority "P0", requirement says "P1""#,
        ),
        case(
            "a requirement demoted without Appendix B's count",
            "appendix_b_matches_the_traces",
            Injection::replace(
                "docs/srs/10-engineering-quality.md",
                "| ENG-25 | P1  |",
                "| ENG-25 | P2  |",
            ),
            cargo_test("srs", "gates", "appendix_b_matches_the_traces"),
            "Appendix B's requirement counts",
        ),
        case(
            "an ADR edited without regenerating DEPENDENCIES.md",
            "mdsmith check",
            Injection::replace(
                TEST_STACK,
                "serde_json reads and writes JSON in the tooling",
                "serde_json reads and writes JSON in tooling",
            ),
            mdsmith(),
            "generated section is out of date",
        ),
    ]
}

/// The drifts the parse gates catch: the specification or a decision
/// record no longer reading cleanly.
fn parse_cases() -> Vec<Case> {
    vec![
        case(
            "a requirement id defined twice",
            "specification_parses",
            Injection::replace(
                "docs/srs/10-engineering-quality.md",
                "| ENG-25 | P1  |",
                "| ENG-24 | P1  |",
            ),
            cargo_test("srs", "gates", "specification_parses"),
            "ENG-24 already defined at",
        ),
        case(
            "a decision record whose front matter no longer reads",
            "decision_records_parse",
            Injection::replace(TEST_STACK, "id: ADR-2610142221", "id ADR-2610142221"),
            cargo_test("adr", "records", "decision_records_parse"),
            "malformed front matter",
        ),
    ]
}

/// The drifts the persona gates catch between §2.5, Appendix C and the
/// persona agents.
fn persona_cases() -> Vec<Case> {
    const APPENDIX_C: &str = "docs/srs/appendix-c-persona-coverage.md";
    let appendix_c_check = || cargo_test("srs", "gates", "appendix_c_covers_every_requirement");
    let agents_check = || cargo_test("srs", "gates", "personas_match_the_agents");
    vec![
        case(
            "a requirement serving no persona",
            "appendix_c_covers_every_requirement",
            Injection::replace(APPENDIX_C, "| SEC-17, ENG-29", "| SEC-17"),
            appendix_c_check(),
            "ENG-29 serves no persona in Appendix C",
        ),
        case(
            "Appendix C listing an id that is no requirement",
            "appendix_c_covers_every_requirement",
            Injection::replace(APPENDIX_C, "SEC-17, ENG-29", "SEC-17, ENG-29, ENG-98"),
            appendix_c_check(),
            "Appendix C lists ENG-98, which is no requirement",
        ),
        case(
            "Appendix C naming a persona §2.5 does not define",
            "appendix_c_covers_every_requirement",
            Injection::replace(
                "docs/srs/02-context.md",
                "\n| U9  | Agent",
                "\n\n| U9  | Agent",
            ),
            appendix_c_check(),
            "Appendix C names U9, which §2.5 does not define",
        ),
        case(
            "a persona agent with no persona",
            "personas_match_the_agents",
            Injection::copy(
                ".claude/agents/persona-agent.md",
                ".claude/agents/persona-stray.md",
            ),
            agents_check(),
            "persona-stray",
        ),
        case(
            "a persona whose agent file is gone",
            "personas_match_the_agents",
            Injection::remove(".claude/agents/persona-reviewer.md"),
            agents_check(),
            "persona-reviewer",
        ),
    ]
}

/// The drifts mdsmith's agent schemas catch: a reviewing agent given a
/// tool beyond reading, and the domain-model agent losing one of its
/// required sections.
fn agent_cases() -> Vec<Case> {
    vec![
        case(
            "a reviewing agent granted a write tool",
            "mdsmith check",
            Injection::replace(
                ".claude/agents/persona-reviewer.md",
                "tools: Read, Grep, Glob",
                "tools: Read, Grep, Glob, Edit",
            ),
            mdsmith(),
            "tools: got",
        ),
        case(
            "the domain-model agent losing a required section",
            "mdsmith check",
            Injection::replace(
                ".claude/agents/domain-model.md",
                "## How you report",
                "## Reporting",
            ),
            mdsmith(),
            "How you report",
        ),
    ]
}

/// The drifts mdsmith's domain-model schemas catch: the hub losing one
/// of the sections that span every concept group, and a concept file
/// losing the summary the hub's catalog reads.
fn domain_model_cases() -> Vec<Case> {
    vec![
        case(
            "the domain-model hub losing a required section",
            "mdsmith check",
            Injection::replace(
                "docs/domain-model/index.md",
                "## Not ParleyInn concepts",
                "## Excluded terms",
            ),
            mdsmith(),
            "Not ParleyInn concepts",
        ),
        case(
            "a domain-model concept file losing its summary",
            "mdsmith check",
            Injection::replace("docs/domain-model/places.md", "summary: >-", "abstract: >-"),
            mdsmith(),
            "summary",
        ),
    ]
}

/// The drift the finding ledger's check exists to catch: a closed
/// domain-model finding whose closing sentence leaves the text.
fn ledger_cases() -> Vec<Case> {
    vec![case(
        "a closed finding's sentence trimmed from the model",
        "finding_ledger_is_carried",
        Injection::replace(
            "docs/domain-model/components-and-surfaces.md",
            "What shows rooms to a person: the browser, through the",
            "What shows rooms to a person: the",
        ),
        cargo_test("finding-ledger", "carried", "finding_ledger_is_carried"),
        "closed findings no longer carried in the text",
    )]
}

/// The drifts the core executable's reach evidence exists to catch
/// (SEC-01, ENG-16): a crate linked in, or the core's own code opening
/// a socket or starting a program.
fn reach_cases() -> Vec<Case> {
    vec![
        case(
            "a crate linked into the core executable",
            "the_core_executable_links_only_crates_its_reach_allows",
            Injection::replace(
                "crates/parinn/Cargo.toml",
                "tracing.workspace = true\n\n[dev-dependencies]\nrusqlite.workspace = true\ntestkit.workspace = true\n",
                "tracing.workspace = true\ntestkit.workspace = true\n\n[dev-dependencies]\nrusqlite.workspace = true\n",
            ),
            cargo_test(
                "parinn",
                "reach",
                "the_core_executable_links_only_crates_its_reach_allows",
            ),
            "parinn links crates its reach does not allow",
        ),
        case(
            "the core executable opening a socket",
            "SEC-01",
            Injection::replace(
                "crates/parinn/src/lib.rs",
                "use std::io::{self, Write};",
                "use std::io::{self, Write};\n\npub fn dial() -> Option<std::net::TcpStream> {\n    None\n}",
            ),
            clippy("parinn"),
            "use of a disallowed type `std::net::TcpStream`",
        ),
    ]
}

/// The drifts ENG-02 exists to catch: a crate depending where its
/// manifest's declared directions forbid, and a manifest leaving out a
/// direction or a responsibility. The edits keep Cargo.lock as it is,
/// so the checks run under `--locked`.
fn architecture_cases() -> Vec<Case> {
    const GATE: &str = "the_workspace_keeps_its_declared_directions";
    let gate = || cargo_test("engineering", "checkout", GATE);
    vec![
        case(
            "the core executable depending on a tooling crate",
            "ENG-02",
            Injection::replace(
                "crates/parinn/Cargo.toml",
                "tracing.workspace = true\n\n[dev-dependencies]\nrusqlite.workspace = true\ntestkit.workspace = true\n",
                "tracing.workspace = true\ntestkit.workspace = true\n\n[dev-dependencies]\nrusqlite.workspace = true\n",
            ),
            scenario("ENG-02"),
            "product crate parinn depends on tooling crate testkit, which a product crate never does",
        ),
        case(
            "a tooling crate depending where its declared directions forbid",
            GATE,
            Injection::replace(
                "tooling/adr/Cargo.toml",
                "may-depend-on = [\"srs\"]",
                "may-depend-on = []",
            ),
            gate(),
            "crate adr depends on crate srs, which its declared directions forbid",
        ),
        case(
            "a crate declaring no directions",
            GATE,
            Injection::replace("tooling/drift/Cargo.toml", "may-depend-on = []\n", ""),
            gate(),
            "crate drift declares no may-depend-on directions",
        ),
        case(
            "a library crate declaring no responsibility",
            "ENG-02",
            Injection::replace(
                "tooling/srs/Cargo.toml",
                "responsibility = \"Reads the SRS requirement tables, appendices and personas for the gates\"\n",
                "",
            ),
            scenario("ENG-02"),
            "library crate srs declares no responsibility",
        ),
    ]
}

/// The drifts the product crates' reach and determinism bans exist to
/// catch: a crate's clippy.toml dropping one (I4, I10), and the core
/// reading a clock or a random order (I10). [`deadline_cases`] holds
/// ENG-03's.
fn product_crate_cases() -> Vec<Case> {
    const LIB: &str = "crates/parinn/src/lib.rs";
    const AT: &str = "use std::io::{self, Write};";
    vec![
        case(
            "a product crate's clippy.toml dropping a reach ban",
            "the_product_crates_keep_their_bans",
            Injection::replace(
                "crates/parinn/clippy.toml",
                "  { path = \"std::net::UdpSocket\", reason = \"the core opens no socket (SEC-01)\" },\n",
                "",
            ),
            cargo_test(
                "engineering",
                "checkout",
                "the_product_crates_keep_their_bans",
            ),
            "crates/parinn/clippy.toml does not ban std::net::UdpSocket in disallowed-types",
        ),
        case(
            "a product crate's clippy.toml dropping a determinism ban",
            "I10",
            Injection::replace(
                "crates/parinn/clippy.toml",
                "  { path = \"std::time::SystemTime::now\", reason = \"derived artifacts read no clock (I10)\" },\n",
                "",
            ),
            cargo_test(
                "engineering",
                "checkout",
                "the_product_crates_keep_their_bans",
            ),
            "crates/parinn/clippy.toml does not ban std::time::SystemTime::now in disallowed-methods",
        ),
        case(
            "the core reading the clock",
            "I10",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub fn now() -> std::time::Instant {\n    std::time::Instant::now()\n}",
            ),
            clippy("parinn"),
            "use of a disallowed method `std::time::Instant::now`",
        ),
        case(
            "the core keeping a hash map's random order",
            "I10",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub type Index = std::collections::HashMap<u8, u8>;",
            ),
            clippy("parinn"),
            "use of a disallowed type `std::collections::HashMap`",
        ),
    ]
}

/// The drifts ENG-03's deadline bans exist to catch: a product crate's
/// clippy.toml dropping one. [`core_wait_cases`] holds the core's code.
fn deadline_cases() -> Vec<Case> {
    vec![
        case(
            "a product crate's clippy.toml dropping a deadline ban",
            "the_product_crates_ban_every_wait_with_no_deadline",
            Injection::replace(
                "crates/parinn/clippy.toml",
                "  { path = \"std::sync::mpsc::Receiver::recv\", reason = \"it waits with no deadline; use recv_timeout (ENG-03)\" },\n",
                "",
            ),
            cargo_test(
                "engineering",
                "checkout",
                "the_product_crates_ban_every_wait_with_no_deadline",
            ),
            "crates/parinn/clippy.toml does not ban std::sync::mpsc::Receiver::recv in disallowed-methods",
        ),
        case(
            "a product crate's clippy.toml dropping the file lock's deadline ban",
            "the_product_crates_ban_every_wait_with_no_deadline",
            Injection::replace(
                "crates/parinn/clippy.toml",
                "  { path = \"std::fs::File::lock\", reason = \"it waits for another process with no deadline; poll try_lock up to a deadline (ENG-03)\" },\n",
                "",
            ),
            cargo_test(
                "engineering",
                "checkout",
                "the_product_crates_ban_every_wait_with_no_deadline",
            ),
            "crates/parinn/clippy.toml does not ban std::fs::File::lock in disallowed-methods",
        ),
    ]
}

/// The drifts ENG-03 exists to catch in the core's code: a wait with no
/// deadline, and mutable global state.
fn core_wait_cases() -> Vec<Case> {
    const LIB: &str = "crates/parinn/src/lib.rs";
    const AT: &str = "use std::io::{self, Write};";
    vec![
        case(
            "the core waiting on a channel with no deadline",
            "ENG-03",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub fn wait(rx: &std::sync::mpsc::Receiver<u8>) -> Option<u8> {\n    rx.recv().ok()\n}",
            ),
            clippy("parinn"),
            "use of a disallowed method `std::sync::mpsc::Receiver::recv`",
        ),
        case(
            "the core joining a thread with no deadline",
            "ENG-03",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub fn wait(h: std::thread::JoinHandle<()>) {\n    let _ = h.join();\n}",
            ),
            clippy("parinn"),
            "use of a disallowed method `std::thread::JoinHandle::join`",
        ),
        case(
            "the core reading standard input with no deadline",
            "ENG-03",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub fn input() -> std::io::Stdin {\n    std::io::stdin()\n}",
            ),
            clippy("parinn"),
            "use of a disallowed method `std::io::stdin`",
        ),
        case(
            "a product crate keeping mutable global state",
            "no_product_crate_keeps_mutable_global_state",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);",
            ),
            cargo_test(
                "engineering",
                "checkout",
                "no_product_crate_keeps_mutable_global_state",
            ),
            "crates/parinn/src/lib.rs:16: mutable global state: pub static SEQ",
        ),
    ]
}

/// The drifts ENG-12's core row exists to catch: the core started
/// outside the end-to-end suite's confinement, and the core opening a
/// socket, which the confined suite fails naming the core and the test.
fn confinement_cases() -> Vec<Case> {
    vec![
        case(
            "the core started outside the end-to-end suite's confinement",
            "the_core_s_tests_start_it_confined",
            Injection::replace(
                "crates/parinn/tests/e2e_parinn.rs",
                "use testkit::boundary::{Component, Confinement};",
                "use testkit::boundary::{Component, Confinement};\n\nfn _unconfined() -> std::process::Command {\n    std::process::Command::new(env!(\"CARGO_BIN_EXE_parinn\"))\n}",
            ),
            cargo_test(
                "engineering",
                "confinement",
                "the_core_s_tests_start_it_confined",
            ),
            "crates/parinn/tests/e2e_parinn.rs starts a component's executable outside the end-to-end suite's confinement (ENG-12)",
        ),
        case(
            "the core opening a socket in the confined suite",
            "ENG-12",
            Injection::replace(
                "crates/parinn/src/lib.rs",
                "        Some(\"version\") => version(args, ctx.build, stdout, stderr),",
                "        Some(\"version\") => {\n            let _ = std::net::UdpSocket::bind((\"127.0.0.1\", 0));\n            version(args, ctx.build, stdout, stderr)\n        }",
            ),
            cargo_test("parinn", "e2e_parinn", "version_prints_the_build_s_version"),
            "ENG-12: the core opened a socket outside its boundary, which denies all network; started by version_prints_the_build_s_version",
        ),
    ]
}

/// The drifts ENG-16's core row exists to catch in what CI gates on: a
/// static check no longer run, left out of the ci job or allowed to
/// fail, and the ci job passing after a failed job.
fn ci_gate_cases() -> Vec<Case> {
    const CI: &str = ".github/workflows/ci.yml";
    let gates = || {
        cargo_test(
            "engineering",
            "static_analysis",
            "the_ci_workflow_gates_on_the_static_checks",
        )
    };
    vec![
        case(
            "CI no longer running cargo-audit",
            "the_ci_workflow_gates_on_the_static_checks",
            Injection::replace(
                CI,
                "      - run: cargo audit\n      - run: cargo audit --file fuzz/Cargo.lock\n",
                "",
            ),
            gates(),
            "no CI job runs cargo-audit: cargo audit",
        ),
        case(
            "a static check's job left out of the ci job",
            "the_ci_workflow_gates_on_the_static_checks",
            Injection::replace(CI, "coverage, lint, supply-chain", "coverage, supply-chain"),
            gates(),
            "job lint runs cargo fmt, but the ci job does not need it",
        ),
        case(
            "a static check's job allowed to fail",
            "the_ci_workflow_gates_on_the_static_checks",
            Injection::replace(
                CI,
                "  lint:\n    runs-on: ubuntu-latest\n",
                "  lint:\n    runs-on: ubuntu-latest\n    continue-on-error: true\n",
            ),
            gates(),
            "job lint runs cargo fmt, but may fail without failing CI: continue-on-error",
        ),
        case(
            "the ci job passing after a failed job",
            "the_ci_workflow_gates_on_the_static_checks",
            Injection::replace(CI, "    if: ${{ always() }}\n    needs: [", "    needs: ["),
            gates(),
            "the ci job does not run after a failed job",
        ),
    ]
}

/// The drifts ENG-16's core row exists to catch in the code itself: a
/// crate switching the unsafe ban off, and the core's reach evidence or
/// the determinism bans going missing.
fn static_analysis_cases() -> Vec<Case> {
    const CLIPPY: &str = "crates/parinn/clippy.toml";
    let check = |name| cargo_test("engineering", "static_analysis", name);
    let unsafe_ban = || check("every_crate_forbids_unsafe_code");
    let reach = || check("the_core_s_reach_evidence_is_present_and_gated");
    vec![
        case(
            "a crate no longer inheriting the workspace lints",
            "every_crate_forbids_unsafe_code",
            Injection::replace(
                "crates/parinn/Cargo.toml",
                "[lints]\nworkspace = true\n",
                "",
            ),
            unsafe_ban(),
            "crate crates/parinn neither inherits the workspace lints nor forbids unsafe code in src/lib.rs",
        ),
        case(
            "the unsafe shim no longer denying unsafe code",
            "every_crate_forbids_unsafe_code",
            Injection::replace(
                "crates/platform/Cargo.toml",
                "unsafe_code = \"deny\"",
                "unsafe_code = \"allow\"",
            ),
            unsafe_ban(),
            "crate crates/platform, the one unsafe shim, does not deny unsafe code in its own lints",
        ),
        case(
            "the workspace lints no longer forbidding unsafe code",
            "every_crate_forbids_unsafe_code",
            Injection::replace(
                "Cargo.toml",
                "unsafe_code = \"forbid\"",
                "unsafe_code = \"deny\"",
            ),
            unsafe_ban(),
            "the workspace lints do not forbid unsafe_code",
        ),
        case(
            "the core's reach test removed",
            "the_core_s_reach_evidence_is_present_and_gated",
            Injection::remove("crates/parinn/tests/reach.rs"),
            reach(),
            "the core's reach evidence is missing: read crates/parinn/tests/reach.rs",
        ),
        case(
            "the core's clippy.toml no longer banning a program start",
            "the_core_s_reach_evidence_is_present_and_gated",
            Injection::replace(
                CLIPPY,
                "  { path = \"std::process::Command\", reason = \"the core starts no program but its kernel worker (SEC-01, CMP-05)\" },\n",
                "",
            ),
            reach(),
            "the core's reach evidence is missing: crates/parinn/clippy.toml does not ban std::process::Command",
        ),
        case(
            "code that computes derived artifacts no longer refused the monotonic clock",
            "derived_code_is_refused_a_clock_or_randomness_at_build_time",
            Injection::replace(
                CLIPPY,
                "  { path = \"std::time::Instant::now\", reason = \"derived artifacts read no clock (I10)\" },\n",
                "",
            ),
            check("derived_code_is_refused_a_clock_or_randomness_at_build_time"),
            "code that computes derived artifacts may read a clock or randomness: crate parinn: crates/parinn/clippy.toml does not ban std::time::Instant::now in disallowed-methods",
        ),
    ]
}

/// The tool setup installs only released binaries: a tool whose backend
/// would build from source is refused before anything downloads.
fn setup_cases() -> Vec<Case> {
    vec![case(
        "a tool that builds from source",
        "scripts/setup-tools.sh --check",
        Injection::replace(
            "mise.toml",
            "[tools]\n",
            "[tools]\n\"cargo:cargo-audit\" = \"0.22.2\"\n",
        ),
        setup_tools(),
        "refusing tools that are not released binaries",
    )]
}

/// The drifts the TOML checks exist to catch: CI no longer running one
/// or no longer gating on its job, which the gate test catches, and a
/// misspelt manifest key and a TOML file out of its layout, which taplo
/// itself catches.
fn toml_cases() -> Vec<Case> {
    const CI: &str = ".github/workflows/ci.yml";
    const GATE: &str = "the_ci_workflow_gates_on_the_toml_checks";
    let gate = || cargo_test("engineering", "static_analysis", GATE);
    vec![
        case(
            "CI no longer running taplo lint",
            GATE,
            Injection::replace(CI, "      - run: taplo lint\n", ""),
            gate(),
            "no CI job runs taplo lint: taplo lint",
        ),
        case(
            "the toml job left out of the ci job",
            GATE,
            Injection::replace(CI, "markdown, toml, workflows", "markdown, workflows"),
            gate(),
            "job toml runs taplo fmt, but the ci job does not need it",
        ),
        case(
            "a manifest key misspelt",
            "taplo lint",
            Injection::replace(
                "crates/parinn/Cargo.toml",
                "edition.workspace = true\n",
                "edition.workspace = true\nlicence = \"MIT\"\n",
            ),
            taplo_lint(),
            "'licence' was unexpected",
        ),
        case(
            "a TOML file out of its layout",
            "taplo fmt",
            Injection::replace(
                "Cargo.toml",
                "unsafe_code = \"forbid\"",
                "unsafe_code=\"forbid\"",
            ),
            taplo_fmt(),
            "the file is not properly formatted",
        ),
    ]
}

/// The drifts the complexity gate exists to catch: the workspace lints
/// no longer denying a complexity lint, and a crate's clippy.toml
/// raising a ceiling, dropping one or switching a lint off. Clippy
/// itself, which CI gates on (ENG-16), catches the core nesting a block
/// past a ceiling.
fn complexity_cases() -> Vec<Case> {
    const GATE: &str = "every_crate_holds_the_complexity_limits";
    let gate = || cargo_test("engineering", "static_analysis", GATE);
    vec![
        case(
            "the workspace lints no longer denying too_many_lines",
            GATE,
            Injection::replace("Cargo.toml", "too_many_lines = \"deny\"\n", ""),
            gate(),
            "the workspace lints do not deny clippy::too_many_lines",
        ),
        case(
            "the unsafe shim no longer denying too_many_lines",
            GATE,
            Injection::replace(
                "crates/platform/Cargo.toml",
                "too_many_lines = \"deny\"\n",
                "",
            ),
            gate(),
            "crate crates/platform, the one unsafe shim, does not deny clippy::too_many_lines",
        ),
        case(
            "a product crate's clippy.toml raising a complexity ceiling",
            GATE,
            Injection::replace(
                "crates/parinn/clippy.toml",
                "too-many-lines-threshold = 80",
                "too-many-lines-threshold = 200",
            ),
            gate(),
            "crates/parinn/clippy.toml sets too-many-lines-threshold to 200, above 80",
        ),
        case(
            "the workspace's clippy.toml dropping a complexity ceiling",
            GATE,
            Injection::replace("clippy.toml", "excessive-nesting-threshold = 5\n", ""),
            gate(),
            "clippy.toml sets no excessive-nesting-threshold",
        ),
        case(
            "a product crate's clippy.toml switching excessive_nesting off",
            GATE,
            Injection::replace(
                "crates/parinn/clippy.toml",
                "excessive-nesting-threshold = 5",
                "excessive-nesting-threshold = 0",
            ),
            gate(),
            "crates/parinn/clippy.toml sets excessive-nesting-threshold to 0, which switches the lint off",
        ),
        case(
            "the core nesting a block past the ceiling",
            "ENG-16",
            Injection::replace(
                "crates/parinn/src/lib.rs",
                "use std::io::{self, Write};",
                "use std::io::{self, Write};\n\npub fn deep(x: u8) -> u8 {\n    if x > 0 { if x > 1 { if x > 2 { if x > 3 { if x > 4 { if x > 5 { return 6; } } } } } }\n    0\n}",
            ),
            clippy("parinn"),
            "this block is too nested",
        ),
    ]
}

/// A case for ENG-20 caught by the gate test that reads the same
/// workflows as the scenario.
fn release_case(
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
const CHECKSUMS_GATE: &str = "the_checkout_s_release_signs_its_checksums_keyless";

/// The gate test of ENG-20's build provenance.
const PROVENANCE_GATE: &str =
    "the_checkout_s_release_attests_provenance_only_in_the_release_build_workflow";

/// The gate test of ENG-20's SBOM.
const SBOM_GATE: &str = "the_checkout_s_release_attaches_and_attests_an_sbom";

/// The drifts ENG-20 exists to catch: provenance signed anywhere but
/// the release-only reusable workflow, a signing permission granted to
/// the build CI runs, a verification that accepts a signer at any ref,
/// a missing check of the published release, and an SBOM that is not
/// attested.
fn release_cases() -> Vec<Case> {
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
fn release_signer_cases() -> Vec<Case> {
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
            Injection::replace(
                RELEASE_WORKFLOW,
                ATTEST_SBOM_STEP,
                "        uses: actions/attest-build-provenance@4d101475d8b20a2381f78447822ac1eab6504dd8 # v4.2.2\n        with:\n          subject-path: \"dist/parinn-*\"\n      - name: Attest the SBOM again\n        uses: actions/attest-sbom@c604332985a26aa8cf1bdc465b92731239ec6b9e # v4.1.0\n",
            ),
            PROVENANCE_GATE,
            "release.yml attests build provenance outside release-build.yml, in job publish",
        ),
        release_case(
            "the build workflow CI runs attesting build provenance",
            Injection::replace(
                BUILD_WORKFLOW,
                "        run: scripts/check-stamp.sh dist/parinn-windows-amd64.exe \"$VERSION\" \"$COMMIT\"\n",
                "        run: scripts/check-stamp.sh dist/parinn-windows-amd64.exe \"$VERSION\" \"$COMMIT\"\n\n  attest:\n    needs: windows\n    runs-on: ubuntu-latest\n    permissions:\n      id-token: write\n      attestations: write\n    steps:\n      - uses: actions/attest-build-provenance@4d101475d8b20a2381f78447822ac1eab6504dd8 # v4.2.2\n        with:\n          subject-path: \"dist/parinn-*\"\n",
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
fn release_build_cases() -> Vec<Case> {
    vec![
        release_case(
            "the release build workflow no longer attesting provenance",
            Injection::replace(
                RELEASE_BUILD_WORKFLOW,
                ATTEST_PROVENANCE,
                "uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1",
            ),
            PROVENANCE_GATE,
            "release-build.yml does not attest build provenance",
        ),
        release_case(
            "the attest job checking out the repository's code",
            Injection::replace(
                RELEASE_BUILD_WORKFLOW,
                "    steps:\n      - uses: actions/download-artifact@",
                "    steps:\n      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1\n      - uses: actions/download-artifact@",
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
fn sbom_format_case() -> Case {
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
fn release_verify_cases() -> Vec<Case> {
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
            Injection::replace(
                RELEASE_WORKFLOW,
                ATTEST_SBOM_STEP,
                "        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1\n",
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
fn protection_case(
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
fn mise_owner_case() -> Case {
    protection_case(
        "the mise toolchain pin without the stakeholder",
        Injection::replace(CODEOWNERS, "/mise.toml                    @jeduden\n", ""),
        "the_checkout_s_codeowners_name_the_stakeholder_on_the_enforcing_paths",
        "mise.toml (gate, CI,) is owned by [], want @jeduden",
    )
}

/// The developer tool pins, which CI's install step reads, must remain
/// owned by the stakeholder.
fn mise_dev_owner_case() -> Case {
    protection_case(
        "the developer tool pin without the stakeholder",
        Injection::replace(CODEOWNERS, "/mise.dev.toml                @jeduden\n", ""),
        "the_checkout_s_codeowners_name_the_stakeholder_on_the_enforcing_paths",
        "mise.dev.toml (gate, CI,) is owned by [], want @jeduden",
    )
}

fn signed_commits_case() -> Case {
    protection_case(
        "commits no longer required to be signed",
        Injection::replace(RULESET, "    { \"type\": \"required_signatures\" },\n", ""),
        "the_checkout_s_ruleset_requires_signed_commits",
        "the ruleset has no required_signatures rule",
    )
}

/// The drifts ENG-21 exists to catch: main's committed ruleset or
/// CODEOWNERS weaker than the requirement says.
fn protection_cases() -> Vec<Case> {
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
        protection_case(
            "a security-sensitive crate without the security reviewer",
            Injection::replace(CODEOWNERS, "/crates/pin/                  @jeduden\n", ""),
            "the_checkout_s_security_crates_belong_to_the_security_reviewer",
            "crates/pin/ is not owned by @jeduden",
        ),
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

/// The drifts the release-build gate exists to catch: the benchmark
/// job building or running the suite in debug, and the NFR-02 budget row
/// timing the development core.
/// The drifts of the calibration workload the NFR-02 hook budget rows
/// measure against: a constant, and its mixing without a constant.
fn calibration_cases() -> Vec<Case> {
    const FROZEN: &str = "the_calibration_workload_is_frozen";
    const SOURCE: &str = "tooling/calibrate/src/lib.rs";
    vec![
        case(
            "the calibration workload's mixing rounds changed",
            FROZEN,
            Injection::replace(
                SOURCE,
                "pub const MIX_ROUNDS: u64 = 1_200_000;",
                "pub const MIX_ROUNDS: u64 = 1_000_000;",
            ),
            cargo_test("engineering", "benchmark_gate", FROZEN),
            "the calibration mixing rounds are 1000000, not the recorded 1200000",
        ),
        case(
            "the calibration workload's mixing changed without a constant",
            FROZEN,
            Injection::replace(SOURCE, "x ^= x >> 27;", "x ^= x >> 28;"),
            cargo_test("engineering", "benchmark_gate", FROZEN),
            "the calibration workload's source hashes to",
        ),
    ]
}

fn release_budget_cases() -> Vec<Case> {
    const CI: &str = ".github/workflows/ci.yml";
    const RELEASE: &str = "benchmarks_and_timed_steps_run_release_builds";
    vec![
        case(
            "the benchmark job building the suite in debug",
            RELEASE,
            Injection::replace(
                CI,
                "cargo build --release --locked -p benchmark -p parinn",
                "cargo build --locked -p benchmark -p parinn",
            ),
            cargo_test("engineering", "benchmark_gate", RELEASE),
            "builds the benchmark suite without --release",
        ),
        case(
            "the benchmark pointed at a debug core",
            RELEASE,
            Injection::replace(
                CI,
                "--core \"$head/parinn\"",
                "--core \"$GITHUB_WORKSPACE/target/debug/parinn\"",
            ),
            cargo_test("engineering", "benchmark_gate", RELEASE),
            "points the benchmark at a debug build",
        ),
        case(
            "the NFR-02 budget row timing the development core",
            RELEASE,
            Injection::replace(
                "tooling/scenario/tests/bdd/non_functional.rs",
                "run_release_core(&mut w, &[\"hook\", &hook], Some(input.as_bytes()))?;",
                "run_core_with(&mut w, &[\"hook\", &hook], Some(input.as_bytes()), &[])?;",
            ),
            cargo_test("engineering", "benchmark_gate", RELEASE),
            "reads a clock and starts the development core with run_core_with(",
        ),
        case(
            "the NFR-02 relative check letting a hook slowed against the reference through",
            "a_hook_made_slow_against_the_reference_fails_the_check",
            Injection::replace(
                "tooling/testkit/src/relative.rs",
                "if own <= base * u128::from(limit) {",
                "if own <= base * u128::from(limit) * 1_000 {",
            ),
            cargo_lib_test(
                "testkit",
                "relative::tests::a_hook_made_slow_against_the_reference_fails_the_check",
            ),
            "a_hook_made_slow_against_the_reference_fails_the_check",
        ),
    ]
}

/// The drifts ENG-15's gate tests exist to catch while its scenario is
/// pending: an NFR id ENG-15 names dropped from the benchmark suite's
/// registry, and the benchmark job no longer gating CI.
fn benchmark_cases() -> Vec<Case> {
    const SUITE: &str = "tooling/benchmark/src/suite.rs";
    const CI: &str = ".github/workflows/ci.yml";
    const REGISTERED: &str = "the_benchmark_suite_registers_every_nfr_id_eng_15_names";
    const GATED: &str = "ci_gates_on_the_benchmark_job";
    [
        release_budget_cases(),
        calibration_cases(),
        vec![
            case(
                "an NFR id ENG-15 names dropped from the benchmark suite",
                REGISTERED,
                Injection::replace(
                    SUITE,
                    "requirement: \"NFR-04\",",
                    "requirement: \"NFR-40\",",
                ),
                cargo_test("engineering", "benchmark_gate", REGISTERED),
                "NFR-04 has no benchmark of real code and is not declared awaiting one",
            ),
            case(
                "the benchmark job no longer running the gate",
                GATED,
                Injection::replace(CI, "\"$head/benchmark\" gate --base", "true --base"),
                cargo_test("engineering", "benchmark_gate", GATED),
                "job benchmark does not run",
            ),
            case(
                "the benchmark job left out of the CI aggregator",
                GATED,
                Injection::replace(CI, ", benchmark", ""),
                cargo_test("engineering", "benchmark_gate", GATED),
                "job ci does not need benchmark",
            ),
            case(
                "SQLite built with extension loading",
                "sqlite_is_built_without_extension_loading",
                Injection::replace(
                    ".cargo/config.toml",
                    "LIBSQLITE3_FLAGS = {",
                    "UNUSED_SQLITE_FLAGS = {",
                ),
                cargo_test(
                    "store",
                    "sqlite_build",
                    "sqlite_is_built_without_extension_loading",
                ),
                "SQLite was built with extension loading",
            ),
            case(
                "an accepted regression recorded with no reason",
                "the_accepted_regressions_parse",
                Injection::replace(
                    "tooling/benchmark/accepted.txt",
                    "with the stakeholder's acceptance.\n",
                    "with the stakeholder's acceptance.\ncore_start_up\n",
                ),
                cargo_test(
                    "benchmark",
                    "accepted_file",
                    "the_accepted_regressions_parse",
                ),
                "want <benchmark> <percent> <reason>",
            ),
        ],
    ]
    .concat()
}

/// The drifts the isolation guard exists to catch (ENG-14): the guard
/// letting real state through, or refusing every place, and a read of
/// real state that bypasses it.
fn isolation_cases() -> Vec<Case> {
    const GUARD: &str = "tooling/testkit/src/guard.rs";
    const UNDER: &str = "    path != temp && path.starts_with(temp)";
    let reads = || {
        cargo_test(
            "engineering",
            "isolation",
            "only_the_isolation_guard_reads_the_home",
        )
    };
    vec![
        case(
            "the isolation guard letting real state through",
            "ENG-14",
            Injection::replace(GUARD, UNDER, "    let _ = (&path, &temp);\n    true"),
            scenario("ENG-14"),
            "resolved real state instead of aborting",
        ),
        case(
            "the isolation guard refusing every place",
            "ENG-14",
            Injection::replace(GUARD, UNDER, "    let _ = (&path, &temp);\n    false"),
            scenario("ENG-14"),
            "refused an isolated place",
        ),
        case(
            "a test helper reading real state around the guard",
            "only_the_isolation_guard_reads_the_home",
            Injection::replace(
                "tooling/testkit/src/lib.rs",
                "/// The repository checkout the tests inspect",
                "pub fn real_home() -> Option<OsString> {\n    std::env::var_os(\"HOME\")\n}\n\n/// The repository checkout the tests inspect",
            ),
            reads(),
            "reads real state outside testkit::guard",
        ),
    ]
}

/// The drifts the decoy `HOME` and the instruction scan exist to catch
/// (ENG-14): CI running the suite with the real `HOME`, and an agent
/// instruction directing a destructive command at real state.
fn instruction_cases() -> Vec<Case> {
    let instructions = || {
        cargo_test(
            "engineering",
            "isolation",
            "the_agent_instructions_direct_no_destructive_command_at_real_state",
        )
    };
    vec![
        case(
            "CI running the suite with the real `HOME`",
            "the_ci_test_job_runs_with_a_decoy_home",
            Injection::replace(
                ".github/workflows/ci.yml",
                "HOME=\"$decoy\" cargo test --workspace --locked",
                "cargo test --workspace --locked",
            ),
            cargo_test(
                "engineering",
                "isolation",
                "the_ci_test_job_runs_with_a_decoy_home",
            ),
            ".github/workflows/ci.yml does not contain",
        ),
        case(
            "the instructions no longer forbidding a command at real state",
            "ENG-14",
            Injection::replace(
                "CLAUDE.md",
                "Never run a command that writes to, deletes from, or purges the real",
                "Run a command that writes to, deletes from, or purges the real",
            ),
            scenario("ENG-14"),
            "CLAUDE.md:",
        ),
        case(
            "a skill telling agents to delete the real ParleyInn home",
            "the_agent_instructions_direct_no_destructive_command_at_real_state",
            Injection::replace(
                ".claude/skills/plan-tidy/SKILL.md",
                "## Read the mess\n",
                "## Read the mess\n\n```sh\nrm -rf ~/.parinn\n```\n",
            ),
            instructions(),
            ".claude/skills/plan-tidy/SKILL.md:",
        ),
        case(
            "a hook in the settings clearing Claude Code's configuration",
            "ENG-14",
            Injection::replace(
                ".claude/settings.json",
                "  \"enabledPlugins\": {",
                "  \"hooks\": {\"Stop\": [{\"command\": \"rm -rf ~/.claude/projects\"}]},\n  \"enabledPlugins\": {",
            ),
            scenario("ENG-14"),
            ".claude/settings.json:",
        ),
    ]
}

/// The drifts the transcript and hook input contract's checks exist to
/// catch (ENG-17, REC-26, SEC-16): a fixture carrying a residual
/// identifier, recorded or hook input shapes that no longer match the
/// recordings, a recorded transcript the core's parser no longer reads
/// without format drift, and a recorded hook input the core's decoder
/// rejects.
fn transcript_contract_cases() -> Vec<Case> {
    let contract = |name| cargo_test("harness-contract", "contract", name);
    vec![
        case(
            "a recorded fixture carrying an account's email address",
            "ENG-17",
            Injection::replace(
                "testdata/claude-code/2.1.296/transcripts/projects/-w-my-app/72c55a35-095c-4ec1-a79e-834f2cc8d630.jsonl",
                "user@example.com",
                "jane.doe@gmail.com",
            ),
            contract("recorded_fixtures_hold_no_residual_identifier"),
            "an email address jane.doe@gmail.com",
        ),
        case(
            "recorded shapes that differ from what the recordings give",
            "REC-26",
            Injection::replace(
                "crates/transcript/shapes/claude-code/2.1.296.json",
                "\"harness\": \"claude-code\",",
                "\"harness\":  \"claude-code\",",
            ),
            contract("recorded_shapes_regenerate_byte_for_byte"),
            "differs from the shapes 2.1.296 records",
        ),
        case(
            "a recorded transcript line of a type no supported version records",
            "ENG-17",
            Injection::replace(
                "testdata/claude-code/2.1.296/transcripts/rewind/1-before-rewind.jsonl",
                "\"type\":\"ai-title\"",
                "\"type\":\"ai-titles\"",
            ),
            contract("every_recorded_transcript_replays_without_an_unparsed_line_or_format_drift"),
            "format drift claude-code:unversioned:ai-titles:unrecorded-type",
        ),
        case(
            "hook input shapes that differ from what the recorded hook inputs give",
            "REC-26",
            Injection::replace(
                "crates/hook/shapes/claude-code/2.1.296.json",
                "\"harness\": \"claude-code\",",
                "\"harness\":  \"claude-code\",",
            ),
            contract("hook_input_shapes_regenerate_byte_for_byte"),
            "differs from the shapes 2.1.296 records",
        ),
        case(
            "a recorded hook input that fails the schema",
            "SEC-16",
            Injection::replace(
                "testdata/claude-code/2.1.296/hooks/startup/01-SessionStart.json",
                "\"session_id\":\"205beaae-e4b8-4798-a1aa-cdf50c807136\"",
                "\"session_id\":205",
            ),
            contract("every_recorded_hook_input_decodes_without_rejection_or_format_drift"),
            "hook input failed schema validation",
        ),
    ]
}

#[cfg(test)]
mod tests;
