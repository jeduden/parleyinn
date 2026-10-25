---
n: 14
title: "Quarantine and parinn purge --range"
status: "✅"
result: true
summary: >-
  @RCL-06, @REC-17 and @SEC-22 off @pending and passing; so are @RCL-08
  and @REC-06, whose purged-range steps now run. A purge erases a range
  and records a tombstone in one transaction, a quarantine holds events
  out of recall by selector, and an ingest purges what a retention
  policy has aged out. @SEC-12 passes seven rows of eight and stays
  @pending: its flag row needs PRV-07 (M5).
---
# Phase 14 result

## Handoff

Outcome:

- RCL-06, REC-17, SEC-22, RCL-08, REC-06: off `@pending`, passing
  against the built core (cucumber). A purged range answers a
  `purged` tombstone and a quarantined address a `quarantined` one in
  `tombstones`; `parinn verify` accepts missing content only under a
  tombstone whose count and digest of commitments the chained headers
  confirm. REC-17's steps scan the home for the erased keys, the content
  and every unkeyed hash of a guess: none remain (SEC-14's vacuum holds).
- SEC-12: seven rows pass; the `--flag` row fails at its Given, since no
  event carries a flag before PRV-07. The scenario stays `@pending`;
  a ruling is wanted (ADM-04's row 5 went to `@PRV-07` the same way).
- Purge, `parinn purge --range`: widening, one transaction; payload
  files, search postings and the vacuum are journalled and finished by
  the next purge, ingest or rebuild. Refused, audited and counted
  (`purge_refused`): events the key set derives from, a run with a
  pending ingest marker, nothing left to erase.
- Quarantine, `add|release|list`: `operator` acts, a cut and a
  widening; the set derives from them alone and holds events recorded
  later. Origins do not change. `Envelope.tombstones` is filled.
- Retention (SEC-22 row 4): time marks, a structural event of the device
  seat, recorded by ingest while a policy is set; an event ages from the
  first mark after it. REC-15 stays `@pending` (M5).
- Gates run: scenario gate, fmt, clippy `-D warnings`, `mdsmith check .`,
  `cargo run -p coverage`.

Inherited: `store::{Erase, Purged, Finished, RangeEvent, EventFacts}`,
`Store::{purge, finish_purge, contents, citations}`;
`administration::{purge, quarantine, retention}`; `EventKind::{Tombstone,
TimeMark}`; counters `purge_refused`, `purge_unfinished`,
`retention_purged`, `retention_failed`.

For the stakeholder:

- Names for review (no domain-model agent here): `Selector`, "time
  mark" kind, `parinn retention` has no verb. Counter names above.
- `retention_purged` counts a policy working, not a failure: it fails
  `doctor` until acknowledged (OQ-50 (i)).
- A tombstone keeps a digest over the commitments, not each (the headers
  keep them); SRS wording on branch claude/m1-phase-14-docs. The
  envelope's `quarantined` reason is new there too.
- A purge, as a widening act, needs the standing risk acceptance; the
  scenario steps record it first. A page of `event_expand` holds about
  15 items, so RCL-06's step follows the cursor.
- `--since` compares transcript timestamps to the second; no event
  without one lies in a window.
- S2 (item `integrity`) untouched.

Parked: SEC-31 copies, other purge scopes, seal rewrites (M3); flags and
REC-15's rest (M5).
