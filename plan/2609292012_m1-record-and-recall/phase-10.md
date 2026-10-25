---
n: 10
title: "The MCP server and live ingest"
status: "✅"
result: false
---
# Phase 10: the MCP server and live ingest

Requirements. REC-13's live ingest, PRV-14's run of a tool call,
SEC-10's run-seat keys in the MCP server's memory alone, REC-06 under
concurrent appends, and REC-19's held observations appended once the
MCP server holds its run-seat key. ENG-08's gap-free `seq` property
runs again on several threads at once.

The session binding (REC-13): a `SessionStart` hook handler records its
nearest harness process beside its process ancestry. The MCP server
reads its own the same way and takes every session binding of that
harness process, marked taken in one transaction, so none is counted
untaken. Before that it ingests nothing as witnessed. Off Linux none
is taken, and `parinn ingest` records the run (REC-22).

Run seats (SEC-10): once bound, the MCP server mints a run-seat key in
memory for the main run and each subagent's run, certified as a run
seat and no ingest seat (`Node::certify_run_seat`). No key reaches the
key store, a file, a log or any output.

Live ingest (REC-13): the MCP server polls each transcript of its bound
harness sessions, subagents' included. Each resolves within the
transcript roots, and no link swapped in later is followed. Each
complete line past the ingest position goes to its run's run seat, of
origin `witnessed`. It opens no socket (I4).

- A held hook observation goes to the main run's run seat at the offset
  where its hook fired (REC-19).
- A witnessed line is never a repeat reference to an ingested event:
  it is its own event, audited (REC-04).
- When its input closes it stops. A transcript left short keeps its
  ingest position as its ingest marker; the stop is audited under
  `ingest_remainder_left`, no failure counter, by the MCP server, or by
  the next reader to find the marker after a kill.
- An append whose ingest position another process moved first is
  refused in its transaction and read again, so every line is recorded
  once (REC-06).

The run of a tool call (PRV-14): only the transcript lines carrying the
call's tool-use id (`_meta` `claudecode/toolUseId`) name it. After a
restart they are read again from the start. A call waits while ingest
reads, at most 2 s. One with no tool-use id, no line in time, or lines
of two runs is refused, counted per cause and audited. Each call is
logged with its tool-use id, the run or cause, and the wait.

Where the gate's scenarios go live:

- `@REC-13`: both rows, against `parinn mcp` under a stand-in harness
  process.
- `@PRV-14`: its calls name `event_search`, phase 12's, so it passes
  once that lands.
- `@REC-06`: its purged range needs phase 14; the rest bound.
- `@REC-19`'s holding step, `@REC-04`'s homes A, B, G and H, `@REC-22`
  and `@SEC-10`'s run-seat steps: bound where they pass.

Gate: `@REC-06`, `@REC-13` and `@PRV-14` pass against a live MCP
server, or name what keeps them pending; REC-19's holding step passes.
