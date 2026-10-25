# Spike S7 report: rmcp or an in-house MCP server

Spike S7 asks how `parinn mcp` speaks MCP over stdio: through the Rust
MCP SDK `rmcp` with HTTP off, or through a minimal server of ParleyInn's
own (§9.2, §12.1). The decision record is ADR-2610102211.

## Decision

An in-house stdio server on `serde_json`. Both stubs speak the
protocol §9.2's MCP tools need, and each passed the same checks
against a raw JSON-RPC driver and against the reference TypeScript
client. The reach decides:

- `rmcp` puts 50 crates in the core executable's normal dependency
  graph, among them `tokio`, `tokio-util`, `futures`, `chrono`,
  `uuid`, `getrandom` and `schemars`. The reach test
  ([reach.rs](../../crates/parinn/tests/reach.rs)) would need an entry
  for each.
- The in-house server puts 5 there: `serde_json` and its own
  `serde_core`, `itoa`, `memchr` and `zmij`.
- `tokio` and `rmcp` carry socket and program-start code behind Cargo
  features (`net`, `process`, `transport-child-process`, the HTTP
  transports). Cargo unifies features across the workspace, so the
  core's reach would rest on no other crate ever turning one on. The
  in-house server has no such code to turn on.

Speed does not decide: both answer a stub `event_search` in well under
a millisecond.

## Setup

- Candidates: `rmcp` 3.5.1 (Apache-2.0) with `default-features =
  false` and the features `server` and `transport-io`, on `tokio`
  1.53.2 with `rt`, `macros`, `io-std` and `time`; and an in-house
  server on `serde_json` 1.0.152 and std threads.
- Both stubs live in [spike](spike/), each a standalone crate outside
  the parinn workspace: [sdk](spike/sdk/src/main.rs) and
  [inhouse](spike/inhouse/src/main.rs). Each serves one tool,
  `event_search`, whose answer is a stub envelope with
  `structuredContent` and the same JSON as a text block. The query
  `slow` stands in for a long search: it waits up to 30 s for a
  cancellation.
- [drive.py](spike/driver/drive.py) drives a stub with raw JSON-RPC:
  `initialize`, `tools/list`, 200 `tools/call` round trips, a cancelled
  call, `ping` and an unknown method.
  [client.mjs](spike/driver/client.mjs) does the same through
  `@modelcontextprotocol/sdk` 1.32.0, the reference client, aborting
  the slow call through its `AbortSignal`.
- Host: 4 vCPU, Linux, Rust 1.99.0, release profile as the workspace's
  (one codegen unit, thin LTO, stripped).

## Results

| Check                                 | `rmcp` 3.5.1         | In-house             |
| ------------------------------------- | -------------------- | -------------------- |
| Protocol version negotiated           | 2025-06-18, as asked | 2025-06-18, as asked |
| `tools/list`, `tools/call`            | pass                 | pass                 |
| `structuredContent` returned          | pass                 | pass                 |
| Cancelled call answered               | never (correct)      | never (correct)      |
| `ping` after the cancellation         | answered             | answered             |
| Unknown method                        | error -32601         | error -32601         |
| Reference TypeScript client           | pass, abort included | pass, abort included |
| Crates in the normal dependency graph | 50 (9 proc-macros)   | 5                    |
| Stripped release binary               | 2.05 MB              | 0.48 MB              |
| Clean release build                   | 40.4 s               | 5.3 s                |
| Startup to `initialize` answered      | 1.8–2.5 ms           | 1.2–1.4 ms           |
| `tools/call` round trip, p50 / p95    | 0.09 / 0.09–0.12 ms  | 0.08 / 0.12–0.18 ms  |
| Socket or program-start libc imports  | none                 | none                 |
| Lines of server code in the stub      | 72                   | 129                  |

The two ranges are two runs. Every license in either graph is MIT,
Apache-2.0, or offered as one of them, except `unicode-ident`'s added
Unicode-3.0, a proc-macro dependency of `rmcp`'s graph that
`deny.toml` already excepts for the tooling.

### Reach in source

A search of each linked crate's `src/` for `std::net`, `tokio::net`,
`TcpStream`, `UnixStream` and `process::Command` found:

- `tokio`: 41 files, all behind its `net` and `process` features, which
  this build leaves off.
- `rmcp`: 5 files, in its HTTP, auth and unix-socket transports, behind
  features this build leaves off.
- `tokio-util`: 4 files, behind `tokio`'s `net` feature.
- `serde_json`: one doc comment. `schemars` and `serde`: schema and
  serialisation impls for `std::net` address types, which open nothing.

Neither binary imports `socket`, `connect`, `bind`, `listen`, `accept`,
`fork`, `execve` or `posix_spawn` from libc.

## What the in-house server must carry

The stub is the whole protocol surface `parinn mcp` needs for §9.2:
`initialize` with version negotiation, `notifications/initialized`,
`ping`, `tools/list`, `tools/call` with structured content and
`isError`, and `notifications/cancelled`. M1 adds, beyond the stub:

- a deadline on every call, fired into the same cancel signal
  (ENG-03), and a `catch_unwind` around each call that fails open and
  audits (ENG-04);
- tool input validated against each tool's schema, and a bound on the
  line length read from stdin (SEC-05);
- the protocol versions it accepts as a list, refreshed when the MCP
  specification revises, and a contract test that replays recorded
  client exchanges (ENG-17).

## Deviations from the plan

- The plan predates the move to Rust and named the MCP Go SDK; the
  SRS's §12.1 row asks about `rmcp`, and this spike measured that.
- The stub tool is `event_search`, §9.2's name, rather than `search`.
- The import-closure test is the Rust reach test now. The spike
  measured each stub's graph by the same rule, normal edges from the
  root, with `cargo tree -e normal`, rather than linking either into
  `parinn` before M1 needs it.
