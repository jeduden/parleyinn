---
n: 16
title: "Exit: the engineering gates, the recall evaluation and the S12 rebuild test"
status: "✅"
result: true
summary: >-
  @ENG-03, @ENG-07, @ENG-09 and @ENG-11 off @pending and passing. parinn-eval
  recalls 216 of 216 planted values after one to three compactions
  within ten hits; the S12 rebuild test passes. @ENG-06,
  @ENG-15, @ENG-23 and @ENG-29 stay @pending, each raised.
---
# Phase 16 result

## Handoff

Outcome:

- ENG-03, ENG-07, ENG-11: passing (cucumber); ENG-07 and ENG-11 have two
  drift cases each.
- ENG-03's hold on injectable I/O: ruled met by the stakeholder's
  [consent on PR #91][consent]; no step checks that clause.
- Verify and the start markers' readers open a transcript source or a
  marker through ingest's guarded open: a link swapped in is not
  followed and a FIFO is not waited on; verify reports such a source as
  failed, unreadable (ADM-09, SEC-18, ENG-03).
- Store schema: `transcript_sources` gained `marked_at`, no version bump.
- parinn-eval: seeded runs, a scripted agent asking event_search then
  event_get. Within ten hits, 72/72 at each depth; within five, depth 3
  falls to 87.5%; without recall, 0/216. CI's `eval` job holds
  `tooling/eval/recall-v1.txt`.
- S12: `crates/store/tests/s12_rebuild.rs` covers either order and
  truncated logs.
- ENG-06: kills of purge and of rebuild. After a kill between a purge's
  transaction and its finishing, verify stays read-only: it names the
  unfinished purge (`pending_purges`), leaves the index unchecked and
  exits 0; the next purge, ingest or rebuild finishes it.
- ENG-09: a soak of 50 writers; at 1M events, 4,293 of 10,000 appends
  passed the 5 s busy deadline and were retried. @ENG-09 passes (ruled
  2 November 2026): 50 certified writers, 10,000 events, then
  `parinn verify` exits 0; nightly runs 1M, read from `nightly.yml`.
- Gates run: scenario gate, fmt, clippy, mdsmith, tests, coverage.

Raised (scenarios stay `@pending`):

- ENG-06: no migration before M3 (ruling 3).
- ENG-15: NFR-01 needs the room view (M7); NFR-05's target needs the
  reference hardware.
- ENG-23: no release and no migration yet.
- ENG-29: no mechanical check; ADR-2610221545 pins no invariants hash.

For the stakeholder:

- Counters (domain-model agent): `verify_unfinished` became
  `verify_timed_out`, report key `unfinished_purges` `pending_purges`.
  `purge_unfinished` does not fit (`purge_failed` or `purge_timed_out`
  do); kept on main, stakeholder to rule. `recall_refused` fits.
- Verify no longer writes (ruled).
- §11.1 measurement wording on claude/m1-phase-16-docs (PR #90).
- Stakeholder paths: `ci.yml`, `nightly.yml`, `tooling/`.

Plan close (stays 🔳): met are recall, S12 and the checks. Raised items
moved (plan.md, §12.2). To M3: ADM-09's landmark row, ENG-06's migration,
`purge --run`. To M5: SEC-12's flag row, SEC-17, ENG-22, ENG-23, NFR-05's
target. To M7: SEC-19, ENG-29, NFR-01, PRV remainders, REC-24's confirm.
To M8: PRV-09.

Open in M1, the M1 ids still `@pending` (cucumber, 2 November 2026):

- SEC-31, ADM-07: steps being bound; nothing claimed yet. Purge residue
  is closed in code (REC-17): only each purged line's position and length
  remain, its `uuid` stays in the place row (REC-04). Open for the
  stakeholder: SEC-31 says only position and length remain.
- ADM-09: 16 of 17 rows pass; the landmark row is M3's.
- REC-22: all steps pass but the first Then, a freshness mark with
  source and ingest position on every surface (VIEW-04, M7).
- REC-02: its last step needs restore blocks and a join (M2).
- REC-24: its confirm step is M7's.
- LANE-02: the stakeholder said leave it (SEC-18).
- In part by §12.2: SEC-12 (flag row, M5), ENG-06, -15, -23, -29, SEC-17,
  -19, PRV rows (above).

Passing and off `@pending`: ENG-09, SEC-18, SEC-05 (64 MiB windows, a
line over 4 MiB refused, `transcript_line_too_large`), REC-04, REC-08,
REC-12, REC-26, PRV-14, RCL-02 (9 of 9, `kind` ruled A), REC-17. Under an
agent harness REC-26's typed rows need `unshare -p -f --mount-proc`.
Resume past a marker: ingest seat, `ingested`.

- [x] ADM-09's other rows pass with `@pending` lifted, all but the
  landmark row, M3's

Ruled: v1.0 ends before M8; PRV-09 and PRV-02's peer rows stay in M8
(§11.2 option C). Verify's 60 s bound (`INDEX_CHECK_SECS`) is postponed.

[consent]: https://github.com/jeduden/parleyinn/pull/91#issuecomment-6402652493
