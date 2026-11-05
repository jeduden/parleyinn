---
id: 2609292016
title: "M5: hardening and evaluation"
status: "🔲"
summary: >-
  The remaining P1 requirements, the full §11 evaluation, an
  external security review and a two-week pilot.
milestone-order: 6
model: opus
depends-on: [2609292014, 2609292015, 2609292008]
---
# M5: hardening and evaluation

## Goal

Reach the v1.0 definition of done: every P0 verified, every P1 done or
signed off, the evaluation published and the pilot clean.

## Context

Milestone M5. Scope: PRV-07 with SEC-12's flag row, PIN-05, PIN-09,
REC-14 and REC-15, SEC-09, SEC-13, OPS-04, OPS-05, ADM-12, ENG-13,
ENG-25, the §11 evaluation, the external review and the pilot.

## Tasks

1. Instruction-like flagging: PRV-07, and SEC-12's flag row, which M1
   left because no event carried a flag
2. Pin candidates and the CLAUDE.md duplicate warning: PIN-05, PIN-09. PIN-05
   finds no candidates while any run on the node leaves R3 or R8 open, which
   holds until OQ-40 gives sandbox states a signature (OQ-51)
3. SDK transcripts and retention: REC-14 and REC-15
4. Encryption at rest and the recall-taint policy hook: SEC-09, SEC-13.
   `store.db` holds content and commitment keys in clear until then; the
   stakeholder accepted CodeQL's cleartext-storage alert on
   jeduden/parleyinn#48 until this task closes it
5. Canary, structured logs, trusted-only export: OPS-04, OPS-05, ADM-12
6. Mutation testing and the maintainer runbook: ENG-13, ENG-25
7. Run the §11.1 evaluation and publish it, with ENG-15's NFR-05
   benchmark against its target on the reference hardware
   and meet NFR-04's target, which M1 benchmarked but did not meet;
   MEM-02 (P1) lands here with the remaining P1
8. External security review and the two-week pilot; SEC-17's recorded
   review and release gate, the rest of ENG-22 (every statement cites a
   passing test) and ENG-23's migration tests from every previous minor version,
   which M1 left
9. M5's initial targets (§12.3): freeze §11's evaluation targets from
   the evaluation's results, removing their † and their §12.3 row
10. Windowed ingest and SEC-05's transcript rows (4 MiB line, 64 MiB window
    and the source-memory ceilings), deferred from M1 by the user's final
    ruling after the review gate's blockers: I1 against an oversize line,
    quadratic cost, capture poll latency. Requirements:

  - no event lost (I1): store an oversize line, for example streamed to the
      payload store, with the whole line redacted before storage
  - recall latency (NFR-03, I9) never waits on a transcript poll
  - an ingest of a resumed source costs linear in what is new: persist the
      hasher, classifier and identifier state in the store (today it re-hashes
      the consumed prefix, replays the classifier from byte 0 and re-learns
      identifiers on each call), against NFR-01 and NFR-05

11. Close PRV-14's ambiguous-run race in code: the M1 binding adds a 300 ms
    settle in `both_runs` of
    `tooling/scenario/tests/bdd/provenance_tool_calls.rs` to hide it; remove
    the settle once the race is closed

## Acceptance Criteria

- [ ] The §11.2 definition of done holds
- [ ] `@SEC-05` is off `@pending` and passing: no event lost to an oversize
  line (I1), recall never waits on a transcript poll, and the ingest of a
  resumed source costs no more than the bytes it adds, within NFR-01 and
  NFR-05
- [ ] PRV-14 passes with no settle in its binding
- [ ] No scenario tagged P0 or P1 is still @pending, or each remaining
  one is deferred with signed-off rationale
