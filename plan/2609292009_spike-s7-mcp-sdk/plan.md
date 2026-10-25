---
id: 2609292009
title: "Spike S7: the Rust MCP SDK or an in-house server"
status: "✅"
summary: >-
  Evaluate the Rust MCP SDK rmcp for stdio, structured content and
  cancellation against a minimal in-house server.
model: sonnet
depends-on: []
---
# Spike S7: the Rust MCP SDK or an in-house server

## Goal

Choose how `parinn mcp` speaks MCP, weighing the SDK's fit against
ENG-18's dependency budget and SEC-01's reach rules.

## Context

Informs §9.2 and RCL-01. The SDK must not pull sockets or program
starts into the shipped binary. Since the move to Rust, §12.1 asks
about `rmcp` with HTTP off, not the MCP Go SDK.

## Tasks

1. [x] Build `event_search` as a stub tool with the SDK and with a
   minimal stdio server
2. [x] Check structured content, cancellation, and the dependency
   graph of each
3. [x] Record the decision as an ADR, naming the SDK crate if it wins
   (ENG-18, ENG-26)

## Outcome

The [spike report](report.md) holds the numbers, and
[ADR-2610102211](../../docs/adr/ADR-2610102211-mcp-server-in-house.md)
proposes the decision: an in-house stdio server on `serde_json`. Both
stubs passed every protocol check, against a raw JSON-RPC driver and
the reference TypeScript client. `rmcp` puts 50 crates in the core's
graph against 5, and keeps its sockets and program starts behind Cargo
features the workspace could turn on.

Deviations from the tasks:

- The stub tool is `event_search`, §9.2's name, not `search`.
- Task 2 measured each stub's graph with `cargo tree -e normal`, the
  reach test's rule, in a standalone crate under [spike](spike/),
  rather than linking either into `parinn` before M1 needs it.

M1's task 6 links the server into `parinn`. It allows the five crates
in the reach test and updates the ADR that lists `serde_json`. It also
adds what the report lists: a deadline, `catch_unwind`, input bounds
and a contract test.

## Acceptance Criteria

- [x] A decision is recorded in
  [docs/srs/09-interfaces.md](../../docs/srs/09-interfaces.md) or an ADR
- [x] The import-closure test passes with the chosen approach: the
  in-house server's graph is `serde_json` and four crates of its own,
  each an entry the reach test takes when M1 links it
