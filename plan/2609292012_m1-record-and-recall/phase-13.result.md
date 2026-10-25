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
  event past the 8,000-estimated-token cap is cut, flagged `cut`, and its
  cursor continues it flagged `continued` without gap or overlap; a cut
  counts `recall_truncated`.
- RCL-05 passes on a bound run: `run` is the writers ingest recorded for
  it, `room` and `rooms` need the parameter and a seat, and each wider
  call is one audit entry. Live ingest now records its run seats'
  writers, which it did not (REC-13).
- RCL-07: each call that returns an envelope appends a recall event to
  the device seat's writer (provenance `assistant`, redacted arguments, the
  returned addresses). It has no `text`, so no search finds it (OQ-11).
- SEC-06, SEC-07 pass. A static check refuses `TrustedText` built outside
  `restore_block`, and inside it by anything but `fixed("..")`, `pin`,
  `field` and `join`; it has a drift case. The envelope's golden files,
  round-trip property and fuzz target (`envelope_encoder`) hold.
- The S8 estimator now lives in `envelope::tokens`, its sample under
  `testdata/model-tokens`, 99% bound tested.
- Writer labels are structural events in the device seat's writer; short,
  range, full and ASCII forms resolve; `parinn event search|expand` prints
  with each control or direction character as `<U+XXXX>`.

Inherited:

- `mcp::{scope, expand, cursor, recording}`, `administration::label`,
  `store::{Reach, Counts}`, `record::range`, `EventKind::{Recall,
  WriterLabel}`, `Counter::{RecallTruncated, RecallNotRecorded}`,
  `Occurrence::RecallExtended`, `engineering::trusted_text`.
- Plan 2609292013's `restore_block` crate must offer exactly the four
  `TrustedText` constructors; its inside-the-crate drift case waits for
  its manifest.

For the stakeholder:

- RCL-08 stays pending: the purge and quarantine clause needs phase 14;
  its step fails naming it. Its other clauses pass for all five rows.
- `recall_calls` is the recall events the record holds, no audit counter:
  one would fail `parinn doctor` on every recall (OQ-50 (i)).
- No M1 command joins a room, so RCL-05 writes a join's record in its
  steps. An unbound server takes `room` for any room named; foreign
  rooms wait for RCL-10.
- The envelope still shows the full address; no M1 surface shows the
  short form. New words: flags `cut` and `continued`, `structural`.
- The domain-model, persona-security-officer and test-engineer agents
  could not be started from this session; a reviewer should run them.
- Docs-only pull request: RCL-03's cap frozen, §8.4 and §9.3 worded.
- Stakeholder paths: `tooling/scenario` (steps, a dev dependency),
  `tooling/engineering`, `tooling/drift`, `features/`, `fuzz/`.

Parked: purge and quarantine in recall (phase 14); recall taint and the
recall event of a command line (OQ-50 (h)); `parinn event` scopes (VIEW-09).
