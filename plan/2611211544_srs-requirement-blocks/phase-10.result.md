---
n: 10
title: "The command-line interface written as a list"
status: "✅"
result: true
summary: >-
  The 83 commands of §9.5 are each a list item, word for word as the
  table held them, their purposes one sentence a line at 80 columns.
  The section falls from 134,889 bytes in 140 lines to 26,179 in 384,
  where each row was padded to 1,545 characters. The srs kind's line
  length is stern, so only a word may run past 80 columns. No
  requirement changes, and no scenario comes off @pending.
---
# Phase 10 handoff

## Done

- The command table of §9.5 moved out of its table, each row a list
  item: its commands, then a colon and its purpose one sentence a
  line, wrapped at 80 columns.
- A code span stays whole where it fits on a line of its own and
  breaks at its own spaces where it does not, as three synopses do.
  CommonMark and GitHub's Markdown render each the same as the table
  did.
- The sentence after the table says entries where it said rows, and
  the domain model's hub points at the `licenses` entry where it
  pointed at its row. No other line changed.
- The srs kind's line length is stern, as the stakeholder chose on 22
  November 2026: the synopsis of `parinn purge` holds 81 columns, its
  scopes one word of 79 characters. `.mdsmith.yml` holds 500 lines,
  its kind's limit, so the rule takes two.

## Verified

- With the section rewritten and the line length not stern,
  `mdsmith check .` failed on the synopsis's line, 81 > 80. With it
  stern, `mdsmith check .` passes.
- The drift suite caught the new case, a sentence of the SRS run on
  past 80 columns at a space.
- A comparison outside the repository found each item's commands and
  purpose the same as its row's, and every other line of the section
  unchanged but the one reworded.
- The srs gates, the scenario gate, the workspace's 4,642 tests and
  the cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32`
  and `@ENG-34` pass.
- `review-eval check` finds every case applies: 235 cases, 74
  components.
- `scripts/review-local.sh` against phase 9's head: 8 of 8 agents
  consulted, 0.21 USD. Their one nit, the hub's pointer to the
  `licenses` row, is taken. Its blocking finding is ENG-21's: no
  stakeholder comment consents to `.mdsmith.yml` yet.

## Next

- Appendix B, with the invariant coverage gate's reader, then the MCP
  tools, with the reader of the recall scenario.
- §6's threats and residual risks as lists would take §6 past the srs
  kind's cap of 700 lines, so they need a split or a ruling first.
