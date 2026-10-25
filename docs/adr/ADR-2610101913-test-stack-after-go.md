---
id: ADR-2610101913
title: "The test stack after Go: cucumber, serde and serde_json"
status: superseded
superseded-by: ADR-2610142221
scope: dependencies
summary: >-
  The scenarios run through cucumber-rs, and the tooling and the tests
  read JSON through serde and serde_json. All three serve tests and
  tooling only; none links into the core executable, whose reach test
  refuses any crate.
---
# ADR-2610101913: The test stack after Go

## Context

Phase 4 of plan
[2610050706](../../plan/2610050706_toolchain-to-rust/plan.md) ports
the `parinn version` stub to the Rust core executable and removes the
last Go source, as
[ADR-2610050528](ADR-2610050528-language-rust-typescript.md) decided.
testify, which asserted in the Go tests, leaves with them. The
repository tooling still runs every scenario under `features/`, keeps
them in step with the SRS, and reads three JSON inputs: the reviewing
agent's review outcome, GitHub's check runs, and the domain model's
finding ledger. The review gate must refuse an unknown field in the
review outcome (ENG-28). The core executable's tests read its
`version --json` output and `cargo metadata`. Nothing chosen here may
link into a shipped executable (I4).

This record supersedes
[ADR-2610101442](ADR-2610101442-rust-test-stack.md), which also named
testify for the Go tests that then remained.

## Decision

| Module       | Purpose                                                                                                           | License           | Maintenance                                        |
| ------------ | ----------------------------------------------------------------------------------------------------------------- | ----------------- | -------------------------------------------------- |
| `cucumber`   | Runs the Gherkin scenarios under features/, and parses them for the scenario gate through its `gherkin` re-export | MIT OR Apache-2.0 | Active; cucumber-rs organisation, regular releases |
| `serde`      | Derives the strict decoding of the review outcome, the check runs and the finding ledger                          | MIT OR Apache-2.0 | Active; the de-facto Rust serialisation framework  |
| `serde_json` | Reads and writes that JSON, refusing trailing data; reads the core executable's output and cargo's in its tests   | MIT OR Apache-2.0 | Active; maintained beside serde                    |

The scenario gate reads features through the same `gherkin` parser
the runner uses, so the two never disagree about which scenarios
exist. The runner drives cucumber on a twenty-line executor of its
own, so no async runtime is a dependency. Temporary directories come
from a small test-only crate in the workspace, not from `tempfile`.
The core executable writes its one JSON object by hand rather than
link serde_json.

## Alternatives

- **Plain `#[test]` functions instead of cucumber.** They lose the
  requirement-readable scenario text the matrix is built on.
- **The `gherkin` crate directly for the gate.** One more direct
  dependency, and a version that could drift from the one cucumber
  runs.
- **`serde_json::Value` walked by hand instead of `serde`'s derive.**
  It saves one dependency, but every field check is code the derive's
  `deny_unknown_fields` already gives, tested.
- **`tokio` or `futures` for the executor, `tempfile` for temporary
  directories.** Each is one more direct dependency for a few lines of
  code.
- **serde_json in the core executable for `version --json`.** One
  object with two string fields does not earn a dependency in the
  core's reach.

## Consequences

Three direct dependencies, all tests and tooling, against ENG-18's
target of ten. cucumber brings about 125 crates transitively; none
reaches a shipped executable, and the core executable's reach test
fails the build if one does. Every license is offered as MIT or
Apache-2.0, both on ENG-18's allow-list. The gherkin parser reads a
description line that opens with a Gherkin keyword, such as
"Scenarios", as that keyword, so a feature's description must not
open a line with one. It also refuses an outline placeholder that no
Examples column fills, so a literal placeholder in an outline's step
is written another way, such as `{id}`.
