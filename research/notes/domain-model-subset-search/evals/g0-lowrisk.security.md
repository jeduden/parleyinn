```json
[
  {
    "type": "security",
    "quote": "An authenticated client where principal acts are taken:",
    "problem": "The candidate never says how the CLI or TUI authenticates the person, and no rule requires the presence proof it defines, so being 'at a terminal' is the only thing that separates the principal from the agent. An agent that runs `parinn` under a pseudo-terminal from its own shell can therefore take widening acts (add a constraint pin, confirm its own pin candidate, turn capture off, release a quarantine). It can also take neutral acts (acknowledge counters, dismiss Needs you items) that hide failures from Health.",
    "fix": "Principal surface: a client the person authenticates to, never one an agent's tool call starts, whatever its output; every widening act carries a presence proof bound to it, and the core refuses and counts any act without one.",
    "severity": "needs-fix"
  },
  {
    "type": "security",
    "quote": "or a read-only CLI verb the agent runs whose output is not a",
    "problem": "Whether CLI output is enveloped depends only on whether it is a terminal. A read-only verb the agent runs under a pseudo-terminal (some harness shells allocate one) returns raw untrusted record text with no envelope. Verbs that are not read-only (ingest, room acts) may echo record text unenveloped. Both paths fall outside I2.",
    "fix": "or any CLI verb an agent's tool call runs, whatever its output; the CLI prints record text unenveloped only at an authenticated principal surface.",
    "severity": "needs-fix"
  },
  {
    "type": "security",
    "quote": "takes its seat's **pin class**, `operator` for a device seat and `assistant`",
    "problem": "A device-seat room act gets provenance `operator`, which is a trusted source. Room acts are taken 'wherever', and the CLI can only sign as the device seat (run-seat keys live only in the MCP server). So text an agent passes to `parinn` from its shell (a `fact` or `decision` pin, a link, a created room) becomes trusted, which contradicts 'Room acts are governance, not I2 trust'.",
    "fix": "Add to Device seat: 'A device seat signs room acts only at a principal surface.'",
    "severity": "needs-fix"
  },
  {
    "type": "security",
    "quote": "The text of a delegation: a subagent's from its harness.",
    "problem": "No provenance class covers a delegated task. It arrives in the subagent's run as harness input, so the hook handlers record it as `user`, which is trusted in `interactive` mode, even though it is the parent model's reply. It then feeds trust marks and ParleyInn-detected pin candidates that are shown as the person's own words.",
    "fix": "In the subagent's run a delegated task takes the provenance of the parent's `tool_call`, never `user`; `user` is only the prompt of a user turn.",
    "severity": "needs-fix"
  },
  {
    "type": "security",
    "quote": "**Ingest** is reading a transcript into the record, by the hook handlers",
    "problem": "Hook handlers read `user` lines from the transcript. The agent can write that file (the candidate itself names rewritten prefixes), and the harness writes user-role lines no person typed (attached files, command output). Because the hook handlers record them in `interactive` mode, a forged or attached line becomes a trusted `user` event.",
    "fix": "Only the prompt a hook reports for a user turn becomes a trusted `user` event; a `user` line read from a transcript is untrusted unless it matches that prompt.",
    "severity": "needs-fix"
  },
  {
    "type": "security",
    "quote": "repository's `.parinn.toml`, its **repository configuration**, may only tighten",
    "problem": "'Tighten' has no defined direction for settings that shape restore blocks or capture (`restore_block.on_prompt`, the pin budget, capture). Fewer injections reads as tighter, yet removing a pin from a restore block or stopping capture is a widening act. A `.parinn.toml` written by an agent or an outside contributor could then drop the principal's constraints after compaction.",
    "fix": "...may only tighten, and never removes a pin from a restore block, turns a restore point off or stops capture; those stay widening principal acts.",
    "severity": "needs-fix"
  },
  {
    "type": "security",
    "quote": "or through a component the node's principal turned on",
    "problem": "The closed component set holds only the core (B0), and I4 now defines only B0. Yet I4 still lets data leave the machine through 'a component the node's principal turned on', and Boundary names B1 and B3 (managed policy also names B2). No rule says what those boundaries may listen on, connect to or send, and turning one on is no named principal act.",
    "fix": "Delete this clause from I4, 'B1 machine and B3 public' from Boundary, 'but for each B1 to B3 component's own entry point' from the CLI, and 'B1 to B3' from Managed policy; or restore I4's B1 and B3 bounds.",
    "severity": "needs-fix"
  },
  {
    "type": "security",
    "quote": "Events after the newest seal are unsigned.",
    "problem": "Trusted sources include 'this node's `operator` events' with no requirement that a seal covers them. An unsigned `operator` tail in a device seat's writer counts as trusted, whether a process of the same OS user appended it or a tampered backup restore brought it back. Nothing requires a qualifying pin's creating event to be the sealed principal act.",
    "fix": "Add: 'An event is trusted only once a seal by a seat key this node's device key certified covers it; a qualifying pin's creating event is a principal act so covered.'",
    "severity": "minor"
  },
  {
    "type": "useless",
    "quote": "addresses and the principal-typed text, its requirement",
    "problem": "None of I2's closed paths uses a fixed template or principal-typed text, so the concept has no use in this candidate. It also invites a write to an agent carrying principal-typed text, which I2 says does not exist.",
    "fix": "Remove Fixed template and principal-typed text, or name in I2 the closed path that uses them.",
    "severity": "minor"
  }
]
```

FINDINGS: 7
