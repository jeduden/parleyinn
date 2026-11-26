---
n: 7
title: "The delivery plan's milestones written as headings"
status: "✅"
result: true
summary: >-
  The delivery plan's nine milestones are each a heading, their scope
  and exit criteria a list of two fields wrapped at 80 columns, word for
  word as the table held them. The section falls from 67,639 bytes in
  93 lines to 25,825 in 272, where a row reached 4,909 characters. No
  requirement changes, and no scenario comes off @pending.
---
# Phase 7 handoff

## Done

- The nine milestones of §12.2 moved out of their table, each a heading
  one level below §12.2's with its scope and exit criteria as a list.
- The paragraph after the table, on LANE-02 in M1 and NFR-04 in M5,
  sits above the first milestone and says scope where it said row. No
  other line of the section changed. The spikes and the initial targets
  stay tables, their rows at 499 and 207 characters.
- Task 5 names every table with a row over 500 characters, measured on
  the tree. The reference architecture's components and decisions, the
  residual risks, the store's tables and the MCP tools joined it; the
  boundary register, at 492, left it.

## Verified

- A comparison outside the repository found each milestone's scope and
  exit criteria the same as its row's, the moved paragraph the same but
  where it said row, and every other line of the section unchanged.
- `mdsmith check .` passes.
- The srs gates, the scenario gate, the workspace's 4,642 tests and the
  cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32` and
  `@ENG-34` pass.
- `review-eval check` finds every case applies: 235 cases, 74
  components. No drift case injects into the section.
- `scripts/review-local.sh` against phase 6's head approved: 4 of 4
  agents consulted, no finding, 0.12 USD.

## Next

- The command-line interface: `parinn purge` with its nine scopes is a
  single word of 79 characters, which no list item holds within 80
  columns. Its form needs a choice before its table can go.
- The open questions, with the step of ASM-09's scenario that reads
  OQ-01, then the rest of task 5's tables, each with its gate's reader.
