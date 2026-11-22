---
n: 11
title: "Appendix B written as a list"
status: "✅"
result: true
summary: >-
  Appendix B's coverage of the ten invariants is a list, an item per
  invariant holding its row's requirement ids word for word, wrapped at
  80 columns, where each row was padded to 860 characters. The appendix
  falls from 11,348 bytes in 35 lines to 4,522 in 72. Its gate reads
  the list. No requirement changes, and no scenario comes off @pending.
---
# Phase 11 handoff

## Done

- Appendix B's coverage table moved out of its table, each row a list
  item: the invariant in bold, then a colon and its requirement ids,
  wrapped at 80 columns.
- The summary and the first sentence speak of the Traces of the
  requirements of §5–§6, where they spoke of a Traces column and of
  tables that §5–§6 no longer hold. The count table stays. No other
  line changed.
- The srs crate's reader of the coverage reads the first list after
  the front matter, an item's lines joined by single spaces, and
  refuses an item it cannot read, naming its line. The gate
  `appendix_b_matches_the_traces` speaks of items where it spoke of
  rows.
- A drift case drops a requirement from I3's item.

## Verified

- With the appendix rewritten and the table's reader, the gate failed:
  no invariant coverage table. With the list's reader, it passes, and
  the reader's unit tests pass.
- A comparison outside the repository found each item's ids the same
  as its row's, and every other line of the appendix unchanged but the
  reworded summary and sentence.
- The drift suite caught the new case. The count case, a requirement
  demoted without the appendix's count, injected by hand, fails the
  gate on its counts.
- `mdsmith check .`, the srs gates, the scenario gate, the workspace's
  4,644 tests and the cucumber scenarios `@ENG-21`, `@ENG-27`,
  `@ENG-28`, `@ENG-32` and `@ENG-34` pass.
- `review-eval check` finds every case applies: 235 cases, 74
  components.
- `scripts/review-local.sh` against phase 10's head approved: 14 of 14
  agents consulted, no finding, 0.54 USD.
- Rebased onto phase 10's head, mdsmith's merge driver rewrote the
  plan's catalog without this phase's row; `mdsmith fix` restored it.

## Next

- The MCP tools of §9.2 as a list, with the reader of the recall
  scenario, `@RCL-01`, reading each tool's parameters from its item.
- §6's threats and residual risks as lists would take §6 past the srs
  kind's cap of 700 lines, so they need a split or a ruling first.
