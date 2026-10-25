---
id: ADR-2610142300
title: "Accepted benchmark regressions: a reviewed record the gate takes once"
status: accepted
summary: >-
  A change that slows a benchmark past ENG-15's 10% on purpose records
  the benchmark, a ceiling and the reason in
  tooling/benchmark/accepted.txt. The gate passes it as accepted while
  no base line names the benchmark, so each acceptance lasts one change.
---
# ADR-2610142300: Accepted benchmark regressions

## Context

ENG-15's gate fails a significant regression of more than 10%. Phase
1 of plan [2609292012](../../plan/2609292012_m1-record-and-recall/plan.md)
linked SQLite, Ed25519, SHA-256 and serde_json into the core, and
`core_start_up` grew 20.8% on the CI runner, from 506 to 612 µs. The
cause is the larger image, measured on jeduden/parleyinn#48:

- the `libm` load SQLite's `log` brings;
- relocations;
- the pages a start touches.

Neither non-PIE nor packed relocations won it back. The stakeholder
accepted the step. The gate had no way to take a deliberate
regression, so it could never turn green on such a change.

## Decision

A committed file, `tooling/benchmark/accepted.txt`, holds one
acceptance a line: the benchmark, a ceiling in percent above the
gate's 10%, and the reason. `benchmark gate` takes the head's file and
the base's (`--accepted`, `--accepted-on-base`). A regression that an
acceptance in the head's file covers, for a benchmark no line of the
base's file names, passes as `accepted` when the head's median stays
within the ceiling. Past the ceiling, or once the base names the
benchmark, it fails as before: a later change that rewords the line or
raises its ceiling accepts nothing.

The record is a reviewed path: `tooling/` is the stakeholder's in
CODEOWNERS, so no acceptance lands without that review. Once the
change merges, its line is in every later base, so it accepts nothing
more and the step it accepted becomes the baseline.

## Alternatives

- **A pull-request label that skips the gate.** It leaves no record
  in the repository. It would also have to be applied by someone with
  triage rights.
- **Raising the threshold.** It hides every later regression of the
  same size.
- **Moving the baseline by hand.** The gate compares with the base
  commit, not a stored baseline, so there is nothing to move.

## Consequences

- Each acceptance stays in the file after it lands. Pruning old lines
  is safe at any time, since a line the base carries accepts nothing.
  Accepting the same benchmark again takes removing its old line in
  an earlier change.
- A drift case proves `the_accepted_regressions_parse` fails on a
  malformed line (ENG-27).
- The `benchmark` job passes both files. A base without the file
  counts as accepting nothing.
