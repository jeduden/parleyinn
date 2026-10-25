---
id: 2610112057
title: "M7: standalone room view"
status: "🔲"
summary: >-
  The room view on one machine: Needs you, Catch up, search, integrity
  status, overlaps and review with what backs each result, plus the
  launcher, seals and the principal surfaces behind them.
milestone-order: 4
model: opus
depends-on: [2609292014]
---
# M7: standalone room view

## Goal

Let a principal work with many agents on one machine, live, after the
fact or asynchronously, from the room view or the CLI alone.

## Context

Milestone M7 of [SRS §12.2](../docs/srs/12-delivery-plan.md). It runs
after M3 and before M4 (execution order M1, M2, M3, M7, M4, M5, M8,
M9). Its priorities are provisional until OQ-30 settles the v1 scope,
and the ENG-29 review gates it. Spikes S9, S10 and S11 come first.

Scope, by requirement family:

- VIEW-01 to VIEW-12, VIEW-14 to VIEW-18, VIEW-20 to VIEW-22
- the OWN P1 rows but OWN-01: the rest of OWN-11 and OWN-12, OWN-21,
  OWN-29
- LANE-03 to LANE-06, LANE-13, LANE-14, LANE-17, LANE-19, LANE-21,
  LANE-24 to LANE-33, the rest of LANE-16 and LANE-23
- PRV-10, REC-18, REC-20, the rest of REC-19, RCL-09, INJ-10
- SEC-20, SEC-21, SEC-23, SEC-27, SEC-29, SEC-32
- ADM-15, ADM-16, OPS-06, NFR-01, NFR-09, NFR-15
- ENG-12 and ENG-16, as changed

## Tasks

1. Spikes S9, S10 and S11: transcript flush, an answerable harness
   prompt, input interfaces, git inside the core
2. The room view and its surfaces: VIEW rows, with the CLI and TUI as
   reduced clients
3. Principal surfaces, presence proofs and the launcher: OWN rows,
   SEC-29
4. The room model: LANE rows, PRV-10, overlaps and range links
5. Seals, head receipts and integrity: REC-18, SEC-27, VIEW-10
6. Quotas, budgets and the B1 boundary: ADM-15, ADM-16, NFR rows,
   SEC-20, SEC-21, ENG-12, ENG-16
7. The initial targets §12.3 gives S9 and M7: freeze each with its
   evidence, removing its † and its §12.3 row

## Acceptance Criteria

- [ ] §12.2's M7 exit criteria hold, the recorded-weekend fixture
  through the CLI alone included: for each room with activity since the
  starting point, Catch up shows changes,
  pending decisions and a next principal action or that none is needed, with
  exact source addresses and visible capture and sync gaps; Needs you opens
  each item in its room with evidence and no pre-filled verdict
- [ ] Every scenario in scope is off @pending and passes
- [ ] The two-week pilot with the room view in daily use is clean and
  the stakeholder signs off
