---
n: 2
title: "§2, §5, §5a and §7 written as blocks"
status: "🔳"
result: false
---
# Phase 2: §2, §5, §5a and §7 written as blocks

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 3's first pull request: the 22 assumptions of §2.3 and
the 66 requirements of §5, §5a and §7 move out of their tables into
blocks. No drift case or evaluation case quotes one of their rows.

The edit. As phase 1 decided, the rewrite is mechanical and made
outside the repository. Each row becomes a heading with its id, one
level below its section's, its other cells as a list of fields in the
table's order, and its text, one sentence a line, wrapped at 80
columns. No other line of the four files changes.

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
