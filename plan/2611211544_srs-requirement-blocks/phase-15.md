---
n: 15
title: "The parser refuses a requirement table"
status: "✅"
result: false
---
# Phase 15: The parser refuses a requirement table

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 4's first pull request. With phase 14, no requirement of
the SRS is a table's row, so the srs crate's parser, which every gate
shares, reads a requirement only as a block and refuses a requirement
table: one whose header leads with "ID" and names a "Requirement" or
an "Assumption" column. Every other table is passed by, as before.

The parser. A requirement table fails the parse with its file, its
header's line and what to do: write each requirement as a block under
a heading that is its id. A table quoted inside a code fence is no
table. The row reader goes, and the crate's stated responsibility says
it reads blocks and refuses a row.

The fixtures. Every test that writes an SRS to disk writes its
requirements as blocks: those of the engineering crate, the review
bundle, the evaluation and review-run. A test of an SRS that does not
read keeps its table and expects the refusal. The one evaluation case
that wrote a requirement table, the review section's new requirement
with no scenario, writes it as a block, and its file's name says
requirement where it said row.

The drift case. A requirement table in a new section of the SRS fails
the parse gate, `specification_parses`, with the refusal.

RED. With the fixtures as blocks and the parser as main has it, the
parser's new tests fail: a requirement table parses.

GREEN. The parser refuses a requirement table, and every test passes.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- the srs gates, the scenario gate, the workspace's tests, clippy and
  the cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32` and
  `@ENG-34` pass;
- `review-eval check` finds every case applies;
- the drift suite catches the new case;
- `scripts/review-local.sh` approves.

What waits. The wording of task 4, a requirement where the tools, the
agents' shared sections, the docs and the scenarios say row or table,
is the next pull request, with the evaluation cases derived from a
row.

BDD. No cross-host behaviour: no `@S` scenario.
