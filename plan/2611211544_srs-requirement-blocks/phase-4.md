---
n: 4
title: "§5c, §5e and §5f written as blocks"
status: "✅"
result: false
---
# Phase 4: §5c, §5e and §5f written as blocks

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 3's third pull request: the 89 requirements of the
principal acts, §5c, of provenance and trust, §5e, and of pins, recall
and restore, §5f, move out of their tables into blocks.

The edit. As in phase 2, each row becomes a heading with its id, one
level below its section's, its other cells as a list of fields in the
table's order, and its text, one sentence a line, wrapped at 80
columns. The principal acts' introduction, which had the security
reviewer review the OWN rows as one block, speaks of the OWN
requirements instead. No other line of the three files changes.

RED. None: no code changes, and main's own tools are the gate.

GREEN. The three files hold a block for each requirement and no
requirement table.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- the srs gates, the scenario gate, the workspace's tests and the
  cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32` and
  `@ENG-34` pass;
- `review-eval check` finds every case applies, and the drift cases
  that inject into the three files are caught;
- `review-bundle` against main shows no requirement changed;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
