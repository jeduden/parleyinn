# Backward trace: the SRS against the pitch

Invariant wording quoted below is historical; the accepted text
is in [§1.3](../../docs/srs/invariants.md).

This file walks the SRS (version 1.4-draft) from the specification's
side and asks where the pitch of 3 October 2026 and the stakeholder's
decisions in [pitch.md](pitch.md) fail to match it. It feeds task 2 of
[plan 2610012322](plan.md), the SRS change. Nothing here is normative.

Also read: the plans [2610012322](plan.md) and
[2610022338](../2610022338_parinn-network-side/plan.md).

Conventions:

- **Quote** is exact SRS or pitch text. Table cells are quoted cell by
  cell, joined with " | ".
- **INVARIANT CHANGE** marks a proposal that changes I1–I10. CLAUDE.md
  requires a security review and a new major version for each. No
  version has shipped, so in practice it means the change lands before
  v1.0 under a named security review, not as an edit to the draft.
- New ids take the next free number and avoid the ids other open pull
  requests hold (ASM-10 to ASM-16, PRV-08, OQ-10 to OQ-13, T10 to
  T13, the CUE family). Proposed here: I11, REC-17 to REC-19, INJ-10,
  PRV-09, SEC-19 to SEC-21, T14 to T16, OQ-14 to OQ-16, ASM-17,
  NFR-15, and a new VIEW family for the lane view. ASM-10, OQ-10,
  OQ-11 and T10 to T12 already exist in the SRS and are only quoted.

## 1. Value the SRS delivers that the pitch leaves out

The pitch moved from the agent to the person watching it. Most of what
the SRS builds in M1 to M3 serves the agent inside its own session,
and the pitch now says almost nothing about that.

### V1. Constraints restored verbatim after compaction

- **SRS:** I3; PIN-01, PIN-03, PIN-08, INJ-01; §3 item 2.
- **Quote:** I3, "Pinned constraints are stored verbatim and
  re-injected verbatim after every compaction." §3 item 2: "Claude
  Code's `/compact` on Sonnet 4.6 kept 53% of safety rules after one
  round and 10% after five."
- **Finding:** the pitch never mentions pins, compaction or standing
  rules. Yet "Only you can instruct your agents" is hollow over a
  multi-day run if your instructions vanish at the third compaction.
  Pins are the one channel by which the owner's words reach the agent
  automatically, and PIN-01 already limits it to trusted actions.
- **Verdict:** core value, and the strongest measured result in §3.
- **Proposed change (pitch):** after "Only you can instruct your
  agents", add: "Your standing rules are pinned: they come back word
  for word after every compaction, and nothing else is pushed into an
  agent's context."

### V2. Exact recall by the agent itself

- **SRS:** §1.2, I1; RCL-01, RCL-03, LMK-05, INJ-01.
- **Quote:** RCL-03, "`expand` (by `seq` range) and `get` (by `seq`)
  MUST return exact post-redaction content, resolving payload
  references, subject to a per-call token cap (default 8,000†) with a
  continuation cursor."
- **Finding:** the pitch's record is read by people ("One view shows
  each agent"). The SRS's record is read first by Claude, through the
  MCP tools and the restore block's landmark index. The pitch drops
  the agent as a reader entirely, though every P0 in §5.4 to §5.6
  serves it and the M1 exit criterion is "Recall ≥ 95% at 1–3
  compactions".
- **Verdict:** core value. "Know exactly what your agents did" should
  include the agents knowing it.
- **Proposed change (pitch):** add to the first paragraph: "Your
  agents can look it up too: after compaction they recall the exact
  message, command or result they need, on request, by its address."

### V3. Quarantine without destroying evidence

- **SRS:** I5; SEC-12, RCL-06, LMK-04; appendix A, B2.
- **Quote:** I5, "Any event, span, session, or derived artifact can be
  quarantined from recall immediately, while the record stays intact
  for forensics." B2: "Exposure accumulates; a lossless store never
  forgets poison".
- **Finding:** the pitch leads with a full, tamper-evident record and
  with peers. A record that keeps everything and is shared keeps
  poison too; quarantine is the SRS's answer. The pitch's security
  sentence covers only who may instruct, not how bad data leaves
  circulation. In a lane, quarantine also has to say whose node it
  binds: a local quarantine cannot reach a peer's copy.
- **Verdict:** core to the security lead, one clause long.
- **Proposed change (pitch):** "Pull bad data out of circulation in
  one step; the evidence stays in the record."
- **Proposed change (SRS):** SEC-12 gains: "Quarantine MUST bind the
  node that records it. A quarantine event MAY be shared with peers as
  an `operator` event of its writer; a peer MUST apply it only to
  that writer's own events, or by its own operator's decision."

### V4. Redaction before storage

- **SRS:** I1, SEC-08, ADR-09.
- **Quote:** SEC-08, "Redaction MUST apply gitleaks-compatible rules
  plus tenant-defined patterns to every text field, payload, and hook
  input before anything is written." I1: "The only exceptions are
  secrets removed by redaction before storage, and data an operator
  explicitly purges or expires by policy."
- **Finding:** "the full, tamper-evident record of each lane of work
  (every message, edit, tool run and result)" overstates I1, which is
  lossless only after redaction. Once segments travel to peers,
  redaction before storage is also what keeps a secret on one machine.
- **Verdict:** belongs in the pitch as a qualifier; it is both honest
  and a selling point against Delta's "Cloudflare-managed keys".
- **Proposed change (pitch):** "keeps the full record … with secrets
  redacted before anything is written".

### V5. Provenance, trust classes and the envelope

- **SRS:** I2; PRV-01 to PRV-06, RCL-04, SEC-06, INJ-03; §6.1 actors.
- **Quote:** PRV-03, "`assistant` and `tool_call` MUST be untrusted,
  because the model can reproduce injected content." Actors:
  "External content author | Controls text Claude reads: web pages,
  repositories, issues, dependency READMEs, file names | Yes — primary
  threat".
- **Finding:** the pitch names other people as the untrusted party.
  The SRS's primary threat is content: web pages, tool output, MCP
  results, file names and the agent's own earlier text. A reader of
  the pitch would think co-authors are the risk and tool output is
  safe. I2 also requires pull-only delivery, not just a label (see C4).
- **Verdict:** core; the pitch's security sentence should carry it.
- **Proposed change (pitch):** replace "everyone else's words reach
  them as untrusted data" with "everything else (other people, web
  pages, tool output, even an agent's own earlier words) reaches an
  agent only when it asks, labelled as untrusted data."

### V6. Completeness you can check, and no silent failures

- **SRS:** I6; REC-08, ADM-09, OPS-01 to OPS-03; appendix A, A7.
- **Quote:** REC-08, "The record MUST NOT depend on source files after
  ingestion. `parinn verify` MUST report any source that disappeared
  before it was fully ingested." A7: "Silent failures dropped 21,279
  events unnoticed".
- **Finding:** "know exactly" is only as good as knowing what is
  missing. The SRS can prove completeness and fail loudly; the pitch
  does not say so.
- **Verdict:** core, as a half sentence; it backs the headline.
- **Proposed change (pitch):** "… and tells you when anything is
  missing."

### V7. Detail that rightly stays out

- **SRS:** I9 (NFR-01, NFR-06), I7 (ADM-01 to ADM-03, NFR-12), I10
  (INJ-06, ADM-08), NG2 and MEM-01, the kernel (CMP-01 to CMP-07),
  NFR-03 to NFR-05.
- **Quote:** I9, "A ParleyInn failure never blocks or slows the agent
  beyond defined budgets." MEM-01: "ParleyInn MUST NOT promote record
  content into any long-term memory, summary store, or cross-project
  store."
- **Finding:** fail-open hooks, install hygiene, determinism, no memory
  promotion, the kernel and the performance targets are engineering
  promises. NFR-05's "≥ 50 concurrent sessions and subagents writing"
  backs "in parallel". The pitch must not contradict MEM-01 or NG7:
  "what it produced" must not become LLM summaries or extracted facts.
- **Verdict:** stays out of the headline. "If ParleyInn fails, your agents
  carry on" (I9) is worth a line of supporting copy, since the UI
  server and the peer are new ways to slow a harness.

## 2. SRS content the pitch or the decisions contradict or obsolete

### C1. The intent statement

- **SRS:** §1.1, §1.2; the index's **Product** line; the CLAUDE.md
  project line (a CODEOWNERS path).
- **Quote:** §1.2, "ParleyInn gives long-running Claude agents
  **unbounded, exact recall without creating a new attack surface**."
- **Finding:** the pitch's promise is "know exactly what your agents
  did, and work with them live". Two conflicts. First, the subject:
  the SRS serves the agent, the pitch serves the person. Second,
  "without creating a new attack surface" cannot hold once a loopback
  UI and an opt-in peer exist; both are new surfaces.
- **Proposed change (SRS, §1.2, first paragraph), new wording:**
  "ParleyInn keeps the complete, tamper-evident record of what long-running
  Claude agents did, one lane of work at a time, on the user's own
  machines. It gives the agents exact recall of it and the people
  working with them a live view of it, without opening any new path
  from untrusted content to an agent. Every surface beyond the local
  process is opt-in, bounded and reviewed." The second paragraph
  ("Where usefulness and safety conflict …") stays as it is.
- **Proposed change (SRS, §1.1):** add "and lets the people who run
  them see and steer that work live" after "remote agents running for
  hours or days".

### C2. I4, SEC-01, CON-04 against the loopback UI and peering

- **SRS:** I4; SEC-01, CON-04, ENG-12, ENG-16, SEC-15; §2.2.
- **Quote:** I4, "No listening sockets, no outbound connections, no
  telemetry. Data leaves the machine only when Claude receives
  recalled content through a tool call, and then travels to the model
  provider like any other context." SEC-01: "Shipped binaries MUST NOT
  open listening sockets or make outbound network connections." CON-04:
  "No network access, at build-verified level, in any binary that
  ships P0 features (SEC-01, ENG-12)."
- **Finding:** decision 9 needs a listening socket on loopback;
  decisions 4 and 5 need outbound and inbound connections to peers.
  Both break I4 as written, not only SEC-01. The pitch's "It runs
  standalone with no network" is also false under the SRS's own
  meaning, since a loopback listener is a listening socket. Plan
  2610022338 already draws the right line (B0 to B3); the SRS has no
  word for it.
- **INVARIANT CHANGE (I4).** Proposed new I4 row:
  "**The core never talks to the network** | The core (hooks, the MCP
  server, the kernel worker and everything that builds what reaches
  the model) opens no socket and makes no connection, and never sends
  telemetry. Without the user's explicit opt-in, no ParleyInn process
  sends data off the machine. Data reaches the model provider only
  when Claude receives recalled content through a tool call."
- **INVARIANT CHANGE (new I11).** "**Network boundaries are explicit
  and opt-in** | Every ParleyInn process sits behind exactly one boundary:
  process, machine (loopback), peer or public. A wider boundary is
  never on by default, each is enforced by a test, and nothing that
  crosses one reaches an agent except as untrusted, pull-only data."
- **Proposed change (SRS, SEC-01), new wording:** "The core MUST NOT
  open listening sockets or make outbound network connections. The UI
  server MUST bind only to loopback addresses and MUST make no outbound
  connection. Only the peer process MAY open network connections, only
  after the tenant enables peering, and only to enrolled peers. CI MUST
  enforce each boundary with an import allow-list per binary and a
  test run under a sandbox that denies all network, or all but
  loopback for the UI server."
- **Proposed change (SRS, CON-04):** "No network access, at
  build-verified level, in the core; every other binary is confined to
  the boundary SEC-01 assigns it (I11)."
- **Proposed change (SRS, ENG-12):** "… MUST run inside a
  network-denied sandbox, loopback-only for the UI server, and fail on
  any socket outside the boundary SEC-01 assigns."
- **Unchanged:** SEC-15 (no telemetry, no update checks) holds as is
  and the pitch agrees with it.
- **Alternative:** ship the network side as a separate product with
  its own SRS. That keeps I4 but puts the UI, which is on the
  standalone path, outside ParleyInn. Not recommended.

### C3. ADR-01, appendix A4 and NFR-09 against a UI server and a peer

- **SRS:** ADR-01; appendix A, A4 and A5; NFR-09; §3 item 6.
- **Quote:** ADR-01 rationale, "Removes the port, token, stale-daemon,
  and wedged-daemon failure classes observed in lcm". A4: "Isolation
  only per OS user; fixed loopback port; token file readable by
  same-user processes" | "No sockets and no tokens; ownership and
  permission checks; tenant binding". NFR-09: "No resident process
  between sessions."
- **Finding:** the loopback UI with "a token per launch" is the design
  A4 rejected, and a live peer is a resident process. The pitch is
  right to want both; the SRS must say why lcm's failures do not
  return. The browser adds an actor §6.1 lacks: any web page open in
  the user's browser can reach loopback (DNS rebinding, cross-site
  requests).
- **Proposed change (SRS, ADR-01):** "**Daemonless core.** Hooks and
  per-session MCP processes open the store directly. The UI server and
  the peer process are optional, started by the user, never on a hook
  or recall path, and their failure never touches a session (I9)."
- **Proposed change (SRS, NFR-09):** "The core MUST leave no resident
  process between sessions. The UI server and the peer process run
  only while the user runs them."
- **Proposed change (SRS, new T14):** "Local web attack on the UI |
  A page in the user's browser reaches the loopback port through DNS
  rebinding, cross-site requests or a leaked launch token | Loopback
  bind, a token per launch, Host and Origin checks, no cross-origin
  access (SEC-19)".
- **Proposed change (SRS, new SEC-19, P0):** "The UI server MUST bind
  only to loopback, MUST require a fresh token per launch, MUST reject
  requests whose Host or Origin is not its own loopback address, and
  MUST allow no cross-origin access."

### C4. I2 against working a lane live with others

- **SRS:** I2; INJ-03, INJ-04, PRV-02, PRV-05; §9.1 `UserPromptSubmit`.
- **Quote:** I2, "Content that originates outside the trusted boundary
  (tool output, web, MCP servers, files, assistant text) reaches the
  model only when Claude explicitly calls a recall tool, and always
  inside an untrusted-data envelope." Pitch: "Turn on peer to peer to
  work a lane live with others … Only you can instruct your agents;
  everyone else's words reach them as untrusted data."
- **Finding:** I2 has two halves, the label and the pull. The pitch
  keeps the label and drops the pull. If a co-author's chat message is
  delivered into an agent's session as it arrives, that is an
  automatic path, enveloped or not, and breaks I2. Decision 8's
  harnesses that "read and write the lane from inside the session" are
  fine only if reading is a recall call.
- **Proposed change (pitch gives way):** "everyone else's words reach
  them only when they ask, as untrusted data." The SRS keeps I2.
- **Proposed change (SRS, new INJ-10, P1):** "When untrusted lane
  messages are waiting for a session, ParleyInn MAY tell it so through
  `TrustedText` holding only their count and sanitized writer names;
  their content MUST reach the session only through a recall tool."
  This lets a live lane feel live without a push path.

### C5. "Only you can instruct your agents" against the trust policy

- **SRS:** PRV-02, PRV-04, PRV-05, PIN-01; §6.1 actors.
- **Quote:** PRV-02, "The default trust policy MUST classify as
  `trusted` only: `operator`; `harness_meta` (structural lifecycle data
  such as timestamps, token counts, session start and end); and `user`
  when the deployment mode is `interactive`." PRV-04: "Deployment mode
  MUST be `automation` by default, in which `user` is untrusted".
  PRV-05: "Configuration MUST NOT be able to classify any provenance
  class as trusted beyond PRV-02."
- **Finding:** three gaps. (1) In the default `automation` mode, which
  fleets and sandboxes use, not even the owner's prompts are trusted
  by ParleyInn. (2) ParleyInn governs only its own paths; an agent that
  fetches a web page inside the harness reads it with no ParleyInn in
  between, so "only you can instruct" promises more than I2 covers.
  (3) A message the owner types into the UI is not harness `user`
  provenance; no class exists for it, and PRV-05 forbids trusting a
  new one. The pitch's open question ("a signed endorsement or a
  granted role") lands here: a granted role widens the trusted
  boundary and needs a PRV-05 change under I2 review; an endorsement
  signed in the owner's own session does not.
- **Proposed change (pitch):** "ParleyInn never hands your agents anyone
  else's words as instructions." This is what ParleyInn can guarantee.
- **Proposed change (SRS, new PRV-09, P0):** "Every event imported
  from another writer MUST carry its writer key beside its one class
  (amending PRV-01) and MUST be `untrusted`, whatever its class or
  its writer's trust policy. An owner message
  from the UI MUST enter as `operator` only when the UI server
  authenticates the local tenant (SEC-19); otherwise it is untrusted."
- **Proposed change (SRS, new OQ-14):** "May another person's message
  reach an agent as an instruction? Options: (a) never; (b) only when
  the owner endorses it in their own session, signed with the owner's
  key; (c) a granted role, which changes PRV-05 and needs I2 review.
  | Plan 2610012322 phase 1; security review".

### C6. NG5, no graphical UI

- **SRS:** NG5; the Working view glossary entry.
- **Quote:** NG5, "Graphical UI | CLI and MCP only". Glossary:
  "**Working view** | Whatever is currently in the model's context
  window. A projection; never the source of truth."
- **Finding:** decision 8 lifts NG5. "One view" in the pitch also
  collides with the glossary's "Working view", which means the
  model's context, not a screen.
- **Proposed change (SRS, NG5):** "A UI that is required, or reachable
  from off the machine | The UI is one more client of the local store;
  harnesses work without it, and it binds to loopback only (SEC-19)".
- **Proposed change (SRS, glossary):** add "**Lane view** | The local
  web page that shows a lane's harnesses live, their results and the
  other harnesses. A client of the record; never the source of truth."

### C7. NG4 and OQ-07 against peer to peer

- **SRS:** NG4, OQ-07; appendix A, B5; §6.1 actors.
- **Quote:** NG4, "Multi-host shared stores | Multiplies blast radius;
  requires provenance-gated sharing design first (OQ-07)". OQ-07:
  "Design for shared stores across hosts: per-tenant row security,
  provenance-gated sharing, blast-radius limits." | "Post-v1
  proposal".
- **Finding:** two parts. The concept is obsolete: plan 2610012322
  does not share a store, it replicates signed per-writer logs, so
  "row security" no longer fits. The timing still holds: I2, I4 and I8
  all change at peering, and decision 6 puts the experience before
  seamless sync.
- **Recommendation:** the pitch gives way on timing, not on intent.
  Keep peering out of v1.0 and specify it as P2 now, which is what P2
  means ("later release, specified now so v1 does not preclude it").
  Build the per-writer record in v1 so peering needs no migration.
- **Proposed change (SRS, NG4):** "Peering in v1.0 | Peer exchange of
  signed per-writer logs is specified as P2 (I11, PRV-09, SEC-20) so
  v1 does not preclude it, and ships after its own security review".
- **Proposed change (SRS, OQ-07):** resolution path becomes "Resolved
  in direction by plan 2610012322: no shared store; signed per-writer
  logs, imported untrusted (PRV-09). Closes with the P2 peering
  requirements."
- **Proposed change (SRS, new T15):** "Malicious peer or co-author |
  An enrolled peer, or a co-author's node, sends forged, replayed or
  poisoned segments | Enrollment by key, signature and chain checks,
  refusal audited (SEC-20); imported untrusted (PRV-09); quarantine
  (SEC-12)".
- **Proposed change (SRS, new SEC-20, P2):** "The peer process MUST
  accept segments only from enrolled writer keys, MUST verify each
  segment's signature and chain before import, and MUST refuse and
  audit any segment that fails (I6)."
- **Proposed change (pitch):** "Turn on peer to peer" becomes "Next:
  turn on peer to peer …", until the P2 requirements are met.

### C8. I8 and the tenant against a node holding other writers' logs

- **SRS:** I8; Tenant glossary entry; SEC-02, RCL-05; §6.1 actors.
- **Quote:** I8, "All state is bound to one tenant's home with strict
  permissions. ParleyInn refuses to operate on state it does not own."
  Tenant: "The security principal that owns a ParleyInn home: one OS user,
  optionally bound to a tenant ID supplied by the runner environment."
  RCL-05: "Recall scope MUST default to the current project.
  Cross-project recall MUST NOT be possible in v1."
- **Finding:** a node in a lane holds logs other tenants wrote. Read
  literally, "refuses to operate on state it does not own" forbids
  that. The local files are still owned by the local UID, so SEC-02
  survives; the invariant's wording does not. The threat model's
  "Other tenant on a shared host" also misses tenants on other hosts.
- **INVARIANT CHANGE (I8).** New meaning: "All local state is bound to
  one tenant's home with strict permissions, and ParleyInn refuses to
  operate on a home it does not own. Content another tenant wrote is
  held as theirs: attributed to its writer's key, untrusted, and never
  a way to widen what this tenant's agents trust or recall."
- **Proposed change (SRS, glossary):** add "**Writer** | One origin
  that appends to its own log: a node and a session, identified by a
  key." and "**Owner** | The tenant whose local session started a
  lane; the only principal whose words may become trusted."
- **Proposed change (SRS, RCL-05):** "Recall scope MUST default to the
  current session. Widening to the current lane MUST require an
  explicit parameter and MUST be logged. Recall across lanes MUST NOT
  be possible in v1."

### C9. Project keying by path against a lane across worktrees

- **SRS:** §8.1; Project glossary entry; §8.2 `meta`.
- **Quote:** §8.1, "project-id = hex(SHA-256(canonical project
  path))[:32]" and "Directory names MUST NOT reveal project paths
  (N)." Project: "A working directory as identified by Claude Code
  (one transcript directory). Each project has its own store."
- **Finding:** worktrees of one clone have different paths, so they
  get different stores; the lane spans them, and spans machines whose
  paths differ again. Separately, an unsalted SHA-256 of a path
  confirms a guessed path, so the (N) rule on names holds only
  against a reader who does not guess.
- **Proposed change (SRS, §8.1):** "project-id = hex(HMAC-SHA-256(
  tenant key, repository identity))[:32], where repository identity is
  the clone's common git directory, or the canonical path outside git
  (N). A lane's id is chosen at its creation, carried by every event,
  and independent of any path (N)."
- **Proposed change (SRS, glossary):** Project becomes "A repository
  as identified by its common git directory, or a working directory
  outside git. Its worktrees share one project." Add "**Lane** | A
  branch with its worktrees, the sessions and people working on it,
  and their conversations, edits and results; the unit ParleyInn records,
  shows and shares. A session belongs to exactly one lane."
- **Session stays.** The pitch does not obsolete "Session"; a lane
  holds sessions. Only the Record entry changes (C10).
- **New ASM-17:** "Worktrees of one clone are distinguishable by
  transcript directory and share one common git directory | git
  worktree docs | Plan 2610012322 phase 1 | §8.1".

### C10. One gap-free seq and one hash chain per project

- **SRS:** REC-06, REC-10, §8.2, §8.3, glossary seq and Record; I1,
  I10; §9.2 `expand`.
- **Quote:** REC-06, "Every event MUST receive a `seq` that is strictly
  increasing, gap-free, and never reused within its project, assigned
  inside the append transaction." REC-10: "Events MUST form a
  per-project hash chain: each event stores `SHA-256(prev_hash ‖
  canonical_encoding(event))`, where the canonical encoding includes
  the payload hash." seq: "An event's address: a strictly increasing
  integer, unique within a project, never reused." Record: "The
  append-only log of events for a project. The source of truth."
- **Finding:** one writer per log (pitch: "Signed, hash-chained log
  per writer") cannot share one gap-free counter or one chain across
  nodes. A node-local `seq` is not stable across nodes, so as an
  address it breaks I1's "stable address" for anyone else. I10's
  rebuild must also be deterministic over whatever set of logs a node
  holds, in any arrival order (phase 1's gate).
- **Proposed change (SRS, REC-06):** "Every event MUST carry an
  address (writer, writer `seq`), where writer `seq` is strictly
  increasing, gap-free and never reused within that writer's log. A
  node MAY keep a local position for indexing; it is not an address."
- **Proposed change (SRS, REC-10):** "Each writer's events MUST form a
  hash chain … , and each event MUST name the heads of the other logs
  it saw, so order across writers is causal, not wall-clock."
- **Proposed change (SRS, glossary):** seq becomes "An event's
  position in its writer's log; with the writer it forms the event's
  address." Record becomes "The append-only set of writers' logs for a
  lane. The source of truth."
- **Proposed change (SRS, §9.2):** `expand` takes a writer and a
  writer `seq` range, or a span id; "Span" in the glossary may cross
  writers.
- **I1 and I10 wording holds** if "address" and "record" are redefined
  as above; no invariant change needed here. Record it in the SRS
  change so the security review sees it.

### C11. Unsigned chains, SEC-10 and "tamper-evident, signed"

- **SRS:** REC-10, SEC-10, OPS-02; §6.1 actors.
- **Quote:** SEC-10, "The v1 core MUST handle no credentials." Actors:
  "Process running as the same OS user | Can read the tenant's files
  directly | **No** — equivalent to the tenant; out of scope". Pitch:
  "Signed, hash-chained log per writer".
- **Finding:** the SRS has no signing anywhere in the record. An
  unkeyed chain stored next to its content shows accidental damage,
  but anyone who can rewrite the file can rewrite the chain. That is
  acceptable locally, where the same user is out of scope, and not
  across peers. Signing needs a key, and a signing key is a credential
  SEC-10 forbids in the v1 core.
- **Proposed change (SRS, new REC-17):** "Each writer MUST sign its
  log's checkpoints with its writer key; a checkpoint covers the
  writer `seq` and the chain head." P1 standalone, P0 once peering
  ships.
- **Proposed change (SRS, SEC-10):** "The v1 core MUST handle no
  credentials other than writer signing keys. A writer key MUST be
  generated locally, stored `0600` or in the OS keychain, never leave
  the node, and never be logged …" (rest unchanged).
- **Proposed change (pitch):** keep "tamper-evident" only with
  "signed"; without REC-17 the claim should read "hash-chained".

### C12. Unkeyed content hashes, purge and "erasure by key"

- **SRS:** REC-09, §8.2 `tombstones`, ADM-07, SEC-08, SEC-09, SEC-14.
- **Quote:** REC-09, "… stored in the content-addressed payload store
  keyed by SHA-256 …". §8.2 tombstones: "purge event `seq`, purged
  range, reason, hash of removed content". SEC-14: "Documentation MUST
  state that physical erasure on SSDs is not guaranteed without
  encryption at rest." SEC-09 (P1): "ParleyInn SHOULD support encryption
  at rest for the database and payload store, with keys obtained
  through a secret reference (SEC-10)."
- **Finding:** an unkeyed hash of purged content lets anyone holding
  the tombstone confirm a guess (SEC-08 already salts its markers for
  this reason). Shared with peers, every payload name leaks the same
  way. The pitch's "erasure by key" means crypto-shredding: purge by
  destroying a per-lane key. That needs encryption at rest, which is
  P1 and optional today, and it cannot reach copies a peer already
  decrypted.
- **Proposed change (SRS, REC-09):** "keyed by HMAC-SHA-256 under a
  per-lane key"; tombstones hold the keyed hash.
- **Proposed change (SRS, new SEC-21):** "Each lane's payloads MUST be
  encrypted under a per-lane key; destroying the key MUST make the
  lane's payloads unreadable on that node, and MUST be recorded as a
  tombstone." P0 if the pitch keeps "erasure by key"; then SEC-09
  rises to P0 as well.
- **Proposed change (pitch):** "erasure by key on your own nodes; a
  peer keeps what it already holds" in the Data control row.

### C13. Deployment context and stakeholders

- **SRS:** §2.1, §2.2; appendix A, D3.
- **Quote:** §2.2, "**Primary:** Claude Code sessions executed by
  self-hosted runners, and Claude Agent SDK workers, inside sandboxes
  we operate." and "Runner workspaces, `~/.claude/`, and `PARINN_HOME`
  live on durable volumes so sessions can pause and resume." §2.1:
  "Platform team (primary operator)".
- **Finding:** the pitch's user is a developer running a fleet across
  worktrees, machines and cloud sandboxes, working with the agents
  live. Claude Code on the web reclaims its sandboxes, so the durable
  volume assumption fails there. No stakeholder row covers a lane
  owner or a co-author.
- **Proposed change (SRS, §2.1):** add "Lane owner | Runs a fleet of
  agents across worktrees and machines; needs to see and steer them
  live and know what verified each result" and "Co-author | Works in
  another person's lane; reads it and writes to it, never instructs
  its agents".
- **Proposed change (SRS, §2.2):** primary becomes developer machines
  running several harnesses, plus self-hosted runners; ephemeral cloud
  sandboxes keep state only by moving their logs to a peer (P2).

### C14. "Every … edit" against transcript-only ingestion

- **SRS:** REC-01, REC-02; §1.4 layer 1.
- **Quote:** REC-01, "ParleyInn MUST ingest Claude Code session transcripts
  from the configured transcript roots (default
  `~/.claude/projects/`)." Pitch: "every message, edit, tool run and
  result".
- **Finding:** the SRS records what transcripts hold. Edits a person,
  a formatter or a shell makes outside an agent's tool calls never
  reach the record. Plan 2610012322 names worktree checkpoints as the
  fix; the SRS has no requirement for them.
- **Proposed change (SRS, new REC-18, P1):** "ParleyInn SHOULD record a
  checkpoint of each lane worktree's changes at session boundaries,
  as an event whose provenance is `file`, so edits outside tool calls
  are recoverable."
- **Proposed change (pitch, until REC-18 lands):** "every message,
  agent edit, tool run and result".

### C15. Decision 7, "good enough against Delta", against §1.2

- **SRS:** §1.2; §11.2 item 4.
- **Quote:** §1.2, "Where usefulness and safety conflict, ParleyInn chooses
  the design that keeps the agent safe and makes the convenience
  opt-in, never the reverse." Decision 7: "Security must be good
  enough, measured against Zed Delta."
- **Finding:** Delta documents no signing and no co-author trust
  model, so "good enough against Delta" is a far lower bar than I2 and
  §1.2 set. Applied to the core, it would license weakening it.
- **Recommendation:** the decision gives way for the core. Delta is
  the bar for the new surfaces' convenience and reach (transport,
  availability, partitions), never for I2, I3, I5 or I6. Record that
  as one sentence in the SRS change's §1.2.

### C16. Public lanes against ADM-12 and MEM-01

- **SRS:** ADM-12, MEM-01, MEM-02.
- **Quote:** ADM-12, "`parinn export --trusted-only` SHOULD write
  trusted events with full provenance as JSONL for downstream
  systems."
- **Finding:** plan 2610022338's B3 public bundles carry untrusted
  content to anyone, bypassing the only export rule the SRS has.
- **Proposed change (SRS, new OQ-15):** "Public lane bundles: which
  classes may leave, under what extra redaction, and who reviews? |
  Plan 2610022338 task 6; security review". Keep public lanes out of
  the pitch until then.

## 3. Priority mismatches

### P1. The live view the pitch leads with has no requirement

- **SRS:** NG5; §9.5 operator recall CLI; no VIEW family.
- **Quote:** pitch, "One view shows each agent as it works, what it
  produced, and what actually verified it." §9.5: "`parinn search` ·
  `parinn expand` · `parinn landmarks` | Operator recall (same envelope,
  same limits)".
- **Finding:** the pitch's second promise is a screen; the SRS's
  nearest P0 is a CLI.
- **Proposed change (SRS):** a new VIEW family, P0 for the standalone
  view if v1 is to match the pitch: VIEW-01 one view of all local
  harnesses, live; VIEW-02 a lane timeline of messages, edits as
  diffs, tool runs and results, by actor; VIEW-03 pending permission
  requests first; VIEW-04 the view shows every recalled or foreign
  item with its provenance and trust; VIEW-05 the view never writes
  to an agent except as SEC-19's authenticated owner. Add NFR-15:
  "An event appears in the lane view within 1 s† of its ingestion."

### P2. "Live" rests on a P1 SHOULD

- **SRS:** REC-13; §9.1.
- **Quote:** REC-13 (P1), "`PostToolUse` and `Stop` hooks SHOULD
  ingest the active transcript incrementally within their budget and
  leave a work marker for any remainder."
- **Finding:** without REC-13, ingestion happens at `PreCompact`,
  `SessionEnd` or by hand, and no view can be live. The pitch's
  headline depends on it.
- **Proposed change (SRS):** REC-13 to P0 and MUST, within NFR-01's
  100 ms budget.

### P3. "What actually verified it" has no requirement

- **SRS:** none; PRV-06 is the nearest.
- **Quote:** pitch, "what actually verified it"; dots lesson: "a
  completed run is not a verified one, so results show what checked
  them".
- **Finding:** the pitch leads with verification; the SRS records tool
  results but no notion of which check backs a claim. A badge built
  from tool output text would itself be forgeable by injected content,
  misleading the person, not the model.
- **Proposed change (SRS, new REC-19, P1):** "Each result ParleyInn
  shows MUST name its evidence class (claim, local run, CI), derived
  only from structural events such as an exit status or a signed check
  result, never from event text."
- **New T16:** "Forged verification | Untrusted output claims a check
  passed | Evidence class from structural events only".

### P4. Signing and peering: pitch headline, absent or post-v1

- **SRS:** REC-10 (P0, unsigned); NG4, OQ-07 ("Post-v1 proposal").
- **Finding:** the pitch puts "tamper-evident" in its first paragraph
  and "Turn on peer to peer" in the present tense; the SRS has no
  signing and excludes peering from v1.
- **Proposed change:** REC-17 as in C11; peering as P2 and the pitch
  says "next", as in C7.

### P5. Recall taint and approval gates: P1, but the pitch's control story

- **SRS:** SEC-13 (P1).
- **Quote:** SEC-13, "… SHOULD ship an example `PreToolUse` policy
  hook that requires approval for configured sensitive actions once
  the flag is set."
- **Finding:** dots' "rule levels per action" and "Only you can
  instruct your agents" both point here: once an agent has read
  someone else's words, its sensitive actions wait for the owner. The
  view already surfaces permission requests first.
- **Proposed change (SRS):** SEC-13 to P0 once co-author messages
  exist; P1 is fine for a standalone v1. Pitch may add "once an agent
  has read untrusted data, risky actions wait for you."

### P6. Encryption at rest: P1, but the pitch's data-control claim

- **SRS:** SEC-09 (P1). "Erasure by key" needs it (C12): raise SEC-09
  and SEC-21 to P0, or drop the claim from the pitch.

### P7. A whole milestone on the P1 kernel the pitch ignores

- **SRS:** §12.2 M4; CMP-01 to CMP-07 (P1).
- **Quote:** "**M4 — Kernel** | CMP P1 | Aggregation over 10M-token
  histories without prompt growth; limits enforced".
- **Finding:** the pitch never mentions the kernel, while the view and
  live ingestion have no milestone at all.
- **Proposed change (SRS, §12.2):** M1 "Record and recall" builds the
  per-writer log and index split (REC-06, REC-10 as in C10); M2 "Pins
  and restore" unchanged; M3 "Landmarks and lifecycle" adds REC-13 at
  P0; new M4 "Standalone lane view" (VIEW P0, SEC-19, NFR-15); M5
  "Hardening and evaluation" unchanged; new M6 "Kernel" (CMP, P1 slip
  to v1.1 with sign-off as §1 conventions allow); M7 "Peering and
  extensions" (P2: SEC-20, REC-17 at P0, public lanes). §11.2 item 5
  becomes "Two-week pilot on a fleet workflow, with the lane view in
  daily use, …"; §11.1 gains a row "What did my agent do: questions
  about a lane answered from the view alone".

### P8. P0 work the pitch ignores that must stay P0

- **SRS:** PIN, INJ, LMK, RCL, SEC-12 (all P0).
- **Finding:** these are the I2, I3 and I5 core and stay P0; the fix
  is in the pitch (V1 to V3). Landmarks can also structure the lane
  timeline, so the view reuses P0 work.

### P9. Harnesses beyond Claude

- **SRS:** NG6, ADR-08. Pitch table: "Any editor; Claude Code first".
- **Finding:** consistent today, since an editor is not a harness. It
  becomes a mismatch if the pitch promises "each agent" whatever its
  harness. New OQ-16: "Is ACP the adapter boundary for harnesses beyond
  Claude Code? | Plan 2610022338 phase 1".
