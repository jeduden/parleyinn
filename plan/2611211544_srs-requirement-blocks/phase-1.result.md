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
  the review section passed its gates and the local review, and its
  bundle showed no requirement changed. The srs kind's cap rises to
  700 lines, a requirement may hold 50, six longer ones listed until
  each is split, and mdsmith checks a block's shape.
---
# Phase 1 handoff

## Done

- The `srs` crate reads a requirement written as a block, and `parse`
  returns a document's rows and blocks in its order. A block reads as
  the row it replaces: Pri, Ver and Traces as those columns, its text
  as its lines joined by single spaces, at its heading's line.
- A heading inside a code fence opens no block. A heading, or a pipe
  row with a delimiter row under it, ends one. A field's lines run to
  the blank line that ends the list, as Markdown reads them. An unknown
  field, a field twice, no field, no text or a malformed trace is
  malformed, at its line. A row and a block that share an id are an id
  defined twice.
- The derived cases find a section's first requirement as a row or as
  a block, outside a code fence, and read a block as the srs crate
  does. For a block, the defect case adds its MUST on a line of its own
  after its text.
- The testing page says how a requirement is written. The srs kind's
  cap in `.mdsmith.yml` holds 700 lines, the stakeholder's ruling of
  21 November 2026.
- A block holds at most 50 lines through mdsmith's
  `max-section-length`, the stakeholder's ruling of 22 November 2026,
  the six longer ones listed at their length.
- mdsmith's schema for the srs kind holds a block to that shape, as the
  stakeholder asked the same day: a list marked with dashes, then one
  paragraph and nothing else, at its level.

## Verified

- Unit tests of the `srs` crate and of the derived cases pass, and
  cover every line the phase adds. rustfmt, clippy denying warnings,
  the workspace's tests and the cucumber scenarios `@ENG-21`,
  `@ENG-27`, `@ENG-28` and `@ENG-32` pass.
- A drift case creates a section holding a block with ENG-25's id,
  and the parse gate fails on it: `10b-drift.md:3: ENG-25 already
  defined at`.
- The drift case that writes a block of 51 lines before §10.4's
  heading fails `mdsmith check .`, which passes it without the cap:
  `section "### ENG-99" too long (51 > 50)`.
- Six drift cases write a block of a broken shape before §10.4's
  heading. `mdsmith check .` passes each without the schema and fails
  each with it.
- With every section rewritten as blocks on a scratch tree, `mdsmith
  check .` passes under the cap and the schema. Any listed length
  lowered by one fails it, naming that requirement alone.
- On a scratch branch, never pushed, the review section's 14 lines,
  the longest of 13,239 characters, became 248 as blocks, none over 80
  characters.
- `mdsmith check .` passes on it, and so do the scenario gate, the srs
  gates and the cucumber scenarios `@ENG-28`, `@ENG-32` and `@ENG-34`.
- The drift suite caught each case that injected into §10.
- The review bundle against this phase shows ENG-28 and ENG-32 with
  main's text alone: no requirement changed.
- `review-eval run --component srs-review-requirements` passed its six
  cases in 12 of 12 runs on the rewritten tree, the block's derived
  defect case among them, at 3.6 turns and 0.04 USD a run against 3.5
  and 0.05 on the rows.
- `scripts/review-local.sh` approved the rewrite: four agents, no
  finding kept, 0.17 USD, 117 seconds.

## Next

- Main's `lang-gherkin` finds a priority with the search
  `^\| <ID> \| P[0-9]`, which matches a row and no block. On #161's
  long-row case it passed 3 of 3 runs on both forms, in 8.0 turns on
  blocks against 7.3 on rows.
- So the review section is rewritten (task 2) only once main's
  `lang-gherkin` reads a requirement as its review context quotes it,
  which #161 lands. Otherwise that pull request changes the lookup
  itself, with its cases.
- Task 6 splits the six the cap lists into requirements of at most
  50 lines, each with its scenario.
- What a rewrite buys a diff: #161's one clause added to ENG-28 is
  106,482 bytes as rows, 728 as blocks.
