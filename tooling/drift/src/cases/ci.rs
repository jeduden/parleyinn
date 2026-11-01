//! The drifts of the CI gates: static analysis, coverage floors,
//! fuzzing, setup, TOML, complexity, calibration and benchmarks.

use super::*;

/// The drifts ENG-33 exists to catch: an action used by a tag rather
/// than a commit sha, an event field expanded into a script, and CI no
/// longer running zizmor over the workflows.
pub(super) fn workflow_cases() -> Vec<Case> {
    vec![
        case(
            "an action used by its tag, not its commit sha",
            "ENG-33",
            Injection::replace(
                ".github/workflows/codeql.yml",
                "github/codeql-action/init@24c54180a607b1449ed407dd24f251e4e9147c8d # v4.38.3",
                "github/codeql-action/init@v4.38.3",
            ),
            scenario("ENG-33"),
            "uses github/codeql-action/init@v4.38.3, not pinned to a full commit sha",
        ),
        case(
            "a pull request's title expanded into a run script",
            "ENG-33",
            Injection::replace(
                CI_WORKFLOW,
                "      - run: taplo lint\n",
                "      - run: taplo lint ${{ github.event.pull_request.title }}\n",
            ),
            scenario("ENG-33"),
            "expands github.event.pull_request.title in a run script, not through env",
        ),
        case(
            "CI no longer runs zizmor over the workflows",
            "ENG-33",
            Injection::replace(CI_WORKFLOW, "run: zizmor .github", "run: zizmor --version"),
            scenario("ENG-33"),
            r#"the CI workflow does not run "zizmor .github""#,
        ),
    ]
}

/// The test job's shards leave a crate out, so its tests run nowhere.
pub(super) fn test_shards_case() -> Case {
    case(
        "a crate the test job's shards leave out",
        "the_ci_test_shards_run_every_crate_once",
        Injection::replace(CI_WORKFLOW, " -p review-run -p srs\"", " -p review-run\""),
        cargo_test(
            "engineering",
            "static_analysis",
            "the_ci_test_shards_run_every_crate_once",
        ),
        "the test job's shards: crate srs runs in no shard",
    )
}

/// The drifts ENG-16's core row exists to catch in what CI gates on: a
/// static check no longer run, left out of the ci job or allowed to
/// fail, and the ci job passing after a failed job.
pub(super) fn ci_gate_cases() -> Vec<Case> {
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
pub(super) fn static_analysis_cases() -> Vec<Case> {
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
        case(
            "the core's code quieting a lint with a bare allow",
            "the core's #![deny(clippy::allow_attributes)]",
            Injection::replace(
                "crates/parinn/src/licenses.rs",
                "\nuse std::io::Write;\n",
                "\n#[allow(unused_imports)]\nuse std::io::Write;\n",
            ),
            clippy("parinn"),
            "#[allow] attribute found",
        ),
    ]
}

/// The drifts ENG-11's checks exist to catch: a security-sensitive
/// crate held below the row's floor, and CI no longer merging the
/// layers' profiles against the floors.
pub(super) fn coverage_floor_cases() -> Vec<Case> {
    vec![
        case(
            "the search crate held below ENG-11's floor",
            "ENG-11",
            Injection::replace(
                "Cargo.toml",
                "pyramid = true\n",
                "pyramid = true\n\n[workspace.metadata.coverage.crates.search.floors]\nall = 85\n",
            ),
            scenario("ENG-11"),
            "search is held to 85% of its lines, below ENG-11's 90%",
        ),
        case(
            "CI no longer holding the merged profile to the floors",
            "ENG-11",
            Injection::replace(
                CI_WORKFLOW,
                "cargo run --locked -p coverage -- --merge",
                "cargo run --locked -p coverage -- --layer unit",
            ),
            scenario("ENG-11"),
            "job coverage does not run \"-p coverage -- --merge\"",
        ),
    ]
}

/// The drifts ENG-07's checks exist to catch: a parser's target
/// dropped from the fuzz workspace, and the nightly job fuzzing with no
/// committed corpus.
pub(super) fn fuzz_cases() -> Vec<Case> {
    vec![
        case(
            "the redactor's fuzz target dropped from the fuzz workspace",
            "ENG-07",
            Injection::replace(
                "fuzz/Cargo.toml",
                "name = \"redactor\"",
                "name = \"scrubber\"",
            ),
            scenario("ENG-07"),
            "fuzz/Cargo.toml declares no target redactor",
        ),
        case(
            "the nightly fuzz job run without the committed corpus",
            "ENG-07",
            Injection::replace(
                ".github/workflows/nightly.yml",
                " \"fuzz/corpus/$target\"",
                "",
            ),
            scenario("ENG-07"),
            "the nightly fuzz job does not fuzz redactor from fuzz/corpus/redactor",
        ),
    ]
}

/// The tool setup installs only released binaries: a tool whose backend
/// would build from source is refused before anything downloads.
pub(super) fn setup_cases() -> Vec<Case> {
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
pub(super) fn toml_cases() -> Vec<Case> {
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

/// The drift SEC-07's check exists to catch: a crate outside
/// `restore_block` constructing `TrustedText`. The drift of
/// `restore_block` building it from other text gets its case when that
/// crate lands (plan 2609292013): a case creates one file, and a crate
/// needs a manifest too.
pub(super) fn trusted_text_cases() -> Vec<Case> {
    const GATE: &str = "restore_content_is_built_from_trusted_text_alone";
    let gate = || cargo_test("engineering", "static_analysis", GATE);
    vec![
        case(
            "a crate outside restore_block constructing TrustedText",
            GATE,
            Injection::create(
                "crates/store/src/leak.rs",
                "fn leak(recalled: String) -> TrustedText {\n    TrustedText::new(recalled)\n}\n",
            ),
            gate(),
            "crates/store/src/leak.rs:2: TrustedText is constructed (TrustedText::new) outside the restore_block crate",
        ),
        case(
            "a crate outside restore_block hiding TrustedText behind an alias",
            GATE,
            Injection::create(
                "crates/store/src/alias.rs",
                "use restore_block::TrustedText as Safe;\nfn leak(s: String) -> Safe {\n    Safe::new(s)\n}\n",
            ),
            gate(),
            "crates/store/src/alias.rs:1: TrustedText is named by a `use .. as` alias, which hides how it is built, outside the restore_block crate",
        ),
        case(
            "a crate implementing a trait that adds to TrustedText",
            GATE,
            Injection::create(
                "crates/store/src/extend.rs",
                "impl Extend<String> for TrustedText {\n    fn extend<I>(&mut self, recalled: I) {}\n}\n",
            ),
            gate(),
            "crates/store/src/extend.rs:1: TrustedText implements the trait Extend, but only Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd, Ord, AsRef<str> and Deref<Target = str> may be, so none adds text to it",
        ),
        case(
            "a crate implementing a trait of its own for TrustedText",
            GATE,
            Injection::create(
                "crates/store/src/grow.rs",
                "impl Grow for TrustedText {\n    fn grow(&self) {}\n}\n",
            ),
            gate(),
            "crates/store/src/grow.rs:1: TrustedText implements the trait Grow, but only Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd, Ord, AsRef<str> and Deref<Target = str> may be, so none adds text to it",
        ),
        case(
            "a crate changing TrustedText through &mut",
            GATE,
            Injection::create(
                "crates/store/src/grow_mut.rs",
                "fn grow(t: &mut TrustedText, recalled: &str) {\n    more(t, recalled)\n}\n",
            ),
            gate(),
            "crates/store/src/grow_mut.rs:1: TrustedText is changed by a &mut TrustedText, though trusted text is built whole by its four constructors and never added to",
        ),
    ]
}

/// The drifts the complexity gate exists to catch: the workspace lints
/// no longer denying a complexity lint, and a crate's clippy.toml
/// raising a ceiling, dropping one or switching a lint off. Clippy
/// itself, which CI gates on (ENG-16), catches the core nesting a block
/// past a ceiling.
pub(super) fn complexity_cases() -> Vec<Case> {
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

/// The drifts the release-build gate exists to catch: the benchmark
/// job building or running the suite in debug, and the NFR-02 budget row
/// timing the development core.
/// The drifts of the calibration workload the NFR-02 hook budget rows
/// measure against: a constant, and its mixing without a constant.
pub(super) fn calibration_cases() -> Vec<Case> {
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

pub(super) fn release_budget_cases() -> Vec<Case> {
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
                "--core \"$1/parinn\"",
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
pub(super) fn benchmark_cases() -> Vec<Case> {
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
                "the suite's run dropped from the shards the gate job needs",
                GATED,
                Injection::replace(
                    CI,
                    "\"$1/benchmark\" run --core",
                    "\"$1/benchmark\" ran --core",
                ),
                cargo_test("engineering", "benchmark_gate", GATED),
                "nor does a job it needs",
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
