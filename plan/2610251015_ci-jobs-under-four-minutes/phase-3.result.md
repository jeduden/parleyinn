---
n: 3
title: "The scenarios and the end-to-end coverage layer sharded"
status: "✅"
result: true
summary: >-
  The scenarios and the end-to-end coverage layer run as four shards
  each. The shards partition the suite; two consecutive runs on the
  same head kept every job under four minutes (slowest 175 s and
  171 s).
---
# Phase 3 result

## Handoff

Outcome:

- `scenarios` and `coverage (e2e shard K)` are four-shard matrices in
  `ci.yml`. A job takes its shard from `strategy.job-index` of
  `strategy.job-total`, so the shards cover the suite once whatever the
  matrix lists. `coverage` and `CI` need the new `coverage-e2e` job.
  The decoy `HOME` check, the count line in each summary and the
  `ci-timings` artifact are kept; no other job changed.
- The runner takes `--shard K/N`, or `PARINN_BDD_SHARD=K/N` for
  `cargo test -p scenario`, which hands its arguments to every test
  target. The coverage tool takes `--layer e2e --shard K/N`, writes
  `e2e-K-of-N.lcov`, and `--merge` adds the files up and fails on a
  missing shard. No scenario was touched or re-pended; none left
  `@pending`. No check on the repository's records landed, so no drift
  case (ENG-27).

Verified, by mechanism:

- Unit tests: the shards partition a synthetic suite for N in 1 to 8,
  in either order; the merge refuses a missing shard or a mixed count.
  Unit line coverage of the two touched crates holds its floor.
- Local run of the real suite, four shards: the scenario names of the
  shards equal the unsharded run's, 116 in all (29, 24, 30 and 33), in
  20 to 34 s each against 86 s whole. The same three NFR-02 scenarios
  fail in the whole run and in their shards here: they need the release
  core CI builds.

Not verified here, left to CI:

- Job times. The shards divide only the scenario run; the build and
  setup each job repeats do not shrink. Read the `ci-timings` of two
  consecutive runs, then flip this phase and the plan to ✅.
- `cargo run -p coverage` end to end. A local instrumented run filled
  the disk, so the coverage jobs are the first full run of the shards.
- zizmor and actionlint were not run locally.

Next: if a shard job still tops four minutes, raise N in both matrices;
`strategy.job-total` follows.
