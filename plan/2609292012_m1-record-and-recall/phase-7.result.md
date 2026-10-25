---
n: 7
title: "The transcript parser and provenance"
status: "✅"
result: true
summary: >-
  @REC-05 and @PRV-03 off @pending and passing. Every transcript line
  is checked against its version's recorded shape and classed from its
  structure; unparsed and drifting lines are kept, counted and audited.
  PRV-08, REC-26, PRV-01 and ENG-17's M1 steps pass narrowed; three
  fuzz targets run nightly.
---
# Phase 7 result

## Handoff

Outcome:

- REC-05 holds: `@REC-05` is off `@pending`, passing against the built
  core, confined. A malformed line and a line of an unknown type are
  `unparsed` events holding the redacted line, counted
  (`events_unparsed`) and audited.
- PRV-03 holds: `@PRV-03` is off `@pending`. Its trust step reads
  `event_get`, where every `ingested` event is untrusted (PRV-02).
- PRV-08's structural rules hold in the `provenance` crate. 14 of
  `@PRV-08`'s 19 rows pass, narrowed locally; none comes off `@pending`.
  No line is `user` until a hook observation records ancestry.
- REC-26's transcript half holds: the `transcript` crate's recorded
  shapes, generated from `testdata/claude-code`, regenerate byte for
  byte. Each drift is audited naming version and difference
  (`format_drift`). Rows 1 and 2 of `@REC-26` pass narrowed.
- `@PRV-01`'s transcript steps and `@ENG-17`'s replay and
  residual-identifier steps pass narrowed. All 32 recorded transcripts
  replay with no unparsed line or drift (`tests/contract.rs`).
- ENG-07: `fuzz/` fuzzes the transcript parser, the redactor and the
  configuration parser. Each ran 90 to 120 s locally under cargo-fuzz
  0.13.2, no crash; the nightly job runs each for 15 minutes.
- `parinn ingest --all` reads every transcript under the roots, as
  `--path` reads one; `parinn verify` accepts a transcript's untrusted
  `harness_meta` in an ingest seat.
- I1, I2 and I6 hold: no line is dropped, no transcript line is
  trusted, and every unparsed line and drift is counted.

Inherited:

- `transcript`: `parse`, `Shapes::supported`, `RecordedShapes`,
  `Drift::describe`. `provenance`: `Classifier`, `Session`.
- `ingest::transcripts`; `Ingested::unparsed`, `drifts`; event kind
  `transcript_line`.
- `harness-contract`: `replay`, `residual`, `shapes`;
  `PARINN_REGENERATE_SHAPES=1` rewrites the shapes.
- Steps in `record_transcript.rs`, `provenance.rs`,
  `engineering_contract.rs`; three drift cases.

For the stakeholder:

- The gate's `@PRV-01`, `@PRV-08` and `@REC-26` cannot pass here. Their
  acts, pins, posts, canary and backup, prompt matches and ancestry,
  and live MCP server come later; PRV-08's first row needs OQ-40.
  PRV-01 cannot pass in M1, which lists it in full.
- `@REC-05` needed phase 8's `--all`: it landed without ingest
  positions, so a second run duplicates, as `--path` already does.
- The SRS names no tool whose result is `web`, `file` or
  `subagent_result`; this phase chose `WebFetch`/`WebSearch`, `Read`
  and `Agent`/`Task`.
- The domain model gains the transcript line and the recorded shape.
- ADR-2610171200 is accepted on the stakeholder's ruling of 19
  October 2026: libfuzzer-sys adds NCSA, admitted by a per-crate
  exception in `deny.toml`, not on the allow-list. The ruling was given
  in chat and awaits its consent comment on jeduden/parleyinn#60. ENG-18's
  check now reads `fuzz/Cargo.toml`, and CI runs cargo-deny and
  cargo-audit over `fuzz/Cargo.lock`.
- `nightly.yml` changed. CODEOWNERS names `crates/provenance/`, not
  `crates/transcript/`.

Parked:

- A tool call ingested apart from its result, runs and ingest
  positions for `--all`, and linked directories: phase 8.
- Hook input shapes, `@REC-26`'s hook row, ancestry and prompt
  matching: phase 9. Live drift detection: phase 10.
- A compaction's derived structural event; its line is `harness_text`.
- `fuzz/` locks `serde_json` 1.0.154, the workspace 1.0.151.
