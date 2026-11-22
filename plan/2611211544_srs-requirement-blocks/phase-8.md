---
n: 8
title: "The open questions written as a list"
status: "✅"
result: false
---
# Phase 8: the open questions written as a list

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 5's third pull request. The open questions of §13.1 were
a table whose 53 rows were each padded to 3,072 characters, so the
section held 175,072 bytes in 82 lines. ASM-09's scenario reads one of
them: its last step checks that OQ-01 records what S6 found.

The edit. Each row becomes a list item, as the change log's did: its id
in bold, then its question one sentence a line, with its resolution
path a field in a list under it, wrapped at 80 columns. Under headings,
as a requirement is written, the section would hold 760 lines, past the
srs kind's cap of 700. The risks stay a table,
their rows at 414 characters. No other line of the section changes.

The reader. ASM-09's step found the line that opens `| OQ-01 `. It
reads OQ-01's item instead, its lines joined by single spaces, and
speaks of OQ-01, not its row, as does the drift case that drops S6's
finding from it.

RED. With the section rewritten and the step not moved, `@ASM-09` fails
in both its examples: SRS §13 holds no OQ-01 row.

GREEN. The step reads OQ-01's item, and `@ASM-09` passes. The drift
suite catches the case that drops S6's finding from OQ-01.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- a comparison outside the repository finds each question and
  resolution path the same as its row's, and every other line of the
  section unchanged;
- the srs gates, the scenario gate, the workspace's tests and
  `@ASM-09` pass, and the drift suite catches the moved case;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
