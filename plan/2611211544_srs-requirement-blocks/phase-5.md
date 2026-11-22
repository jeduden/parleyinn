---
n: 5
title: "§6, §6b and §10 written as blocks"
status: "✅"
result: false
---
# Phase 5: §6, §6b and §10 written as blocks

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 3's last pull request: the 64 requirements of security,
§6, the boundary register's own, §6b, and engineering quality, §10,
move out of their tables into blocks. The threat table of §6 and the
register of §6b stay tables: task 5 takes them.

The edit. As in phase 2, each row becomes a heading with its id, one
level below its section's, its other cells as a list of fields in the
table's order, and its text, one sentence a line, wrapped at 80
columns. No other line of the three files changes.

The drift cases. Three cases quoted §10's rows, and quote its blocks
instead: the one that changes ENG-25's priority, the one that renames
ENG-25 to ENG-24 to redefine an id, and the one that drops PRV-11 from
ENG-29's list of the requirements marked for I2 review, which now
wraps. The case that writes a block in a new section with an id §10
defines names it as another section's, where it said a row's.

RED. With the three files rewritten and the cases not moved, the drift
registry's test that every case applies fails: none of the three finds
the text it replaces.

GREEN. The three files hold a block for each requirement, and the
three cases inject into the blocks.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- the srs gates, the scenario gate, the workspace's tests and the
  cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32` and
  `@ENG-34` pass;
- `review-eval check` finds every case applies, and the drift suite
  catches every case that injects into the three files;
- `review-bundle` against main shows no requirement changed;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
