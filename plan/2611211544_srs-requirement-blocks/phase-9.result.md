---
n: 9
title: "The components, decisions, store tables and concerns as lists"
status: "✅"
result: true
summary: >-
  The reference architecture's 6 components and 10 design decisions,
  the store's 9 tables of §8.2 and Appendix A's 45 concerns are each a
  list item, word for word as their tables held them, their other
  columns fields under each. Their widest rows held 741, 1,178, 564
  and 503 characters. No requirement changes, and no scenario comes
  off @pending.
---
# Phase 9 handoff

## Done

- The reference architecture's components and design decisions, the
  store's logical schema of §8.2 and Appendix A's concerns moved out
  of their tables, each row a list item: its lead, its text one
  sentence a line, and its other columns as fields in a list under it,
  wrapped at 80 columns.
- A component leads with its name, a decision with its ADR, a table of
  the schema with its name and kind, a concern with its id. Appendix
  A's four narrower tables follow its widest, so its five read alike.
- The core's parts and the lifecycle grid stay tables, their rows at
  369 and 303 characters. No other line changed.
- §4 falls from 32,797 bytes in 166 lines to 19,729 in 252, §8 from
  19,608 in 129 to 15,415 in 151, and Appendix A from 18,578 in 79 to
  11,165 in 219.

## Verified

- A comparison outside the repository found each item's lead, text and
  fields the same as its row's cells, and every other line of the
  three sections unchanged.
- `mdsmith check .` passes.
- The srs gates, the scenario gate, the workspace's 4,642 tests and
  the cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32`
  and `@ENG-34` pass.
- `review-eval check` finds every case applies: 235 cases, 74
  components. No drift case injects into the three sections.
- `scripts/review-local.sh` against phase 8's head approved: 6 of 6
  agents consulted, no finding, 0.22 USD.

## Next

- The command-line interface as a list, with mdsmith's stern line
  length for the srs kind, as the stakeholder chose: a line may pass
  80 columns only where it holds no space past them, as the synopsis
  of `parinn purge` and its nine scopes does. The change to
  `.mdsmith.yml` needs the stakeholder's consent.
- Appendix B, with the invariant coverage gate's reader, then the MCP
  tools, with the readers of the recall and envelope scenarios.
- §6's threats and residual risks as lists would take §6 past the srs
  kind's cap of 700 lines, so they need a split or a ruling first.
