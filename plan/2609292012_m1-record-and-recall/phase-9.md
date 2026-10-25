---
n: 9
title: "The hook handlers: hook observations held at the ingest marker"
status: "✅"
result: false
---
# Phase 9: the hook handlers

Requirements. SEC-16 on the hook input's schema; PRV-04 and PRV-08 on
the attended mark, prompt commitments and process ancestry; REC-19's
holding of a run's hook observations; NFR-02's hook budgets and start
markers. ENG-07 gets the hook input decoder's fuzz target, ENG-17 the
hook input contract replay, ENG-15 the NFR-01 and NFR-02 benchmarks.
`@SEC-16` and `@NFR-02` come off `@pending`.

The hook input decoder (SEC-16, REC-26), before anything is recorded:

- Each field a handler uses must have its type, and an id
  (`session_id`, `prompt_id`) must be a plain id; else the input is
  rejected, audited as "hook input failed schema validation" and
  counted `hook_input_rejected`. A malformed one stays `hook_rejected`.
- Its keys are checked against the hook input shapes generated from
  `testdata/claude-code`. A missing or extra key is format drift,
  audited and counted; the handler still records what it can.

What each handler records, as state beside the record, never in a
writer: no MCP server holds a run-seat key before phase 10 (REC-19).

- `SessionStart`: the session binding (`session_id`, `transcript_path`,
  `cwd`), its `source` (a fork observation when `fork`), and the
  process ancestry read from `/proc` inside the core: a harness process
  above the session's own makes it nested. Elsewhere none is recorded,
  so no line is `user` there yet.
- `UserPromptSubmit`: the `prompt_id`, whether the attended mark is in
  the handler's environment, and a prompt commitment to the redacted
  prompt under its own 256-bit key. While a principal settings layer
  fails, the commitment is dropped, counted (phase 4's question).
- `PermissionRequest`: nothing yet; OWN-04 decides later.
- Each held at the transcript's ingest marker, at the transcript's size
  when its hook fired.

`parinn ingest` uses them. A fork observation names a harness fork
(REC-04); the ancestry lets a typed line be `user`. The first prompt
line carrying a prompt commitment's `prompt_id` uses it up: matched
when marked typed and its redacted text matches; later ones conflict,
counted. In `interactive` mode a match whose handler found no attended
mark is audited and counted. Each held observation, and each match, is
appended as a structural event of the device seat; every event stays
untrusted (R3 and R8 stay open until OQ-40). A held source is read
again once the source its lines repeat is ingested or gone.

Budgets (NFR-02, §9.1): `SessionStart` 150 ms, `UserPromptSubmit` and
`PermissionRequest` 50 ms, from the handler's start. Past it the
handler answers empty, audited as `hook_budget_exceeded`. Each leaves a
start marker it clears. One left past its hook's harness timeout is
counted `hook_unfinished`, once, by the next hook handler, the MCP
server as it starts, or `parinn status`. A session binding no MCP server
took within 30 s is counted once, by the next hook handler, `parinn
status` or `parinn ingest`.

Benchmarks (ENG-15): each hook handler over a fresh home, for NFR-02.
NFR-01's store of 1M events stays awaiting.

Where the gate's scenarios go live:

- `@PRV-04`: its last row needs trust (phase 11) and a signed sandbox
  state (OQ-40); rows 1 to 3 bound.
- `@REC-19`: the holding steps bound; sealing is M7's, the MCP server
  phase 10's.
- `@NFR-02`: `parinn mcp` counts a marker as it starts, so all rows
  pass.
- `@REC-04`'s homes and `@REC-22`'s ingest marker need several homes:
  with phase 10.
- PRV-08 rows 2, 4, 5 and 19 and REC-26's hook row bound where they
  pass.

Gate: `@SEC-16` and `@NFR-02` pass, and `@PRV-04`'s first three
rows. REC-19's holding steps are bound, their MCP server half phase
10's; a hook handler fails open within its hook budget; NFR-01 and
NFR-02 are benchmarked.
