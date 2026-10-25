---
n: 13
title: "Recall and the envelope: the six recall tools, scopes and recall events"
status: "✅"
result: true
summary: >-
  @RCL-01, @RCL-03, @RCL-04, @RCL-05, @RCL-07, @SEC-06 and @SEC-07 off
  @pending and passing. The six recall tools answer in the envelope under
  the 8,000-token cap and a cursor, in the calling run's scope unless the
  call widens it, and each leaves a recall event; the envelope's golden
  files match. @RCL-08 stays @pending on purge and quarantine (phase 14).
---
# Phase 13 result

## Handoff

Outcome:

- RCL-01, RCL-03, RCL-04, RCL-07: off `@pending`, passing against the
  built core. `event_expand`, `landmark_list`, `pin_list` and `stat_list`
  join the tools; the last answers a structural envelope of counts. An
  event past the 8,000-estimated-token cap is cut, its item's
  `continuation` holding `cut`, and its cursor continues it with
  `continued`, without gap or overlap; a cut counts `recall_truncated`.
  `flags` holds PRV-07 marks alone.
- RCL-05 passes on a bound run: `run` is the writers ingest recorded for
  it, `room` and `rooms` need the parameter and a seat, and each wider
  call is one audit entry. An unbound server serves `room` for the
  personal room alone. A call refused for its scope, and `event_expand`
  over a record gap, count `recall_refused`. `pin_list` and `stat_list`
  take `scope` and `room` like the rest.
- RCL-07: each call that returns an envelope appends a recall event to
  the device seat's writer (provenance `assistant`, redacted arguments,
  the returned addresses), no `text`. The scenario reads the stat
  "recalls" through `stat_list` before and after.
- SEC-06, SEC-07 pass. The static check refuses `TrustedText` built
  outside `restore_block`, and aliased by `as`, `type` or `<..>::`
  anywhere; a text check, no type-level guarantee. `parinn event` indents
  every content line, so none reads as a header, and shows tag
  characters and variation selectors as placeholders. Golden files
  cover a continued item and a tombstone; the round trip and
  `envelope_encoder` cover score, structural, continuation, tombstones.
- Live ingest records its run seats' writers (REC-13); the S8 estimator
  lives in `envelope::tokens`; writer labels (RCL-08) resolve in short,
  range, full and ASCII forms.

Inherited:

- `mcp::{scope, expand, cursor, recording}`, `mcp::RecallWriter`,
  `scope::RecallScope`, `store::{RecallReach, Stats}`, `Store::head`,
  `administration::label`, `record::range`, `Counter::{RecallTruncated,
  RecallNotRecorded, RecallRefused}`, `envelope::{Continuation,
  Tombstone}`, `engineering::trusted_text`.
- Plan 2609292013's `restore_block` crate offers exactly the four
  `TrustedText` constructors; its inside-the-crate drift case waits for
  its manifest.

For the stakeholder:

- RCL-08 stays pending on purge and quarantine (phase 14).
- No M1 command joins a room, so RCL-05 writes a join's record in its
  steps; foreign rooms wait for RCL-10.
- `features/recall.feature` line 96 now reads: the stat "recalls"
  increases by 1 (a scenario-text edit). SRS §9.3 must say
  `continuation` (branch recall-docs); §9.2 should list `scope` and
  `room` for `pin_list` and `stat_list`.
- `recall_refused` is a new counter. Item `integrity` is fixed
  (`unsigned`, `unverified` on hits): derive per writer or omit?
- Recall events from a device seat carry provenance `assistant`:
  right word? `tombstones` still empty until purge. Windows reserved
  file names in the repository have no check yet.
- The agent reviews of this phase ran by the lead; fixes are in.

Parked: purge and quarantine in recall (phase 14); recall taint and the
recall event of a command line (OQ-50 (h)); `parinn event` scopes
(VIEW-09).
