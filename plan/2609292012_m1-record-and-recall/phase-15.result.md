---
n: 15
title: "Operations: verify, status, audit, the boundary and the threat model"
status: "✅"
result: true
summary: >-
  @ADM-11, @OPS-01, @SEC-15, @SEC-17, @SEC-19 and @ENG-22 off @pending
  and passing. SEC-01's core row passes as gate tests. @ADM-09 passes
  seven rows of eight and stays @pending: no landmark exists before M2.
---
# Phase 15 result

## Handoff

Outcome:

- ADM-11, OPS-01, SEC-15, SEC-17, SEC-19, ENG-22: off `@pending`,
  passing (cucumber, built core and repository checkout). `parinn
  status` adds the store's schema version and what fork detection
  covers; OPS-01's five operations each audit and count once.
- ADM-09: seven rows pass (run with the tag lifted); the landmark row
  cannot run before M2, so the scenario stays `@pending` and is raised.
  `parinn verify` now checks the store's shape (foreign schema objects,
  the index's ranking, the index against a derivation rolled back),
  names the check it failed, and finds transcript sources short of
  their end.
- SEC-01's core row: gate tests in `tooling/engineering/tests/boundary.rs`
  (the scenario stays pending, as ENG-12 and ENG-16). SEC-19, SEC-15,
  SEC-17 and ENG-22 each have a check, a gate test and drift cases.
- Gates run: scenario gate, fmt, clippy `-D warnings`, `mdsmith check .`,
  `cargo run -p coverage` (100% in all layers, unit 99% or more).

Findings, fixed on the way:

- A writer with a seat and no event got an index slot on rebuild and
  none on first indexing: the index differed by arrival (I10).
- Counter `redacted` is now `redactions`, as OPS-01 names it.
- A transcript path that cannot be looked at is `hook_failed`, not a
  path outside the roots.

For the stakeholder:

- Names for review (no domain-model agent here): `redactions`, check
  names in `parinn verify`, `pending_sources`, `docs/security/`.
- Rulings wanted: ADM-09 says an unserved source past its position
  fails. ENG-06 needs verify to pass after a kill mid-ingest, so a
  source an ingest stopped short of is pending; a lost source now
  fails verify (exit 3, was 0). "Replaced first" is not checked.
- The review row in the threat model names an agent as reviewer; the
  security reviewer's review is still to be given.
- Stakeholder paths touched: `tooling/`, `SECURITY.md` (one citation).
- OQ-50: raising `hook_budget_exceeded` in ADM-11's scenario fails
  `doctor` until acknowledged; the scenario does not run doctor.
- S2 (item `integrity`) untouched; the phase 14 counters reused.

Parked: landmark rows (M2), the other components' evidence (M7 to M9),
SEC-01 whole (M7).
