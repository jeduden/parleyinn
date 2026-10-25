---
n: 6
title: "The store: writer logs, commitments, payloads, segments and format versions"
status: "✅"
result: true
summary: >-
  @REC-09 off @pending and passing. Content above the payload threshold
  goes to the payload store under a keyed name; segments close at turn
  ends; a newer store or segment is refused, parinn status exits 3.
  REC-12, REC-19's segment and ADM-05's status steps pass narrowed;
  10,000 kills of ingest run nightly; NFR-09 and REC-17's secret frozen.
---
# Phase 6 result

## Handoff

Outcome:

- REC-09 holds: `@REC-09` is off `@pending`, passing against the built
  core, confined. Content whose canonical encoding passes the threshold
  goes to `payloads/<name>`, the name HMAC-SHA-256 under its commitment
  key; the row keeps a preview, the name and size.
- REC-12 holds but for its last step: lines append in transcript order;
  a well-formed transcript timestamp is kept as metadata, else none.
- REC-19's segment closing: a core append closes the segment it ends;
  ingest closes one at each turn end and subagent end, and at the end.
  `parinn verify` checks segments tile each log; it and `parinn status`
  report each writer chain ended without a closed segment.
- ADM-05's M1 part (ruling 3): every segment and the store carry
  format version 1. A newer store is refused before anything in it
  changes; `parinn status` and `parinn verify` exit 3 naming it, as they
  do a newer segment.
- ENG-08's first property: `store/tests/seq_property.rs`, 64 seeded
  random records over two connections, keeps each writer gap-free.
- ENG-06 over ingest: `e2e_crash_consistency.rs` kills the confined
  core at seeded points; 1,000 kills (seed 11) passed locally. It found
  that a first start cut short failed `parinn verify`, now fixed.
- Frozen with evidence: NFR-09's overhead, 1.34x over every recording
  (`store_overhead.rs`), once events moved to a rowid table (1.78x
  before); REC-17's 256-bit secret (an ingest unit test).
- I1, I6 and I10 hold: nothing is lost to a kill, every refusal is
  named, and no clock enters the chain (a seeded re-ingest gives the
  same chain head).

Inherited:

- `store`: `append` closes, `append_open` and `close_segment`;
  `segments`, `open_segments`, `newer_segments`, `open_read_only`,
  `with_payload_threshold`, `with_disk`; `payload::{name, Disk}`.
- `NewEvent.transcript_timestamp`; `Stored.payload`; errors
  `NewerSegment` and `Payload`. `record::commitment::commit_encoded`.
- `ingest::ingest_with` and `Limits`; `node::verifying_key`;
  `testkit::boundary::Confinement::run_killed`.
- Steps in `record_store.rs`; verify classes `newer_segment`, `segment`
  and `payload`.

For the stakeholder:

- SEC-05's third row cannot pass as written: SEC-08 redacts each JSON
  string whole, PRV-08 parses the line whole. `@REC-12` waits on
  `parinn event expand` (phase 13).
- REC-12's chain head step runs two seeded homes, as the core has no
  entropy seam; REC-09's export step checks names, there being no
  export. Confirm both readings.
- Commits with stakeholder paths: the SRS freeze, `nightly.yml`, and
  `tooling/` (steps, testkit, the scenario manifest).
- A segment is kept beside the log, never an event, as REC-25's merge
  keeps chain heads; a turn end is a reply stopping for neither a tool
  call nor a pause. The domain-model agent was not consulted.

Parked:

- A payload whose append a crash rolled back stays as an orphan file;
  cleaning it is purge's (phase 14) or SEC-31's.
- The 30 s and harness-session-end closes, and excluding a writer that
  still appends from the open-segment report: phase 10.
- `parinn migrate` and its backup: M3. `doctor` and principal acts
  refuse a newer store with exit 1, not 3.
