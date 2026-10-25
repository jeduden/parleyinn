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

Milestone M5. Scope: PRV-07, PIN-05, PIN-09, REC-14 and REC-15,
SEC-09, SEC-13, OPS-04, OPS-05, ADM-12, ENG-13, ENG-25, the §11
evaluation, the external review and the pilot.

## Tasks

1. Instruction-like flagging: PRV-07
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
7. Run the §11.1 evaluation and publish it
8. External security review and the two-week pilot
9. M5's initial targets (§12.3): freeze §11's evaluation targets from
   the evaluation's results, removing their † and their §12.3 row

## Acceptance Criteria

- [ ] The §11.2 definition of done holds
- [ ] No scenario tagged P0 or P1 is still @pending, or each remaining
  one is deferred with signed-off rationale
