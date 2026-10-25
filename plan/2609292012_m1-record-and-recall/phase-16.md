---
n: 16
title: "Exit: the engineering gates, the recall evaluation and the S12 rebuild test"
status: "✅"
result: false
---
# Phase 16: exit

Requirements. ENG-03, ENG-06, ENG-07, ENG-09, ENG-11, ENG-15, ENG-23
and ENG-29; the NFR-05 benchmark; the `parinn-eval` module of §11.1 and
its recall evaluation; the S12 rebuild test. It closes plan 2609292011's
ENG-03 and ENG-15. I1, I6, I9, I10.

Measure first: lift each gate scenario's `@pending` locally, run it, and
record what fails and what each needs before building.

- ENG-03: the steps are bound and pass; the stakeholder held it until
  M1 adds injectable I/O. Name the seams M1 added and ask.
- ENG-06: the nightly kills of ingest gain kills of purge and rebuild.
  Migration waits for M3 (ruling 3), so the scenario stays `@pending`.
- ENG-07: bind the steps to `fuzz/`, its corpora and the nightly loop.
- ENG-09: a soak of 50 writers appending 1M events, then `verify`'s
  checks, beside the Miri job over the platform shim.
- ENG-11: bind the steps to the coverage floors CI holds.
- ENG-15: an NFR-05 benchmark of concurrent runs. NFR-01's target needs
  the room view (M7), so the scenario stays `@pending`.
- ENG-23, ENG-29: bind what M1 can check; a clause M1 cannot meet keeps
  the scenario `@pending` and is raised.
- `parinn-eval`: versioned, seeded long runs built from the recorded
  Claude Code 2.1.296 line shapes, exact values planted before 1 to 3
  compactions, recalled through `event_search` and `event_get` by a
  scripted agent. Recall at each depth is measured, never written in,
  and must reach 95%.
- S12: two writers' logs rebuild one search index identical in content
  in either order, row ids from addresses; a purge sets secure delete
  and truncates the WAL; a `user` event's trust needs the deployment
  mode.

SRS wording goes to claude/m1-phase-16-docs. Keep the counters and
names earlier phases left for the stakeholder; rename none.

Gate: `@ENG-03`, `@ENG-06`, `@ENG-07`, `@ENG-09`, `@ENG-11`, `@ENG-15`,
`@ENG-23` and `@ENG-29` pass, or stay `@pending` with a finding; 95%
recall in `parinn-eval`; the S12 rebuild test passes.
