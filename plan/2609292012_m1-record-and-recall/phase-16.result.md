---
n: 16
title: "Exit: the engineering gates, the recall evaluation and the S12 rebuild test"
status: "✅"
result: true
summary: >-
  @ENG-03, @ENG-07 and @ENG-11 off @pending and passing. parinn-eval
  recalls 216 of 216 planted values after one to three compactions
  within ten hits; the S12 rebuild test passes. @ENG-06, @ENG-09,
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
- parinn-eval: seeded runs in the 2.1.296 line shapes; a scripted agent
  asks event_search then event_get. Within ten hits, 72/72 at each depth
  (Wilson 95% from 94.9%). Within five, depth 3 falls to 87.5%. Without
  recall, 0/216. CI's `eval` job holds `tooling/eval/recall-v1.txt`.
- S12: `crates/store/tests/s12_rebuild.rs` covers either order and
  truncated logs.
- ENG-06: kills of purge and of rebuild. After a kill between a purge's
  transaction and its finishing, verify stays read-only: it names the
  unfinished purge (`pending_purges`), leaves the index unchecked and
  exits 0; the next purge, ingest or rebuild finishes it.
- ENG-09: a soak of 50 writers; at 1M events, 4,293 of 10,000 appends
  passed the 5 s busy deadline and were retried. It runs nightly.
- Gates run: scenario gate, fmt, clippy, `mdsmith check .`,
  `cargo test --workspace`, `cargo run -p coverage` (unit 99% or more).

Raised (scenarios stay `@pending`):

- ENG-06: no migration before M3 (ruling 3).
- ENG-09: 1M events cannot run in a pull request's scenario budget; a
  ruling on splitting nightly from PR steps is wanted.
- ENG-15: NFR-01 needs the room view (M7); NFR-05's target needs the
  reference hardware.
- ENG-23: no release and no migration yet.
- ENG-29: no mechanical check; ADR-2610221545 pins no invariants hash.

For the stakeholder:

- Names (no domain-model agent): `parinn-eval`, `ingest_fifty_runs`,
  `run_killed_at_terminal`.
- Counters, domain-model agent verdict: `verify_unfinished` did not fit
  (the cause is a deadline); renamed here to `verify_timed_out`, and the
  `parinn verify` report key `unfinished_purges` to `pending_purges`
  (purges awaiting the next rebuild). `purge_unfinished` does not fit
  either: the model says `purge_failed` or `purge_timed_out`, and the
  report key's analogue likewise; both kept for now on main, stakeholder
  to rule. `recall_refused` fits; `recall_rejected` is the strict-literal
  alternative.
- Verify's derived-index recheck is bounded at 60 s
  (`INDEX_CHECK_SECS`). On a store whose derivation always takes longer,
  ADM-09's derived artifact check never runs (NFR-05). A ruling on the
  bound is wanted.
- Verify no longer writes (ruled).
- §11.1 measurement wording on claude/m1-phase-16-docs (PR #90).
- Stakeholder paths: `.github/workflows/ci.yml` and `nightly.yml`,
  `tooling/`, `Cargo.toml`, `tooling/*/Cargo.toml`.
- Inherited items untouched: S2 `integrity`, phase 14/15 counters and
  renames, `tombstones`.

Plan close (stays 🔳): met are recall, S12 and the checks. The raised
items moved (plan.md, §12.2). To M3: ADM-09's landmark row, ENG-06's
migration, SEC-08's `purge --run`. To M5: SEC-12's flag row, SEC-17,
ENG-22, ENG-23, NFR-05's target. To M7: SEC-19, ENG-29, NFR-01, the PRV
remainders. To M8: PRV-09. Open in M1: RCL-02, below; rulings
on LANE-02, REC-24, ENG-09, the residue.

- [x] ADM-09's other rows pass with `@pending` lifted, all but the
  landmark row, M3's

Last P0 sweep (cucumber):

- REC-08, REC-12, PRV-14 and REC-26 are off `@pending` and pass. REC-26's
  run is served by a live MCP server; a harness process below another is
  nested (PRV-08), so under an agent harness its typed rows need
  `unshare -p -f --mount-proc`.
- REC-02 stays `@pending`: its last step needs restore blocks and a join
  (M2's own-principal join, LANE-23). It is in plan.md's table and M2's
  tasks; its other 16 steps pass.
- REC-04 is off `@pending` and passes (cucumber). A purged event keeps
  its `uuid`, byte offset and prefix hash: the residue finding.
- RCL-02 stays `@pending` until bound. Ruled 2 November 2026: `kind`
  is the provenance class family (SRS 9.2, RCL-02); the scenario agrees.
- Ruled: v1.0 ends before M8, so PRV-09's and PRV-02's peer rows stay in
  M8 (SRS 11.2, 12.2; OQ-30 narrowed). REC-02 is in part in M1's row.

[consent]: https://github.com/jeduden/parleyinn/pull/91#issuecomment-6402652493
