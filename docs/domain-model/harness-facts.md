---
title: "Harness facts"
order: "02"
summary: >-
  The harness's own facts that ParleyInn records and names: the harness, harness sessions and transcripts, hooks, turns and compaction.
---
# Harness facts

ParleyInn records and names the harness's own facts. It does not manage them.
It changes only harness configuration, under I7.

- **Harness**: The external program that runs agents, such as Claude Code or the
  Agent SDK. It keeps their transcripts, compacts their context and calls
  ParleyInn
  through hooks where ParleyInn writes to an agent; ParleyInn reads the rest of
  a run
  from its transcripts.
- **Harness session**: The harness's own unit, which yields one transcript, plus
  one per subagent where the harness writes them apart (ASM-03). Named only when
  describing the harness, never as a ParleyInn unit. One whose harness process
  descends from a run's tool call records no `user` event (PRV-08); recall
  taint passes between its runs and that run (SEC-13). Such a harness session
  is **nested**. Its **process ancestry** is the chain of processes a hook
  handler of it descends from, read inside the core: a harness process above
  the harness session's own makes it nested.
- **Transcript**: The harness's file of what a harness session did. A
  **transcript source** is a transcript ParleyInn ingests (Ingest marker,
  `parinn
  ingest`); a shrunk transcript or a rewritten prefix starts a new **transcript
  generation** (REC-07). The **transcript roots** (`transcript.roots`) are the
  directories ingest reads transcripts from by default (REC-01, SEC-18). An
  **allowed root** is a directory within which every path ParleyInn receives
  must
  resolve: the transcript roots, the home, and, for reading only, a run's
  worktree and its repository's git directory (SEC-18). A path outside every
  allowed root is refused. As a
  ParleyInn term, "source" is a transcript source or a trusted source; the
  harness's
  `source` field and §6.1's threat sources keep their own sense. Each
  **transcript line** is one JSON value, recorded as one event. Its event
  keeps the line's well-formed `timestamp` as its **transcript timestamp**, a
  field outside the chained header that is no structural field and never
  trusted; `seq` alone orders events (REC-12), and time marks alone age them
  (REC-15). A harness
  version's **recorded shape** is, per line type (an attachment by its type, a
  `system` line by its subtype), the keys its recorded lines all carry and those
  any carries, generated from that version's recorded transcripts (ENG-17), and
  per hook the same of its recorded hook inputs. A
  transcript line or hook input whose shape differs from the one recorded for
  its harness version is **format drift**, counted and audited (REC-26): such a
  line is kept whole, redacted, as untrusted `harness_text`, and such a hook
  input gets the empty answer.
- **Hook**: The harness's callback into ParleyInn, carrying its **hook input**,
a
  JSON object (§9.1). The core's hook handlers answer it. ParleyInn installs
  only
  the hooks through which it writes to an agent: `SessionStart`,
  `UserPromptSubmit` and `PermissionRequest`. Git's hooks are always qualified,
  as the commit hook.
- **Attended mark**: The harness's mark, in each hook handler's environment,
  that a harness session runs in the harness's own terminal interface, not in
  print mode or the Agent SDK (Claude Code: `CLAUDE_CODE_SESSION_ATTENDED` set
  to `1`, ASM-22). The harness sets it whatever started it; the MCP server gets
  none of its own and inherits what started the harness, so only a hook handler
  reads it. A `UserPromptSubmit` hook observation records it as a structural
  field, which can only keep `user` untrusted, never make anything trusted
  (PRV-04). A **prompt match** is the structural event naming the `user` event a
  prompt commitment was used up by, its `prompt_id` and whether it matched
  (PRV-08).
- **Tool-use id**: The harness's id for one tool call, carried by the call's
  transcript line and by its MCP call (Claude Code: `_meta`
  `claudecode/toolUseId`). The MCP server finds the run that made a call only by
  the transcript line carrying it (PRV-14).
- **Turn**: One exchange between the harness and the model, from an input to the
  model reply that ends it. A **turn end** is that reply: one that stops for no
  tool call and no harness pause. A **subagent end** is a subagent's
  `subagent_result` event (REC-19). A **user turn** is the turn that text typed
  at the harness's own input starts (its prompt, or the terminal the launcher
  hosts): a person's message in `interactive` deployment mode, a pipeline's in
  `automation`. A line the launcher carried in starts no user turn: the launcher
  records the carried text's commitment as its **turn trigger** (what started a
  turn, LANE-14). Only that typed text is `user`, trusted only when matched to
  the prompt commitment of the text a `UserPromptSubmit` hook got and its hook
  handler found the attended mark (PRV-04), while no run on the node leaves R3
  or R8 open (PRV-08). Other free text the harness writes or adds to the model's
  input is `harness_text`, never `user` and never trusted, unless it is a turn's
  model reply, a tool result, a file's or an MCP server's text or a subagent's
  report (PRV-08, PRV-03).
- **Compaction**: The harness replacing earlier context with a compaction
  summary when the context window fills or when asked. ParleyInn neither
  performs
  nor controls it (NG1), and sends the harness nothing for it; it records it
  from the transcript and restores pins after it (I3). The harness's own call
  that writes the summary is a harness internal, no agent and no run, and
  ParleyInn
  records nothing for it (REC-02). While the `SessionStart` hook after it runs,
  nothing names the compacting run (ASM-06).
