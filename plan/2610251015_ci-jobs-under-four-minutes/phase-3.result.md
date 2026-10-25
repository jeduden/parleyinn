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

Verified in CI, on two consecutive green runs (40198728851 and
40199668108) of one head:

- Job times. The slowest job took 175 s and 171 s; the `scenarios` and
  `coverage (e2e shard K)` jobs took 92 to 134 s. The gate of two runs
  under four minutes holds.
- `cargo run -p coverage` end to end. The four shard jobs and the
  merging `coverage` job passed, the floors held at 100% in every layer
  together, and the scenario count read 50 bound, 256 pending as on
  main. The line totals differ from main's only by this branch's own
  code and tests.
- zizmor, in the `workflows` job. actionlint is not a CI job and was
  not run.

Next: if a shard job still tops four minutes, raise N in both matrices;
`strategy.job-total` follows.
