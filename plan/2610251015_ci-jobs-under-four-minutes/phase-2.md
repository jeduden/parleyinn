---
n: 2
title: "The benchmark gate stratified by shard"
status: "✅"
result: false
---
# Phase 2: the benchmark gate stratified by shard

Requirements. ENG-15 keeps its wording: a statistically significant
regression greater than 10%. Its scenario stays `@pending` on the NFR
benchmarks it awaits. No scenario comes off `@pending`.

BDD coverage: none. The change is inside the `benchmark` tooling crate
and the CI pooling step.

RED spec. Phase 1 pooled ten runners' samples into one Mann–Whitney
test. Runner speed differences widen both sides alike, so a real 12%
slowdown on every runner can come out not significant: the stats test
`stratified_z_sees_a_slowdown_the_runners_spread_hides` builds three
runners of very different speed and shows the pooled z under the
critical value. The stakeholder ruled for a stratified test.

GREEN sites:

- `tooling/benchmark/src/samples.rs`: a `# stratum NAME` line starts a
  stratum; `parse_strata` keeps them apart, `pooled` joins them.
- `tooling/benchmark/src/stats.rs`: `stratified_z`, the van Elteren form
  of the rank test; one stratum is the plain test, so every existing
  fixture decides as before.
- `tooling/benchmark/src/gate.rs`: the medians over every sample, the
  significance over the strata the two sides share; a benchmark both
  sides sampled on no common stratum fails as `Unpaired`, never falling
  back to the weaker test.
- `.github/workflows/ci.yml`: the gate job pools each shard under a
  stratum line naming it.

## Result

The three-runner fixture passes the stratified test and fails the
pooled one, in the stats and gate unit tests. The engineering helper
that slows one benchmark in a single-stratum file still makes the gate
fail naming it, so the ENG-15 bindings decide as before. The CI run on
the change shows the gate's report unchanged in shape.
