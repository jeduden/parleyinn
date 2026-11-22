---
n: 11
title: "Appendix B written as a list"
status: "✅"
result: false
---
# Phase 11: Appendix B written as a list

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 5's sixth pull request. Appendix B's coverage table held
a row per invariant, each padded to 860 characters. The gate
`appendix_b_matches_the_traces` reads it: the requirements an
invariant's row lists must be those whose Traces name it.

The edit. Each row becomes a list item: the invariant in bold, then a
colon and its requirement ids, wrapped at 80 columns. The appendix's
summary and first sentence spoke of the Traces column of §5–§6 and of
the tables, which §5–§6 no longer hold: they speak of the Traces of
the requirements. The count table stays, its rows at 73 characters.
No other line changes.

The reader. The srs crate's `invariant_coverage` reads the first list
after the front matter, an item per invariant, its lines joined by
single spaces, where it read the table. The gate speaks of items, not
rows. A drift case drops a requirement from I3's item, and the gate
catches it.

RED. The reader's unit tests on the list fail against the table
reader.

GREEN. The reader reads the list, its unit tests pass, and the gate
passes on the rewritten appendix.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- a comparison outside the repository finds each item's ids the same
  as its row's, and every other line of the appendix unchanged but the
  reworded summary and sentence;
- the srs gates, the scenario gate and the workspace's tests pass, and
  the drift suite catches the new case and the count case;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
