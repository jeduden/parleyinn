---
n: 10
title: "The MCP server and live ingest"
status: "✅"
result: true
summary: >-
  @REC-13 off @pending and passing against a live MCP server. It takes
  its own harness process's session binding, ingests its transcripts
  into run seats whose keys it holds in memory, and leaves an ingest
  marker when it stops short. @REC-06 and @PRV-14 pass narrowed:
  purge (phase 14) and event_search (phase 12) keep them pending.
---
# Phase 10 result

## Handoff

Outcome:

- REC-13 holds: `@REC-13` is off `@pending`, both rows passing
  against `parinn mcp` under a stand-in harness process, confined. A
  `SessionStart` hook handler records its nearest harness process; the
  MCP server takes only that harness process's session bindings,
  marked taken, and ingests nothing as witnessed before. It polls its
  transcripts, subagents' included, every 100 ms, resolved within the
  transcript roots or the run's worktree (SEC-18). A held hook
  observation lands in the main run's run seat where its hook fired.
  A link swapped in is refused and audited.
- A stop short of a transcript's end is audited under
  `ingest_remainder_left` (truncated, no failure) by the MCP server.
  A killed server leaves its start marker; the next MCP server or
  `parinn ingest` audits the remainder. Either resumes at the ingest
  marker, every line recorded once.
- SEC-10's M1 part: each run-seat key is minted in the MCP server's
  memory (OQ-55's option a) and certified as a run seat and no ingest
  seat, so its events are witnessed; no key reaches the key store, a
  file, a log or any output.
- PRV-14 holds: a call is served only under the run whose transcript
  line its tool-use id names, waiting for ingest at most 2 s; else it
  is refused in fixed text, counted per cause and audited. Each call
  is logged with its tool-use id, run or cause, and wait. All nine
  rows pass narrowed to `event_get`.
- REC-06: an append behind an ingest position another ingest moved is
  refused in its transaction (`PositionMoved`) and read again; a
  position moves only forward. `@REC-06` passes narrowed but for its
  purged range. ENG-08's seq property runs on three threads at once;
  `concurrent_ingest.rs` races live ingest against `parinn ingest`.
- REC-19: the holding step passes; live ingest closes its run seats'
  segments every 30 s and when its input closes. `parinn verify`
  accepts structural events in a run seat that is no ingest seat.
- REC-04: homes A, B and H bound and passing narrowed. A witnessed
  line is never a repeat reference nor held, audited
  (`witnessed_repeat_recorded`).
- I1, I4, I6 and I9 hold: no line is lost or recorded twice, the core
  opens no socket, every refusal, failure and stop is audited, and
  live ingest runs beside the calls it serves.

Inherited:

- `hook::ancestry::Ancestry::harness`; `store`: `Observation::harness`,
  `Held::taken`, `take_bindings`, `StoreError::PositionMoved`.
- `ingest::live`: `RunSeats`, `ingest_live`; `Ingested::witnessed_repeats`,
  `IngestError::position_moved`, `ATTEMPTS`.
- `mcp::tool_call`: `Runs`, `Found`, `Cause`, `WAIT`;
  `Server::with_runs`, `take_found`.
- `parinn::capture`: `Capture`, `Running`, `Finder`, `count_stopped`;
  `Context::live_limit`, seam `PARINN_TEST_INGEST_LIMIT`. `parinn mcp`
  now starts the node.
- Counters `ingest_remainder_left`, `witnessed_repeat_recorded`,
  `live_ingest_failed`, `tool_call_without_tool_use_id`,
  `tool_call_run_not_found`, `tool_call_run_ambiguous`.
- `testkit::boundary::Confinement::spawn`, `Started`; steps in
  `live.rs`, `record_live.rs`, `provenance_tool_calls.rs`,
  `record_homes.rs`.

For the stakeholder:

- `@PRV-14` names `event_search`, phase 12's, held on its branch.
  Merged, it should pass as written; the merge meets this phase in
  `mcp::Server` and `call`, `StoreError`, the counters and `parinn mcp`.
- `@REC-06` needs `parinn purge --range` (phase 14) for its purged range.
- `@SEC-10`'s last step handed run-seat keys over at launch; the MCP
  server mints them, OQ-55's recommended option. SEC-10 and its step now
  say the server mints every key, pending the stakeholder's comment.
- REC-22 stays unbound: SEC-18 still refuses its transcript from
  outside the roots (raised in phase 8).
- A session binding is matched by the harness process's id; a reused
  process id could match a stale binding. Off Linux no binding is taken.
- Unit line coverage (`cargo llvm-cov --lib`): mcp and hook 100%,
  ingest 99.9%, audit 99.7%, administration 99.6%, store 99.0%, parinn
  98.4%, its misses mostly hook handler and audit-failure paths.
- Stakeholder paths: `tooling/` (testkit, steps), `features/`.
  New terms: live ingest; an MCP server's start marker.

Parked:

- REC-04's homes C to G and I; G needs live ingest to hold a line for a
  transcript an MCP server holds a binding for.
- REC-26 against a live MCP server; sealing and rooms (M7).
- Leaving a writer an MCP server still appends to out of the
  open-segment report.
