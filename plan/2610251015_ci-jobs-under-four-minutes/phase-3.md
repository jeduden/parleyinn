---
n: 3
title: "The scenarios and the end-to-end coverage layer sharded"
status: "🔳"
result: false
---
# Phase 3: the scenarios and the end-to-end coverage layer sharded

Requirements. None applies: this phase changes how CI runs, not what
ParleyInn does. It takes no scenario off `@pending` and weakens none.

BDD coverage: none. The change is in the scenario runner's selection,
the coverage tool and the workflow; the runner's unit tests prove the
shards partition the suite, and a local run of every shard proves it on
the real scenarios.

RED spec. As scenarios are written, the `scenarios` job (every
cucumber scenario in one job) and `coverage (e2e)` (the same suite on
the instrumented build) each run past their five-minute cap and are
cancelled: about 5 min 15 s uninstrumented at 125 scenarios. A run
where either is cancelled at the cap fails this phase.

GREEN sites:

- `tooling/scenario`: `--shard K/N` (and `PARINN_BDD_SHARD=K/N` for
  `cargo test -p scenario`, which hands its arguments to every test
  target). A scenario belongs to the shard its feature path, name and,
  for an outline row, line hash to (FNV-1a, dependency-free), so the
  shards partition the suite in any order. It composes with `--tags`,
  `--skip` and name filters; pending scenarios stay skipped; the count
  line ends in `(shard K/N)` and counts the whole suite.
- `tooling/coverage`: `--layer e2e --shard K/N` runs `bdd --shard K/N`
  (the first shard also runs the layer's other targets) and writes
  `e2e-K-of-N.lcov`; `--merge` adds the shard files up as `e2e.lcov`
  with `lcov::merge` and fails when a shard is missing or the files
  disagree on N. The test and scenario counts read the sources and the
  features, so they are the totals, not per shard.
- `.github/workflows/ci.yml`: `scenarios` is a four-shard matrix; the
  end-to-end layer leaves `coverage-layer` for `coverage-e2e`, a
  four-shard matrix; `coverage` and `CI` need it. A job's shard is its
  `strategy.job-index` of `strategy.job-total`, so the shards cover the
  suite once whatever the matrix lists. The decoy `HOME` check, the
  count line in the summary and the `ci-timings` artifact are kept.

No new check on the repository's records lands, so no drift case. A
job's shard is by construction. The merge refuses a missing shard in
code with unit tests, which is no record check (ENG-27).

Gate. Two consecutive CI runs show every `scenarios` and `coverage
(e2e shard K)` job under four minutes in their `ci-timings`, and the
coverage table's end-to-end figure and the test counts match an
unsharded run's.
