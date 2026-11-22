---
n: 14
title: "The review section written as blocks"
status: "✅"
result: false
---
# Phase 14: The review section written as blocks

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 2: the two requirements of the review section, 10a,
ENG-28 and ENG-32, move out of their table into blocks. Phase 1 held
it back until main's `lang-gherkin` read a requirement as its review
context quotes it, not by a search for a row; #161 landed that.

The edit. As in phase 5, each row becomes a heading with its id, one
level below its section's, its other cells as a list of fields in the
table's order, and its text, one sentence a line, wrapped at 80
columns. The sentence that named the table's columns says how a
requirement is written, as the room view's does. No other line of the
file changes.

ENG-28 as a block holds 193 lines, the length the srs kind lists it
at. No drift case quotes the section, so none moves.

RED. None: no code changes, and main's own tools are the gate.

GREEN. The file holds a block for each requirement and no requirement
table.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- a comparison outside the repository finds each block's fields and
  text the same as its row's cells;
- the srs gates, the scenario gate, the workspace's tests and the
  cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32` and
  `@ENG-34` pass;
- `review-eval check` finds every case applies, the cases derived from
  the section among them;
- `review-bundle` against main shows no requirement changed;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
