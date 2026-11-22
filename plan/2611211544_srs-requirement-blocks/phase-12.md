---
n: 12
title: "The MCP tools written as a list"
status: "✅"
result: false
---
# Phase 12: The MCP tools written as a list

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 5's seventh pull request. The tool table of §9.2 held a
row per tool, 30 in all, each padded to 969 characters. The recall
scenario, `@RCL-01`, reads it: each tool the MCP server offers must
have a row, and must declare the parameters its row's Parameters cell
names outside parentheses.

The edit. Each row becomes a list item: the tool's name, then its
priority, parameters, returns and limits as fields named by their
columns, wrapped at 80 columns. An empty cell gives no field; a cell
holding a dash keeps it. No other line changes.

The reader. The scenario's reader of §9.2 reads the section's items,
each Parameters field's lines joined by single spaces, where it read
every table row of the file. `recall.rs` shrinks, and its entry on the
size list is lowered to its length. A drift case gives `event_get` a
parameter on a field's second line that the tool does not declare, and
the scenario catches it.

RED. With the list and the table's reader, the scenario fails: §9.2
specifies no `event_search`.

GREEN. With the list's reader, the scenario passes.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- a comparison outside the repository finds each item's fields the
  same as its row's cells, each tool's parameters as the table's
  reader read them, and every other line of the section unchanged;
- the srs gates, the scenario gate, the workspace's tests, clippy and
  the scenarios `@RCL-01` and the ENG ones pass, and the drift suite
  catches the new case;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
