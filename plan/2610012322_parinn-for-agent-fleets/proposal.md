# ParleyInn for agent fleets: the reconciled proposal

This proposal drove plan 2610012322's SRS change, which has landed
as SRS 2.0-draft. Where a section landed, it now points at the SRS,
the only source; what stays here is the reasoning and the findings.
It was the one canonical proposal for that change. It
replaces the forward and backward traces, the four UX drafts and the
persona review as the source the SRS change is written from. Those
files stay as evidence. Nothing here is normative until the
stakeholder approves it and a named human security reviewer accepts
the invariant changes in §3 (ENG-29).

Sources reconciled: [pitch.md](pitch.md), [plan.md](plan.md),
[phase-1.md](phase-1.md), [trace-forward.md](trace-forward.md),
[trace-backward.md](trace-backward.md), the four drafts under `ux/`,
the persona review under `persona-review/`, plan
[2610022338](../2610022338_parinn-network-side/plan.md) with its
[phase 1](../2610022338_parinn-network-side/phase-1.md), and the SRS
1.4-draft under [docs/srs](../../docs/srs/index.md).

## 1. Summary

ParleyInn holds the lane: a branch, its worktrees, its agents and people,
and their conversations, edits, tool runs and results, as one
hash-chained record per writer on the user's own machines, which
shows tampering by anyone but a process of the same user until a
receipt leaves the machine. The
agent recalls it exactly; the person sees it live in one view and
steers through the harness. Peering extends it later, with no central
service.

What this proposal settles:

- **One contract version.** The SRS goes to 2.0-draft. I1, I2, I4,
  I5, I8 and I10 change wording (§3); each change waits for the named
  security reviewer, the stakeholder (@jeduden), to record approval in
  an ADR (ENG-29).
- **One id set.** 106 new requirements and 22 changed ones, numbered
  from the next free id on disk, avoiding every id another open pull
  request holds (§6, §9).
- **Four new families, chosen once.** LANE (what a lane is and what
  it derives), VIEW (what a person reads, on any surface), OWN (every
  act by which a person reaches an agent) and PEER (replication, P2).
  The traces' UI family becomes VIEW; boundary controls stay in SEC.
- **One rule per contested question** (§6.0): trust by local writer or
  certified owner device, never by claimed origin; content committed
  through a per-event key; recall defaulting to the session; every
  owner act `operator`.
- **v1 scope.** The record, pins, recall, the per-writer identity
  model and the standalone lane view with steering. Peering, the
  merge gate, public lanes and bridges are P2, specified now so v1
  does not preclude them.
- **Every persona finding dispositioned** (§10): 121 items from the
  first review, 2 not adopted, 4 adopted in part, each with its
  reason; the second review's 18 merged findings, R1–R18 (§10.12);
  the third review's 17, R3-1 to R3-17 (§10.13); the fourth review's
  8, R4-1 to R4-8 (§10.14); the fifth review's 6, R5-1 to R5-6
  (§10.15); the sixth review's 6, R6-1 to R6-6 (§10.16); the seventh
  review's 5, R7-1 to R7-5 (§10.17); and the stakeholder's OQ-29
  decision (§10.18), and the decisions on D1 to D7 (§10.19).

## 2. Defaults applied

The stakeholder decided six of these seven questions on 3 October 2026
(§10.19); D7 stays open for a longer discussion. Requirement rows
that implement one carry its tag, for example `(D4)`, at the start of
their text.

| Tag | Decision                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | Alternative kept visible                                                                                                                   | Where it lands                                             |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------- |
| D1  | DECIDED. I4 is restated by network boundary: B0 core, no sockets; B1 machine, loopback only; B2 peer, opt-in; B3 public, opt-in. Managed policy can disable each of B1–B3. The record becomes one signed, hash-chained log per writer, addressed by (writer, seq). The requirements name no technology: formats, storage engines and hash, commitment and signature algorithms are chosen by ADR (ENG-26). The SRS goes to 2.0-draft, and the invariant changes wait for the named security reviewer's recorded approval (ENG-29).                                                                                                                                                                                                                                                                                                                                                                                                                      | Keep I4 as written and ship the UI and the network side as a separate product with its own SRS.                                            | §3, SEC-01, SEC-19, SEC-22, REC-06, REC-10, REC-18, ENG-29 |
| D2  | DECIDED, with the granted-role alternative kept for later (OQ-14, behind its own I2 review). Each agent has exactly one principal; co-authors drive their own agents in the lane. Any principal forwards a post to their own agents with a signed one-click Endorse that sends exactly the text shown inside a fixed template naming its source, recorded as their act. No automatic notices by default; an opt-in notice is `TrustedText` built only from a count, key fingerprints and recall addresses, audited, and never starts a turn.                                                                                                                                                                                                                                                                                                                                                                                                            | A granted role that makes a co-author's messages trusted, which changes PRV-05 and needs its own I2 review; notices on by default.         | OWN-01, OWN-08, OWN-09, INJ-10, LANE-10, LANE-12           |
| D3  | DECIDED. An owner key certifies device keys, each with a scope and a maximum rule level, under an I2-reviewed requirement. A phone may only allow or deny held permission requests.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Every imported event untrusted, including the owner's own other devices, so a phone or second machine cannot answer anything.              | PRV-02, PRV-10, OWN-16, OWN-17, PEER-07                    |
| D4  | DECIDED, with OQ-17 answered yes: a permission decision computed from the owner's signed rules may return through a hook with a fixed template. ParleyInn never denies on a timeout unless the owner turned on an away policy, and the harness's own prompt stays answerable while ParleyInn holds a request. Holds get their own NFR-01 budget row. `parinn run` is a separate, optional launcher outside the core, with its own boundary row and review. Plain-hook Claude Code gets reduced controls, stated at install. Owner steering, approvals and replies reach an agent only through the harness's own input, never through hook output, and carry only owner-typed text or fixed templates that reference ids.                                                                                                                                                                                                                                | A 3-minute hold that parks and denies by default; steering through hook output; no launcher, so plain hooks are the only Claude Code path. | OWN-03..07, OWN-15, OWN-19, NFR-01, SEC-29                 |
| D5  | DECIDED by the stakeholder (3 October 2026, OQ-29). Against an unsandboxed agent of the same user, no software surface proves who acted, so ParleyInn rests owner acts on the sandbox around the agent (plan 2610031222) and does not try to defeat an unsandboxed one. Owner acts fall in three classes; cut and neutral acts need only an authenticated surface or a terminal. Widening acts need only the same while every session on the node runs in a sandbox that blocks the residual risks; while any session leaves a risk open, they need the owner's recorded acceptance of the open risks (OWN-22), and ParleyInn labels every such session. WebAuthn per act stays available as hardening that the owner or managed policy may require. With only the core, the person also acts through the harness's own channels (its prompt, `/pin` and `/unpin` scoped to the session, typing to it), which ParleyInn records but does not vouch for. | Presence optional, so an unsandboxed same-user agent's only obstacle is the launch credential and the CLI code.                            | OWN-11, OWN-12, LANE-11                                    |
| D6  | DECIDED. Search scope is a choice: on every search the person selects session, lane, project or every lane the tenant holds, and the surface shows the scope it used. An agent selects its own scope through RCL-05's explicit `scope` parameter, defaulting to the session.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | Operator search scoped like agent recall, one project at a time.                                                                           | VIEW-09, RCL-05                                            |
| D7  | OPEN: deferred by the stakeholder for a longer discussion; rows tagged (D7) keep a provisional priority until then. The default under discussion: The standalone lane view is P1 in v1. Peering is P2, specified now so v1 does not preclude it. Whatever D7 decides for the browser view, the CLI forms of Catch up, search, verify and the gap display (VIEW-08..11) stay P1.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | The view moves to P2 after v1.0; its P1 rows become P2 unchanged. Or the view becomes P0, which delays v1 by a milestone.                  | VIEW, OWN, NG4, NG5, §11                                   |

D4 includes OQ-17, answered yes by the stakeholder: a permission decision
(allow, ask, deny) computed from the owner's signed rules is harness
control, not text, so a hook may return it with a fixed template that
references ids (OWN-04). Owner-typed text never goes through a hook.

## 3. Invariant and constraint changes

Every change in this section needs a human security review and is the
reason the SRS becomes 2.0-draft (CLAUDE.md, §1.3). Each row is
flagged **SECURITY REVIEW**. ENG-29 makes the review a gate: no
component that crosses B0 is built past a prototype, and no row below
is accepted, until an ADR records the named reviewer's approval.

### 3.1 Invariants

| #   | Flag            | Old wording (1.4-draft)                                                                                                                                                                                                                                                                                          | Now                                                                                      | Why                                                                                                                                                                                                                                                                                                     |
| --- | --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| I1  | SECURITY REVIEW | **Nothing is lost.** Every event an agent saw or produced remains recoverable by a stable address across any number of compactions and sessions. The only exceptions are secrets removed by redaction before storage, and data an operator explicitly purges or expires by policy. Both exceptions are recorded. | Accepted as ADR-2610032155 change 1; the text is in [§1.3](../../docs/srs/invariants.md) | A node-local `seq` is not stable for anyone else (D1). Redaction on import (REC-23) is a new exception.                                                                                                                                                                                                 |
| I2  | SECURITY REVIEW | **No automatic path from untrusted content to the model.** Content that originates outside the trusted boundary (tool output, web, MCP servers, files, assistant text) reaches the model only when Claude explicitly calls a recall tool, and always inside an untrusted-data envelope.                          | Accepted as ADR-2610032155 change 2; the text is in [§1.3](../../docs/srs/invariants.md) | Names the new untrusted sources, bounds the trusted boundary by key, and admits the one path by which another person's words reach an agent: the principal's recorded Endorse (D2, D3, D4). The automatic writes are listed as a closed set, so the invariant holds as written (security re-review #3). |
| I4  | SECURITY REVIEW | **ParleyInn never talks to the network.** No listening sockets, no outbound connections, no telemetry. Data leaves the machine only when Claude receives recalled content through a tool call, and then travels to the model provider like any other context.                                                    | Accepted as ADR-2610032155 change 3; the text is in [§1.3](../../docs/srs/invariants.md) | D1. A loopback UI and opt-in peering both break the old wording; the core keeps it unchanged as B0.                                                                                                                                                                                                     |
| I5  | SECURITY REVIEW | **Bad data can be removed from circulation without destroying evidence.** Any event, span, session, or derived artifact can be quarantined from recall immediately, while the record stays intact for forensics.                                                                                                 | Accepted as ADR-2610032155 change 4; the text is in [§1.3](../../docs/srs/invariants.md) | A local quarantine cannot bind a peer's copy; saying so keeps I5 true (SEC-12, PEER-11).                                                                                                                                                                                                                |
| I8  | SECURITY REVIEW | **Isolation follows the tenant.** All state is bound to one tenant's home with strict permissions. ParleyInn refuses to operate on state it does not own.                                                                                                                                                        | Accepted as ADR-2610032155 change 5; the text is in [§1.3](../../docs/srs/invariants.md) | A lane node holds logs other tenants wrote; read literally, the old text forbids it.                                                                                                                                                                                                                    |
| I10 | SECURITY REVIEW | **Everything derived is rebuildable.** All derived state (indexes, landmarks, active pins, quarantine set, statistics) is a deterministic function of the append-only record. Rebuilding from the record reproduces it exactly.                                                                                  | Accepted as ADR-2610032155 change 6; the text is in [§1.3](../../docs/srs/invariants.md) | Trust is relative to the node (PRV-02), and peers merge logs in any order (PEER-03, phase 1's gate).                                                                                                                                                                                                    |

Unchanged in wording: I3 (pin scope in PIN-10 narrows where a pin is
restored, never what it says), I6 (it now binds every binary, OPS-06),
I7 (SEC-22 and SEC-23 tighten it) and I9. A hold is the owner's
policy, bounded by its own NFR-01 row, not a ParleyInn failure; a ParleyInn
failure during a hold still falls back to the harness's prompt
(OWN-06).

The backward trace's separate I11 ("network boundaries are explicit")
is not adopted: it would split one concern across two invariants, and
D1 restates I4 instead.

### 3.2 Constraints

Landed in [§2.4 Constraints](../../docs/srs/02-context.md).

### 3.3 Non-goals, stakeholders, deployment and ADRs

Landed in [§1.5 Non-goals](../../docs/srs/01-introduction.md), [§2.1 to
§2.2](../../docs/srs/02-context.md) and
[§4](../../docs/srs/04-reference-architecture.md).

## 4. Glossary changes

Landed in [§1.6 Glossary](../../docs/srs/01-introduction.md).

## 5. Personas U1–U9

### 5.1 The personas for a new §2.5

Landed in [§2.5 Personas](../../docs/srs/02-context.md).

### 5.2 Persona coverage

Landed in [Appendix C](../../docs/srs/appendix-c-persona-coverage.md).

### 5.3 Draft Appendix C: existing requirements by persona

Landed in [Appendix C](../../docs/srs/appendix-c-persona-coverage.md).

## 6. The requirement set

Columns follow §5: **ID** · **Pri** · **Requirement** · **Ver** ·
**Inv** (invariant traces) · **Personas**. Each new or changed id
lands with its `@pending` scenario in the SRS change (CLAUDE.md).
Priorities: P0 for v1.0, P1 expected for v1.0, P2 later, specified now.

### 6.0 One rule per contested question

1. **Trust by local writer or certified device, never by claimed
   origin.** The forward trace trusted by origin; the backward trace
   trusted nothing imported. Rule: an event is trusted only if this
   node's own writer wrote it in a class PRV-02 trusts, or it is an
   `operator` event signed by a device key this tenant's owner key
   certified, within that key's scope and rule level (PRV-02, PRV-10).
   Every other imported event is untrusted, whatever its class. The
   writer comes from the verified key alone, and an import claiming a
   local writer is refused (PRV-09). Reason: "nothing imported" leaves
   the owner's phone and second machine powerless (D3), while trust by
   claimed origin lets any peer forge it; a certified, scoped key is
   the narrowest rule that serves both.
2. **Per-event keyed commitment, not tenant-keyed or lane-keyed
   hashing.** The traces disagreed on the key. Rule: content enters
   the chain, seals, tombstones and anything replicated only as a
   keyed commitment to the content under `k_e`, where `k_e` is drawn
   per event, stored with the content and erased with it (REC-17).
   Local file and directory names use a tenant-local storage key that
   never leaves the node (REC-09, LANE-02). Reason: a tenant key
   cannot be shared with co-authors' nodes, so they could not check
   content; a lane key is shared by every holder, so any holder could
   confirm a guess at purged content from the retained commitment.
   Only a key erased with the content meets "no retained or replicated
   value confirms purged content".
3. **Recall defaults to the session; lane and project are explicit.**
   Rule: RCL-05 keeps the current session as the default, adds an
   explicit, logged `lane` and `project` scope, puts foreign lanes
   outside every widened scope (RCL-10), and keeps cross-project
   recall out of v1. Reason: a lane-wide default widens what every
   agent reads, and recall taint (SEC-13) with it, for no request of
   the agent's; the explicit scope gives lane work to agents that ask.
4. **Every owner act is `operator`.** The UX drafts gave owner acts
   `user`, `operator` or `harness_meta`. Rule: every owner act the
   glossary lists, written by an authenticated owner surface, is
   `operator` (OWN-02). A prompt typed into the harness stays `user`,
   trusted only in interactive mode (PRV-04). An answer given at the
   harness's own prompt is recorded as `harness_meta`: an outcome, not
   an act. A post is `post`, always untrusted. Reason: one class per
   kind of proof; `user` cannot carry the owner's authority because
   automation mode distrusts it, and an outcome the harness reports
   proves nothing about who answered.

### 6.1 Changed requirements

| ID     | Pri | Old wording, in short                                                              | New requirement                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | Ver  | Inv     | Personas   |
| ------ | --- | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- | ------- | ---------- |
| REC-06 | P0  | `seq` strictly increasing, gap-free, never reused within its project               | (D1) Every event MUST carry an address (writer, `seq`), where `seq` is strictly increasing, gap-free and never reused within that writer's log, and assigned only by that writer inside its append transaction. A node MAY keep a local position for indexing; it is not an address.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | T    | I1, I10 | U9, U4, U3 |
| REC-09 | P0  | payloads content-addressed, keyed by SHA-256                                       | Event content above a threshold (default 8 KiB†) MUST be stored in the payload store, named by a keyed hash under the tenant's local storage key, which never leaves the node, with the event holding a preview (default ≤ 512 bytes†), the reference and the commitment (REC-17). Payload writes MUST be atomic (write, fsync, rename).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | T    | I1      | U9, U8     |
| REC-10 | P0  | one per-project hash chain over the canonical event                                | (D1) Each writer's events MUST form their own hash chain: each event stores a collision-resistant hash of `prev_hash` and the canonical encoding of its header, where `prev_hash` is the writer's previous event and the header holds only the address, kind, provenance class, `prev_hash`, the heads of the other writers' logs the writer had seen, and the commitment (REC-17). Order across writers is causal, never wall-clock.                                                                                                                                                                                                                                                                                                                                                                                                                      | T    | I10     | U8, U3, U4 |
| REC-13 | P1  | `PostToolUse` and `Stop` hooks SHOULD ingest incrementally                         | `PostToolUse`, `Stop`, `SubagentStop` and `PermissionRequest` hooks MUST ingest the active transcript incrementally within their budget and leave a work marker for any remainder. Raised from SHOULD because the live view (VIEW-02) rests on it.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | T    | I9      | U2, U4     |
| PRV-01 | P0  | one class from a closed set of twelve                                              | Every event MUST carry its writer and exactly one provenance class from this closed set: `user`, `assistant`, `tool_call`, `tool_result:<tool>`, `web`, `mcp:<server>`, `file`, `subagent_result`, `harness_meta`, `harness_text`, `operator`, `post`, `unparsed`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | T    | I2      | U8, U9, U6 |
| PRV-02 | P0  | trusted only `operator`, `harness_meta`, and `user` in interactive mode            | (D3) The default trust policy MUST classify as `trusted` only: events a writer of this node wrote in class `operator`, `harness_meta`, or `user` when the deployment mode is `interactive`; and `operator` events trusted under PRV-10. Every other event MUST be `untrusted`, including every event another tenant wrote, every non-`operator` event from another node, every `post`, and every event ingested from a transcript the hooks did not report (REC-22). A widening `operator` event (OWN-11) MUST take effect in derived state (restore blocks, rule levels, grants, trust, enrolments) only when the sandbox states and risk acceptance recorded when it was written permitted it under OWN-22, and, where an authenticator was required, its stored assertion verifies; `parinn verify` MUST report every widening event that fails either. | T    | I2, I8  | U8, U9, U3 |
| RCL-05 | P0  | default current project; widening from session to all sessions explicit and logged | Recall scope MUST default to the current session. Widening to the current lane (its sessions on every worktree and writer this node holds) or to the current project MUST require an explicit `scope` parameter and MUST be logged. A foreign lane MUST NOT fall inside a widened scope (RCL-10). Recall across projects MUST NOT be possible in v1.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | T    | I8      | U9, U8, U7 |
| ADM-07 | P0  | purge scopes; tombstone per range; compact                                         | `parinn purge` MUST delete the selected scope (project, lane, session, writer, actor, address range, time window, or provenance class) from the sealed segment files, events, FTS index, projections and payload store, MUST erase the commitment key and payload reference of every purged event (REC-17, REC-09) and every copy SEC-31 names, MUST append a tombstone event per purged range holding only addresses, counts, reason and commitments, and MUST compact the database and rewrite the affected segments keeping their seals verifiable. `parinn uninstall` MUST offer purge.                                                                                                                                                                                                                                                                | T    | I1, I5  | U1, U8     |
| ADM-08 | P0  | rebuild regenerates derived state byte-identically                                 | `parinn rebuild` MUST regenerate all derived state from the record, and a subsequent `parinn verify` MUST confirm it is byte-identical. Derived state MUST depend only on the set of writer logs held and the node's own keys, never on the order in which logs or segments arrived.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | T    | I10     | U1, U3     |
| PIN-03 | P0  | pin stores verbatim text, type, priority, creating `seq`, author, text hash        | Each pin MUST store: verbatim text (≤ 1,000 characters†), type, priority, scope (PIN-10), the address (writer, `seq`) of its creating event, author, and the creating event's commitment (REC-17), never a bare hash of its text.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | T    | I3, I5  | U9, U8     |
| ADM-02 | P0  | install shows a diff; uninstall reverses exactly what install made                 | `parinn install` MUST display a diff of every configuration change and require confirmation or `--yes`. `parinn uninstall` MUST list every artifact any ParleyInn binary created (hooks, plugin and MCP registration, `parinn ui` credentials, `parinn run` sockets, writer and device keys, enrolments, git-carrier refs, managed state it wrote), offer to remove each, reverse exactly the configuration changes `install` made, and audit what it left.                                                                                                                                                                                                                                                                                                                                                                                                | T    | I7, I6  | U1, U2     |
| ADM-04 | P0  | TOML, strict validation; layers defaults, tenant, project                          | (D1) Configuration MUST be TOML with strict validation. Layers are defaults, then tenant config, then project `.parinn.toml`, then managed policy, which overrides all three. Managed policy MUST be read from a documented system path per OS that the tenant cannot write. When that file or its directory is writable by the tenant, or the file is unparsable or invalid, every ParleyInn binary MUST refuse to start a B1–B3 component or `parinn run`, MUST keep the core recording, and MUST audit and count the condition. A tenant- or project-configuration change that adds or ends a pin, changes the deployment mode, loosens redaction, recall scope, rule levels or holds, or turns on a B1–B3 component MUST take effect only after a widening act under OWN-11 records its digest, or when managed policy sets it.                        | T    | I6, I7  | U1, U8     |
| ADM-05 | P0  | forward-only migrations after a verified backup                                    | Format changes to segments (REC-21) and schema changes to derived state MUST be versioned forward-only migrations, preceded by an automatic verified backup under ADM-06. ParleyInn MUST refuse to open a segment or store with a newer version than it supports.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | T    | I1      | U1         |
| ADM-06 | P0  | backup and restore with SQLite's online backup                                     | `parinn backup` MUST capture the sealed segments, the open segment up to a fresh seal, the payload store, derived state and the OPS-02 audit log with its hash chain as one consistent copy, without writer or device keys (SEC-10), and MUST record each backup as an audited event. `parinn restore` MUST be a widening act (OWN-11), MUST verify the copy and its audit chain with `parinn verify`, MUST reapply every purge, revocation, quarantine, pin end and rule tightening recorded after the backup was taken or refuse, MUST start a new writer for every local writer it restores, audited, and MUST never append to a restored writer's log or reuse a `seq`.                                                                                                                                                                                | T    | I1, I6  | U1, U3     |
| SEC-01 | P0  | shipped binaries open no listening socket and make no outbound connection          | (D1) The core binary `parinn` (B0) MUST NOT open any socket or make outbound connections. Components behind B1–B3 MUST ship as separate binaries the core neither links nor starts. `parinn ui` and `parinn run` (B1) MAY listen only on loopback or a `0600` Unix socket in `PARINN_HOME`, and MUST connect nowhere else. This MUST be enforced per binary by an import allow-list in CI and by suites run under each boundary's sandbox (ENG-12).                                                                                                                                                                                                                                                                                                                                                                                                        | T, I | I4      | U8, U1     |
| SEC-10 | P0  | the v1 core handles no credentials                                                 | The core MUST handle no credential other than its own writer key (REC-18) and, from PEER, the device and writer certificates (PRV-10). A key MUST be generated on the node, held in an OS key store the core reaches without a socket, cgo or a child process where one exists, named per OS, or else in a `0600` file, never inherited from the environment unless the tenant opts in (PEER-06), and never logged, exported, backed up or placed in a segment; `parinn status` MUST say when a key is a file an unsandboxed agent of the same user could read. Every credential of another binary (enrolment tokens, forge tokens, git-remote and notification credentials) MUST follow the same rules and be readable only by the binary that uses it.                                                                                                   | I    | I4      | U8, U1     |
| SEC-12 | P0  | quarantine by range, session, provenance, flag, time; `operator` event; immediate  | Quarantine MUST be available by address range, session, lane, writer, provenance class, flag and time window; a quarantine that removes a pin or any trusted event from a restore block MUST be a widening act (OWN-11); MUST be recorded as an `operator` event with the stated reason; MUST take effect immediately for all recall, landmark and injection operations on the node that records it; and release MUST be equally recorded. A quarantine reaches peers only as a request (PEER-11).                                                                                                                                                                                                                                                                                                                                                         | T    | I5      | U8, U4, U7 |
| SEC-17 | P0  | threat model maintained and reviewed each minor release                            | A threat-model document MUST be maintained in the repository and reviewed at every minor release. It MUST cover every row of the boundary register (SEC-19) and the actors acting across each, and MUST compare ParleyInn control by control with Zed Delta on record signing, co-author trust, central-service dependence and key custody.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | I    | —       | U8         |
| NFR-01 | —   | hook p95 budgets per hook                                                          | (D4) Hook wall-clock time at p95 on a project with 1M events†: `UserPromptSubmit` ≤ 50 ms; `SessionStart` ≤ 150 ms; `PostToolUse`, `Stop`, `SubagentStop`, `Notification` ≤ 100 ms; `PreCompact` ≤ 2 s; `SessionEnd` ≤ 1 s. Held `PermissionRequest`: ParleyInn's own work ≤ 50 ms† beyond the wait, and the wait itself ≤ the owner's hold window, capped 10 s† below the harness timeout. These budgets MUST hold with `parinn ui` open and ten harnesses writing.                                                                                                                                                                                                                                                                                                                                                                                       | A    | —       | U2, U9     |
| NFR-09 | —   | no resident process between sessions; per-hook RSS ≤ 50 MiB; store ≤ 1.5× text     | The core MUST leave no resident process between sessions; per-hook peak RSS ≤ 50 MiB†; store overhead ≤ 1.5× stored text†. `parinn ui`, `parinn run`, `parinn peer`, `parinn publish` and `parinn bridge` run only while the user runs them, each with peak RSS ≤ 256 MiB† and steady-state CPU ≤ 5% of one core† while idle, measured on the reference hardware; for `parinn run` these budgets MUST hold in total for ten concurrent instances, and it MUST add at most 10 ms† p95 to keystroke-to-echo latency.                                                                                                                                                                                                                                                                                                                                         | A    | —       | U1, U2     |
| ENG-12 | P0  | end-to-end suite in a network-denied sandbox                                       | The end-to-end suite MUST run each binary inside its boundary's sandbox: all network denied for `parinn`; all but loopback denied for `parinn ui` and `parinn run`; each of `parinn peer`, `parinn publish` and `parinn bridge` confined to its register rows. It MUST fail on any socket outside the boundary the register assigns (SEC-19).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | T    | I4      | U8, U1     |
| ENG-16 | P0  | CI gates, forbidden imports outside allow-listed packages                          | CI MUST gate on `go vet`, `staticcheck`, `gosec`, `errcheck`, `govulncheck` and custom analyzers, with one import allow-list per binary: `net`, `net/http` and `os/exec` forbidden in the core outside the kernel-worker re-exec; `os/exec` allowed only in `parinn run`; listening sockets allowed only in `parinn ui`, `parinn run` (loopback), `parinn peer` and `parinn publish`; outbound connections allowed only in `parinn peer`, `parinn publish` and `parinn bridge`.                                                                                                                                                                                                                                                                                                                                                                            | T    | I4      | U8, U1     |

### 6.2 The requirement tables

Sections 6.2 to 6.14 drafted the requirement rows. They landed, and were
revised since, in [§5](../../docs/srs/05-functional-requirements.md),
[§5b](../../docs/srs/05b-lane-requirements.md),
[§5c](../../docs/srs/05c-principal-and-peer-requirements.md),
[§6](../../docs/srs/06-security.md),
[§7](../../docs/srs/07-non-functional-requirements.md),
[§9](../../docs/srs/09-interfaces.md) and
[§10](../../docs/srs/10-engineering-quality.md); the SRS is the only
source.

## 7. Boundary register

Landed in [§6.3 Boundary register](../../docs/srs/06-security.md).

## 8. Unified UX vocabulary

Landed in [§9b](../../docs/srs/09b-lane-vocabulary.md).

## 9. Id mapping table

"—" means the trace proposed nothing there. New ids start from the next
free number on disk and skip ASM-10..16, PRV-08, OQ-10..13, T10..T13
and the CUE family.

| Old forward-trace id          | Old backward-trace id        | New id                                 | Note                                                |
| ----------------------------- | ---------------------------- | -------------------------------------- | --------------------------------------------------- |
| —                             | —                            | REC-17                                 | per-event keyed commitment (security officer #1)    |
| REC-18 (origin signing)       | REC-17 (writer signing)      | REC-18                                 | the swap the persona review found; signing keeps 18 |
| —                             | —                            | REC-19                                 | seal cadence (multi-machine #2)                     |
| REC-17 (checkpoint)           | REC-18 (checkpoint)          | REC-20                                 | worktree checkpoints                                |
| —                             | REC-19 (evidence class)      | LANE-05                                | one evidence set                                    |
| C3 (REC-06, REC-10)           | C10 (REC-06, REC-10)         | REC-06, REC-10 changed                 | per-writer address and chain                        |
| C7 (REC-09, tombstone)        | C12 (REC-09, per-lane key)   | REC-09 changed, REC-17, ADM-07 changed | per-event key replaces tenant and lane keys         |
| C5 (PRV-02)                   | PRV-09 (imported untrusted)  | PRV-02 changed, PRV-09                 | §6.0 rule 1                                         |
| —                             | C8 (glossary Writer, Owner)  | §4                                     | glossary                                            |
| RCL-08 (lane scope)           | C8 (RCL-05 by session)       | RCL-05 changed                         | §6.0 rule 3                                         |
| ADM-13                        | —                            | ADM-13                                 | no git writes                                       |
| LANE-01                       | —                            | LANE-01                                | lane identity                                       |
| C6 (path of repository)       | C9 (HMAC of common git dir)  | LANE-02                                | repository root commit, path-free                   |
| LANE-02 (actor)               | —                            | LANE-03                                | actor                                               |
| LANE-03 (results)             | VIEW-02 (timeline by actor)  | LANE-04                                | results and hunk attribution                        |
| LANE-04 (level)               | P3, REC-19                   | LANE-05                                | `external` dropped                                  |
| LANE-05 (CRDT)                | —                            | LANE-09                                | automatic merge                                     |
| LANE-06 (gate)                | —                            | LANE-07                                | author rule, strict default                         |
| UI-01                         | VIEW-01                      | VIEW-01                                | one view                                            |
| UI-02                         | NFR-15 (1 s)                 | VIEW-02, NFR-15                        | 2 s from harness write; 1 s from ingest             |
| UI-03                         | —                            | VIEW-03                                | optional client                                     |
| —                             | VIEW-03 (requests first)     | VIEW-05                                | Needs you                                           |
| —                             | VIEW-04 (provenance shown)   | VIEW-07                                | trust marks                                         |
| UI-04 (steer via harness)     | VIEW-05 (writes as owner)    | OWN-02, OWN-03                         | steering family                                     |
| PEER-01                       | —                            | PEER-01                                | plus unattended start                               |
| PEER-02                       | —                            | PEER-02                                | —                                                   |
| PEER-03                       | —                            | PEER-03                                | —                                                   |
| PEER-04 (5 s from close)      | —                            | PEER-04                                | 5 s from write                                      |
| SEC-19 (register)             | I11                          | SEC-19; I11 folded into I4             | —                                                   |
| SEC-20 (UI loopback)          | SEC-19 (UI loopback)         | SEC-20                                 | plus argv and origin storage                        |
| SEC-21 (inert render)         | —                            | SEC-21                                 | —                                                   |
| SEC-22 (peer accept)          | SEC-20 (peer accept)         | SEC-25                                 | —                                                   |
| SEC-23 (publish)              | C16, OQ-15                   | SEC-26; OQ-15 kept                     | —                                                   |
| —                             | SEC-21 (per-lane encryption) | not adopted                            | REC-17 meets the purge goal; SEC-09 stays P1        |
| C2 (SEC-01)                   | C2 (SEC-01, CON-04, ENG-12)  | SEC-01, CON-04, ENG-12 changed         | per binary                                          |
| —                             | C11 (SEC-10)                 | SEC-10 changed                         | writer and device keys only                         |
| —                             | V3 (SEC-12)                  | SEC-12 changed                         | binds the recording node                            |
| C11 (SEC-17)                  | C15                          | SEC-17 changed                         | Delta comparison                                    |
| —                             | INJ-10 (sanitized names)     | INJ-10                                 | fingerprints only, off by default                   |
| —                             | P2 (REC-13 to P0)            | REC-13 changed                         | MUST at P1, not P0, since the view is P1            |
| —                             | P5 (SEC-13 to P0)            | not adopted now                        | SEC-13 stays P1; revisit in M8 when posts arrive    |
| C1 (I4)                       | C2 (I4, I11)                 | I4 changed                             | §3                                                  |
| C4 (glossary Record, ADM-08)  | C10 (Record, seq)            | §4, ADM-08 changed, I10 changed        | —                                                   |
| —                             | C8 (I8)                      | I8 changed                             | —                                                   |
| C8 (NG4)                      | C7 (NG4, OQ-07)              | NG4 changed                            | —                                                   |
| C9 (NG5)                      | C6 (NG5, Lane view)          | NG5 changed                            | —                                                   |
| C10 (§2.2)                    | C13 (§2.1, §2.2)             | §3.3                                   | platform team stays primary                         |
| C12 (§12: M7, M8, M9)         | P7 (renumbered M4–M7)        | §11                                    | existing ids kept; new M7–M9                        |
| ASM-17 (flush)                | —                            | ASM-17                                 | —                                                   |
| —                             | ASM-17 (common git dir)      | ASM-20                                 | root commit readable without a process              |
| S9                            | —                            | S9                                     | plus ASM-18                                         |
| OQ-14 (co-author path)        | OQ-14 (co-author path)       | resolved by OWN-08 (D2)                | OQ-14 is reused for scoped delegation               |
| —                             | OQ-16 (ACP)                  | OQ-16                                  | —                                                   |
| T14 (co-author injection)     | T15 (malicious peer)         | T14, T17                               | —                                                   |
| T15 (local web)               | T14 (local web)              | T15                                    | —                                                   |
| T16 (rendered content)        | —                            | T16                                    | —                                                   |
| T17 (forged segments)         | T15                          | T17                                    | —                                                   |
| T18 (erasure gap)             | —                            | T18                                    | —                                                   |
| —                             | T16 (forged verification)    | T19                                    | —                                                   |
| UX onboarding U1 (solo)       | —                            | persona U2                             | the onboarding draft's U-numbers are retired        |
| UX onboarding U2 (many nodes) | —                            | persona U3                             | —                                                   |
| UX onboarding U3 (reviewer)   | —                            | persona U5                             | —                                                   |
| UX onboarding U4 (OSS)        | —                            | persona U7                             | —                                                   |
| UX onboarding U5 (teammate)   | —                            | persona U6                             | —                                                   |
| UX onboarding U6 (returning)  | —                            | persona U4                             | —                                                   |
| UX attention classes P1–P4    | —                            | Q1–Q4                                  | no clash with priorities                            |

## 10. Persona finding dispositions

Kinds: **req** (requirement ids), **pitch**, **design** (this
document's §8 or the UX drafts), **plan** (§11), **not adopted** or
**in part** with the reason.

### 10.1 Merged findings (persona-review/README.md)

| #   | Finding, in short                                               | Kind        | Disposition                                                                                           |
| --- | --------------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------- |
| 1   | The traces conflict                                             | design      | One id set (§9) and one rule per question (§6.0)                                                      |
| 2   | The UX designs contradict each other                            | design      | §8: one vocabulary, queue order, keymap, Endorse, notices off, one review surface, phone answers only |
| 3   | Journeys have no ids or phase                                   | req, plan   | VIEW-08, VIEW-09, VIEW-10, VIEW-11, LANE-10, LANE-11, LANE-06, LANE-07, SEC-22; phase gates in §11    |
| 4   | Post notices push peer bytes into context                       | req         | INJ-10 off by default, local strings only; OWN-09                                                     |
| 5   | Steering launders untrusted text; Re-run here                   | req         | OWN-03, OWN-04, OWN-07, OWN-18, SEC-29                                                                |
| 6   | Trust across own nodes contradicts import rule                  | req         | PRV-02, PRV-10, OWN-17, PIN-11 (§6.0 rule 1)                                                          |
| 7   | Network surface beyond B0–B3                                    | req, design | §7 register; SEC-19, SEC-22, SEC-24, SEC-28; HTTPS import rejected (R1)                               |
| 8   | Purged content confirmable; no fleet purge; no import redaction | req         | REC-17, ADM-07, ADM-14, PEER-11, REC-23                                                               |
| 9   | Project identity                                                | req         | LANE-02, PIN-10                                                                                       |
| 10  | Verification classes forgeable or inconsistent                  | req         | LANE-05, LANE-07                                                                                      |
| 11  | Holds deny on timeout and may block the prompt                  | req         | OWN-06, OWN-07, NFR-01, ASM-18                                                                        |
| 12  | Phase 1 gates miss speed, weekend, review                       | req, plan   | NFR-01, NFR-15, ENG-29; §11.3                                                                         |
| 13  | Owner authentication gaps                                       | req         | OWN-11, SEC-20                                                                                        |
| 14  | Sandbox tail loss, unattended start, no fallback                | req, plan   | REC-19, PEER-01, PEER-05, PEER-08; sandbox sync and the git carrier move into M8                      |
| 15  | Author can approve; double review with a forge                  | req         | LANE-07, LANE-08                                                                                      |
| 16  | Lane handover undefined                                         | req         | LANE-11                                                                                               |
| 17  | Operator role demoted; no quotas, bounds, CLI view              | req, plan   | U1 primary (§5); SEC-22, SEC-23, ADM-15, ADM-16, OPS-06, NFR-09; fleet-rollout exit in M7             |

### 10.2 Agent (U9)

| #   | Finding, in short                                                   | Kind    | Disposition                                                                                                                                                                                                                                                        |
| --- | ------------------------------------------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | Blocking: steering puts owner-voice text outside TrustedText        | req     | OWN-03, OWN-04                                                                                                                                                                                                                                                     |
| 2   | Blocking: owner devices contradict trust; pins lost                 | req     | PRV-10, PIN-11                                                                                                                                                                                                                                                     |
| 3   | Waiting-message notice                                              | req     | INJ-10                                                                                                                                                                                                                                                             |
| 4   | Envelope lacks origin                                               | req     | RCL-09                                                                                                                                                                                                                                                             |
| 5   | Re-keying widens pins                                               | req     | PIN-10                                                                                                                                                                                                                                                             |
| 6   | Holds break budgets; expiry as fixed "held" text, never a rejection | in part | NFR-01 and OWN-06 adopted: without an away policy nothing expires. Not adopted: under "keep going" the harness gets a deny carrying the fixed held text and id, because no harness offers a "held" decision; a deny is the only value that lets the agent carry on |
| 7   | Watchdog nudge with no owner act                                    | req     | OWN-09; automatic nudges removed from the attention design                                                                                                                                                                                                         |
| 8   | One owner act, three classes                                        | design  | §6.0 rule 4                                                                                                                                                                                                                                                        |
| 9   | One address scheme                                                  | req     | RCL-08                                                                                                                                                                                                                                                             |
| 10  | Recall scope disagreement                                           | req     | RCL-05 (§6.0 rule 3)                                                                                                                                                                                                                                               |
| 11  | When steering ships                                                 | plan    | OWN is P1 in M7, not P2 in M9                                                                                                                                                                                                                                      |

### 10.3 Fleet developer (U2)

| #   | Finding, in short                                   | Kind        | Disposition                                                                                                                                                                                                                                          |
| --- | --------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Blocking: timeout denies; native prompt unusable    | req         | OWN-06, OWN-07, ASM-18                                                                                                                                                                                                                               |
| 2   | Blocking: `parinn run` needed per agent for control | in part     | OWN-15 states the missing controls at install. Not adopted: interrupt and stop for a plain-hook session, because hooks have no input path and OWN-03 forbids steering through hook output                                                            |
| 3   | UX files contradict each other                      | design      | §8                                                                                                                                                                                                                                                   |
| 4   | Overlapping edits flagged quietly                   | req         | LANE-13                                                                                                                                                                                                                                              |
| 5   | Shell answers need a code from a second screen      | in part     | OWN-12 as revised by R4-1 to R6-2: cut and neutral acts at the terminal, widening acts by per-act assertion. Not adopted: dropping the code, since an unsandboxed agent can fake a terminal and the code is the CLI's only defence short of presence |
| 6   | Notices on by default; recall raises prompts        | req, design | INJ-10 off by default; the plugin marks recall tools read-only so the owner can allow them once in the harness (I7 forbids ParleyInn changing the prompt itself)                                                                                     |
| 7   | No lane from an issue or worktree                   | req         | LANE-01                                                                                                                                                                                                                                              |
| 8   | Phase 1 never checks speed                          | req, plan   | VIEW-02, NFR-01; §11.3 gate                                                                                                                                                                                                                          |
| 9   | New token per launch breaks a pinned tab            | not adopted | A lasting credential widens T15. `parinn ui --print` and the stale-tab page naming the command stay (design)                                                                                                                                         |
| 10  | "Unsandboxed" on every tile                         | design      | One line on Health (§8.6)                                                                                                                                                                                                                            |
| 11  | Automation mode default on a workstation            | not adopted | PRV-04's default protects runners fed by issue text; Setup asks the question and shows the CLI diff (VIEW-18)                                                                                                                                        |

### 10.4 Live collaborator (U6)

| #   | Finding, in short                                                        | Kind   | Disposition                                |
| --- | ------------------------------------------------------------------------ | ------ | ------------------------------------------ |
| 1   | Blocking: handover undefined                                             | req    | LANE-11                                    |
| 2   | Blocking: words reach the agent unseen                                   | req    | RCL-11                                     |
| 3   | Blocking: no ids for invites, roles, endorse, status, presence, handover | req    | LANE-10, OWN-08, LANE-12, PEER-09, LANE-11 |
| 4   | Adopt versus Endorse                                                     | design | §8.10: Endorse                             |
| 5   | Requests ignored with no sign                                            | req    | LANE-12                                    |
| 6   | Joining is slow; no role on the invite                                   | req    | LANE-10                                    |
| 7   | Pairing talk mixed with requests                                         | req    | LANE-12                                    |
| 8   | Watcher cannot contribute                                                | req    | LANE-10                                    |
| 9   | Presence forgeable                                                       | req    | PEER-09                                    |

### 10.5 Multi-machine developer (U3)

| #   | Finding, in short                               | Kind      | Disposition                           |
| --- | ----------------------------------------------- | --------- | ------------------------------------- |
| 1   | Blocking: one repo, different projects per node | req       | LANE-02                               |
| 2   | Blocking: reclaimed sandbox loses its tail      | req, plan | REC-19, PEER-05; sandbox sync into M8 |
| 3   | Remote liveness unbounded                       | req       | PEER-04                               |
| 4   | No owner key hierarchy for relayed segments     | req       | PRV-10, PEER-07                       |
| 5   | No B2 confidentiality                           | req       | SEC-24                                |
| 6   | Concurrent metadata needs manual repair         | req       | LANE-09                               |
| 7   | No unattended peer start in a sandbox           | req       | PEER-01                               |
| 8   | One inbound-reachable node, no fallback         | req, plan | PEER-08; the carrier moves into M8    |
| 9   | Sandbox token cannot continue a lane            | req       | PEER-06                               |
| 10  | Sandbox trust marks disagree with C5            | design    | §6.0 rule 1, §8.6                     |
| 11  | Sandbox token is a bearer secret                | req       | PEER-06                               |
| 12  | REC-18 means two things                         | design    | §9                                    |

### 10.6 Open-source maintainer (U7)

| #   | Finding, in short                                     | Kind        | Disposition                                     |
| --- | ----------------------------------------------------- | ----------- | ----------------------------------------------- |
| 1   | Blocking: Re-run here feeds the contributor's command | req         | OWN-18                                          |
| 2   | Blocking: real versus fabricated lane                 | req         | LANE-15                                         |
| 3   | No redaction on import; private paths                 | req         | REC-23, SEC-26                                  |
| 4   | `parinn import https://…` breaks B0                   | req, design | REC-23; register R1                             |
| 5   | Contributor's sharing path unspecified                | req         | SEC-26 (plain file or git ref)                  |
| 6   | Trust rule disagreement; sender names in TrustedText  | req         | PRV-09, INJ-10                                  |
| 7   | Accepted contribution's link                          | req         | LANE-15, LANE-06                                |
| 8   | No reject action or reason                            | req         | LANE-15, VIEW-07                                |
| 9   | Adoption per displayed text                           | req         | OWN-08                                          |
| 10  | "From @kai" shows a sender's name                     | req, design | VIEW-07; petnames (§8.6)                        |
| 11  | Lane scope and foreign origins                        | req         | RCL-05, RCL-10                                  |
| 12  | Prompts left out of public bundles                    | open        | OQ-15; the withheld-count banner stays (SEC-26) |

### 10.7 Platform operator (U1)

| #   | Finding, in short                                | Kind      | Disposition                                                                                                                                      |
| --- | ------------------------------------------------ | --------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | Blocking: no policy lock on listeners            | req       | SEC-22                                                                                                                                           |
| 2   | Blocking: browser changes trust mode and peering | req       | SEC-23, VIEW-18                                                                                                                                  |
| 3   | Blocking: no fleet-wide purge                    | req       | ADM-14, PEER-11                                                                                                                                  |
| 4   | Blocking: role demoted; no rollout journey       | req, plan | U1 primary (§5); M7 exit demonstrates a managed-policy rollout                                                                                   |
| 5   | Peers fill disks                                 | req       | ADM-15                                                                                                                                           |
| 6   | No resource bounds for resident processes        | req       | NFR-09                                                                                                                                           |
| 7   | Failures only in a browser                       | req       | OPS-06, ADM-16                                                                                                                                   |
| 8   | Conflicting ids                                  | design    | §9                                                                                                                                               |
| 9   | Upgrade and restore undefined                    | req       | REC-21, LANE-02                                                                                                                                  |
| 10  | UI token in argv                                 | req       | SEC-20                                                                                                                                           |
| 11  | Phone over B2 not allowed                        | design    | Register: phone uses the SSH tunnel (row 11); phone view over B2 rejected (R2)                                                                   |
| 12  | No fleet-wide cap                                | in part   | ADM-15 quotas set by managed policy (SEC-22). Not adopted: a fleet-wide spend cap, since ParleyInn only estimates spend (VIEW-16); kept as OQ-26 |

### 10.8 Returning owner (U4)

| #   | Finding, in short                                  | Kind        | Disposition                                                                                 |
| --- | -------------------------------------------------- | ----------- | ------------------------------------------------------------------------------------------- |
| 1   | Blocking: journeys have no ids                     | req         | VIEW-08, VIEW-09, VIEW-10, VIEW-11                                                          |
| 2   | Blocking: phase 1 misses the weekend               | plan        | §11.3 recorded-weekend fixture                                                              |
| 3   | Blocking: cross-lane search forbidden              | req         | VIEW-09 (D6)                                                                                |
| 4   | Blocking: failed capture looks quiet               | req         | VIEW-08                                                                                     |
| 5   | "Verified" overclaims before signing               | req         | VIEW-10, REC-18                                                                             |
| 6   | Three catch-up designs                             | design, req | §8.9, VIEW-08                                                                               |
| 7   | Pitch dropped "tells you when anything is missing" | pitch       | §10.11 restores it                                                                          |
| 8   | "Since last visit" lives on one machine            | req         | VIEW-08                                                                                     |
| 9   | Catch-up lags; state the frontier                  | req         | VIEW-08                                                                                     |
| 10  | Are thinking blocks searchable                     | design      | Yes: they are `assistant` events, untrusted, indexed under REC-11 like other assistant text |
| 11  | Who writes stuck marks under NFR-09                | design      | Nobody: `stuck?` is computed in the viewer (§8.1); no resident watchdog                     |

### 10.9 Reviewer (U5)

| #   | Finding, in short                                | Kind   | Disposition                        |
| --- | ------------------------------------------------ | ------ | ---------------------------------- |
| 1   | Blocking: five vocabularies for what verified it | req    | LANE-05, LANE-07, §8.3             |
| 2   | Blocking: authors can approve                    | req    | LANE-07                            |
| 3   | Blocking: landed link has no id                  | req    | LANE-06                            |
| 4   | Gate is P2 but the pitch sells it                | pitch  | §10.11: "next"                     |
| 5   | Double review with a forge                       | req    | LANE-08                            |
| 6   | Re-run here runs untrusted text                  | req    | OWN-18, LANE-05                    |
| 7   | Strict landing should be the default             | req    | LANE-07                            |
| 8   | "Why" depends on marked decisions                | req    | LANE-04                            |
| 9   | Phone approval scope contradicts itself          | req    | OWN-16, OWN-17                     |
| 10  | Two review surfaces                              | design | §8.9                               |
| 11  | "Carried: rebase only" should name the tool      | design | "carried: empty range-diff" (§8.3) |

### 10.10 Security officer (U8)

| #   | Finding, in short                               | Kind        | Disposition                            |
| --- | ----------------------------------------------- | ----------- | -------------------------------------- |
| 1   | Blocking: erasure leaves confirming hashes      | req         | REC-17, REC-10, ADM-07                 |
| 2   | Blocking: peer bytes enter the model unasked    | req         | INJ-10, OWN-09, I2                     |
| 3   | Blocking: untrusted text laundered as operator  | req         | OWN-03, OWN-04, OWN-07, OWN-18         |
| 4   | Blocking: sockets beyond the register           | req, design | §7, SEC-19, SEC-24                     |
| 5   | Blocking: trust across nodes contradicts itself | req         | PRV-02, PRV-10 with I2 review (ENG-29) |
| 6   | Cookie leaks across loopback ports              | req         | SEC-20                                 |
| 7   | Same-user agent; presence                       | req         | OWN-11; actor and T21 in §6.12         |
| 8   | Verification from agent-written fields          | req         | LANE-05                                |
| 9   | Erasure end to end undefined                    | req         | SEC-30, PEER-11, VIEW-11               |
| 10  | Integrity cannot be verified alone              | req         | REC-19, SEC-27                         |
| 11  | Phase 1 builds a socket before review           | req, plan   | ENG-29; §11.3                          |
| 12  | Ids collide; UX cites backward numbers          | design      | §9                                     |
| 13  | Presence as unsigned hints                      | req         | PEER-09                                |
| 14  | LAN listener binding unstated                   | req         | SEC-24                                 |
| 15  | Desktop notifications push untrusted text       | req         | VIEW-17                                |

### 10.11 Pitch changes

Applied in pitch v9, with the second and third reviews' wording fixes
(R18, R3-17); the pull-request sentence was in plan.md and is fixed
there.

| Pitch text now                                                               | New text                                                                                                                          | Reason                              |
| ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| "ParleyInn keeps the full, tamper-evident record …"                          | "… keeps the full, hash-chained record … and tells you when any of it is missing or changed."                                     | Returning owner #7; REC-18, VIEW-08 |
| "what verified it: the agent's own claim, a local run, or CI"                | "what verified it: the agent's claim, a run on its own machine, a reviewer's re-run, or CI for that exact commit"                 | LANE-05                             |
| "Signed approvals and required checks on the lane" (table)                   | "Next: signed approvals and required checks on the lane"                                                                          | Reviewer #4; LANE-07 is P2          |
| "Your own nodes; erasure by key after the keyed-hash fix" (table)            | "Removal on your own nodes, with no hash left that confirms removed content; peers are asked to remove theirs"                    | REC-17, PEER-11; SEC-09 stays P1    |
| "That is a pull request in all but its interface."                           | "It holds what a pull request's conversation holds, plus the runs and results behind it."                                         | Reviewer #4; the gate is P2         |
| "Others can post to your lane; only you decide what becomes an instruction." | unchanged; add "Endorsing a post sends exactly what you saw, signed as yours."                                                    | D2, OWN-08                          |
| Decision 9: "Standalone means nothing leaves the machine."                   | "Standalone means ParleyInn sends nothing off the machine; what an agent recalls travels to its model provider as it does today." | I4 carve-out                        |

### 10.12 Second review (persona-review/round-2/README.md)

| #   | Finding, in short                                          | Disposition                                                                                                              |
| --- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| R1  | Owner acts outside OWN-11 and OWN-12                       | OWN-02, OWN-11 and OWN-12 cover every owner act the glossary lists; allow once is not a loosening; D5; OQ-18 closed      |
| R2  | Erasure leaves copies                                      | SEC-31; ADM-07 names segment files and payload references; PIN-03 stores a commitment; OQ-24 narrowed                    |
| R3  | Shallow clones mint a second identity                      | LANE-02: identity from the enrolment token or an owner bind; never minted without the bound commit                       |
| R4  | A lane started with capture broken is invisible            | VIEW-08 lists uningested transcripts and risen counters; VIEW-09 names capture gaps; VIEW-04 worst freshness             |
| R5  | Managed policy has no source and no ceiling                | ADM-04 adds the managed layer from a path the tenant cannot write; SEC-22 caps rule levels, away, hooks, holds; OWN-06   |
| R6  | Backup and restore assume SQLite; cloned images share keys | ADM-05, ADM-06: segments backed up, restore starts new writers; REC-24 mints a new key on a copied home                  |
| R7  | A bundle cannot be told real from fabricated               | SEC-26 keeps withheld headers; REC-23 refuses colliding lane ids and writers; LANE-15 checks a cross-signed git key      |
| R8  | Remote answering rests on ASM-18; parked approval lost     | OWN-15 states it at install if S9 fails; OWN-07 delivers the template on the next prompt or lists its absence            |
| R9  | Text paths into the agent                                  | I2 lists automatic writes as a closed set; OWN-07 away never starts a turn; OWN-08 sourced template, both texts          |
| R10 | `lane_status` fields; writer-chosen names                  | VIEW-15 closed field list in the envelope; LANE-01 random lane ids; VIEW-07 fingerprints                                 |
| R11 | Key chain gaps                                             | PRV-10 writer certificates and revocation by held set; SEC-10 keychain first; VIEW-08 and VIEW-10 off-node receipts      |
| R12 | Peer sync                                                  | REC-19 seals every hook; PEER-03 sealed ranges; LANE-01 merges concurrent lanes; LANE-10 review; PEER-08 encrypted       |
| R13 | Boundary register gaps                                     | Rows 25 and 26; `parinn publish` and `parinn bridge` (CON-02, ENG-16); SEC-29 listener; SEC-10 credentials; SEC-24 binds |
| R14 | Review gaps                                                | Bound human (glossary, LANE-07); LANE-05 `unbound`; VIEW-19; LANE-08 forge hand-off; LANE-06; §8.3 rank. OQ-22 stays     |
| R15 | Lane states without a source                               | OWN-21 Ready for review; LANE-01 branch moves; LANE-12 drops "seen"; LANE-13 acknowledgement; LANE-10, LANE-11 states    |
| R16 | Delivery                                                   | D7 keeps the CLI views P1; M8 exit criteria; ENG-29 defines a prototype; §11.3 applied to the phase files                |
| R17 | Uninstall; segment count                                   | ADM-02 lists every artifact; REC-25 merges segments                                                                      |
| R18 | Pitch overclaims                                           | Pitch v8                                                                                                                 |

Not adopted: the maintainer's ask that their journeys work in v1
(maintainer, round 2, finding 4). Import of a stranger's bundle is the
widest new input path, so REC-23, LANE-15 and SEC-26 stay P2 behind
their own proposal and ADR; the pitch names them as next.

### 10.13 Third review (persona-review/round-3/README.md)

| #     | Finding, in short                      | Disposition                                                                                                                       |
| ----- | -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| R3-1  | No owner act with only the core        | OWN-12: a confirmation typed at the controlling terminal, a code from `parinn ui` or `parinn run` when one runs, never stored; D5 |
| R3-2  | Branch switch or lane merge drops pins | PIN-10 restores every lane the session has belonged to; LANE-01 resolves merged ids                                               |
| R3-3  | Ingest trusts a stranger's transcript  | REC-22 and PRV-02: untrusted whatever the class; outside the harness's directory, a foreign lane                                  |
| R3-4  | Cross-signature runs the wrong way     | LANE-15 and SEC-26: a binding statement signed by the git signing key; foreign writers chain to the bundle's owner key            |
| R3-5  | Export bypasses review                 | SEC-26 and OWN-12: every export is a reviewed, audited owner act                                                                  |
| R3-6  | Sandbox needs its issuing peer         | PEER-06 token carries identity, delegation and delivery addresses; PEER-01 entrypoint; `enrolled, never synced`                   |
| R3-7  | Erasure copies                         | ADM-06 audits backups, keeps the audit log, reapplies purges; SEC-31 P0, backups, receipts; PEER-11 replicates every purge        |
| R3-8  | Keys and identity                      | PRV-10 revocation covers issued keys; REC-24 node identity; PEER-05 retired writers                                               |
| R3-9  | Managed policy and resources           | ADM-04 fails closed on invalid policy; SEC-22 retention and authenticator; SEC-29 one exception                                   |
| R3-10 | Lane membership                        | PEER-02 members only; LANE-16 roles; LANE-11 former owner, pins, expiry; LANE-12 routing; RCL-11 shown; LANE-10 first view        |
| R3-11 | Review gate                            | LANE-07 human verdicts only, changes block, checkpoint authorship; LANE-08 forge `asserted`; OWN-02 one key; OQ-22 closed         |
| R3-12 | Catch up                               | VIEW-08 receipt heads, removals, order; VIEW-04 `unrecorded`; VIEW-09 writer time; VIEW-10 sealed prefix; §8 normative (§11.3)    |
| R3-13 | Delivery and phase tests               | OWN-08 and LANE-16 into M8; M7 CLI exit; both phase files                                                                         |
| R3-14 | Hooks and holds                        | NFR-01 `Notification`; OWN-15 OQ-17; OWN-06 connected; OWN-16 allow once; VIEW-17 and row 6                                       |
| R3-15 | Agent-facing details                   | OWN-07 request ids; fingerprints, never petnames, on agent paths; §8.5 `unverified`; §9.3 `origin`                                |
| R3-16 | Peer details                           | PEER-04 relay hop; M8 exit; SEC-26 bind; rows 24 to 26; PEER-06 token act; PEER-09 presence                                       |
| R3-17 | Document defects                       | Rows rejoined to their tables; SEC-30 columns; §10.12 text; D7; pitch v9                                                          |

### 10.14 Fourth review (persona-review/round-4/README.md)

| #    | Finding, in short                                                      | Disposition                                                                                                                                                                                                    |
| ---- | ---------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| R4-1 | Terminal confirmation reopens T21; the code path makes the shell worse | Security wins by default (OWN-11, OWN-12, D5): terminal alone only for acts that cut; the harness's own prompt and `/pin` with the core alone; managed policy may opt in, with a Needs you line per act. OQ-29 |
| R4-2 | Another tenant's pins reach a restore block                            | PIN-10 restores only the session's own principal's trusted pins; LANE-11 re-signing; M2's exit and the phase-1 test switch branch                                                                              |
| R4-3 | An updated bundle is refused; the binding statement cannot be signed   | REC-23 accepts a continuing chain; LANE-15 detached signature made with the contributor's own git tooling; REC-22 `imported`; `parinn reject` row                                                              |
| R4-4 | Offline sandbox certification and retired writers                      | PRV-10 admits the token link; PEER-05 accepts continuing segments and offers each sealed range at once; PEER-06 carries lane pins, token source                                                                |
| R4-5 | `parinn run` cost and setup                                            | NFR-09 totals for ten instances and keystroke latency; OWN-15 launch path and an install opt-in                                                                                                                |
| R4-6 | Same-user tampering                                                    | VIEW-10 warns on a writable receipt path and shows a short code; SEC-10 names the key store per OS; §1.2 qualified                                                                                             |
| R4-7 | Agent-facing details                                                   | VIEW-15 recall addresses of waiting posts; PIN-10 names both merged ids; §9 note                                                                                                                               |
| R4-8 | Phase tests                                                            | Plan 2610022338's phase-1 test covers widening acts by a same-user process and a branch switch; M7 answers at the harness prompt                                                                               |

### 10.15 Fifth review (persona-review/round-5/README.md)

| #    | Finding, in short                                                        | Disposition                                                                                                                                                                                           |
| ---- | ------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| R5-1 | An agent can start its own `parinn ui` or `parinn run` and read the code | Three act classes (glossary, D5, OWN-11); widening acts need a WebAuthn assertion or the passkey login of a `parinn ui` session; OWN-02 drops `parinn run` as an owner surface; SEC-20, OWN-16; OQ-29 |
| R5-2 | Screen reads and keystroke injection; `/pin` restores lane-wide          | T21 names them; an unconfirmed `/pin` or `/unpin` stays in its session (PIN-10, OWN-11)                                                                                                               |
| R5-3 | Core-only gaps: ending a pin, marking ready, finding the prompt          | `/unpin`; marking ready is neutral; `parinn needs` names the terminal; install shows the policy diff; batch-clearing                                                                                  |
| R5-4 | Re-signing undefined; pin tests unprovable; trust by current mode        | PIN-10 re-sign act, recorded mode, filter first; M2, M8, M9 exits; phase-1 test checks old and new lanes' pins                                                                                        |
| R5-5 | Lists drift; weak receipt code; §1 unqualified                           | One act list with a drift gate (§11.3); 80-bit receipt code; §1 qualified                                                                                                                             |
| R5-6 | `lane_status` floods; pitch decision 9                                   | Range form capped at 20†; decision 9 reworded                                                                                                                                                         |

### 10.16 Sixth review (persona-review/round-6/README.md)

| #    | Finding, in short                                                | Disposition                                                                                                                 |
| ---- | ---------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| R6-1 | First enrolment undefined; revoking is a cut act                 | OWN-11: an attested hardware key or a root-owned policy entry; enrolling more or revoking is widening                       |
| R6-2 | A session login authorizes widening; GUI injection; shared RP ID | OWN-11: one assertion per widening act, bound to its digest and exact origin; OWN-12, OWN-16, SEC-20; T21                   |
| R6-3 | Quarantine and revocation remove pins on a cut act               | Glossary and SEC-12: any act that removes a pin or stops acceptance is widening                                             |
| R6-4 | Unasserted acts recorded as trusted; unlisted acts               | OWN-11 surface mark and untrusted free text; every listed act classified; unlisted means widening; the gate checks the body |
| R6-5 | Harness-prompt pins; reads skip the envelope                     | `/unpin` ends only its session's pins, `/pin` marked as harness input; non-terminal reads enveloped and logged              |
| R6-6 | Core-only clarity (fleet A to E)                                 | `/pin` and `/unpin` say their scope; `parinn lanes` names terminals; CLI widening verbs wait for the tab's assertion        |

### 10.17 Seventh review (persona-review/round-7/README.md)

| #    | Finding, in short                                      | Disposition                                                                                          |
| ---- | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------- |
| R7-1 | Writer-signed widening events need no assertion        | PRV-02: a widening event takes effect only with a verified stored assertion; `parinn verify` reports |
| R7-2 | Tenant configuration widens with no assertion          | ADM-04: widening configuration changes need an OWN-11 act recording their digest, or managed policy  |
| R7-3 | Attestation roots too loose                            | OWN-11: roots compiled in, limited to device-enforced user verification                              |
| R7-4 | Restore, command confirmation, witness runs, rotation  | Classified widening; ADM-06 restore reapplies later tightenings                                      |
| R7-5 | T21 gaps; clearing without an authenticator; SSH phone | T21 extended; terminal clearing; same-port forward; residual risks listed after T24 for OQ-29        |

### 10.18 The stakeholder's OQ-29 decision

On 3 October 2026 the stakeholder put sandboxing on its own arm, plan
2610031222, and decided that for an unsandboxed session the residual
risks are accepted rather than fought. The proposal follows:

- D5 is decided: owner acts rest on the sandbox; widening acts need
  only an authenticated surface while every session blocks the
  residual risks, and the owner's recorded acceptance otherwise.
- OWN-22 is new: the sandbox state of each session, from a source the
  agent cannot write, and the risk acceptance, which managed policy
  may forbid.
- OWN-11, OWN-12, OWN-16, PRV-02, SEC-20 and SEC-22 drop the mandatory
  per-act WebAuthn assertion; it stays as hardening the owner or
  managed policy may require, with the enrolment rules of round 6.
- Rounds 4 to 7 of the persona review stand as the analysis behind the
  decision: they established that no software-only check holds against
  an unsandboxed agent.

### 10.19 The stakeholder's decisions on D1 to D7

Recorded on 3 October 2026:

| Decision        | Answer                                                                                                                              |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| D1, I4          | Restate I4 by network boundary, B0 to B3                                                                                            |
| D1, record      | One signed log per writer; the requirements name no technology, and ADRs choose formats and algorithms                              |
| D2              | One principal per agent, endorsement, notices off; granted roles later (OQ-14)                                                      |
| D3              | An owner key certifies device keys; the phone only allows once or denies                                                            |
| D4              | No deny on a timeout without an away policy; steering through the harness's input; OQ-17 yes                                        |
| D5              | Decided earlier through OQ-29 (§10.18)                                                                                              |
| D6              | Search scope is the person's choice on every search; an agent selects its own under RCL-05                                          |
| D7              | Open, for a longer discussion                                                                                                       |
| ENG-29 reviewer | The stakeholder, @jeduden                                                                                                           |
| D1, packaging   | Not decided: boundaries hold per component and per process, so one executable or several both fit (ADR-2610032155, change 3; OQ-32) |

The packaging answer came during the security review: a process runs
exactly one component, no core process starts another, and the
boundaries hold whether the components behind B1 to B3 ship in one
executable or several. Which one waits for the tech re-evaluation
(OQ-32). The rows above that still say "separate binary", or name a
mechanism, are superseded by SRS 2.0-draft's SEC-01, SEC-19, SEC-28,
SEC-29, PEER-01, CON-02, CON-04, ENG-12 and ENG-16.

## 11. Delivery plan changes for §12

Landed in [§12](../../docs/srs/12-delivery-plan.md).

## 12. Open questions that remain

Landed in [§13](../../docs/srs/13-open-questions-and-risks.md).
