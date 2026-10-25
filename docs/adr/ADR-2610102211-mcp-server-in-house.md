---
id: ADR-2610102211
title: "The MCP server: ParleyInn's own stdio server on serde_json, not rmcp"
status: accepted
summary: >-
  `parinn mcp` speaks MCP through a minimal stdio server of ParleyInn's own,
  reading and writing JSON through serde_json. The Rust MCP SDK rmcp
  works, but links 50 crates into the core and keeps its sockets and
  program starts behind Cargo features. Spike S7 answers §12.1's
  question with this record.
---
# ADR-2610102211: The MCP server

## Context

The MCP server is the core component that serves the MCP tools of
§9.2 to one harness session over stdio. It is part of B0: it opens no
socket and starts no program (I4, SEC-01). The core executable's
reach test fails the build when a crate links into `parinn` that its
reach does not allow, and ENG-18 targets ten direct dependencies.

Spike S7 (§12.1) asks whether the Rust MCP SDK `rmcp`, with HTTP off,
fits: stdio, structured content and cancellation. The spike built
`event_search` as a stub tool both on `rmcp` 3.5.1 and on a minimal
server of ParleyInn's own, and drove each with raw JSON-RPC and with the
reference TypeScript client. The numbers are in the [spike
report][report].

## Decision

`parinn mcp` is a stdio MCP server of ParleyInn's own. It speaks the subset
§9.2 needs: `initialize` with protocol-version negotiation,
`notifications/initialized`, `ping`, `tools/list`, `tools/call` with
one text block and `isError`, and `notifications/cancelled`. The text
block is the envelope in §9.3's field order, its warning before any
recalled content. The call returns no `structuredContent`: `serde_json`
writes an object's keys in sorted order, which would put an item's
content ahead of the warning. It
reads and writes JSON through `serde_json`, and runs each call on a
std thread whose cancel signal a cancellation or the call's deadline
fires (ENG-03). No async runtime links into the core.

It adds no direct dependency. When M1 links the server into `parinn`,
`serde_json` joins the core: the ADR that lists it then says it links
into the core executable, and the reach test allows `serde_json`,
`serde_core`, `itoa`, `memchr` and `zmij`, under review.

## Alternatives

- **`rmcp` 3.5.1 with `server` and `transport-io` only.** It passed
  every check the in-house stub passed, in 72 lines against 129. It
  puts 50 crates in the core's graph (`tokio`, `tokio-util`,
  `futures`, `chrono`, `uuid`, `getrandom`, `schemars` among them),
  quadruples the binary (2.05 MB against 0.48 MB) and its clean build
  time. `tokio` and `rmcp` keep socket and program-start code behind
  Cargo features, and Cargo unifies features across a workspace: the
  core's reach would hold only while no other crate turned `net` or
  `process` on. The reach test would catch a new crate, not a new
  feature of an allowed one.
- **The MCP server on `serde_json::Value` without `serde_json`.** A
  JSON reader of ParleyInn's own saves one dependency, but it is parser
  code the core must fuzz and keep correct, and the store, the
  envelope and the hook handlers read and write JSON too.

## Consequences

- The MCP specification revises. The server keeps the protocol
  versions it accepts as a list, and a contract test replays recorded
  client exchanges against it (ENG-17), so a revision shows up as a
  failing test rather than a broken harness session.
- The server owns its entry point: each call runs under `catch_unwind`
  that fails open and audits (ENG-04), and the line length read from
  stdin and each tool's input are bounded and validated (SEC-05).
- [ADR-2610101913](ADR-2610101913-test-stack-after-go.md) says
  `serde_json` serves tests and tooling only. The change that links it
  into `parinn` updates that record, or supersedes it, in the same
  change.
- If a later need (an MCP feature the subset lacks, or a transport
  beyond stdio) outgrows the in-house server, this record is
  revisited with `rmcp` measured again; stdio alone never needs it.

[report]: ../../plan/2609292009_spike-s7-mcp-sdk/report.md
