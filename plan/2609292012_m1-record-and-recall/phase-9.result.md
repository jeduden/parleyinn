---
n: 9
title: "The hook handlers: hook observations held at the ingest marker"
status: "✅"
result: true
summary: >-
  @SEC-16 and @NFR-02 off @pending and passing. Hook handlers decode
  each hook input against its schema and recorded shapes, and hold
  their observations at the ingest marker within their hook budgets;
  ingest uses them for user lines and prompt matches. PRV-04 rows 1-3,
  PRV-08, REC-19 and REC-26's hook steps pass narrowed; NFR-01 and
  NFR-02 benchmarked.
---
# Phase 9 result

## Handoff

Outcome:

- SEC-16 holds: `@SEC-16` is off `@pending`. A field a handler uses
  must have its type, an id a plain id, `hook_event_name` its hook;
  else `hook_input_rejected`, fail-open.
- NFR-02 holds: `@NFR-02` is off `@pending`. Each handler answers by
  its §9.1 budget on a thread of its own, else `hook_budget_exceeded`.
  Its start marker, left past its harness timeout, is counted once
  (`hook_unfinished`) by the next hook handler, `parinn mcp` as it
  starts, or `parinn status`.
- REC-19's hold: no handler appends to a writer. `SessionStart` holds
  the session binding, `source`, worktree and process ancestry;
  `UserPromptSubmit` the `prompt_id`, the attended mark and a prompt
  commitment (PRV-04, PRV-08). Paths are redacted first (SEC-08). A
  binding untaken 30 s is counted once (`session_binding_not_taken`).
- Phase 4's question: while a principal settings layer fails, a hook
  keeps its observation but drops the commitment, counted.
- `parinn ingest` appends each held observation and each prompt match
  as structural events of the device seat. A typed line is `user` once
  ancestry is observed, not nested. Conflicts, and matches without the
  attended mark in `interactive` mode, are counted. A fork observation
  holds a fork; a held source is read again.
- ENG-17: hook input shapes regenerate byte for byte; all 47 recorded
  inputs of installed hooks decode with no rejection or drift; two
  drift cases.
- ENG-07: `fuzz/` gains `hook_input_decoder`, from recorded inputs;
  90 s locally, 200,075 runs, no crash.
- ENG-15, release build, fresh home: `SessionStart` p95 15 ms (NFR-02
  Target), `UserPromptSubmit` p95 15 ms (NFR-01 Floor).
- I2, I6, I8 and I9 hold: no user event is trusted, every rejection,
  drift, overrun and stopped handler is counted, a refused home is left
  untouched, and every path fails open.

Inherited:

- `hook`: `decode`, `Decoded`, `HookInput`, `shape`, `ancestry`
  (`Ancestry`, `ProcFs`, `own`), `Hook::budget`, `harness_timeout`.
- `store::hook`: `Observation`, `Held`, `PromptCommitment`;
  `hold_observation`, `hook_observations`, `prompt_commitment`,
  `append_observed`, `untaken_bindings`, `count_binding`;
  `Store::open_read_only_made`.
- `ingest`: `observed`, `prompt`; `Inputs::interactive`; four new
  `Ingested` counts. Event kinds `hook_observation`, `prompt_match`;
  eight counters.
- `parinn::hook_handlers`: `count_unfinished`, `count_untaken`;
  `Context` gains `attended`, `hook_budget`, `ancestry`, `hook_hangs`.
  Seams `PARINN_TEST_CLOCK`, `PARINN_TEST_HOOK_BUDGET_MS`,
  `PARINN_TEST_FAULT=hook-hang`.
- Steps in `record_hooks.rs`, `provenance_hooks.rs`,
  `security_hooks.rs`, `non_functional.rs`.

For the stakeholder:

- `@PRV-04`'s last row cannot pass in M1: a trusted `user` event needs
  trust (phase 11) and a signed sandbox state (OQ-40). Rows 1 to 3
  pass narrowed; the gate's "`@PRV-04` passes" is raised.
- REC-19's holding step fails, naming its MCP server half (phase 10).
  Until then every harness session counts an untaken session binding.
- Hook observations are structural events in the device seat; PRV-08
  wants them untrusted while R3 or R8 is open. Phase 11 must.
- `hook_rejected` (malformed) and `hook_input_rejected` (schema) stay
  two counters. A repository key is now audited "rejected repository
  setting", PRV-04's words.
- No `PermissionRequest` hook input is recorded (S9): only its schema
  is checked. A harness process is one named `claude`.
- Every hook checks the whole home (SEC-02), payloads included.
- Stakeholder paths: `tooling/`, `features/`, `crates/hook/Cargo.toml`.
  New terms: process ancestry, nested, prompt match, hook input shapes.

Parked:

- The MCP server taking bindings and appending held observations, and
  `@REC-04`'s home G: phase 10. Homes A, B, H and `@REC-22`'s ingest
  marker, which need several homes, with them.
- `PermissionRequest` decisions (OWN-04); restore blocks (M2).
- Purge erasing prompt commitments: phase 14, SEC-31.
- Process ancestry off Linux; a staged start marker a kill leaves.
