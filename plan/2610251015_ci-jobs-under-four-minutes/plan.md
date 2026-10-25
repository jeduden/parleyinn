---
id: 2610251015
title: "Hold every CI job under four minutes"
status: "🔳"
summary: >-
  Restructure the jobs of ci.yml and build.yml so each finishes in four
  minutes or less, one minute under the five-minute cap, without
  weakening any gate, floor or statistic.
model: sonnet
depends-on: []
---
# Hold every CI job under four minutes

## Goal

Every job in `ci.yml` and `build.yml` finishes in four minutes or less
on a pull request, so the five-minute `timeout-minutes` cap is a signal
and not a daily failure. [docs/ci.md](../../docs/ci.md) is the standard;
the `ci-timings` artifact of each run is the measurement.

## Context

The cap went to five minutes on every job in those two workflows. Five
jobs run over it today. Times are from main run 765, the last full run
before the cap:

| Job            | Time    | Where it goes                                                                                                            |
| -------------- | ------- | ------------------------------------------------------------------------------------------------------------------------ |
| test           | 5.3 min | release build 50 s, `cargo test --workspace` 2 m 54 s, a second scenario run only to count 1 m 17 s                      |
| coverage       | 6.5 min | release build 1 m 14 s, then `cargo run -p coverage` 4 m 52 s, running cargo-llvm-cov once per test layer in one process |
| drift          | 5 min   | one test, `every_drift_is_caught`, runs each injected case in turn in a copy of the checkout                             |
| build (ubuntu) | 4.8 min | one Build step compiles three release targets in turn (3 m 34 s)                                                         |
| benchmark      | 15 min  | `ROUNDS=30` of `SAMPLES=10` on one runner, base and head interleaved (12 m 25 s)                                         |

No job caches the cargo registry or the target directory; only the mise
tools and the pinned Rust toolchain are cached. The benchmark job's
comment says "a second or two of sampling"; the step takes twelve
minutes, so the comment or the sampling is wrong.

Reuse, searched first:

- **`setup-mise`** already caches tools and the toolchain, keyed on the
  pins. A cargo cache extends it, for ci jobs only: the action says
  release builds never consume a branch-writable cache.
- **The coverage tool** runs the three layers in `measure`; a layer
  flag reuses that, and the merge for the floors reuses `lcov::merge`.
- **The drift suite** already builds once into one target directory;
  sharding the case list reuses `cases()`.
- **`build.yml`** already has a matrix over builders; a matrix over
  targets extends it, and the arm64, Windows and provenance consumers
  keep their artifact names.
- **The benchmark gate** already pools samples files; shards append to
  the same files the gate reads.
- **The `ci-timings` artifact** (ci.yml) is the measurement. Nothing
  new measures.

Stakeholder paths: every phase touches `/.github/`, and most touch
`/tooling/`. [CODEOWNERS](../../.github/CODEOWNERS) owns both. Each phase
is its own pull request and waits for the stakeholder's approval. A
phase that changes `/tooling/` also lands the unit tests that keep each
crate at its coverage floor. Where it adds a check on the repository's
records, it adds a drift case too (ENG-27).

## Tasks

1. Proving slice: every job of `ci.yml` and `build.yml` under its cap
   (phase 1): the release core built once, the scenarios in their own
   job, one job per coverage layer, one job per release target, the
   benchmark rounds sharded and pooled
2. A cargo registry and target cache for ci jobs only, never for the
   release build, if a job still runs over four minutes
3. The drift cases sharded across four jobs (phase 1)
4. The benchmark gate stratified by shard, so the runners' speed
   differences do not blunt it (phase 2)
5. The scenarios and the end-to-end coverage layer sharded in four,
   once the growth of the suite took both over the cap (phase 3)

Later phases are declared when the prior phase's result shows the real
shape.

## Execution

| Phase | Model  | Gate                                                                                                                                 |
| ----- | ------ | ------------------------------------------------------------------------------------------------------------------------------------ |
| 1     | sonnet | The `ci-timings` of two consecutive runs show every job under four minutes; the summaries and the provenance hold                    |
| 2     | sonnet | A three-runner fixture fails the pooled test and passes the stratified one; the single-stratum fixtures decide as before             |
| 3     | sonnet | The shards partition the scenarios in unit tests and on the real suite; two consecutive runs show every shard job under four minutes |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                                                                                                                              |
| --- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | ✅     | [Every job of ci.yml and build.yml under its five-minute cap](phase-1.md)                                                                                          |
| 2   | ✅     | [The benchmark gate stratified by shard](phase-2.md)                                                                                                               |
| 3   | 🔳     | [The scenarios and the end-to-end coverage layer sharded](phase-3.md)                                                                                              |
|     | ↳      | The scenarios and the end-to-end coverage layer run as four shards each. The shards partition the suite; the CI gate of two runs under four minutes is still open. |
<?/catalog?>

## Acceptance Criteria

- [x] Every job of `ci.yml` and `build.yml` finishes in four minutes or
  less in two consecutive runs, read from their `ci-timings` artifacts
- [x] Every job keeps `timeout-minutes: 5`, and `CI` stays the one
  required check
- [x] No gate, coverage floor or requirement scenario is weakened; the
  benchmark still runs `ROUNDS=30` of `SAMPLES=10` and gates at 0.01,
  and ranks each shard's samples against its own runner's, so the
  runners' speed differences do not blunt the test
- [x] The release build is unchanged in what it produces: the same
  binaries, checksums and provenance, under per-target artifact names
- [x] Each phase that changes `/tooling/` keeps its crates at their
  coverage floors and adds a drift case where it checks the
  repository's records
- [ ] The `scenarios` and `coverage (e2e shard K)` jobs finish in four
  minutes or less in two consecutive runs, with the scenario count and
  the coverage table unchanged (phase 3)
- [x] `mdsmith check .` passes
