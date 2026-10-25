Feature: Engineering quality (ENG)

  The scenarios for SRS §10, one per requirement, tagged with its id and
  priority; the engineering tables carry no Traces column. A scenario
  still tagged @pending is declared but not yet written: its steps are
  bound when the plan that implements the requirement lands. The
  scenarios without @pending inspect the repository itself and run on
  every `cargo test --workspace`.

  @ENG-01 @P0
  Scenario: the toolchain is pinned and release builds are static, trimmed and stamped
    Given the repository checkout
    Then "rust-toolchain.toml" pins the Rust toolchain to an exact release
    And the release workflow builds with "--remap-path-prefix"
    And the release workflow builds with "PARINN_COMMIT"
    And the release workflow builds with "x86_64-unknown-linux-musl"
    And the release workflow builds with "aarch64-unknown-linux-musl"

  @ENG-02 @P0
  Scenario: dependency directions between the workspace crates are declared and enforced
    Given the declared dependency directions for the workspace crates
    When a crate depends on a crate its direction forbids
    Then the architecture check fails naming both crates
    And every library crate has exactly one declared responsibility

  @ENG-03 @P0 @pending
  Scenario: no mutable global state, injectable I/O and deadlines everywhere
    Given the repository's code
    When the static analyzers run
    Then no crate declares mutable global state
    And every public blocking operation takes a deadline, directly or through a cancellation signal a deadline fires
    And no crate that computes derived artifacts reads the wall clock or randomness

  @ENG-04 @P0
  Scenario: errors keep their cause, are typed per counter, and panics stop at the entry points
    Given an isolated ParleyInn home
    And a hook handler that panics
    When the hook "SessionStart" runs with a valid input
    Then the command exits 0
    And the hook output is empty
    And the hook handler wrote nothing to stderr, the default panic message included
    And an audit entry records "panic recovered"

  @ENG-05 @P0
  Scenario: structured logs pass through redaction
    Given an isolated ParleyInn home
    And a log field containing an AWS access key
    When ParleyInn writes the log line
    Then the log line is JSON emitted through tracing
    And the key appears only as a "[REDACTED:" marker

  @ENG-06 @P0 @pending
  Scenario: the crash-consistency test kills ParleyInn mid-operation and every invariant stays true
    Given an isolated ParleyInn home
    When the crash-consistency test kills ParleyInn at randomized points during ingestion, purge, migration and rebuild
    Then after every kill "parinn verify" exits 0
    And each nightly job completes at least 10000 iterations

  @ENG-07 @P0 @pending
  Scenario Outline: coverage-guided fuzzing covers every parser with a committed corpus
    Given the cargo-fuzz target for the <input>
    Then its seed corpus is committed under fuzz/corpus
    And the nightly fuzz job fuzzes it

    Examples:
      | input                     |
      | transcript parser         |
      | hook input decoder        |
      | event_search query parser |
      | envelope encoder          |
      | field sanitizer           |
      | redactor                  |
      | configuration parser      |

  @ENG-08 @P0 @pending
  Scenario Outline: property-based tests check the record's core laws
    Given randomly generated records
    Then the property "<property>" is true of every one

    Examples:
      | property                                                                                                                                                      |
      | within each writer's log, seq is strictly increasing and gap-free, so no two events share an address (writer, seq)                                            |
      | ingest twice equals ingest once                                                                                                                               |
      | every order of ingesting a harness fork's transcripts, with a fork observation, records each repeated line in the same writer with the same repeat references |
      | after any sequence of appends and lifecycle events, each derived artifact's canonical digest equals that of a rebuild from nothing                            |
      | envelope encoding round-trips any byte sequence                                                                                                               |
      | sanitized fields never contain restore delimiters                                                                                                             |

  @ENG-09 @P0 @pending
  Scenario: data races are excluded and the store survives a concurrency soak
    Given an isolated ParleyInn home
    When 50 concurrent writers append 1000000 events
    Then no two events share an address (writer, seq), and no seq is duplicated or skipped within any writer's log
    And "parinn verify" exits 0
    And every unsafe block that shares memory between threads passes under Miri or ThreadSanitizer

  @ENG-10 @P0 @pending
  Scenario: golden files fix the exact bytes of restore blocks and envelopes
    Given a fixture record
    When the restore block and an envelope are rendered
    Then both equal their committed golden files byte for byte

  @ENG-11 @P0 @pending
  Scenario: line coverage meets the floor per crate class
    Given a coverage profile of the whole suite
    Then every security-sensitive crate is at least 90% covered
    And the workspace as a whole is at least 80% covered

  @ENG-12 @P0 @pending
  Scenario Outline: the end-to-end suite runs each component confined to its boundary
    Given the end-to-end suite runs "<component>" confined to its boundary, which <confinement>
    When "<component>" opens a socket outside the boundary the register assigns it
    Then the suite fails naming the component and the caller

    Examples:
      | component           | confinement                   |
      | core                | denies all network            |
      | room-view component | denies all but loopback       |
      | launcher            | denies all but loopback       |
      | peer component      | allows only its register rows |
      | publish component   | allows only its register rows |
      | bridge component    | allows only its register rows |

  @ENG-13 @P1 @pending
  Scenario: mutation testing scores the security-sensitive crates
    Given the security-sensitive crates
    When mutation testing runs
    Then the mutation score is at least 70%

  @ENG-14 @P0
  Scenario: tests cannot touch the real `HOME` or Claude Code configuration
    Given a test that resolves a path under the real `HOME`
    When the suite runs
    Then the isolation guard aborts that test
    And no agent instruction file in the repository directs a destructive command at non-isolated state

  @ENG-15 @P0 @pending
  Scenario: the benchmark gate fails on a significant regression
    Given the benchmark suite covering NFR-01 to NFR-05
    When a change slows a benchmark by more than 10% with statistical significance
    Then the benchmark CI job fails naming the benchmark

  @ENG-16 @P0 @pending
  Scenario Outline: CI gates on the static checks and on build-time reach evidence per component
    Given the CI workflow
    When CI reads the build-time reach evidence for the "<component>", dependencies and start-up code included
    Then the workflow gates on cargo clippy, cargo fmt, cargo-deny, cargo-audit, the browser room view client's strict TypeScript compile and lint, and the custom checks
    And the browser room view client's TypeScript package imports nothing of the core but its generated types
    And every crate that parses untrusted content or decides trust forbids unsafe code
    And a build-time check fails when code that computes derived artifacts reads a clock or randomness
    And a build-time check fails when restore content is built from anything but trusted text
    And CI fails when the evidence for the "<component>" is missing or shows that it <violation>

    Examples:
      | component           | violation                                                                                                      |
      | core                | opens any socket, or starts a program other than its own kernel worker                                         |
      | launcher            | makes an off-machine connection or listens beyond loopback or a local endpoint only the same OS user can reach |
      | room-view component | starts a program, makes an off-machine connection or listens beyond loopback                                   |
      | peer component      | starts a program or reaches beyond its register rows                                                           |
      | publish component   | starts a program or reaches beyond its register rows                                                           |
      | bridge component    | starts a program, listens, or reaches beyond its register rows                                                 |

  @ENG-17 @P0 @pending
  Scenario: contract tests replay every supported Claude Code version and catch format drift, and fixtures hold no residual identifier
    Given recorded hook inputs and transcripts for each supported Claude Code version
    When the contract suite replays them
    Then every one parses without an unparsed event or format drift
    And a nightly job drives real Claude Code through compaction, restore and recall
    And the nightly job records a transcript and hook inputs from the newest Claude Code release and fails on any format drift
    And the residual-identifier check runs in CI over every file under "testdata/claude-code", and in the nightly job before any recording is logged or kept as an artifact
    And it fails on a recorded fixture that carries an account's email address, organization id, a home path or the name of one of its skills or connectors, with its drift case

  @ENG-18 @P0
  Scenario: every direct dependency is justified by an accepted ADR with an allow-listed license
    Given the repository checkout
    When the direct dependencies are read from "Cargo.toml"
    And the direct dependencies are read from "fuzz/Cargo.toml"
    And the ADRs are read from "docs/adr"
    Then each direct dependency is named by exactly one accepted ADR
    And every module an accepted ADR names is a direct dependency
    And each named module has a purpose, a license and a maintenance status, and its ADR weighs alternatives
    And each license is on the allow-list the ENG-18 requirement states, or a crate's exception in "deny.toml"
    And the direct dependencies linked into a shipped executable stay within the target the ENG-18 requirement states
    And "DEPENDENCIES.md" lists every ADR that names a module

  @ENG-20 @P0
  Scenario: the release workflows sign, attest and verify every release they publish
    Given the repository checkout
    When the release workflows are read
    Then the checksum file is signed keyless with Sigstore and verified once published
    And only the release-only build workflow publishes each binary's SLSA Build Level 3 provenance as a GitHub artifact attestation, verified against that workflow's signing identity at main
    And an SPDX SBOM is attached, published as a GitHub artifact attestation and verified once published

  @ENG-30 @P0
  Scenario: a dependency with no license text stops the build in the pull request that adds it
    Given the repository checkout
    When every crate linked into a shipped executable for a release target is read through "cargo metadata" and the sources cargo has fetched
    Then each crate has a license file in its source
    And the repository keeps no license text for any crate by name and version
    And each license file's text fits the canonical text of a license in the crate's declared expression, from the allow-list in "deny.toml" or granted by one of its exceptions, ignoring case, whitespace and copyright lines, with holder, year and name left free and the Apache-2.0 appendix optional
    And the CI workflow runs the check on every pull request, with the locked dependency tree
    And the check adds no dependency to any shipped executable

  @ENG-31 @P0
  Scenario: the core executable carries the license texts of what it links for its own target, kept in step by CI
    Given the repository checkout
    When the release targets are read from the build workflow
    Then each target has a committed "licenses/<target-triple>/THIRD_PARTY_LICENSES" and no other target has one
    When each target's file is regenerated twice with "cargo metadata --filter-platform <target-triple>"
    Then both regenerations give the same bytes, with no timestamp, host path or randomness
    And the committed file equals the regeneration
    And it holds the license text and copyright lines of every crate a normal-dependency path from a binary under "crates/" reaches for that target
    And no build-only, proc-macro, test, fuzz or tooling crate appears in it
    And it names SQLite, the Rust standard library, on the musl targets the C library, and any other runtime the target links statically
    And the committed files of any two targets whose linked crates differ differ in bytes
    When each target's build selects its embedded file
    Then the embedded text equals that target's committed file
    And the built core executable's "parinn licenses" output equals it
    And any other executable a component ships in carries the same for the crates linked into it
    And the CI workflow fails a pull request whose committed file differs from a regeneration
    And the release build fails when a built executable's output differs from its target's regenerated file
    And the core gains no dependency and opens no socket

  @ENG-21 @P0
  Scenario: main only moves through reviewed, signed pull requests
    Given the repository's branch protection for main
    Then direct pushes are rejected
    And commits must be signed
    And every pull request needs an approval from a reviewer other than its author
    And CODEOWNERS names the stakeholder on the requirement text and on every path that enforces it
    And CODEOWNERS names no code owner on any other path
    And the security-sensitive crates are owned by the designated security reviewer, whose approval they need

  @ENG-22 @P0 @pending
  Scenario: every privacy or data-flow statement cites its proving test
    Given the documentation
    When every privacy or data-flow statement is listed
    Then each one references a test that exists and passes

  @ENG-23 @P0 @pending
  Scenario: releases are semver with a changelog, upgrade notes and migration tests
    Given a release candidate version
    Then its version is valid semantic versioning
    And the changelog and upgrade notes have a section for it
    And migrations from every previous minor version are tested

  @ENG-24 @P0
  Scenario: SECURITY.md names a private channel and a 90-day disclosure policy
    Given the repository checkout
    Then "SECURITY.md" links the private vulnerability reporting channel
    And "SECURITY.md" states a 90-day coordinated disclosure policy
    And "SECURITY.md" states that security fixes are backported to the latest minor release

  @ENG-25 @P1 @pending
  Scenario: two maintainers and an incident runbook before general availability
    Given the v1.0 release checklist
    Then at least two active maintainers are listed
    And an incident-response runbook exists in the repository

  @ENG-26 @P0
  Scenario: every design decision lives in one ADR file and a changed decision supersedes it
    Given the repository checkout
    When the ADRs are read from "docs/adr"
    Then every ADR has an id, a title, a status and a summary
    And every ADR's file is named for its id
    And every ADR's status is proposed, accepted or superseded
    And no two ADRs share an id
    And every superseded ADR names an ADR that exists as its successor

  @ENG-27 @P0
  Scenario: every check that keeps the records in step is proven by an injected drift
    Given the repository checkout
    When the drift cases are read
    Then the CI workflow runs the drift suite with "cargo test --locked -p drift --test suite -- --ignored"
    And every drift case's edit applies to the checkout
    And every non-pending scenario that inspects the repository checkout has a drift case guarding its id
    And the requirement-scenario gate, the Appendix B check and the persona gates each have a drift case

  @ENG-28 @P0
  Scenario: an agent's approval passes through a gate the agent cannot reach
    Given the repository checkout
    When the review workflow is read from ".github/workflows/review.yml"
    Then it runs only when the "CI" workflow completes or a pull request gets a comment, as the default branch defines it
    And the job that runs the reviewing agent holds no write permission and not the "JEDUDEN_REVIEW_AGENT_KEY" key
    And only one job holds the "JEDUDEN_REVIEW_AGENT_KEY" key, and it runs no agent
    And that job runs only after the agent's job succeeded
    And that job decides the review with "cargo run --locked --quiet -p review-gate"
    And that job decides the review with "--checked checked.json"
    And the review gate decides:
      | verdict         | finding  | check finding | head    | CI     | other check | review          |
      | approve         | none     | none          | current | passed | passed      | APPROVE         |
      | approve         | nit      | none          | current | passed | passed      | APPROVE         |
      | approve         | blocking | none          | current | passed | passed      | REQUEST_CHANGES |
      | approve         | none     | nit           | current | passed | passed      | APPROVE         |
      | approve         | none     | blocking      | current | passed | passed      | REQUEST_CHANGES |
      | approve         | none     | malformed     | current | passed | passed      | an error        |
      | approve         | none     | none          | current | passed | failed      | REQUEST_CHANGES |
      | request_changes | none     | none          | current | passed | passed      | REQUEST_CHANGES |
      | approve         | none     | none          | moved   | passed | passed      | nothing         |
      | approve         | none     | none          | current | failed | passed      | an error        |
      | malformed       | none     | none          | current | passed | passed      | an error        |
      | missing         | none     | none          | current | passed | passed      | an error        |
      | approve         | none     | missing       | current | passed | passed      | an error        |

  @ENG-29 @P0 @pending
  Scenario: an invariant or I2-review change lands only with an ADR recording a named security reviewer's approval
    Given the repository checkout
    When a change to an invariant or to a requirement marked for I2 review is proposed
    Then it lands only with an accepted ADR that records a named human security reviewer's approval
    And until that ADR is accepted, every B1, B2 or B3 component is built only as a prototype
    And evidence from the release build proves no release artifact or tagged release contains prototype code
