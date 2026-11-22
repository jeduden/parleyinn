---
n: 3
title: "The room sections and §5d written as blocks"
status: "✅"
result: true
summary: >-
  The 67 requirements of the room family, LANE, the room collaboration
  and the room view, and of the peer network are each written as a
  block, and none changes: main's review bundle shows each with main's
  text alone. The four files fall from 204,921 bytes to 70,291, and no
  line of a block passes 80 columns, where a row reached 3,757
  characters. No scenario comes off @pending.
---
# Phase 3 handoff

## Done

- The 14 requirements of LANE, the 19 of the room collaboration, the
  22 of the room view and the 12 of the peer network moved out of
  their tables into blocks, each a heading with its id, its fields as
  a list and its text one sentence a line.
- The room view's introduction says how a requirement is written,
  where it said what each table lists, and its prose speaks of
  requirements where it said rows. No other line of the four files
  changed.
- The four files hold 1,500 lines where they held 152, and 70,291
  bytes where they held 204,921. The longest, the room collaboration,
  holds 505 lines, under the srs kind's cap of 700.

## Verified

- `mdsmith check .` passes, with the srs kind's shape schema and its
  50-line cap on a block.
- The srs gates, the scenario gate, the workspace's 4,642 tests and
  the cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32`
  and `@ENG-34` pass.
- `review-eval check` finds every case applies: 235 cases, 74
  components. No drift case injects into the four files.
- `review-bundle` against main shows each of the 67 with main's text
  alone: no requirement changed.
- `scripts/review-local.sh` against main approved: 7 of 7 agents
  consulted, no finding, 0.26 USD.

## Next

- Phase 4 writes §5c, §5e and §5f as blocks.
- Phase 5 writes §6, §6b and §10 as blocks, with the drift cases that
  quote §10's rows moved.
