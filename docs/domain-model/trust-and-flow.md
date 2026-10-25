---
title: "Trust and flow"
order: "09"
summary: >-
  Trust and how content reaches an agent: trust levels and sources, recall, endorsement, trust grants, delegation, rule levels, away policies, fixed templates, sandbox state and risk acceptance, quotas and spend, counters, canary and stats, run status and room status, the Needs you queue and focus set, overlap, forks, presence hints and petnames.
---
# Trust and flow

- **Trust level**: `trusted` or `untrusted`, per event for one principal's
  agents on one node, derived by the trust policy from the event's provenance,
  origin (as Origin derives it, PRV-13), recorder, writer
  and recorded deployment mode, for `user` the attended mark its hook
  observation recorded, the node's key set, and that principal's stamps and
  trust grants as its writer logs carry them (PRV-02, PRV-04, I10).
- **Trusted sources**: The sources I2 lists, whose events the trust policy
  classifies as `trusted` (PRV-02), narrowed in a foreign room (RCL-10) and by a
  missing attended mark (PRV-04);
  everything else is untrusted.
- **Deployment mode**: `interactive` (a person types at the harness) or
  `automation` (a pipeline does, the default), set per node by managed policy
  or accepted configuration (PRV-02, PRV-04, ADM-04, `node.deployment_mode`).
- **Trust policy**: The rule that derives each event's trust level (PRV-02); no
  settings layer changes it.
- **Trust mark**: The sign beside an item saying where it came from and whether
  it is trusted, from §9.7.6's closed set (VIEW-07).
- **Taint**: The trust level a derived artifact, or an output built from events
  (a recall result, kernel output, an exported bundle, rendering or trusted-only
  export), inherits: untrusted when any event it derives from is untrusted or
  it holds a cut or neutral act's free text (OWN-11), except for its sanitized
  structural fields.
- **Recall taint**: A run's mark after it recalls untrusted content, which
  tightens the rule levels of the action classes SEC-13 configures as
  sensitive (OWN-10). It passes from run to run, whatever agent each belongs
  to, as SEC-13 says.
- **Principal surface**: An authenticated client where principal acts are taken:
  the browser room view under SEC-20, a phone's browser included before the
  peer component (§6.3 row 11), the CLI or TUI at a terminal, or a paired phone
  within its scope. An unsandboxed agent can still pass
  the CLI's or TUI's terminal test, through a terminal it opens or a process it
  did not start: that is §6.1's residual risk R7, and its writing the store or
  signing outside ParleyInn's code is R3, which only a sandbox removes (Sandbox
  state).
- **Presence proof**: A hardware-backed, user-verified proof of a person's
  presence, bound to one principal act (OWN-11, PRV-12).
- **Recall**: An agent's call to a **recall tool**: an MCP tool that returns
  enveloped content (§9.2), such as `event_search`, or `kernel_exec` through
  its table functions (CMP-03), each call of which is a recall, or a
  read-only CLI verb whose output is not a terminal (OWN-12); each records
  a recall event (RCL-07). Pull-only, defaulting to the agent's current
  run. A recall tool that stops at its per-call cap returns a
  **continuation cursor** (`next_cursor`); passed back as `cursor`, it
  continues without gap or overlap (RCL-03). What else a tool call may return,
  RCL-04 says.
- **Closed path**: One of the ways I2 lists by which ParleyInn writes to an
agent
  without the agent's recall, such as a restore block, an opt-in notice or a
  delegated task in its template. No other write to an agent exists (I2).
- **Recall scope**: `run`, `room` or `rooms`, or a foreign room named in the
  call (RCL-05). A wider scope is said to extend recall; "widening" belongs to
  acts.
- **Endorsement**: A principal act sending a post's text, exactly as the
  principal confirmed it, to one of that principal's own agents, inside a fixed
  template naming the author's seat key fingerprint and the post's address
  (OWN-08).
- **Trust grant**: A principal's widening act trusting another principal's key
  for its own agents, in one room or everywhere, its revocation a cut act
  (OWN-29). It covers that principal's posts and pins written from device seats
  a device key of that principal certified, within that key's device scope
  (PRV-10), never a cross-room post shown in another room, run seats,
  token-key-only nodes or room summaries, and nothing in a foreign room but the
  pins a leave, kick or bar keeps there (Foreign room). OWN-29 says which
  service accounts it may name and when a recorded one covers nothing (Service
  account).
- **Delegation**: One agent handing another a delegated task: its subagent, or,
  under a delegation grant, another agent of the same principal, an agent on
  another of its nodes, or another principal's agent (OWN-23 to OWN-26). How
  a delegated task and its delegate report travel between two principals'
  nodes rests on OQ-14.
- **Delegated task**: The text of a delegation: a subagent's from its harness,
  any other in OWN-24's template.
- **Delegate report**: What a delegate returns, recorded on the delegating side
  so it stays in the delegating run's recall scope (OWN-24); it reaches the
  delegating agent as OWN-25 says.
- **Delegation grant**: The delegating principal's widening act naming who may
  delegate, to which targets, at what rule level, within what **delegation
  budget** (its cap in spend), how many delegates at once, how deep, and until
  when (OWN-23).
- **Acceptance grant**: The receiving principal's widening act on its own node,
  naming the delegating principal, the targets, the maximum rule level, the
  delegation budget and the expiry (OWN-26). Delegation to another principal's
  agent needs both a delegation grant and an acceptance grant.
- **Permission grant**: The grant that lets an approved action pass again when
  the harness next raises it at its permission prompt (OWN-07). Separate from
  trust and delegation grants.
- **Notice allowance, notice opt-in**: The owner's per-room allowance and the
  agent's principal's opt-in. An opt-in notice needs both.
- **Action class**: A kind of tool action, such as edits, commands or network
  use (OWN-10).
- **Rule level**: For each action class, one of, loosest to tightest: act
  without asking, act when told, ask first, hand off (OWN-10). A maximum rule
  level is the loosest allowed.
- **Quota**: A limit on storage or events, per room, node, writer received from
  a peer, peer or worktree checkpoint, that the node's principal or managed
  policy sets; ADM-15 says what reaching one does (I1, I6, I10).
- **Spend**: What runs cost in model tokens or money; no quota limits it until
  OQ-26 closes.
- **Away policy**: A principal's opt-in choice of what an unanswered held
  request does while it is on: keep going, pause or stop (OWN-07). Only that
  principal act sets it, never a settings key.
- **Residual risk**: One of the risks §6.1 lists for an unconfined run.
- **Sandbox state**: What confines a run, its policy digest, and which residual
  risks it blocks, recorded from outside the sandbox, where the agent cannot
  write; a run with none recorded counts as unsandboxed (OWN-22). A residual
  risk counts as blocked only by a sandbox state its measuring component signed
  out of the agent's reach (open: OQ-40); until then a run leaves it open
  (OWN-22, PRV-08).
- **Risk acceptance**: The principal's act accepting the residual risks a run
  leaves open (OWN-22).
- **Hand-off, hand-back**: An agent passes control to its principal as a held
  request; the principal returns control with a note and a worktree checkpoint
  (OWN-20).
- **Run controls**: Steer, interrupt, pause, resume and stop: principal acts on
  a run. **Terminal takeover**, the principal typing in the harness's terminal
  the launcher hosts, is the harness's own channel, never a principal act
  (OWN-02, OWN-19).
- **Fixed template**: Wording ParleyInn ships, filled only with ids, counts,
version
  numbers, key fingerprints, addresses and the principal-typed, endorsed or
  delegated text, or the text of a post a trust grant covers, its requirement
  names; **principal-typed text** is text a principal typed at a principal
  surface for that act.
- **Correction, retry**: After a verdict: principal-typed text in a fixed
  template, or a new run from a worktree checkpoint, whose join to the room the
  retry itself asks for (OWN-28, LANE-23).
- **Counter**: A count of dropped, rejected, redacted, truncated, coalesced,
  timed-out or failed operations of one kind, each written to the node's
  append-only **audit log** of ParleyInn's own operations (OPS-01), shown by
  `parinn status` (ADM-11), failing `parinn doctor` until acknowledged
  (`parinn counter ack`, I6, OPS-03). The audit log also records every purge
  (ADM-07).
- **Structured log**: A line for each operation ParleyInn's components perform,
  one JSON object a line in `logs/parinn.jsonl` or on stderr, a hook handler's
  only in the file, every field redacted first (ENG-05, OPS-05). It is
  neither the audit log nor a writer log: it holds no counter and no hash
  chain, and nothing derived is built from it.
- **Canary**: `parinn canary`'s end-to-end test: it writes a **canary event**
  through capture's ingest path and recalls it through the MCP server (OPS-04).
- **Stat**: A count derived from the record within a recall scope, such as
  events, runs, compactions or recalls, read through `stat_list` (RCL-01).
- **Run status**: A run's one status from §9.7.1's closed set (one waiting on
  its principal is Asking, never Needs you), with its **freshness mark**, which
  says how current the run's events are on this node, whether ingest keeps up
  with its transcript (`unrecorded`), or that it is an ingested run
  (`ingested`); `stuck?` is a watchdog observation (VIEW-04).
- **Room status**: A room's one status from §9.7.2's closed set, such as
  Running, Quiet or Ready for review, computed where shown and never recorded
  or set directly (VIEW-04); OWN-21's ready and abandoned marks feed it.
- **Queue class**: One of Needs you's classes Q1 to Q4, which set its order
  (§9.7.4, VIEW-05).
- **Focus set**: The rooms a principal marks to come first in Needs you, changed
  by a recorded neutral principal act, so every device shows one order.
- **Open room**: A room with no ready mark whose branch heads still stand
  (OWN-21) and no abandoned mark, that names no branch or names one that has not
  landed.
- **Exposure**: Of a branch, the untrusted and flagged items its runs read and
  their recall taint (VIEW-13).
- **Overlap**: Runs in two open rooms editing one file, which raises a Needs you
  item for each room's owner (LANE-13).
- **Fork**: Either of two events one writer sealed at one seq, both kept for
  forensics; the node's principal chooses which fork to keep (PEER-10).
  Unqualified, "fork" means this; the harness's own is always a harness fork.
- **Presence hint**: An ephemeral sign that a seat is connected, or typing (a
  **typing hint**), never stored in the record (PEER-09).
- **Watchdog observation**: ParleyInn's note, computed where shown and never
  recorded, that a run looks stuck, shown as the `stuck?` freshness mark; it
  never starts or resumes a turn (OWN-09).
- **Petname**: A name the person viewing chose for a key or a room, never one
  another principal sent (VIEW-07).
