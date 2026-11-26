---
n: 5
title: "§6, §6b and §10 written as blocks"
status: "✅"
result: true
summary: >-
  The 64 requirements of security, the boundary register's own and
  engineering quality are each written as a block, and none changes:
  main's review bundle shows each with main's text alone. The three
  drift cases that quoted §10's rows inject into its blocks and are
  caught. Every requirement of task 3's fourteen sections is now a
  block. No scenario comes off @pending.
---
# Phase 5 handoff

## Done

- The 31 requirements of §6, the one of §6b and the 32 of §10 moved
  out of their tables into blocks, each a heading with its id, its
  fields as a list and its text one sentence a line. The threat table
  of §6 and the register of §6b stay tables. No other line of the
  three files changed.
- The three files hold 1,267 lines where they held 262, and 97,777
  bytes where they held 224,682. The longest, §6, holds 663 lines,
  under the srs kind's cap of 700.
- The drift cases that change ENG-25's priority, rename ENG-25 to
  ENG-24 and drop PRV-11 from ENG-29's list inject into §10's blocks.
  The case that writes a block with an id §10 defines in a new section
  names it as another section's.

## Verified

- Red: with the three files rewritten and the cases not moved, the
  drift registry's test that every case applies failed on the three,
  each naming the row it no longer found. Green: it passes.
- The drift suite catches the 16 cases that inject into the three
  files, the three moved ones among them.
- `mdsmith check .` passes, with the srs kind's shape schema and its
  50-line cap on a block.
- The srs gates, the scenario gate, the workspace's 4,642 tests and
  the cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32`
  and `@ENG-34` pass.
- `review-eval check` finds every case applies: 235 cases, 74
  components.
- `review-bundle` against main shows each of the 64 with main's text
  alone: no requirement changed.
- `scripts/review-local.sh` against main approved: 12 of 12 agents
  consulted, no finding, 0.75 USD.

## Next

- Task 2, the review section, once #161 lands.
- Tasks 4 to 6. The Execution table sits at mdsmith's column width
  ratio: a later phase's gate fits in about 60 characters.
