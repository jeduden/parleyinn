---
n: 7
title: "The transcript parser and provenance"
status: "✅"
result: false
---
# Phase 7: the transcript parser and provenance

Requirements. REC-05, REC-26, PRV-08, PRV-03 and PRV-01's closed set
on every line `parinn ingest` records; ENG-17's contract replay and
residual-identifier check; ENG-07's first `fuzz/` crate. `@REC-05` and
`@PRV-03` come off `@pending`; the other rows keep steps a later phase
reaches, their M1 part bound and passing.

The recorded shapes, a new `transcript` crate:

- Per supported harness version and line type (an attachment by its
  type, a `system` line by its subtype): the keys every recorded line
  carries and those any carries. Generated from every transcript under
  `testdata/claude-code`; a test regenerates them byte for byte, with
  its drift case (ENG-27).
- A line of a type no supported version records is `unparsed`, counted
  as format drift too. A line missing a needed key, naming another
  version, or of an unrecorded shape is kept whole, redacted, as
  `harness_text`; so is a `user` or `harness_meta` line with a key
  beyond its shape. Each is audited as format drift, naming the
  version and what differed, and counted; ingest goes on.

Provenance, a new `provenance` crate: one class per line from its
structural fields, never `renderedRole` or text.

- `user` only for typed input (`promptSource` `typed` or `sdk`; not
  `isMeta`, a compaction summary, a delegated task or command output)
  in a harness session whose ancestry an observation records. Those
  observations are phase 9's, so every such line is `harness_text` now.
- A tool result takes its call's tool: an MCP server's `mcp:<server>`,
  `WebFetch` and `WebSearch` `web`, `Read` `file`, `Agent` and `Task`
  `subagent_result`, else `tool_result:<tool>`; with no call held, or
  a name not plain or changed by redaction, `harness_text`.
- An assistant tool call is `tool_call`, its text `assistant`, a
  harness refusal `harness_text`. Attachments: `instructions` `file`,
  an `mcp_instructions_delta` naming one server `mcp:<server>`, else
  `harness_text`.
- Metadata lines without free text (`cost-state`, `mode`,
  `permission-mode`, `atis-latch`, `file-history-snapshot`, the
  `turn_duration` system line) are `harness_meta`; any other line is
  `harness_text`.

Ingest:

- Kind `transcript_line` with its class, or `unparsed_line` with
  `unparsed`, audited and counted (`events_unparsed`).
- `parinn ingest --all` reads every transcript under `transcript.roots`
  as `--path` reads one; positions and runs stay phase 8's.
- `parinn verify` accepts `harness_meta` outside device seats: a
  transcript's is untrusted (PRV-08), only the launcher's trusted.

ENG-17, in `harness-contract`: each recorded transcript parses with
no unparsed line and no drift. A residual-identifier check reads each
file under `testdata/claude-code`. It refuses an email, an org id, a
home path, a connector or a skill the fixtures do not list.

ENG-07: a `fuzz/` crate, its own cargo workspace. It fuzzes the
transcript parser, seeded from the recorded transcripts, the redactor
and the configuration parser, each corpus committed. `libfuzzer-sys`
gets an ADR. The nightly fuzz step runs every target.

Where the gate's scenarios go live:

- `@REC-05`, `@PRV-03`: here. Every `ingested` event is untrusted
  (PRV-02); phases 10 and 11 recheck PRV-03 on a witnessed run.
- `@PRV-08`: rows needing no hook observation, bound. Prompt matches
  and ancestry need phase 9; row 1's signed sandbox state, OQ-40.
- `@REC-26`: rows 1 and 2, bound. Its MCP server is phase 10's; rows
  3 to 5 need phase 9.
- `@PRV-01`: the transcript steps, bound. Acts, pins, posts,
  tombstones, canary and backup come later; a call ingested apart from
  its result needs phase 8.
- `@ENG-17`: replay and residual identifiers; the nightly run is M2's.

For the stakeholder: the SRS names no tool whose result is `web`,
`file` or `subagent_result`; this phase chooses the mapping above.

Gate: `@PRV-03` and `@REC-05` pass; the bound rows of `@PRV-01`,
`@PRV-08` and `@REC-26` pass narrowed; the parser's fuzz target and
the residual-identifier drift case run.
