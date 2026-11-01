//! The drifts of the records themselves: dependencies, decisions, the
//! repository's own files, agents, personas, the domain model and the
//! finding ledger.

use super::*;

/// The drifts ENG-18 exists to catch: the manifests and the dependency
/// ADRs disagreeing.
pub(super) fn dependency_cases() -> Vec<Case> {
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
pub(super) fn decision_cases() -> Vec<Case> {
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
pub(super) fn license_text_cases() -> Vec<Case> {
    [license_check_cases(), license_edit_cases()].concat()
}

/// The license cases that edit no source the checks are built from.
pub(super) fn license_check_cases() -> Vec<Case> {
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
            Injection::replace(
                "scripts/ci-gates.txt",
                "--bin license-check",
                "--bin protection-check",
            ),
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
pub(super) fn license_edit_cases() -> Vec<Case> {
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
pub(super) fn repository_cases() -> Vec<Case> {
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
                "scripts/ci-gate.sh drift",
                "true",
            ),
            scenario("ENG-27"),
            r#"does not contain "cargo test --locked -p drift --test suite -- --ignored""#,
        ),
    ]
}

/// The drifts the gate tests and mdsmith catch between the SRS, the
/// scenarios and the generated sections.
pub(super) fn gate_cases() -> Vec<Case> {
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
pub(super) fn parse_cases() -> Vec<Case> {
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
pub(super) fn persona_cases() -> Vec<Case> {
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
pub(super) fn agent_cases() -> Vec<Case> {
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
pub(super) fn domain_model_cases() -> Vec<Case> {
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
pub(super) fn ledger_cases() -> Vec<Case> {
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
