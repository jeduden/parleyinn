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
  review, whose bundle showed no requirement changed. Drift cases land
  with it. The srs kind's cap rises to 700 lines, and a requirement may
  hold 50, six longer ones listed until each is split.
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
- A requirement written as a block holds at most 50 lines, from its
  heading to the next, through mdsmith's `max-section-length`: the
  stakeholder's ruling of 22 November 2026. The six longer ones are
  listed at their length, ENG-28's with #161's clause.
- The derived cases read a block as the srs crate does: its heading's
  id has two digits, and a fence inside it is part of its text.

## Verified

- Unit tests of the `srs` crate and of the derived cases pass, and
  cover every line the phase adds. rustfmt, clippy denying warnings,
  the workspace's tests and the cucumber scenarios `@ENG-21`,
  `@ENG-27`, `@ENG-28` and `@ENG-32` pass.
- The new drift case creates a section holding a block with ENG-25's
  id. The parse gate fails on it: `10b-drift.md:3: ENG-25 already
  defined at`.
- The drift case that writes a block of 51 lines before §10.4's
  heading fails `mdsmith check .`, which passes it without the cap:
  `section "### ENG-99" too long (51 > 50)`.
- With every section rewritten as blocks on a scratch tree, `mdsmith
  check .` passes under the cap. Any listed length lowered by one
  fails it, naming that requirement alone, ENG-28's with #161's clause.
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
  finding kept, 0.17 USD, 117 seconds.

## Next

- Main's `lang-gherkin` finds a requirement's priority with the search
  `^\| <ID> \| P[0-9]`, which matches a row and no block. On #161's
  long-row case it passed 3 of 3 runs on both forms, but took 8.0
  turns on blocks against 7.3 on rows. Its turns ran out on pull
  request 154.
- So the review section is rewritten (task 2) only once main's
  `lang-gherkin` reads a requirement as its review context quotes it,
  which #161 lands. Otherwise that pull request changes the lookup
  itself, with its cases.
- Task 6 splits the six requirements the cap lists, each into
  requirements of at most 50 lines with their scenarios, and drops
  its entry.
- What a rewrite buys a diff: #161's one clause added to ENG-28 is a
  diff of 106,482 bytes as rows, which repads all four lines of the
  table. As blocks it is 728 bytes over 3 lines.
