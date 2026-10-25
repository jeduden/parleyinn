---
n: 1
title: "Every job of ci.yml and build.yml under its five-minute cap"
status: "🔳"
result: false
---
# Phase 1: every job of ci.yml and build.yml under its five-minute cap

Requirements. None applies: this phase changes how CI runs, not what
ParleyInn does. It takes no scenario off `@pending`. ENG-15's gate test
(`benchmarks::gated`) learns that the suite may run in a job the gate
job needs; its unit tests cover the new shape.

BDD coverage: none. The change is in the workflows and the tooling that
reads them, and no cross-host or lease behavior is involved.

RED spec. Main run 765 has test at 5.3 min, coverage at 6.5 min, build
(ubuntu) at 4.8 min and benchmark at 15 min, all over the cap. The
first run on this branch with the cap shows each timing out. Any run
where a job of `ci.yml` or `build.yml` is over four minutes fails this
phase.

GREEN sites:

- `.github/workflows/ci.yml`: a `release-core` job builds the release
  core once and uploads it; `test` runs every test but the scenario
  crate's, and `scenarios` runs that crate with the count line taken
  from the run itself; `coverage (unit|integration|e2e)` each measure
  one layer, and `coverage` merges them and holds the floors;
  `benchmark (shard N)` each run two of the thirty rounds, and
  `benchmark` pools the samples and runs the gate; `CI` names each job
  that did not succeed.
- `.github/workflows/build.yml`: one job per release target, the target
  read from the `*_TARGETS` lists, with `<name>-parinn` and
  `<name>-licenses` artifacts; the arm64 and Windows checks and the
  release provenance read the per-target artifacts.
- `tooling/coverage`: `--layer NAME` and `--merge`, with `Host::read`.
- `tooling/engineering`: the benchmark gate check follows `needs:`.

Gate. The `ci-timings` artifacts of two consecutive runs show every job
of `ci.yml` and `build.yml` under four minutes; the scenario count, the
coverage table and the benchmark report still appear in the job
summaries; the release provenance covers the same binaries.
