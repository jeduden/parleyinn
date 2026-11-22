---
n: 7
title: "The delivery plan's milestones written as headings"
status: "✅"
result: false
---
# Phase 7: the delivery plan's milestones written as headings

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 5's second pull request. The milestones of §12.2 were a
table whose nine rows were each padded to 4,909 characters, M1's scope
alone 3,812, so a search cut every row and a change to one repadded
all nine. No gate reads them.

The edit. Each milestone becomes a heading one level below §12.2's,
its scope and its exit criteria a list of two fields, wrapped at 80
columns, as a requirement's fields are. The paragraph after the table,
on LANE-02 in M1's row and NFR-04 in M5's, would sit under the last
milestone: it moves above the first and says scope where it said row.
The spikes and the initial targets stay tables, since no row of theirs
passes 500 characters. No other line of the section changes.

The command-line interface was to come with it, but one command's
synopsis, `parinn purge` with its nine scopes, is a single word of 79
characters. No list item holds it within 80 columns, so its form is
left for a later phase.

RED. None: no code changes, and no gate reads the milestones.

GREEN. The milestones are headings whose fields hold the table's
scopes and exit criteria word for word.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- a comparison outside the repository finds each milestone's scope and
  exit criteria the same as its row's, the moved paragraph the same but
  where it said row, and every other line of the section unchanged;
- the srs gates, the scenario gate and the workspace's tests pass;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
