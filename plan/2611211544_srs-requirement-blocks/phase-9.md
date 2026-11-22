---
n: 9
title: "The components, decisions, store tables and concerns as lists"
status: "✅"
result: false
---
# Phase 9: the components, decisions, store tables and concerns as lists

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 5's fourth pull request: the four tables no gate reads
whose rows pass 500 characters. They are the reference architecture's
components, 741 characters a row, and its design decisions, 1,178; the
store's logical schema of §8.2, 564; and Appendix A's concerns, 503.

The edit. Each row becomes a list item, as the open questions did: its
lead, then its text one sentence a line, then its other columns as
fields in a list under it, wrapped at 80 columns. A component leads
with its name, its responsibility the text. A decision leads with its
ADR, its decision the text, its alternatives and rationale the fields.
A table of the schema leads with its name and kind, its key contents
the text, its constraints a field. A concern leads with its id, the
concern the text, ParleyInn's response and its requirements the fields.
Appendix A's four narrower tables follow, so its five read alike. The
core's parts, at 369 characters a row, and the lifecycle grid, at 303,
stay tables. No other line changes.

RED. None: no code changes, and no gate reads these tables.

GREEN. The four tables are lists whose items hold their rows' cells
word for word.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- a comparison outside the repository finds each item's lead, text and
  fields the same as its row's cells, and every other line of the
  three sections unchanged;
- the srs gates, the scenario gate and the workspace's tests pass;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
