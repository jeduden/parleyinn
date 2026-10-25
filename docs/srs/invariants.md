---
title: "1.3 Invariants"
summary: >-
  The single source of ParleyInn's ten invariants, I1 to I10, which every
  requirement serves; other files include it through mdsmith.
---
# 1.3 Invariants

The invariants are ParleyInn's contract. Every requirement serves at least one
of them. A change that would break an invariant is a design change requiring
security review and a new major version, not a bug fix.

- **I1 — Nothing is lost.** Every event an agent saw or produced remains
  recoverable by its stable address, (writer, seq), across any number of
  compactions and agent runs, on every node that holds its writer's log. The
  only exceptions are secrets removed by redaction before storage or on import,
  and data the node's principal explicitly purges, or removes under a retention
  policy that principal or managed policy sets. Every exception is recorded.
- **I2 — No automatic path from untrusted content to the model.** Content that
  originates outside the trusted sources (tool output, web, MCP servers, files,
  the model's replies, and anything another node or principal produced, except
  the agent's principal's acts signed by a device key it certified, posts and
  pins written from a device seat such a key certified, a pin version that
  principal stamped, or what a trust grant of that principal covers) reaches the
  model only inside an untrusted-data envelope when the agent explicitly calls a
  recall tool, or through one of the closed paths below that a principal act
  names. The trusted sources are this node's own `operator` and structural
  events (hook observations only as PRV-08 allows), the `harness_meta` events
  its launcher recorded, and, while the deployment mode is `interactive` and no
  run on the node leaves R3 or R8 open, the `user` events matched to a prompt
  (PRV-08), and, once PRV-10 ships, principal acts signed by a device key the
  agent's principal certified and posts and pins written from a device seat such
  a key certified, within that key's scope, a pin version a principal stamped,
  for that principal's own agents, and the posts and pins a trust grant of the
  agent's principal covers. ParleyInn writes to an agent only through a closed
  set of paths. Without a principal act: restore blocks (INJ-01, INJ-02,
  INJ-04), built only from qualifying pins, trusted structural fields and fixed
  text ParleyInn ships; and opt-in notices (INJ-10) and the fixed templates of
  OWN-04 and OWN-07, each built only from fixed text ParleyInn ships, trusted
  structural fields and ids. On a principal act recorded at that time: through
  the harness's own input, only principal-typed text, a fixed template that
  references ids, or a post a principal endorsed exactly as shown inside the
  template of OWN-08. Under a delegation grant its principal recorded (OWN-23),
  and, for an agent of another principal, an acceptance grant that agent's
  principal recorded (OWN-26): a delegated task inside the fixed template of
  OWN-24. Once its requirements ship, a principal may also trust another
  principal by key for its own agents, in one room or everywhere; the pins that
  principal wrote from device seats one of its device keys certified then
  restore to those agents, and its posts reach them inside the fixed template of
  OWN-29. ParleyInn applies such grants and never grants trust itself. No other
  write to an agent exists.
- **I3 — Constraints are never summarized.** Every pin that restores is stored
  verbatim and restored verbatim after every compaction, or named by id and
  count when the budget or run ambiguity (INJ-05) omits it.
- **I4 — Each component stays inside one declared network boundary, and only the
  core builds what reaches the model.** B0, the core (hook handlers, the MCP
  server, the kernel worker, the CLI, the TUI and everything that builds what
  reaches the model), opens no socket and makes no outbound connection. B1,
  machine: a component may listen only on loopback or on a local endpoint only
  the same OS user can reach, and connect nowhere else; the launcher carries
  into the harness's input only text the core built. B2, peer: encrypted,
  mutually authenticated connections to nodes and paired phones the node's
  principal enrolled by key, off until that principal turns it on. B3, public:
  read-only publishing and outbound exchange with hosts the node's principal
  names, off until turned on. Managed policy can disable B1, B2 and B3. No
  component sends telemetry or depends on a central or third-party service.
  ParleyInn's components send data off the machine only as recalled content an
  agent receives through a tool call, or as what ParleyInn writes to an agent
  through I2's closed paths, both of which the harness sends to its model, or
  through a B2 or B3 component the node's principal turned on; whatever such a
  component brings in is trusted only as I2 allows.
- **I5 — Bad data can be removed from circulation without destroying the
  record.** Any event, span, run, writer or derived artifact can be quarantined
  from recall immediately on the node that records the quarantine, while the
  record stays intact for forensics. A quarantine reaches another node only as a
  quarantine request that node's principal applies.
- **I6 — No silent failures.** Every dropped, rejected, redacted, timed-out, or
  failed operation is counted, logged, and visible to the node's principal.
- **I7 — Harness configuration changes only on explicit instruction.** ParleyInn
  changes harness configuration only through explicit install and uninstall
  operations, shows the change first, and never overrides managed policy or the
  harness's managed settings.
- **I8 — Isolation follows the principal.** All local state is bound to one
  principal's home with strict permissions, and ParleyInn refuses to operate on
  a home that does not belong to the OS user running it. Content another
  principal wrote, or another node wrote other than this principal's acts signed
  by a device key it certified and its posts and pins written from a device seat
  such a key certified, is kept as theirs: attributed to its seat key, and
  untrusted unless this principal's own stamp or trust grant covers it. On its
  own it never makes this principal's agents trust it, nor extends their recall.
- **I9 — ParleyInn never degrades the agent.** A ParleyInn failure never blocks
  or slows the agent beyond defined budgets. ParleyInn fails open, except where
  continuing would violate I2, I4, or I8.
- **I10 — Everything derived is rebuildable.** All derived artifacts (the search
  index, landmarks, active pins, quarantine set, room state, origins, trust
  levels, run and integrity statuses, queues, evidence and proof classes, stats)
  are a deterministic function of the set of writer logs a node holds and the
  node's own key set, independent of the order in which logs arrived. Rebuilding
  reproduces their content exactly, whatever their storage layout.
