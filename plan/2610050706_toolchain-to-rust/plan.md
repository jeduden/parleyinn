---
id: 2610050706
title: "Move the whole toolchain from Go to Rust"
status: "✅"
summary: >-
  Port everything the repository runs from Go to Rust: the scenario
  runner (cucumber-rs in place of godog), the gates that keep the SRS,
  scenarios, ADRs and reviews in step, the drift harness, the review
  gate, and CI, release and coverage. Then remove Go. Each gate runs
  in both languages until the drift suite shows they agree.
model: sonnet
depends-on: []
---
# Move the whole toolchain from Go to Rust

## Goal

Every tool the repository builds and runs is Rust. That holds for the
product and for the gates that check it. The page's TypeScript is the
only other language, as ADR-2610050528 and SRS 2.1-draft set out.

## Context

[ADR-2610050528](../../docs/adr/ADR-2610050528-language-rust-typescript.md)
made the product Rust and first let repository tooling stay Go. On 5
October 2026 the stakeholder chose to move the whole toolchain. SRS
2.1-draft still lets tooling stay Go in CON-01, ENG-01 and ENG-16;
those clauses go when Go does.

Spike S2's benchmark is its own Go module of about 1,900 lines. It is
evidence ADR-07 cites, not tooling, and no task ports it. The tooling
is about 4,800 lines, about 3,000 of them tests:

| Go code                                      | What it does                                                          |
| -------------------------------------------- | --------------------------------------------------------------------- |
| `cmd/parinn/main.go`                         | the `parinn version` CLI, the only product code so far                |
| `cmd/parinn/bdd_test.go`                     | runs every scenario through godog, `@pending` skipped, isolated HOME  |
| `cmd/parinn/bdd_engineering*_test.go`        | step bindings for the six bound scenarios: ENG-01, 18, 24, 26, 27, 28 |
| `cmd/parinn/imports_test.go`                 | the import-closure half of SEC-01                                     |
| `internal/srs`                               | parses the SRS tables, Appendix B and C, the personas                 |
| `internal/scenario`                          | parses features and checks the SRS and scenarios agree                |
| `internal/adr`                               | reads ADRs for ENG-18 and ENG-26                                      |
| `internal/drift`                             | injects each registered drift and requires its check to fail (ENG-27) |
| `internal/review`, `cmd/review-gate`         | the reviewer app's gate (ENG-28)                                      |
| `scripts/check-coverage.sh`, `.golangci.yml` | the 100% coverage gate and the linters                                |

Most of these paths are the stakeholder's under
[CODEOWNERS](../../.github/CODEOWNERS), as are the CI workflows, so
every phase needs the stakeholder's approval. The Rust replacements
join CODEOWNERS in the phase that adds them. The security-sensitive
owner lines move from `internal/<name>/` to the matching crate paths
before the first such crate lands, so ENG-21's second approval keeps
applying.

Reuse, searched first:

- **The scenario runner.** The `cucumber` crate (0.23, MIT OR
  Apache-2.0) runs Gherkin with step macros. A prototype in the
  session that wrote this plan showed the shape that replaces godog:
  a `[[test]]` target with `harness = false`;
  `filter_run_and_exit` skipping `@pending`; `fail_on_skipped`, which
  turns a step with no definition into a failure (exit 101), as
  godog's strict mode does; and `--tags @REC-01` to run one scenario,
  in place of `-run 'TestFeatures/^REC-01:'`.
- **The feature parser.** The `gherkin` crate (0.16, MIT OR
  Apache-2.0) parses the features for the scenario gate.
- **Kept as external tools.** mdsmith and frit are pinned binaries,
  not repository code; they stay.
- **Not reused.** Keeping godog to drive a Rust binary as a black box
  would keep Go in the repository, which the stakeholder ruled out.

The port follows the vendored-fork discipline of the research's
[options](../../research/notes/implementation-path/options.md). While
both exist, the Go and the Rust gate run side by side. A Go gate is
deleted only after the drift suite shows each registered drift caught
by both.

## Tasks

1. Proving slice: a Cargo workspace, a pinned toolchain, the cucumber
   runner and the scenario gate in Rust, beside Go (phase 1)
2. Port the SRS parser's remaining checks (Appendix B and C, personas)
   and the ADR reader, with the ENG-18 and ENG-26 bindings reading
   `Cargo.toml` as well as `go.mod` (phases 1 and 2)
3. Port the drift harness and its cases, and run the drift suite
   against both languages (phase 1)
4. Port the review gate and its workflow (ENG-28) (phases 1 and 2)
5. Port `parinn version` to the Rust core executable; the release
   workflow builds it; ENG-01's steps check the Rust release flags;
   reach evidence per crate replaces the import-closure test (SEC-01)
   (phase 4)
6. CI on Rust: clippy, `cargo fmt`, `cargo-deny`, `cargo-audit` (phase
   2), and coverage through `cargo-llvm-cov`, measured per test layer,
   in place of `scripts/check-coverage.sh` (phase 3)
7. Remove Go: the gates and godog first (phase 2); then `go.mod`,
   `go.sum`, the last Go sources and `.golangci.yml`, and the Go
   clauses of CON-01, ENG-01 and ENG-16 (phase 4)
8. Test-engineer agents that review the test pyramid, and a skill
   that runs them (phase 3)

## Execution

| Phase | Model | Gate                                                                                                                |
| ----- | ----- | ------------------------------------------------------------------------------------------------------------------- |
| 1     | opus  | `cargo test --workspace` and `go test ./...` pass; CI's `drift` and `rust-drift` jobs pass on one commit            |
| 2     | opus  | CI green with no Go gate left; the drift suite catches every case through the Rust checks                           |
| 3     | opus  | CI's coverage job prints line coverage per test layer and fails a crate below its floor                             |
| 4     | opus  | No Go source remains outside the archived spike S2 benchmark; CI runs the release build of the Rust core executable |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                                                                                                                                                                                                                                                                                                                            |
| --- | ------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | ✅     | [Every gate in Rust, beside its Go original](phase-1.md)                                                                                                                                                                                                                                                                                                         |
|     | ↳      | Every Go gate has a Rust port under tooling/, and cucumber-rs runs the six bound scenarios. CI run 38060936531 on 594102c passed the Go drift job and the Rust one side by side: every registered drift is caught by both languages.                                                                                                                             |
| 2   | ✅     | [Remove the Go tooling; ENG-18 reads Cargo](phase-2.md)                                                                                                                                                                                                                                                                                                          |
|     | ↳      | The Go gates and godog are gone; the six bound scenarios run in cucumber-rs alone. ENG-18 reads go.mod and cargo metadata, ENG-01 checks rust-toolchain.toml, and the review workflow decides with the Rust review gate. 30 drift cases, all caught.                                                                                                             |
| 3   | ✅     | [Coverage per test layer](phase-3.md)                                                                                                                                                                                                                                                                                                                            |
|     | ↳      | The coverage executable measures each test layer, holds 99% unit and 100% overall per crate, and fails an inverted pyramid. Locally every crate is at 100% from its unit tests; the shape is unit 199, integration 18, end-to-end 14. Four test-engineer agents and two skills review and shape the pyramid.                                                     |
| 4   | ✅     | [The core executable in Rust; Go removed](phase-4.md)                                                                                                                                                                                                                                                                                                            |
|     | ↳      | `parinn version` is the Rust core executable, built by one workflow CI and the release share: one Linux and one macOS builder, four targets, each run on its own platform. Its clippy config and reach test hold it to no socket, no program start and no linked crate. Go is gone outside the archived spike S2 benchmark; SRS 2.32-draft drops the Go clauses. |
<?/catalog?>

## Acceptance Criteria

- [x] `cargo test --workspace` runs every non-pending scenario through
  cucumber-rs, with `@pending` skipped and undefined steps failing
- [x] Every gate that Go runs today runs in Rust, and the drift suite
  shows each registered drift caught by the Rust gate
- [x] The release workflow builds the Rust core executable, and
  ENG-01's scenario checks the Rust toolchain and release flags
- [x] CI gates on clippy, `cargo fmt`, `cargo-deny`, `cargo-audit`
  and the coverage floors
- [x] No Go source, `go.mod` or `go.sum` remains outside spike S2's
  archived benchmark, and CON-01, ENG-01 and ENG-16 no longer mention
  Go
- [x] `mdsmith check .` passes
