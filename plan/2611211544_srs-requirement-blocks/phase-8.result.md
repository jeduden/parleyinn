---
n: 8
title: "The open questions written as a list"
status: "✅"
result: true
summary: >-
  The 53 open questions of §13.1 are each a list item, word for word as
  the table held them, the resolution path a field under each, wrapped
  at 80 columns. The section falls from 175,072 bytes in 82 lines to
  41,197 in 564, where each row was padded to 3,072 characters.
  ASM-09's scenario reads OQ-01's item. No requirement changes, and no
  scenario comes off @pending.
---
# Phase 8 handoff

## Done

- The open questions of §13.1 moved out of their table, each a list
  item: its id in bold, its question one sentence a line, and its
  resolution path a field in a list under it. The risks stay a table,
  their rows at 414 characters. No other line of the section changed.
- ASM-09's last step reads OQ-01's item, its lines joined by single
  spaces, and speaks of OQ-01, not its row, as does the drift case
  that drops S6's finding from it. The step file keeps its 1,297 lines,
  the length the size list gives it.
- The hub's exclusion of compaction guidance said the term appears
  nowhere else, though OQ-02 and OQ-51 record its withdrawal: a breach
  of ENG-35 on main. Its row names those two records now.

## Verified

- With the section rewritten and the step not moved, `@ASM-09` failed
  in both its examples, as SRS §13 held no OQ-01 row. With the step
  moved, it passes.
- A comparison outside the repository found each question and
  resolution path the same as its row's, and every other line of the
  section unchanged.
- `mdsmith check .` passes.
- The srs gates, the scenario gate, the workspace's 4,642 tests and
  the cucumber scenarios `@ASM-09`, `@ENG-21`, `@ENG-27`, `@ENG-28`,
  `@ENG-32` and `@ENG-34` pass.
- The drift suite caught the three cases that inject a file the change
  touches: S6's finding dropped from OQ-01, an outline's row moved to
  an Examples block held pending, and the hub losing a required
  section.
- `review-eval check` finds every case applies: 235 cases, 74
  components.
- `scripts/review-local.sh` against phase 7's head approved: 10 of 10
  agents consulted, 0.27 USD. Its one nit is on main: OQ-56 cites §8.4
  for token estimates §8.4 does not hold.
- Its run before the hub's fix requested changes. The hub's specialist
  found the breach above and cited OQ-51 for it, an id no requirement
  or invariant has, so the review refused its report.

## Next

- Phase 9: the components, decisions, store tables and concerns as
  lists. Then the command-line interface, with mdsmith's stern line
  length for the srs kind, as the stakeholder chose.
- The model specialists' report section does not say, as the other
  specialists' does, that an open question's id refuses the report:
  the refusal above came from that gap.
