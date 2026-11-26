---
n: 3
title: "The room sections and §5d written as blocks"
status: "✅"
result: false
---
# Phase 3: the room sections and §5d written as blocks

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 3's second pull request: the 67 requirements of the room
family, LANE, the room collaboration and the room view, and of the
peer network, §5d, move out of their tables into blocks. Phase 2's
handoff named ten sections for this phase. They land a few to a pull
request, as task 3 asks, so phases 4 and 5 take the other six.

The edit. As in phase 2, each row becomes a heading with its id, one
level below its section's, its other cells as a list of fields in the
table's order, and its text, one sentence a line, wrapped at 80
columns. The room view's introduction, which said what each table
lists, says how a requirement is written instead, and its prose speaks
of requirements where it said rows. No other line of the four files
changes.

RED. None: no code changes, and main's own tools are the gate.

GREEN. The four files hold a block for each requirement and no
requirement table.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- the srs gates, the scenario gate, the workspace's tests and the
  cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32` and
  `@ENG-34` pass;
- `review-eval check` finds every case applies, and the drift cases
  that inject into the four files are caught;
- `review-bundle` against main shows no requirement changed;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
