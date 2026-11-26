---
n: 12
title: "The MCP tools written as a list"
status: "✅"
result: true
summary: >-
  The 30 MCP tools of §9.2 are each a list item, word for word as the
  table held them, their priority, parameters, returns and limits
  fields under the tool's name, wrapped at 80 columns. The section
  falls from 46,675 bytes in 245 lines to 23,901 in 393, where each row
  was padded to 969 characters. The recall scenario reads the list. No
  requirement changes, and no scenario comes off @pending.
---
# Phase 12 handoff

## Done

- The tool table of §9.2 moved out of its table, each row a list item:
  the tool's name, then its priority, parameters, returns and limits as
  fields named by their columns, wrapped at 80 columns. An empty cell
  gave no field; a cell holding a dash kept it. No other line changed.
- The recall scenario's reader of §9.2, `@RCL-01`'s, reads the
  section's items, each Parameters field's lines joined, where it read
  every table row of the file, the hook and settings tables' among
  them.
- `recall.rs` shrank to 1,038 lines, and its entry on the size list
  says so.
- A drift case gives `event_get` a parameter on the second line of its
  Parameters field, which the tool does not declare.

## Verified

- With the list and the table's reader, `@RCL-01` failed: §9.2
  specifies no tool `event_search`. With the list's reader, it passes.
- A comparison outside the repository found each item's fields the
  same as its row's cells, each tool's parameters as the table's
  reader read them, and every other line of the section unchanged.
- The drift suite caught the new case and the five size-list cases.
- `mdsmith check .`, clippy, rustfmt, the srs gates, the scenario gate,
  the workspace's 4,644 tests and the cucumber scenarios `@ENG-21`,
  `@ENG-27`, `@ENG-28`, `@ENG-32` and `@ENG-34` pass.
- `review-eval check` finds every case applies: 235 cases, 74
  components.
- `scripts/review-local.sh` against phase 11's head approved: 9 of 9
  agents consulted, 0.45 USD. Its one nit stands, below.

## Next

- The reader keeps every code span outside parentheses as a
  parameter, as the table's reader did, so `room_pin`'s `intent` and
  `verdict`, values of its `type`, read as parameters. No offered tool
  meets it yet; the reader needs to stop there before `room_pin`
  ships.
- §6's threats and residual risks as lists would take §6 past the srs
  kind's cap of 700 lines, to about 760, so they wait on the
  stakeholder's ruling.
