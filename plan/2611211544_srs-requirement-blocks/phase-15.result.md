---
n: 15
title: "The parser refuses a requirement table"
status: "✅"
result: true
summary: >-
  The srs crate's parser, which every gate shares, reads a requirement
  only as a block and refuses a requirement table with its file, its
  header's line and what to do. A drift case proves the parse gate
  catches one. No requirement changes, and no scenario comes off
  @pending.
---
# Phase 15 handoff

## Done

- The parser reads a requirement only as a block under a heading that
  is its id. A table whose header leads with "ID" and names a
  "Requirement" or an "Assumption" column fails the parse:
  `<file>:<line>: a requirement table: write each requirement as a
  block under a heading that is its id`, at the header's line. Every
  other table, and one quoted in a code fence, is passed by. The row
  reader is gone, and the crate's stated responsibility says it reads
  blocks and refuses a row.
- Every test that writes an SRS to disk writes its requirements as
  blocks: the engineering crate's, the review bundle's, the
  evaluation's and review-run's. A test of an SRS that does not read
  keeps its table and expects the refusal.
- The evaluation case of the review section's new requirement with no
  scenario writes ENG-36 as a block, in
  `defect-new-requirement-without-scenario.toml`.
- The drift case "a requirement written as a table's row" adds a table
  in a new section, `docs/srs/10b-drift.md`.

## Verified

- RED: with the parser's new tests and main's parser, 3 of the crate's
  47 unit tests fail, each because a requirement table parses.
- `mdsmith check .` passes.
- The srs gates, the scenario gate, the workspace's 4,651 tests, clippy
  denying warnings, rustfmt and the cucumber scenarios `@ENG-21`,
  `@ENG-27`, `@ENG-28`, `@ENG-32` and `@ENG-34` pass: 5 scenarios, 77
  steps.
- `review-eval check` finds every case applies: 236 cases, 74
  components.
- The drift suite catches the new case: `specification_parses` fails
  with `10b-drift.md:3: a requirement table`.
- `scripts/review-local.sh` against main: the agents approve, 13 of 13
  consulted, no finding, 0.94 USD. It blocks only on the consent the
  paths CODEOWNERS assigns under `tooling/review-*` need, which the
  pull request asks the stakeholder for.

## Next

- Phase 16, the rest of task 4: a requirement where the tools, the
  agents' shared sections, the docs and the scenarios say row or table,
  with the evaluation cases derived from a row and the evaluation's row
  path, which reads no SRS now.
- Task 6: split each requirement the srs kind lists past 50 lines.
