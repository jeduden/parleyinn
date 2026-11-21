---
n: 1
title: "Every reader takes a requirement written as a block"
status: "✅"
result: true
summary: >-
  Every reader of the SRS now takes a requirement written as a block
  beside the tables: the srs crate's parser, which every gate shares,
  and the evaluation's derived cases. No requirement changes, and no
  scenario comes off @pending. Rewritten as blocks on a scratch branch,
  the review section passed its gates, its derived cases and the local
  review, whose bundle showed no requirement changed. A drift case
  lands with it, and the srs kind's cap rises to 700 lines.
---
# Phase 1 handoff

## Done

- The `srs` crate reads a requirement written as a block, and `parse`
  returns a document's rows and blocks in its order. A block reads as
  the row it replaces: Pri, Ver and Traces as those columns, its text
  as its lines joined by single spaces, at its heading's line.
- A heading inside a code fence opens no block. A heading, or a line
  that opens a table, ends one. An unknown field, a field twice, no
  field, no text or a malformed trace is malformed, at its line. A row
  and a block that share an id are an id defined twice.
- The derived cases find a section's first requirement as a row or as
  a block, outside a code fence. For a block, the defect case adds its
  MUST on a line of its own after the block's text.
- The testing page says how a requirement is written. The srs kind's
  cap in `.mdsmith.yml` holds 700 lines, the stakeholder's ruling of
  21 November 2026.

## Verified

- Unit tests of the `srs` crate and of the derived cases pass, and
  cover every line the phase adds. rustfmt, clippy denying warnings,
  the workspace's tests and the cucumber scenarios `@ENG-21`,
  `@ENG-27`, `@ENG-28` and `@ENG-32` pass.
- The new drift case creates a section holding a block with ENG-25's
  id. The parse gate fails on it: `10b-drift.md:3: ENG-25 already
  defined at`.
- On a scratch branch, never pushed, the review section's two
  requirements were rewritten as blocks. Its 14 lines, the longest of
  13,239 characters, became 248, none over 80 characters.
- `mdsmith check .` passes on it, and so do the scenario gate, the srs
  gates and the cucumber scenarios `@ENG-28`, `@ENG-32` and `@ENG-34`.
- The drift suite catches the six cases that inject into §10. No case
  injects into §10.5.
- The review bundle against this phase shows ENG-28 and ENG-32 with
  main's text alone: no requirement changed.
- `review-eval run --component srs-review-requirements` passed its six
  cases in 12 of 12 runs on the rewritten tree, the block's derived
  defect case among them. A run took 3.6 turns and 0.04 USD, against
  3.5 turns and 0.05 USD on the rows.
- `scripts/review-local.sh` approved the rewrite: four agents, no
  finding kept, 0.17 USD, 117 seconds. The hub's specialist asked for
  two words the section already used; the generalist kept neither.

## Next

- Main's `lang-gherkin` finds a requirement's priority with the search
  `^\| <ID> \| P[0-9]`, which matches a row and no block. On #161's
  case of a step that says other than its long row, it passed 3 of 3
  runs on both forms, but took 8.0 turns on blocks against 7.3 on
  rows. Its turns ran out on pull request 154.
- So the review section is rewritten (task 2) only once main's
  `lang-gherkin` reads a requirement as its review context quotes it,
  which #161 lands. Otherwise that pull request changes the lookup
  itself, with its cases.
- What a rewrite buys a diff: #161's one clause added to ENG-28 is a
  diff of 106,482 bytes as rows, which repads all four lines of the
  table. As blocks it is 728 bytes over 3 lines.
