---
title: "Record"
order: "04"
summary: >-
  The record and what derives from it: writers, events, addresses, segments, seals, provenance and origin, spans and landmarks, quarantine and purge, receipts, backups, bundles and exports.
---
# Record

- **Record**: The set of writer logs a node holds (I10); the audit log,
  counters and **configuration** sit beside it: the principal's settings
  (§9.6), some changes to which apply only once accepted (ADM-04), and which a
  repository's `.parinn.toml`, its **repository configuration**, may only
  tighten (SEC-11). Managed policy, configuration and repository configuration
  are the three **settings layers**. The **store** is the home's files
  containing the record, its derived artifacts and payloads (§8.1).
- **Writer**: One seat's append-only log, written on one node or paired phone
  and held by any node, named by its **writer id**, derived from the seat's
  first key. Its events are hash-chained; the **chain head** at a seq is the
  hash over every event up to it.
- **Retire a writer**: Mark a writer retired at its last accepted seq, by a
  principal act of the principal its seat key chains to, or by the expire act
  ending the access token whose token key certified its seat key (PEER-05).
- **Event**: One immutable entry in a writer, such as a message, tool call,
  tool result, **hook observation** (what a hook reported), act, **recall
  event** (one recall, RCL-07), tombstone or unparsed line. Its **event kind**
  is which kind of entry it is, recorded apart from its provenance.
- **seq**: An event's position in its writer: strictly increasing, gap-free and
  never reused (REC-06).
- **Address**: (writer, seq), shown as short `A2·4812` (its writer label and
  seq), range `A2·4812–5025` or full `parinn:room/<room>/w/<writer>/<seq>`. One
  ASCII input form, `<label>:<seq>` or `<label>:<from>-<to>`, the writer id
  allowed in place of the label, is accepted with them (RCL-08). A **writer
  label** is a writer's short display alias, unique per node, each
  **writer-label assignment** a structural event (RCL-08). An address range
  lies in one writer; "range" on its own means an address range. The only
  meaning of "address".
- **Payload**: The full content of an event whose canonical encoding passes the
  payload threshold (`payload.threshold_bytes`, REC-09), stored in the store's
  **payload store** under a name that confirms no guess at its content. The
  event keeps its preview, its **payload reference** (the payload's name and
  size) and the event's commitment.
- **Preview**: The first bytes of an event's or post's content, at most 512 by
  default, cut at a character boundary (REC-09). It is what a payload event
  keeps in place of its content, and what a recall hit, the kernel or a view
  shows of any event or post (OWN-08). A preview is content: it carries its
  event's trust level, is no structural field, and goes with the content under
  purge (ADM-07).
- **Segment**: A range of one writer, sealed as a unit when it closes; what
  peers exchange. A unit of the record, not a storage layout: a merge (REC-25)
  or a purge (ADM-07) rewrites closed segments without changing any address.
  The **open segment** is a writer's newest, still growing, closing as REC-19
  says; peers exchange its sealed prefix. A closed segment or an open segment's
  sealed prefix is a **sealed range**. A refused segment is recorded as a
  structural event in this node's personal room (REC-21). Each segment carries
  a **format version** (REC-21), and the store a **schema version** (ADM-05,
  ADM-11). A node refuses to open a store or segment newer than it supports
  (ADM-05), and refuses and audits a segment of any version it does not support
  (REC-21).
- **Seal**: A seat key's signature over its writer id, a seq and the chain head
  at that seq (REC-18), made where that key lives (REC-19). Events after the
  newest seal are unsigned.
- **Prompt commitment**: Unlike an event's commitment, a commitment to the
  redacted text a `UserPromptSubmit` hook handler got, under its own random key
  kept only on this node, recorded with the hook input's `prompt_id` in a
  structural hook observation; the first transcript line carrying that
  `prompt_id` uses it up, and purge erases it with its key (PRV-08, SEC-31).
- **Commitment**: A keyed commitment to an event's content under a per-event
  random key, its **commitment key**, kept with the content and erased with it;
  the only way the chain, seals and tombstones refer to content, but for what a
  tombstone keeps (REC-17, Purge).
- **Provenance**: An event's class from a closed set, saying what produced its
  content (PRV-01): `user`, `assistant`, `tool_call`, `tool_result:<tool>`,
  `web`, `mcp:<server>`, `file`, `subagent_result`, `harness_meta`,
  `harness_text` (Turn), `operator`, `post`, `summary`, `structural` and
  `unparsed`. `harness_meta` is what the harness reports, or the launcher
  records, about the harness's operation, never free text: turn triggers, model
  tokens, metadata lines and sandbox state (PRV-08, OWN-22); only what the
  launcher records is trusted, never what a hook handler or a transcript line
  says.
  `operator` is the class of principal acts, expire acts and device-seat pins,
  and `assistant` of run-seat pins; posts are `post`, room summaries `summary`,
  and every other room act takes its seat's **pin class**, `operator` for a
  device seat and `assistant` for a run seat (PRV-01, OWN-02). REC-20, RCL-07,
  OWN-18 and PEER-11 class the events they name; PRV-01 lists the structural
  ones.
- **Origin**: Where and how an event entered this node's record (RCL-09),
  derived from the record and the node's key set alone, in this order (PRV-13):
  `bundle` (imported, as an arrival event names it, whatever other one does);
  else `peer` (received from a peer, paired phone or backup restore, as an
  arrival event names it); else `ingested` (in the writer of an ingest seat);
  else `witnessed`; else `peer` (such as a writer whose seat key a key that a
  node identity change replaced certified, Node, REC-24). Only arrival events,
  ingest seats and writers whose seats a key of this node's own certified count
  (Key event). A quarantine never changes it; the trust policy reads `bundle`
  as `peer` (PRV-02); origin is independent of provenance. An **arrival event**
  is the structural event this node's device seat in its personal room records
  the first time it accepts a sealed range by a way, naming the writer, the seq
  range, a digest of the range's chained headers, never of its content, and how
  it arrived. A **witnessed run** is one this node's
  capture recorded live. Every event but one from a peer or a bundle also
  records its **recorder**, the part that recorded it: the hook handlers, CLI,
  TUI, MCP server, launcher, room-view component or bridge component.
- **Span**: A contiguous range of one run's events in one writer, bounded as
  LMK-01 says.
- **Landmark**: A structural headline over one span, bound to its address range
  (LMK-02). A **natural-language headline** is one LMK-06 may add to a span
  whose every event is trusted; it is no structural field and never reaches a
  restore block. Landmarks roll up into **tiers** of **landmark blocks**, each
  a sequence of landmarks; a full tier collapses its older ones to one line
  each, listing one address range per writer (LMK-05).
- **Structural field**: A field ParleyInn derives from an event's shape, never
from
  its text: ids, kinds, counts, tool names, key fingerprints, addresses, version
  numbers, paths and exit status. Sanitized under LMK-03, it is trusted
  whatever the event's trust level (PRV-06).
- **Structural event**: An event of provenance `structural`, whose meaning lies
  only in structural fields. A hook observation keeps only a hook's structural
  fields; content a hook carries is recorded under its own class. An act's
  structural fields count as structural fields, never the text it carries; the
  act keeps its own class.
- **Capture**: This node's MCP servers and hook handlers recording its runs'
  events; turning it off is widening (OWN-11, ADM-02).
- **Ingest marker**: What ingest leaves when it stops short of a transcript's
  end, so the next MCP server or `parinn ingest` resumes it (REC-13); a hook
  handler holds its observation there, where its hook fired, for its MCP server
  to append (REC-19). A **start marker** is what a hook handler
  or MCP server leaves as it starts and clears as it finishes; one left behind
  is counted and audited (NFR-02), an MCP server's with each transcript it left
  short (REC-13). **Ingest** reads a transcript into the record: the MCP
  server's **live ingest** as it grows (REC-13), or `parinn ingest` (REC-22). A
  line a harness fork repeats is recorded once, in the writer of the run it
  belongs to; every other run that repeats it records a **repeat reference**
  to that event's address, never a copy (REC-04). The
  run it belongs to is the one no **fork observation**, a `SessionStart` hook
  observation with `source` = `fork`, names; with none, the run that first
  recorded it, as a structural **repeat owner** event in the device seat
  records. A fork's line read before its owner's is **held** at an ingest
  marker until its owner's transcript is ingested or gone, or 30 s pass. An
  **ingest position** is how far ingest read a transcript source (REC-03).
- **Worktree checkpoint**: An event recording a worktree's commit, branch and
  redacted diff since the previous worktree checkpoint (REC-20).
- **Derived artifact**: Anything computed from the record and the node's key
  set, such as those I10 lists, the **search index** over event text among them.
  A derived artifact is its content, never its storage layout: two copies are
  the same when their **canonical digest**, a digest of their content, schema
  and the stored settings that decide what a read returns, that no storage
  layout, insertion history or arrival order affects (§8.3), matches
  (ADM-08). Unqualified, "content" is an event's content; a derived artifact's
  content is always said of the artifact. The **quarantine set** is what a node
  holds quarantined.
- **Redaction**: Removing secrets from content before it is stored, on import,
  or from what an export's, a publish's or an invite's review step sends,
  recorded (I1, SEC-08). Three kinds of **redaction rule** apply: the
  **redaction rule set**, ParleyInn's maintained and versioned rules for known
  secret shapes; the **harness account identifiers**, the account email and
  organization id a transcript's own harness fields name, learnt from each
  transcript; and the principal's **redaction patterns**
  (`redaction.extra_patterns`). Each match becomes a marker naming its rule.
- **Retention policy**: A rule (`retention_policy.*`), set by the node's
  principal (ADM-04) or managed policy, that purges content per room and
  provenance class after a time (I1, SEC-22).
- **Flag**: A mark ParleyInn sets on an event whose text matches an injection
  pattern (PRV-07); flagged text contributes only counts to landmarks (LMK-04).
- **Quarantine**: Recorded, reversible exclusion of an event, span, run, writer
  or derived artifact from every path that carries content to an agent and from
  trusted-only exports (SEC-12), on the node that records it or carries it over
  (ADM-06), without deletion (I5).
- **Purge**: Deletion of content, appending a **tombstone** naming the purged
  range, which views show in its place (ADM-07): by a principal act, or by the
  node under a retention policy (REC-15), which ages events by **time marks**,
  structural events the node records at intervals. The only way stored content
  is destroyed (I1), never undoing an act's effect (Relations); no purge scope
  covers an arrival event, node identity change event, key event or certificate
  record (ADM-07).
- **Gap marker**: What stands where content is missing: a tombstone for a purged
  range, a **quarantine marker** for a quarantined address, or a **truncation
  marker** on capped kernel output. A **missing range** is part of a writer this
  node does not hold, such as a peer's lost tail; a **capture gap** is part of a
  run its capture did not record.
- **Integrity status**: What a room or writer shows about its chain and seals,
  one of §9.7.5's values (VIEW-10); `refused` shows a refused segment
  (Segment). The UI never says "secure".
- **Receipt**: A signed statement about a node's record, made to be kept apart
  from the home and checked with no network. Always qualified: a **head
  receipt** lists every writer's chain head at a moment, the tamper evidence of
  VIEW-10 and SEC-27; a **purge receipt** states what a purge erased and what it
  could not (SEC-31).
- **Backup**: A copy of a home's store and audit log, with none of its keys
  (`parinn backup create`, ADM-06, SEC-10); reading it back is a backup
  restore (Node). Creating one records no principal act, only an audited event
  (§9.5), and a copy the principal takes off the machine is its own tool,
  outside ParleyInn's components (I4).
- **At-rest key**: The key that encrypts the store when encryption at rest is on
  (SEC-09).
- **Bundle**: A reviewed export of a room, signed by the device key of the
  device that exported it, its **exporter**, which chains to the **bundle's
  principal key**, carried as a file or a git ref.
- **Trusted-only export**: The JSONL of the events trusted for the principal's
  agents, with full provenance, that downstream memory systems read (ADM-12,
  MEM-02); an export like a bundle (SEC-26).
- **Export**: Writing a bundle, a rendering or a trusted-only export (`parinn
  export`), a widening principal act under SEC-26. To **publish** is to serve a
  room's bundle read-only through the publish component, only as the room's
  visibility allows; it is widening, and stopping it cut (OWN-11).
- **Rendering**: A room rendered for people to read, with no keys or commitments
  (`parinn export --rendering`, SEC-26).
- **Import**: Reading a bundle into this node's record (`parinn import`,
REC-23).
  A transcript is ingested and a peer's segments are received; neither is
  imported.
- **Kernel**: The hermetic environment where an agent runs code over
  the record: SQL by default (CMP-01, ADR-04), Python only as CMP-08's
  opt-in. Each run's **kernel variables** (in the SQL kernel, the
  temporary tables and views its SQL creates) live in its own **kernel
  namespace** (CMP-02), reset when what it read is quarantined or purged
  (CMP-07).
