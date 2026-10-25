---
n: 2
title: "Drift injection: prove every check catches its drift"
status: "✅"
result: true
summary: >-
  ENG-27 added and off @pending. 14 registered drifts, 13 caught and
  one known gap; the drift CI job fails when a check lets one through.
---
# Phase 2 result

## Handoff

Outcome, in the SRS's terms:

- ENG-27 (P0) is new, and its scenario passes off `@pending` (godog
  `TestFeatures/^ENG-27:`). It checks that CI runs the drift suite,
  that every case's edit still applies to the checkout, and that every
  non-pending scenario opening on the checkout has a case. So does each
  of the requirement-scenario gate and the Appendix B check.
- The drift suite (`go test -tags drift ./internal/drift`, CI job
  `drift`, part of the required `CI` check) proves the unedited copy
  passes every check. It then injects 14 drifts: 13 are caught with
  their expected message. One known gap is recorded: the SRS citing an
  ADR with no file, which phase 3 closes.
- Guarded today: ENG-01, ENG-18, ENG-24, ENG-26, ENG-27, the
  requirement-scenario gate, the Appendix B check and mdsmith's
  generated sections.
- Verified by sabotage, then reverted. With ENG-18's one-ADR-per-module
  check stubbed out and ENG-24 put back on `@pending`, the suite failed
  on exactly those three cases.

What the next phase inherits:

- A new check lands with its case in `internal/drift/cases.go`. The
  cases are data: one edit, one check, one expected message.
- Phase 3 must flip the known gap: when its check catches an ADR the
  SRS cites without a file, the suite fails until `KnownGap` is
  dropped from that case.
- `scenario.Scenario` now carries step texts, which ENG-27 uses to
  find the scenarios that inspect the checkout.

Found on the way: the suite's first run caught ENG-27's own drift
going unseen while ENG-27 was still `@pending`. A pending scenario
skips, and a skip passes, so re-pending any guarded scenario now fails
the suite.
