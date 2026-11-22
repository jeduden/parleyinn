---
n: 6
title: "The change log written as a list"
status: "✅"
result: false
---
# Phase 6: the change log written as a list

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 5's first pull request. The SRS's change log, in its
index, was a table whose 48 rows were each padded to 12,027
characters, so the index held 611,025 bytes in 136 lines, and an entry
longer than the longest repadded every row. No gate reads it.

The edit. Each row becomes a list item: its version in bold and its
date, then its summary, one sentence a line, wrapped at 80 columns, as
a requirement's text is. No other line of the index changes.

RED. None: no code changes, and no gate reads the change log.

GREEN. The change log is a list whose 48 entries hold the table's
versions, dates and summaries word for word.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- a comparison outside the repository finds each entry's version,
  date and summary the same as its row's, and every other line of the
  index unchanged;
- the srs gates, the scenario gate and the workspace's tests pass;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
