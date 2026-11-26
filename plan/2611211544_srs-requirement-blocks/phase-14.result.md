---
n: 14
title: "The review section written as blocks"
status: "✅"
result: true
summary: >-
  ENG-28 and ENG-32, the review section's two requirements, are each
  written as a block, and neither changes: main's review bundle shows
  each with main's text alone. ENG-28 holds 193 lines, the length the
  srs kind lists it at. No requirement table is left in the SRS. No
  scenario comes off @pending.
---
# Phase 14 handoff

## Done

- ENG-28 and ENG-32 moved out of the review section's table into
  blocks, each a heading with its id, its priority and verification
  method as a list and its text one sentence a line, wrapped at 80
  columns. The sentence that named the table's columns says how a
  requirement is written. No other line of the file changed.
- The section holds 16,540 bytes in 249 lines, where it held 53,457 in
  14: ENG-28's row was 13,306 characters, and the table padded ENG-32's
  row and its separator to that width.

## Verified

- A comparison outside the repository found each block's fields and
  text the same as its row's cells.
- `review-bundle` against main shows ENG-28 and ENG-32 with main's text
  alone: no requirement changed. No drift case injects into the
  section.
- `mdsmith check .` passes, ENG-28 at its listed 193 lines.
- The srs gates, the scenario gate, the workspace's 4,652 tests and the
  cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32` and
  `@ENG-34` pass.
- `review-eval check` finds every case applies, the cases derived from
  the section among them: 236 cases, 74 components.
- `scripts/review-local.sh` against main approved: 4 of 4 agents
  consulted, no finding, 0.19 USD.

## Next

- Task 4: no requirement table is left, so the parser can refuse one,
  with a drift case, and the SRS's tools and agents speak of a
  requirement where they say row.
- #154's two sentences take ENG-28's block to 202 lines, past the 193
  it is listed at. The stakeholder rules whether it is listed at its
  new length or split first (task 6).
