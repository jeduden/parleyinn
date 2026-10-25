---
title: "1.4 Scope"
summary: >-
  Scope by context layer: ParleyInn implements the record, pin, recall and
  kernel layers; long-term memory is out of scope.
  Claude first, room summaries from the facilitator, and the room
  view on loopback (B1), then on enrolled devices (B2), with any
  wider reach an open question (OQ-38).
---
# 1.4 Scope

ParleyInn implements layers 1–4 of the agent context architecture:

| Layer | Name             | In ParleyInn v1                                             |
| ----- | ---------------- | ----------------------------------------------------------- |
| 1     | Record           | Append-only, provenance-tagged event log of every agent run |
| 2     | Pins             | Qualifying pins, restored verbatim after every compaction   |
| 3     | Recall           | Pull-only recall, landmark index, restore blocks            |
| 4     | Kernel           | Hermetic kernel with read-only access to the record         |
| 5     | Long-term memory | **Not in scope.** Export interface only (§5.9)              |

ParleyInn serves Claude first. Version 1 supports Claude Code and the Claude
Agent SDK; other harnesses come later, and the design MUST NOT preclude
them (ADR-08, OQ-16).

Room summaries are in scope. When a run joins a room with much
content, its agent makes a summary request for a room summary of the
size it wants (LANE-33). The room's one facilitator, the service
account the owner appoints, writes it through its own node's CLI
(`parinn room-summary write`), signed with its device seat's key there;
the program that writes it is neither a ParleyInn component nor an agent,
and the core makes no model calls (I4). A room summary is untrusted
and never replaces the record (I1, I2).

The room view reaches the principal in two steps. It stays a client of
the record under VIEW-03, and everything it lets a person do also
exists in the CLI or MCP.

1. The browser room view on loopback (B1). Without the peer component, a
   browser on the principal's phone reaches it only as the browser room
   view, a principal surface, through the principal's own tunnel outside
   ParleyInn, which ParleyInn sees as loopback; that phone is not a paired phone
   (§6.3).
2. The principal's other devices, as enrolled, mutually authenticated
   peers and paired phones (B2).

I4 binds what ParleyInn's components send, not what the principal's own
tools carry: its tunnel, its `git push`, its copies. The room-view
component listens only on loopback (B1), and the peer component
connects only to enrolled peers and paired phones (B2). Whether the
room view may reach browsers on other machines, or be served from a
host the principal names, is an open question: each would need new I4
wording, approved by ADR with a security review, before any
requirement asks for it (OQ-38).

A ParleyInn feature whose purpose is to reach off the machine through a tool the
principal runs outside ParleyInn, such as its own tunnel, has its own §6.3 row,
is
off by default and is turned on by a widening principal act (SEC-19). A commit's
room trailer and any trailer links are not such a feature: they name the room in
the repository's own commits, always on, and leave only with the principal's own
push (LANE-28, §6.3).
