---
id: ADR-2610101442
title: "The test stack after the move to Rust: cucumber, serde, serde_json and testify"
status: superseded
superseded-by: ADR-2610101913
scope: dependencies
summary: >-
  The repository tooling in Rust runs the scenarios through cucumber-rs
  and reads JSON through serde and serde_json; testify asserts in the
  Go tests that remain. All four serve tests and tooling only; none
  links into a shipped executable.
---
# ADR-2610101442: The test stack after the move to Rust

## Context

Plan [2610050706](../../plan/2610050706_toolchain-to-rust/plan.md)
moves the repository tooling from Go to Rust, as
[ADR-2610050528](ADR-2610050528-language-rust-typescript.md) decided.
The tooling runs every scenario under `features/`, keeps them in step
with the SRS, and reads three JSON inputs: the reviewing agent's
review outcome, GitHub's check runs, and the domain model's finding
ledger. The review gate must refuse an unknown field in the review
outcome (ENG-28). Until phase 4 of that plan, the `parinn version`
stub and its import-closure test stay Go. Nothing chosen here may link
into a shipped executable (I4).

This record supersedes
[ADR-2609292234](ADR-2609292234-test-stack.md), whose godog and
cucumber messages modules left with the Go gates.

## Decision

| Module                        | Purpose                                                                                                           | License           | Maintenance                                        |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------- | ----------------- | -------------------------------------------------- |
| `cucumber`                    | Runs the Gherkin scenarios under features/, and parses them for the scenario gate through its `gherkin` re-export | MIT OR Apache-2.0 | Active; cucumber-rs organisation, regular releases |
| `serde`                       | Derives the strict decoding of the review outcome, the check runs and the finding ledger                          | MIT OR Apache-2.0 | Active; the de-facto Rust serialisation framework  |
| `serde_json`                  | Reads and writes that JSON, refusing trailing data                                                                | MIT OR Apache-2.0 | Active; maintained beside serde                    |
| `github.com/stretchr/testify` | assert and require helpers in the Go tests that remain (test only)                                                | MIT               | Active; the de-facto Go assertion library          |

The scenario gate reads features through the same `gherkin` parser
the runner uses, so the two never disagree about which scenarios
exist. The runner drives cucumber on a twenty-line executor of its
own, so no async runtime is a dependency. Temporary directories come
from a small test-only crate in the workspace, not from `tempfile`.

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

## Consequences

Four direct dependencies, all tests and tooling, against ENG-18's
target of ten; testify leaves with the last Go source. cucumber brings
about 125 crates transitively; none reaches a shipped executable.
Every license is MIT, or offered as MIT or Apache-2.0, both on
ENG-18's allow-list. The gherkin parser reads a description line that
opens with a Gherkin keyword, such as "Scenarios", as that keyword, so
a feature's description must not open a line with one. It also refuses
an outline placeholder that no Examples column fills, so a literal
placeholder in an outline's step is written another way, such as
`{id}`.
