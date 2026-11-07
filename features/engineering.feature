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

  @ENG-03 @P0
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

  @ENG-07 @P0
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

  @ENG-11 @P0
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

  @ENG-33 @P0
  Scenario: workflows pin every outside action to a commit sha and expand no untrusted value in a script
    Given the repository checkout
    When every workflow under ".github/workflows" and composite action under ".github/actions" is read
    Then every action and reusable workflow they use from outside the repository is pinned to a full commit sha, with its version in a comment, and every docker:// image to its sha256 digest
    And no run script expands a github.event field, github.head_ref or an inputs value with an expression
    And the CI workflow runs "zizmor .github"

  @ENG-21 @P0
  Scenario: main only moves through reviewed, signed pull requests
    Given the repository's branch protection for main
    Then direct pushes are rejected
    And commits must be signed
    And every pull request needs an approval from a reviewer other than its author
    And every pull request needs the "review-gate" check, taken from the reviewer app alone
    And CODEOWNERS names the stakeholder on the specification, the decision records, the scenarios and the process that builds and checks the code
    And CODEOWNERS names no code owner on any other path
    And CODEOWNERS names no code owner on Rust source, so a code change needs only a reviewer other than its author

  @ENG-22 @P0 @pending
  Scenario: every privacy or data-flow statement cites its proving test
    Given the documentation
    When every privacy or data-flow statement is listed
    Then each one cites a test after "test:" or "tests:", by a test function's name or a non-pending scenario's "@ID"
    And each citation resolves to a test that exists and is not ignored

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
    And the CI workflow runs each gate of "scripts/ci-gates.txt" through "scripts/ci-gate.sh", with no gate's command written inline
    And every drift case's edit applies to the checkout
    And every non-pending scenario that inspects the repository checkout has a drift case guarding its id
    And the requirement-scenario gate, the Appendix B check and the persona gates each have a drift case

  @ENG-28 @P0
  Scenario: an agent's approval passes through a gate the agent cannot reach
    Given the repository checkout
    When the review workflow is read from ".github/workflows/review.yml"
    Then it runs only when the "CI" workflow completes or a pull request gets a comment, as the default branch defines it
    And the job that runs the review's agents holds no write permission and not the "JEDUDEN_REVIEW_AGENT_KEY" key
    And that job runs every agent the review bundle dispatches with "cargo run --locked --quiet --offline -p review-run -- --workdir .", and no agent routes the review
    And review-run runs each specialist the review bundle dispatches as its own agent, told to read only its own diff, its part's when the diff runs in parts, and the review context the review bundle wrote for it, which names the review class, the touched ids, its dispatch row and the deterministic findings on its files, never the whole bundle, assembles its report from its run log, runs each generalist once its own specialists have ended and the mass failure is decided, and writes the review outcome from what they answered
    And review-run passes each agent the cost cap of its tier, starts a run only once its tier's cap fits in the pull request's cap beside what the earlier reviews spent, what the runs that ended were charged and the caps of the runs still running, none at all when the earlier reviews reached it, charges a run paid for that left no result its tier's cap and one whose result reported no cost nothing, names each agent it did not start over budget, so the review requests changes, and keeps what the pull request's reviews spent
    And review-run starts no generalist once more than a third of the dispatched specialists, those a cost cap stopped or kept from starting among them, ended without an accepted report, and opens the review outcome with a blocking finding naming the mass failure
    And review-run waits with backoff before it runs again an agent the API turned away as overloaded or rate-limited, runs it a third time at most, leaves it a named gap when no deadline is left to wait, and never runs again one a cost cap stopped, whose credential was refused or that was charged
    And review-run runs a specialist whose diff is past its split limit in parts, each on whole files of its diff, merges their reports into its one report, reads a part cut off at its turn budget after it read its whole part as it stands, and fails the review naming a part that did not run, so the gate posts nothing
    And review-run names a specialist's finding that cites an id no requirement or invariant has in a blocking finding, keeps it out of the report, does not run that specialist again, it was paid for, and names a generalist's finding line it cannot read in a blocking finding, so the review requests changes and the gate posts that request rather than failing
    And review-run reads a generalist's weighing from its whole run log, so a finding it stated before a final answer that only names its scope is kept, and itself decides partial a weighing that weighs no finding at a place its specialist reported, naming that place in a blocking finding and the generalist as not reviewed, whatever the generalist says of its scope
    And review-run reads every specialist's run log whatever result it ended with and decides from its Read calls whether it read its whole diff: one cut off at its turn budget after reading its whole diff is consulted and the review decided on its finding, one cut off before it stated any line or an outcome is named partial, never a clean pass, one that read part of its diff is named partial with the lines it did not read, so the gate posts the request for changes, a line saying it is no finding is set aside and listed in review/ignored.json, a finding-shaped line that does not read is its own blocking finding beside the finding kept, and one whose log holds no model call did not run, runs once more as it cost nothing, and the gate posts nothing
    And review-run writes each dispatched agent left without an accepted report beside the review outcome and names it not reviewed, and a review resumed from that list runs again that agent, each agent whose answer key changed and each generalist whose specialist runs again, keeps every agent whose key is unchanged on a new commit, keeps nothing across a changed file every key reads, and approves as a full run would
    And review-run given an answer store takes every accepted answer stored under an unchanged key, runs on a new commit only the specialist whose diff changed and its generalist, never stores a report holding a line that does not read, and runs again an agent whose stored report fails the report check
    And review-run removes the answer stored under a key on request and no other, so a review given the store runs that agent again and nothing else
    And review-run runs no more an agent whose gap recurred the same way in two resumes in a row, and names it stalled in a blocking finding so a person acts
    And review-run names a blocking finding kept in three reviews in a row with its cited file unchanged in a blocking finding of its own, as needing a stakeholder ruling
    And review-run carries the findings of the previous review of a pull request from the newest artifact under the pull request's own name that a run of main's review workflow uploaded, refuses and counts an artifact of that name from any other run, names a findings file past 1 MiB or one that does not read, and the review job fetches them before the Claude Code OAuth token is staged, passes them as --previous, scrubs the findings it kept and keeps them for 90 days for the next review, and says in the job summary which earlier review it carried or that it found none
    And review-run sums what each earlier review of a pull request spent from every artifact under the pull request's own name, of any run attempt, that a run of main's review workflow uploaded, an earlier attempt of its own run's included, refuses and counts an artifact of that name from any other run, names a spend file past 1 KiB or one that does not read, and the review job fetches them before the Claude Code OAuth token is staged, fails on a run behind one it cannot look up, passes the sum as --spent-before, scrubs what this review spent and keeps it for 90 days for the next review, under its run's attempt and replacing none, however the review ended, and says in the job summary how many earlier reviews it summed and what they spent or that it found none
    And its agents can write nothing: review-run allows them Read, Glob and Grep alone
    And no step that runs an agent holds the "CLAUDE_CODE_OAUTH_TOKEN" token: one step in each agent job stages it where no agent reads it, review-run refuses its agents every read of the staged credential's directory by name, of the process, system and device files and outside their trees, and every file an agent job uploads waits for a scrub of that token
    And every text the review writes, prints or uploads is scrubbed of that token: review-run, its agents' output holding the token, leaves it in no file under review/ and in no line it prints
    And a review in which an agent's output held that token requests changes, naming the leak in a blocking finding, and fails, so nothing is posted
    And only one job holds the "JEDUDEN_REVIEW_AGENT_KEY" key, and it runs no agent
    And that job runs only after the agents' job succeeded
    And that job decides the review with "cargo run --locked --quiet -p review-gate"
    And that job decides the review with "--checked checked.json"
    And that job decides the review with "--dispatch dispatch.json"
    And that job decides the review with "--consulted consulted.json"
    And that job decides the review with "--reports reports --answers answers.json"
    And that job posts its decision as the "review-gate" check
    And every agent the review bundle can dispatch is defined, specialists at low effort, generalists at medium and their deep instances each naming its own effort, model and turn budget
    And every language the repository's source and configuration are written in has its specialist
    And the review gate decides:
      | verdict         | finding  | check finding | head    | CI     | other check | consulted | dispatch  | own check | specialist             | review          |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | reported               | APPROVE         |
      | approve         | nit      | none          | current | passed | passed      | all       | given     | none      | reported               | APPROVE         |
      | approve         | blocking | none          | current | passed | passed      | all       | given     | none      | reported               | REQUEST_CHANGES |
      | approve         | none     | nit           | current | passed | passed      | all       | given     | none      | reported               | APPROVE         |
      | approve         | none     | blocking      | current | passed | passed      | all       | given     | none      | reported               | REQUEST_CHANGES |
      | approve         | none     | malformed     | current | passed | passed      | all       | given     | none      | reported               | an error        |
      | approve         | none     | none          | current | passed | failed      | all       | given     | none      | reported               | REQUEST_CHANGES |
      | request_changes | none     | none          | current | passed | passed      | all       | given     | none      | reported               | REQUEST_CHANGES |
      | approve         | none     | none          | moved   | passed | passed      | all       | given     | none      | reported               | nothing         |
      | approve         | none     | none          | current | failed | passed      | all       | given     | none      | reported               | an error        |
      | malformed       | none     | none          | current | passed | passed      | all       | given     | none      | reported               | an error        |
      | missing         | none     | none          | current | passed | passed      | all       | given     | none      | reported               | an error        |
      | approve         | none     | missing       | current | passed | passed      | all       | given     | none      | reported               | an error        |
      | approve         | none     | none          | current | passed | passed      | not all   | given     | none      | reported               | an error        |
      | approve         | none     | none          | current | passed | passed      | malformed | given     | none      | reported               | an error        |
      | approve         | none     | none          | current | passed | passed      | all       | missing   | none      | reported               | an error        |
      | approve         | none     | none          | current | passed | passed      | all       | malformed | none      | reported               | an error        |
      | approve         | none     | none          | current | passed | passed      | all       | given     | failed    | reported               | APPROVE         |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | no report              | an error        |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | empty report           | an error        |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | incomplete             | an error        |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | no outcome answered    | an error        |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | other outcome answered | an error        |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | requested              | APPROVE         |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | malformed finding      | an error        |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | revised nothing        | APPROVE         |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | withdrew nothing       | APPROVE         |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | none beside a finding  | an error        |
      | approve         | none     | none          | current | passed | passed      | all       | given     | none      | findings with none     | an error        |
      | request_changes | blocking | none          | current | passed | passed      | all       | given     | none      | unknown id             | REQUEST_CHANGES |

  @ENG-29 @P0 @pending
  Scenario: an invariant or I2-review change lands only with an ADR recording a named security reviewer's approval
    Given the repository checkout
    When a change to an invariant or to a requirement marked for I2 review is proposed
    Then it lands only with an accepted ADR that records a named human security reviewer's approval
    And until that ADR is accepted, every B1, B2 or B3 component is built only as a prototype
    And evidence from the release build proves no release artifact or tagged release contains prototype code

  @ENG-32 @P0
  Scenario: every reviewer component is evaluated against its cases on the model it names
    Given the repository checkout
    When the reviewer evaluation cases are read from "tooling/review-eval/cases"
    Then every reviewer component, the agents the review bundle can dispatch, each deep instance and every other agent under .claude/agents among them, has a case that plants a defect, a case that is clean and a case that addresses the reviewer
    And every case applies to the checkout, and the review bundle dispatches its component for it where it dispatches that component at all
    And the review workflow's evaluate job runs "cargo run --locked --quiet -p review-eval" on the components the pull request changes, after the agents' job and before the job that holds the "JEDUDEN_REVIEW_AGENT_KEY" key
    And a pull request that changes only the shared section "docs/agents/specialist-report.md" runs the cases of one component per distinct set of such sections instead of every component's
    And a run of a case is graded:
      | outcome     | finding | forbidden | answer       | stated         | answered | run                     |
      | as expected | found   | none      | outcome line | before answer  | yes      | passes                  |
      | other       | missing | none      | outcome line | before answer  | yes      | fails on outcome        |
      | other       | found   | none      | outcome line | before answer  | yes      | passes, slips on answer |
      | as expected | missing | found     | outcome line | before answer  | yes      | fails on finding        |
      | as expected | found   | found     | outcome line | before answer  | yes      | fails on forbidden      |
      | as expected | found   | none      | more         | before answer  | yes      | passes, slips on answer |
      | as expected | found   | none      | no outcome   | before answer  | yes      | passes, slips on answer |
      | as expected | found   | none      | outcome line | only in answer | yes      | fails on stating        |
      | as expected | found   | none      | outcome line | before answer  | no       | fails on no answer      |
      | as expected | found   | none      | outcome line | before answer  | cut      | passes, slips on cut    |
      | other       | missing | none      | no outcome   | before answer  | cut      | fails on cut            |
    And a case passes with two of three runs, stopping once decided:
      | runs             | case   | runs made |
      | pass, pass       | passes | 2         |
      | pass, fail, pass | passes | 3         |
      | fail, fail       | fails  | 2         |
      | fail, pass, fail | fails  | 3         |
      | none, pass, pass | passes | 3         |
    And a case that fails is a blocking finding at its component's file, naming the case, the model and the first failure
    And a scorecard line records a component's cases, runs, slips, cost, turns and seconds on its model
    And the scheduled run's workflow ".github/workflows/review-eval.yml" fails on a failed case, with the report that names it in its summary
    And the stability measurement ".github/workflows/stability.yml" runs on demand alone and compares three fresh reviews of each snapshot of "tooling/review-eval/stability-snapshots.toml", and every review keeps a run list whose cost is its run logs' and a run report that escapes what a run wrote

  @ENG-34 @P0
  Scenario: the review bundle flags a file past its line limit and a file no specialist reads
    Given the repository checkout
    Then the review bundle's line limits are those the ENG-34 row states
    When the review bundle is built for a change that adds a Rust file past its limit and a file no review specialist reads
    Then its deterministic findings hold a nit at the Rust file in the finding form, citing ENG-34
    And its deterministic findings hold a nit at the unread file in the finding form, citing ENG-28
    And the review bundle lists both nits
    When the review bundle is built for a change within its limits that review specialists read
    Then its deterministic findings hold no nit citing ENG-34 or ENG-28
    And no file of the checkout is past its kind's line limit unless the size list gives it at no fewer lines than it holds
    And the size list gives no file that fits its limit, no file that is gone and no file more lines than it holds
    When the size check reads a tree with a listed file grown, a listed file shrunk but still past its limit, a file past its limit that is not listed, a listed file that fits and a listed file that is gone
    Then it names the grown file, the shrunk file, the unlisted file, the file that fits and the file that is gone
    And it names no file held at its listed size or within its limit
