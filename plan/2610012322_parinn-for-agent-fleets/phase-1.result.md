---
n: 1
title: "Scope proposal and the record/index proving slice"
status: "✅"
result: true
summary: >-
  Spike S12 passes: on each node, two writers' segments in either order
  rebuild a byte-identical index file, foreign events stay untrusted,
  one branch link stands, a purge leaves nothing confirming a guess.
  Byte-identity per append needs a decision.
---
# Phase 1 result

## Handoff

Outcome, in the SRS's terms:

- Spike S12 (§12.1) passes its exit criterion, this phase's gate test.
  Mechanism: `cargo test` in the [slice](slice/report.md)'s own crate,
  outside the parinn workspace: 4 tests in `tests/proving_slice.rs`
  over 4,006 events of S2's corpus, and 21 unit tests.
- ADM-08 and I10, for a rebuild: on each node, segments in any arrival
  order, reordered or duplicated, rebuild a byte-identical index file,
  holding the search index and room state. The two nodes' index files
  differ, since trust levels and origin derive per node; PEER-03 as
  worded does not hold for them.
- PRV-02, with RCL-09's origin values: every event keeps its address
  and provenance. The other writer's events read `peer` and untrusted,
  the node's own trusted classes trusted.
- LANE-31: concurrent branch links resolve by the lower commitment,
  causal ones by the first in causal order, each conflict recorded
  with both acts.
- REC-17, ADM-07 and PEER-11: after a purge, no stored or served value
  confirms a guess at the content, whatever the order of copies. A
  peer erases only once its principal applies the erasure request,
  which the slice stands in for with the tombstone.
- The test fails with the node's tombstone handling switched off.
- No SRS id closes and no scenario leaves `@pending`: this phase is a
  spike. The proposal and persona-review parts of the gate closed
  earlier, with SRS 2.0-draft and the stakeholder's OQ-29 decision.

What M1 inherits, from the report's findings:

- Byte-identity (ADM-08, PEER-03, I10) holds only for an index file
  rebuilt from nothing. One kept per append keeps equal rows, but
  FTS5's structure record differs, even after `optimize` and
  `VACUUM`. The report recommends a canonical digest per derived
  artifact, which changes I10's wording. That is the stakeholder's
  call, as are the report's two PEER-03 questions.
- Row ids derive from the address or event hash, not arrival.
- A purge sets SQLite's `secure_delete` and FTS5's `secure-delete`, or
  rebuilds the index file. Without them, 18 of 40 purges left text,
  and every purge left postings. In WAL mode it also checkpoints.
- Trust levels of `user` events need the recorded deployment mode;
  applying an erasure request must be a recorded principal act (I10).
- Full rebuild runs at about 35,000 events/s on a 4 vCPU host: about
  five minutes per 10M events.

Not covered, for M1 or later: seals and forks, segment merges, the
payload store, open segments, ingest, quarantine, erasure requests as
signed events, purge receipts, purging an act, one branch in two
rooms, missing ranges, a real peer exchange, WAL mode and Windows.
