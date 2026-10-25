# Trace forward: ParleyInn's pitch to the SRS

Invariant wording quoted below is historical; the accepted text
is in [§1.3](../../docs/srs/invariants.md).

This file traces the pitch of 3 October 2026 and the stakeholder's ten
decisions in [pitch.md](pitch.md) into the SRS
([index](../../docs/srs/index.md), 1.4-draft), with detail from
[plan 2610012322](plan.md),
[plan 2610022338](../2610022338_parinn-network-side/plan.md) and its
[phase 1](../2610022338_parinn-network-side/phase-1.md). It is evidence
for this plan's SRS change, not normative text.

- **Status.** *covered*: a requirement makes the claim testable.
  *partial*: the gap is named. *contradicted*: an id, invariant or
  non-goal says otherwise. *missing*: no requirement exists.
- **Action.** (a) new requirement, (b) change existing text, (c) change
  the pitch: exactly one per claim that is not covered. Knock-on edits
  the action forces are listed with it.
- **Post-v1** marks a claim whose answer is a milestone after v1.0 in
  §12.

Ids take the next free number on disk. Not used, because other open
pull requests hold them: ASM-10..16, PRV-08, OQ-10..13, T10..13 and the
CUE family. Each new or changed id lands with its `@pending` scenario
in the same change (CLAUDE.md).

## Counts

| Status       | Claims |
| ------------ | ------ |
| covered      | 12     |
| partial      | 9      |
| contradicted | 8      |
| missing      | 22     |
| total        | 51     |

## Claim table

Source: **pitch** is the quoted pitch, **lane** the paragraph after
it, **D1**–**D10** the decisions, **table** the Delta comparison,
**open** the list "Open before the SRS change", and a plan id that
plan's text.

### Claims from the pitch

| #   | Claim                                                                   | Source    | Status       | Supporting ids and gap                                                                                                                | Action               |
| --- | ----------------------------------------------------------------------- | --------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------- | -------------------- |
| P1  | Every message is recorded                                               | pitch     | covered      | REC-01 "MUST ingest Claude Code session transcripts"; PRV-01 `user`, `assistant`; REC-05 "MUST NOT be dropped"                        | —                    |
| P2  | Every edit an agent makes through its tools is recorded                 | pitch     | covered      | PRV-01 `tool_call`; REC-09; RCL-03 "MUST return exact post-redaction content". Raw tool input, not a diff                             | —                    |
| P3  | Edits outside the agent's tools (person, formatter, shell) are recorded | pitch     | missing      | None: transcripts hold only what the harness logged, and nothing reads the worktree                                                   | (a) REC-17           |
| P4  | Every tool run is recorded                                              | pitch     | covered      | PRV-01 `tool_call`; REC-02 "link each to its parent session"                                                                          | —                    |
| P5  | Every result is recorded                                                | pitch     | covered      | PRV-01 `tool_result:<tool>`; REC-09; RCL-03                                                                                           | —                    |
| P6  | The record is tamper-evident                                            | pitch     | partial      | REC-10 "per-project hash chain"; ADM-09; OPS-02. Gap: unkeyed, unsigned, unanchored; SEC-10 "The v1 core MUST handle no credentials." | (a) REC-18           |
| P7  | The record stays on your own machines                                   | pitch     | covered      | I4 "No listening sockets, no outbound connections, no telemetry."; SEC-01; SEC-15; OPS-05                                             | —                    |
| P8  | One record spans agents on several machines                             | pitch     | contradicted | NG4 "Multi-host shared stores"; RCL-05 "Cross-project recall MUST NOT be possible in v1."                                             | (b) C8. Post-v1      |
| P9  | Parallel agents are all recorded                                        | pitch     | covered      | NFR-05 "≥ 50 concurrent sessions and subagents writing"; NFR-08; ENG-09                                                               | —                    |
| P10 | There is a view, a graphical interface                                  | pitch, D8 | contradicted | NG5 "Graphical UI", "CLI and MCP only"                                                                                                | (b) C9               |
| P11 | The view shows each agent live, as it works                             | pitch, D8 | missing      | None. REC-13 bounds ingestion, not display                                                                                            | (a) UI-02            |
| P12 | The view shows what each agent produced                                 | pitch, D8 | partial      | LMK-02 "touched file paths, and an error indicator". Gap: no per-agent results of files changed and commands run                      | (a) LANE-03          |
| P13 | The view shows what actually verified each result                       | pitch     | missing      | None; LMK-02's error indicator is not verification                                                                                    | (a) LANE-04          |
| P14 | You work with your agents live: steer them from the view                | pitch     | missing      | None. Steering by injection would break INJ-03 "MUST accept only values of type `TrustedText`" and INJ-04                             | (a) UI-04            |
| P15 | It runs standalone, with no service behind it                           | pitch, D3 | covered      | CON-03 "No runtime dependency on Python, Node.js, or a database server"; NFR-12 "zero required configuration"                         | —                    |
| P16 | Standalone needs no network                                             | pitch, D4 | covered      | SEC-01; ENG-12 "network-denied sandbox"; CON-04                                                                                       | —                    |
| P17 | Peer to peer can be turned on                                           | pitch, D4 | contradicted | I4 "No listening sockets, no outbound connections"; SEC-01; CON-04; NG4                                                               | (b) C1. Post-v1      |
| P18 | Peering is built on standalone and off by default                       | D4        | missing      | None                                                                                                                                  | (a) PEER-01. Post-v1 |
| P19 | You work a lane live with others                                        | pitch     | missing      | None                                                                                                                                  | (a) PEER-04. Post-v1 |
| P20 | No central service: every node is a full peer; self-hosting is normal   | pitch, D3 | missing      | None for the network side; standalone is P15                                                                                          | (a) PEER-02. Post-v1 |
| P21 | No break when the network splits: each side keeps writing               | pitch, D5 | contradicted | REC-06 "gap-free, and never reused within its project"; REC-10; §8.2 `seq` "gap-free". Two cut-off writers cannot extend one chain    | (b) C3               |
| P22 | The sides merge on reconnect and converge                               | D5, table | missing      | None                                                                                                                                  | (a) PEER-03. Post-v1 |
| P23 | Derived state is the same whatever order logs arrive in                 | D5        | partial      | I10 "a deterministic function of the append-only record"; ADM-08. Gap: one record in one order is assumed                             | (b) C4               |
| P24 | CRDTs merge state several participants edit at once                     | D5        | missing      | None                                                                                                                                  | (a) LANE-05. Post-v1 |
| P25 | Live co-editing of files in one worktree                                | table     | missing      | None; the table marks it "Planned"                                                                                                    | (b) C12. Post-v1     |
| P26 | Only you can instruct your agents                                       | pitch     | partial      | PIN-01 "created only by trusted actions"; INJ-03; PRV-04. Gap: the harness reads others' CLAUDE.md, files and web pages directly      | (c) pitch            |
| P27 | Everyone else's words reach your agents only as untrusted data          | pitch     | partial      | I2; RCL-04. Gap: PRV-02 trusts "`user` when the deployment mode is `interactive`", so an imported co-author prompt is trusted         | (b) C5               |

### Claims from the lane and the decisions

| #   | Claim                                                                     | Source         | Status       | Supporting ids and gap                                                                                                                       | Action      |
| --- | ------------------------------------------------------------------------- | -------------- | ------------ | -------------------------------------------------------------------------------------------------------------------------------------------- | ----------- |
| P28 | The lane is the unit ParleyInn holds                                      | lane, D2       | missing      | None; the record is per project, an "event log of every session" (§1.4)                                                                      | (a) LANE-01 |
| P29 | A lane spans a branch and all its worktrees                               | lane, D2       | contradicted | §8.1 "hex(SHA-256(canonical project path))[:32]"; glossary Project "A working directory"; RCL-05 then forbids crossing worktrees             | (b) C6      |
| P30 | A lane holds its agents and humans, and who did what                      | lane           | partial      | REC-02 "recording agent identity when the transcript provides it". Gap: no human identity                                                    | (a) LANE-02 |
| P31 | A lane is a pull request in all but its interface                         | lane, D2       | missing      | None: no review, approval, required check or landing                                                                                         | (c) pitch   |
| P32 | Git keeps the code                                                        | lane           | partial      | ADM-02 diffs configuration writes. Gap: nothing forbids ParleyInn writing to a repository                                                    | (a) ADM-13  |
| P33 | Keep the name ParleyInn                                                   | D1             | covered      | SRS title page; not requirement-bearing                                                                                                      | —           |
| P34 | The experience comes first; seamless sync second                          | D6             | missing      | §12 has no lane, view or peer milestone                                                                                                      | (b) C12     |
| P35 | Security is good enough, measured against Zed Delta                       | D7             | missing      | SEC-17 asks for a threat model; §6.1 names no co-author, peer, network attacker or browser page                                              | (b) C11     |
| P36 | One view shows the other harnesses beside the one being worked with       | D8             | missing      | None                                                                                                                                         | (a) UI-01   |
| P37 | Harnesses reach each other's lane work directly, with no UI between       | D8             | partial      | RCL-05 "Widening ... to all sessions of the project MUST require an explicit parameter". Gap: worktrees are separate projects; no lane scope | (a) RCL-08  |
| P38 | The UI is one more client, never a required hop                           | D8             | missing      | None                                                                                                                                         | (a) UI-03   |
| P39 | The standalone UI is a browser page on a loopback-only port               | D9             | contradicted | SEC-01 "MUST NOT open listening sockets"; NFR-09 "No resident process between sessions."                                                     | (b) C2      |
| P40 | The UI guards against local attacks: launch token, Host and Origin checks | D9, 2610022338 | missing      | None                                                                                                                                         | (a) SEC-20  |
| P41 | Others' words shown in the view cannot act in the browser                 | pitch, D8      | missing      | SEC-06 protects the envelope only, not rendering                                                                                             | (a) SEC-21  |
| P42 | Standalone means nothing leaves the machine                               | D9             | partial      | SEC-15; I4 "Data leaves the machine only when Claude receives recalled content through a tool call". Gap: recall leaves, by I4's carve-out   | (c) pitch   |
| P43 | Boundaries B0–B3 are explicit; a wider one is never on by default         | D10            | missing      | None; SEC-01 is a single ban                                                                                                                 | (a) SEC-19  |
| P44 | B0: the core that feeds the model has no network capability               | D10            | covered      | SEC-01; ENG-16 "forbidden imports (`net`, `net/http`, `os/exec` outside allow-listed packages)"; ENG-12                                      | —           |
| P45 | No telemetry and no third-party service at any boundary                   | D10            | covered      | SEC-15 "MUST NOT include telemetry, crash reporting to remote services, or update checks"; OPS-05                                            | —           |

### Claims from the comparison and the plans

| #   | Claim                                                          | Source      | Status       | Supporting ids and gap                                                                      | Action               |
| --- | -------------------------------------------------------------- | ----------- | ------------ | ------------------------------------------------------------------------------------------- | -------------------- |
| P46 | B2: only signed segments from peers enrolled by key, untrusted | D10         | missing      | None                                                                                        | (a) SEC-22. Post-v1  |
| P47 | B3: public, read-only, reviewed and signed lane bundles        | D10         | missing      | None; ADM-12 export is local                                                                | (a) SEC-23. Post-v1  |
| P48 | Signed approvals and required checks gate the lane             | table       | missing      | None                                                                                        | (a) LANE-06. Post-v1 |
| P49 | Purged content cannot be confirmed afterwards (keyed hashes)   | table, open | contradicted | REC-09 "keyed by SHA-256"; §8.2 tombstone "hash of removed content" confirms a guessed text | (b) C7               |
| P50 | Claude Code first; other harnesses later                       | table       | covered      | NG6 "Design MUST NOT preclude them (ADR-08)"; NFR-13                                        | —                    |
| P51 | Agents in reclaimed cloud sandboxes keep their record          | 2610012322  | contradicted | §2.2: `PARINN_HOME` must "live on durable volumes so sessions can pause and resume"         | (b) C10. Post-v1     |

Not traced: dots' rule levels per agent action. pitch.md states them
as a lesson, and SEC-13 already ships an example approval hook.

## Proposed requirement texts

### New families

- **LANE** (§5.11). The lane sits above session and project. No family
  owns its identity, actors, results or merge gate.
- **UI** (§5.12). NG5 kept a graphical client out, so no family covers
  one. Its security goes to SEC.
- **PEER** (§5.13). Replication is new behavior with its own priority
  and milestone. Its controls go to SEC-22.
- No NET family: boundaries are security controls, so they extend SEC.

The standalone lane and view are P1, since D6 puts the experience
first; anything needing more than loopback is P2. If the stakeholder
wants the view after v1.0, those P1 rows become P2 unchanged.

Each entry gives id, priority, verification and traces, then the text.

### §5.1 Record

- **REC-17** · P2 · T · I1 — On `Stop` and `SessionEnd`, ParleyInn SHOULD
  record a worktree checkpoint as an event with provenance `file`: the
  checked-out commit id, the branch, and a redacted diff (SEC-08) of
  tracked and non-ignored untracked files against the previous
  checkpoint, stored as a payload. Reading the worktree MUST stay
  within SEC-18 and the hook budget, leaving the rest to a work marker
  (NFR-02).
- **REC-18** · P1 · T · I6 — Each origin MUST sign the head of its hash
  chain (REC-10) with a per-origin Ed25519 key whenever it closes a
  segment, and `parinn verify` MUST check every signature against the
  origin's public key, exiting 3 on a mismatch. The private key MUST
  be generated on the node, held under SEC-10's secret-reference
  rules, and MUST NOT enter the record, an export, a backup or a
  segment.

REC-17 needs a git reader that works within the core's boundary, with
no program start: own code
or a dependency with its ADR (ENG-18). REC-18 changes SEC-10's "The v1
core MUST handle no credentials" to "The core MUST handle no credential
other than its own origin signing key (REC-18)". A same-UID attacker
stays out of scope (§6.1), so REC-18 makes the record tamper-evident
once it leaves the node: backups, exports, peers.

### §5.4 Recall

- **RCL-08** · P1 · T · I8 — `search`, `expand` and `landmarks` MUST
  accept the scope `session = lane`, covering every session of the
  current lane (LANE-01), including other worktrees of the project.
  The scope MUST be explicit, MUST be logged like the all-sessions
  scope (RCL-05), and MUST NOT reach another project.

### §5.8 Administration

- **ADM-13** · P0 · T · I7 — ParleyInn MUST NOT write to any git repository
  or working tree (objects, refs, notes, configuration, hooks or
  files), except the project settings file that `parinn install --scope
  project` writes after confirmation (ADM-02). It MAY read repositories
  within SEC-18.

### §5.11 Lane

- **LANE-01** · P1 · T · I1, I2, I10 — ParleyInn MUST hold each lane as one
  record, identified by its project and a lane id. Every session MUST
  belong to exactly one lane, chosen deterministically from the
  record: the lane of the branch recorded as a `harness_meta` event at
  the session's first event, or the project's default lane. A
  reassignment MUST be an `operator` event. Branch names MUST be
  sanitized per LMK-03 before any structural use.
- **LANE-02** · P1 · T · I2 — Every event MUST name its actor: an agent
  session (with subagent identity per REC-02) or a human, who is the
  local tenant for local events and the enrolled key's owner for
  imported ones. The actor MUST be derived from origin and source,
  never from content.
- **LANE-03** · P1 · T · I2, I10 — ParleyInn MUST derive per session and per
  lane the files changed and the commands run with their exit status,
  each bound to the `seq` range that produced it, from structural
  fields only (LMK-03), as a deterministic function of the record.
- **LANE-04** · P1 · T · I2, I4, I10 — Each LANE-03 result MUST carry one
  verification level: `none`, `claimed` (assistant text only),
  `local_run` (a command recorded with its exit status) or `external`
  (an off-machine check known only from tool output), derived
  deterministically from the record. A level resting on untrusted
  events MUST be marked untrusted, and ParleyInn MUST NOT fetch check
  results itself.
- **LANE-05** · P2 · T · I10 — Lane metadata several participants may
  change (title, status, labels, assignment) MUST be stored as events
  and merged by a documented conflict-free rule, a CRDT chosen by ADR,
  so every node holding the same events derives the same metadata.
- **LANE-06** · P2 · T, D · I2, I10 — A lane MAY declare required checks
  and approvers. An approval MUST be an event signed by an enrolled
  approver's key over the lane heads it approves. ParleyInn MUST report a
  lane landable only when every required check is `local_run` or
  stronger on those heads and every required approval is present, and
  MUST NOT push or merge itself (ADM-13).

### §5.12 User interface

- **UI-01** · P1 · D, T · I1 — `parinn-ui` MUST show in one view every
  live and recorded session of the tenant's projects, grouped by lane.
  The selected session shows prompts, tool calls, permission requests,
  output, each edit as a diff, each tool run with its result and
  verification level (LANE-04), and each event's actor (LANE-02).
  Every other session MUST appear as a tile that opens full size.
- **UI-02** · P1 · A · I9 — `parinn-ui` MUST show each transcript line of
  a running session within 2 s† of the harness writing it (ASM-17),
  and its reads MUST NOT push any hook past its NFR-01 budget.
- **UI-03** · P1 · T · I9, I10 — `parinn-ui` MUST be an optional client:
  it MUST read the store only through the core library, read-only, and
  MUST hold no state the record cannot rebuild. Every capability it
  offers MUST also be in the CLI or MCP, and no hook, ingestion or
  recall path MAY depend on it running.
- **UI-04** · P2 · T · I2, I6 — `parinn-ui` MAY send a prompt or a
  permission decision to a running local harness only on the local
  tenant's action in an authenticated session (SEC-20), only through
  the harness's own client interface, never through ParleyInn's injection
  path (INJ-03, INJ-04). It MUST NOT forward content from a peer. Each
  action MUST be recorded with provenance `user` and audited.

### §5.13 Peer network

- **PEER-01** · P2 · T · I4, I9 — Peering MUST ship as a separate binary,
  `parinn-peer` (B2), off by default, started only by an explicit
  tenant action, and neither linked nor started by the core. With it
  absent or stopped, every requirement in §5–§10 MUST hold as in
  standalone.
- **PEER-02** · P2 · T, D · I4 — Every peer MUST be able to hold
  complete copies of the lanes it shares and serve them to any
  enrolled peer. No peer MAY be required for two others to exchange
  segments, and no third-party service for enrollment, discovery or
  relay.
- **PEER-03** · P2 · T · I9, I10 — Peers MUST exchange whole, signed
  origin segments (REC-18). Two peers holding the same segments MUST
  derive byte-identical lane state and index, whatever order or
  partition they arrived through (ADM-08). A writer cut off from every
  peer MUST keep appending to its own log without slowing its agents.
- **PEER-04** · P2 · A, T · I2 — While connected, enrolled peers MUST
  receive a lane's newly closed segments within 5 s†. Every imported
  event MUST keep its origin, MUST be untrusted on the importing node
  (PRV-02), and MUST reach an agent only through pull-only recall.

### §6.2 Security

- **SEC-19** · P0 · T, I · I4 — Each ParleyInn binary MUST be assigned to
  exactly one boundary in a register kept in the repository: B0
  process (the core, no socket), B1 machine (loopback only), B2 peer
  (enrolled peers only) or B3 public (read-only). B2 and B3 MUST be off
  by default. CI MUST fail when a binary's import closure, or a test
  under its boundary's sandbox, shows more reach than the register
  grants.
- **SEC-20** · P1 · T · I4, I6, I8 — `parinn-ui` MUST bind only to a
  loopback address on an ephemeral port and MUST require on every
  request a random token of at least 128 bits, minted per launch and
  shown only to the launching user. It MUST reject a request whose
  `Host` is not its own address and port or whose `Origin` is not its
  own, MUST allow no cross-origin request, and MUST audit every
  rejection.
- **SEC-21** · P1 · T · I2, I4 — `parinn-ui` MUST render record content as
  inert text, never interpreting HTML, script, or Markdown links and
  images from it. A Content-Security-Policy MUST forbid any resource
  from outside its own origin, so content can neither run code nor
  make the browser reach the network. Untrusted content MUST be
  visibly marked.
- **SEC-22** · P2 · T · I2, I6, I8 — `parinn-peer` MUST accept segments
  only from peers enrolled by public key, MUST verify each segment's
  signature (REC-18) and chain continuity before import, MUST refuse
  and audit a segment failing either, and MUST store imported events
  under the local home (SEC-02) as untrusted (PRV-02).
- **SEC-23** · P2 · T, I · I2, I4 — Publishing a lane (B3) MUST require
  the tenant's review of each bundle, redaction at least as strict as
  SEC-08 plus a publish-only rule set, and a signature. The public
  host MUST serve bundles read-only with no write path, and an
  imported public bundle MUST be untrusted.

### Register entries

- **ASM-17** (§2.3) — Claude Code appends each transcript line within
  1 s of the event while a session runs. Evidence: unverified.
  Verified in: S9. Affects: UI-02.
- **S9** (§12.1) — How soon does a running session flush transcript
  lines, and can a read-only tail keep up? Resolves ASM-17. Exit: lag
  measured, UI-02's † frozen.
- **OQ-14** (§13.1) — How does a co-author's message reach an agent
  without the owner relaying it: an owner's signed endorsement, or a
  granted role? Until resolved, no imported event is trusted. Owner:
  security review, before M8.
- **§6.1 actors**, all in scope: a co-author or enrolled peer, who
  writes their own origin's events into a shared lane; a network
  attacker between peers; a web page in the tenant's own browser,
  which can call loopback ports and try DNS rebinding.

| #   | Threat                       | Vector                                                        | Controls                     |
| --- | ---------------------------- | ------------------------------------------------------------- | ---------------------------- |
| T14 | Co-author injection          | A peer's message written to instruct the owner's agents       | PRV-02, PEER-04, OQ-14       |
| T15 | Local web attack on the UI   | DNS rebinding, CSRF or cross-origin reads from a browser page | SEC-20                       |
| T16 | Rendered-content attack      | Record content that runs script or loads remote resources     | SEC-21                       |
| T17 | Forged or rewritten segments | A forged origin or rewritten chain from a peer or the network | REC-18, SEC-22               |
| T18 | Erasure gap                  | Purged content persists on peers and in published bundles     | SEC-23 review; documentation |

## Proposed changes to existing text

### C1 — I4, for P17 (invariant change)

**Flag:** this weakens an invariant, so it needs a security review and
a new major version (CLAUDE.md, §1.3): the SRS becomes 2.0-draft.

Old: "**ParleyInn never talks to the network.** No listening sockets, no
outbound connections, no telemetry. Data leaves the machine only when
Claude receives recalled content through a tool call, and then travels
to the model provider like any other context."

New: "**The core never talks to the network, and nothing wider is on by
default.** The core (record, hooks, recall, restore, and everything
that can reach the model) opens no socket and makes no outbound
connection: boundary B0. Every other component sits behind exactly one
wider boundary (SEC-19): B1 loopback, B2 enrolled peers, B3 public
read-only. B2 and B3 are off unless the tenant turns them on. No
component sends telemetry or calls a third-party service. Data leaves
the machine only when Claude receives recalled content through a tool
call, or, once the tenant enables B2 or B3, as signed segments to
enrolled peers or bundles the tenant reviewed. Whatever a peer delivers
is untrusted (I2)."

Knock-on: the glossary gains **Core**, the B0 binary `parinn` and its
library. CON-04 stands if `parinn-ui` and `parinn-peer` are separate
binaries shipping no P0 feature.

### C2 — SEC-01, for P39

Old: "Shipped binaries MUST NOT open listening sockets or make outbound
network connections. This MUST be enforced by an import allow-list
check in CI and by a test suite run under a network-deny sandbox."

New: "The core binary (`parinn`, B0) MUST NOT open any socket or make
outbound connections. Components behind B1–B3 MUST ship as separate
binaries the core neither links nor starts. `parinn-ui` (B1) MUST listen
only on loopback and make no outbound connection. This MUST be enforced
per binary by an import allow-list check in CI, and by suites run under
a sandbox denying all network to the core and all but loopback to
`parinn-ui`."

Knock-on: NFR-09 becomes "The core keeps no resident process between
sessions; `parinn-ui` and `parinn-peer` run only while the tenant started
them." ENG-12 and ENG-16 run per binary. A new ADR records the UI
server; the core stays daemonless (ADR-01).

### C3 — REC-06 and REC-10, for P21

Old REC-06: "Every event MUST receive a `seq` that is strictly
increasing, gap-free, and never reused within its project, assigned
inside the append transaction."

New REC-06: "Every event MUST receive an address (`origin`, `seq`):
`origin` names the writing node and session, and `seq` is strictly
increasing, gap-free and never reused within that origin's log,
assigned only by that origin's writer inside the append transaction."

Old REC-10: "Events MUST form a per-project hash chain: each event
stores `SHA-256(prev_hash ‖ canonical_encoding(event))`, where the
canonical encoding includes the payload hash."

New REC-10: "Each origin's events MUST form their own hash chain: each
event stores `SHA-256(prev_hash ‖ canonical_encoding(event))`, where
`prev_hash` is the previous event of the same origin and the canonical
encoding includes the payload hash and the other origins' heads the
writer had seen."

I1's text stands, but its "stable address" becomes (`origin`, `seq`).
Knock-on: glossary **seq**, §8.2 `events`, `seq` in §9.2 and §9.3, LMK
ranges, ENG-08 and NFR-08 per origin. This is the identity model M1
blocks on.

### C4 — ADM-08 and glossary "Record", for P23

**Flag:** I1, I5 and I10 keep their text, but redefining "record"
changes what they range over. Review it with C1.

Old glossary: "The append-only log of events for a project. The source
of truth." New: "The set of append-only, per-origin logs a node holds
for a project. The source of truth."

ADM-08 keeps its text and adds: "Derived state MUST depend only on the
set of origin logs held, never on the order in which logs or their
segments arrived."

### C5 — PRV-02, for P27

Old: "The default trust policy MUST classify as `trusted` only:
`operator`; `harness_meta` (…); and `user` when the deployment mode is
`interactive`. Every other class MUST be `untrusted`."

New: "The default trust policy MUST classify as `trusted` only events
whose origin is the local tenant's own node and whose class is
`operator`, `harness_meta` (…), or `user` when the deployment mode is
`interactive`. Every other event MUST be `untrusted`, including any
event whose origin is another node or tenant."

This tightens I2, so it is not an invariant change. Trust becomes
relative to the node; rebuild stays deterministic per node.

### C6 — §8.1 and glossary "Project", for P29

Old §8.1: "project-id = hex(SHA-256(canonical project path))[:32]
(N)". New: "project-id = hex(SHA-256(canonical repository path))[:32]
(N), where the repository path is the git common directory when the
working directory lies in a git work tree, so all worktrees of one
clone share a project, and the working directory otherwise."

Old glossary: "A working directory as identified by Claude Code (one
transcript directory). Each project has its own store." New: "A
repository clone, identified by its git common directory, with all its
worktrees; outside git, a working directory. Each project has its own
store and may span several transcript directories."

Knock-on: ASM-03 maps several transcript slugs to one project; SEC-18
admits the git common directory as a read root.

### C7 — REC-09 and the §8.2 tombstone, for P49

REC-09: "keyed by SHA-256" becomes "keyed by HMAC-SHA-256 under the
tenant's content key (`tenant.json`), so no stored or tombstoned hash
confirms a guessed content without that key". §8.2 `tombstones`: "hash
of removed content" becomes "keyed hash of removed content"; §4.3
follows. Peers cannot check a keyed address, so their payload
integrity rests on REC-18 signatures.

### C8 — NG4, for P8 (post-v1)

Old: "Multi-host shared stores — Multiplies blast radius; requires
provenance-gated sharing design first (OQ-07)". New: "Multi-host stores
in v1.0, and a shared mutable store at any version — Later releases
replicate per-origin, append-only logs between enrolled peers (PEER,
P2)." OQ-07 resolves through the PEER design in M8.

### C9 — NG5, for P10

Old: "Graphical UI — CLI and MCP only". New: "A graphical interface
that is required, or reachable from off the machine — The lane view
(UI-01..04) is an optional local client behind B1; every operation
stays in the CLI and MCP (UI-03)."

### C10 — §2.2, for P51 (post-v1)

Add: "**Later, with peering:** ephemeral sandboxes whose `PARINN_HOME`
is reclaimed. Their record survives only by syncing outbound to an
enrolled peer before reclaim (PEER-02, SEC-22); until then they are out
of scope."

### C11 — SEC-17 and §6.1, for P35

Old: "A threat-model document MUST be maintained in the repository and
reviewed at every minor release." New adds: "It MUST cover every
boundary in the register (SEC-19) and the actors acting across them,
and MUST compare ParleyInn control by control with the closest product the
stakeholder names (Zed Delta as of 2026) on record signing, co-author
trust, central-service dependence and key custody." The actors and
T14..T18 above join §6.1.

### C12 — §12, for P25 and P34

New milestones, listed after M4 so existing ids stay stable; the order
follows D6, experience before sync:

- **M7, lane and view, standalone:** LANE-01..04, UI-01..03, RCL-08,
  REC-18, SEC-19..21, S9. Exit: with all but loopback denied, two
  harnesses on separate worktrees show in one view with their results,
  and the stakeholder signs off.
- **M8, peer network:** PEER-01..04, SEC-22, LANE-05, OQ-14; needs C1
  approved. Exit: two partitioned peers keep working and converge to
  byte-identical lane state on reconnect.
- **M9, seamless sync and beyond:** discovery, outbound-only sandboxes
  (C10), live co-editing in one worktree, LANE-06, SEC-23, UI-04,
  REC-17. Exit: each item lands through its own proposal and ADR.

P8, P17..P20, P22, P24, P25, P46..P48 and P51 are post-v1. pitch.md
calls itself direction, but before it is used outside the project it
should say peer to peer is the next step, not the first release.

## Proposed pitch changes

### For P26: only you can instruct your agents

Reason: ParleyInn mediates only its own channels, the restore block
(INJ-03) and recall (RCL-04). The harness still reads others' CLAUDE.md,
files, web pages and tool output directly. In automation mode (PRV-04)
ParleyInn trusts not even the owner's prompts.

Old: "Only you can instruct your agents; everyone else's words reach
them as untrusted data."

New: "Nothing ParleyInn carries instructs your agents unless you put it
there: everyone else's words reach them only as untrusted data, and
only when they ask for it."

### For P31: a pull request in all but its interface

Reason: a pull request also runs review, approvals, required checks and
landing. No requirement covers them; LANE-06 is P2 and the merge gate
is plan 2610022338's task 5.

Old: "That is a pull request in all but its interface. Git keeps the
code; ParleyInn keeps the lane."

New: "It holds what a pull request's conversation holds, plus the runs
and results behind it. Git keeps the code and lands it; ParleyInn keeps the
lane."

### For P42: standalone means nothing leaves the machine

Reason: I4 carves out recall, which reaches the model provider through
the harness. The decision as worded is stronger than I4.

Old (decision 9): "Standalone means nothing leaves the machine."

New: "Standalone means ParleyInn sends nothing off the machine; what an
agent recalls travels to its model provider with the rest of its
context, as it does today."
