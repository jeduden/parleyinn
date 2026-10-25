---
id: ADR-2610142221
title: "The test stack and the core's JSON: cucumber, serde and serde_json"
status: accepted
scope: dependencies
summary: >-
  The scenarios run through cucumber-rs and the tooling decodes JSON
  through serde. serde_json reads and writes JSON in the tooling and,
  from M1, in the core executable: the MCP server, the store's
  content, the envelope and the canonical encoding.
---
# ADR-2610142221: The test stack and the core's JSON

## Context

[ADR-2610101913](ADR-2610101913-test-stack-after-go.md) chose
cucumber, serde and serde_json for tests and tooling alone. The MCP
server of [ADR-2610102211](ADR-2610102211-mcp-server-in-house.md)
reads and writes JSON-RPC through serde_json, and that record says the
change linking it into the core records it. Phase 1 of plan
[2609292012](../../plan/2609292012_m1-record-and-recall/plan.md) does:
the MCP server, the store's content, the envelope (§9.3), seat
certificates, key events and the canonical encoding all hold JSON
values. This record supersedes ADR-2610101913.

## Decision

| Module       | Purpose                                                                                                                                             | License           | Maintenance                                        |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------- | -------------------------------------------------- |
| `cucumber`   | Runs the Gherkin scenarios under features/, and parses them for the scenario gate through its `gherkin` re-export; tests only                       | MIT OR Apache-2.0 | Active; cucumber-rs organisation, regular releases |
| `serde`      | Derives the strict decoding of the review outcome, the check runs and the finding ledger; tooling only                                              | MIT OR Apache-2.0 | Active; the de-facto Rust serialisation framework  |
| `serde_json` | Reads and writes JSON: in the core, the MCP server's messages, event content, the envelope, certificates and key events; in the tooling, its inputs | MIT OR Apache-2.0 | Active; maintained beside serde                    |

The core uses `serde_json::Value` alone, with no derive. The canonical
encoding (RFC 8785) is the `record` crate's own writer over a
`Value`, since serde_json's output is not canonical: it sorts members
by UTF-16 code units and refuses numbers that are not safe integers.

## Alternatives

- **A JSON reader of ParleyInn's own.** Parser code the core must fuzz and
  keep correct, for one dependency saved
  ([ADR-2610102211](ADR-2610102211-mcp-server-in-house.md) weighed
  it).
- **`simd-json` or `sonic-rs`.** Faster, with `unsafe` the core would
  link for speed it does not need.
- **serde's derive in the core.** It brings the proc-macro crates into
  the core's build for types `Value` serves as well.

## Consequences

- The core's reach test allows `serde_json`, `serde_core`, `itoa`,
  `memchr` and `zmij`. Cargo unifies features across the workspace, so
  its metadata also shows `serde` with its derive, which the tooling
  turns on, and the proc-macro crates; they run at build time only.
- Three direct dependencies here; the workspace's direct set is seven
  with this phase's four, against ENG-18's target of ten.
