---
n: 4
title: "§5c, §5e and §5f written as blocks"
status: "✅"
result: true
summary: >-
  The 89 requirements of the principal acts, provenance and trust, and
  pins, recall and restore are each written as a block, and none
  changes: main's review bundle shows each with main's text alone. The
  three files fall from 298,088 bytes to 70,944, and no line of a block
  passes 80 columns, where a row reached 9,037 characters. No scenario
  comes off @pending.
---
# Phase 4 handoff

## Done

- The 29 requirements of §5c, the 14 of §5e and the 46 of §5f moved
  out of their tables into blocks, each a heading with its id, its
  fields as a list and its text one sentence a line.
- The principal acts' introduction speaks of the OWN requirements,
  where it said rows. No other line of the three files changed.
- The three files hold 1,641 lines where they held 151, and 70,944
  bytes where they held 298,088. The longest, §5c, holds 622 lines,
  under the srs kind's cap of 700.

## Verified

- `mdsmith check .` passes, with the srs kind's shape schema and its
  50-line cap on a block.
- The srs gates, the scenario gate, the workspace's 4,642 tests and
  the cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32`
  and `@ENG-34` pass.
- `review-eval check` finds every case applies: 235 cases, 74
  components. No drift case injects into the three files.
- `review-bundle` against main shows each of the 89 with main's text
  alone: no requirement changed.
- `scripts/review-local.sh` against main approved: 6 of 6 agents
  consulted, no finding, 0.32 USD. One part of `domain-model-hub`
  answered at 23 turns, inside the budget of 30 plan 2611190753's
  phase 13 gave it.

## Next

- Phase 5 writes §6, §6b and §10 as blocks, with the drift cases that
  quote §10's rows moved.
