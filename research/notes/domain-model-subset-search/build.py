import json, os, collections
HERE = os.path.dirname(os.path.abspath(__file__))

def E(stem, *terms):
    return [f"{stem}: {t}" for t in terms]

H = lambda *w: [f"hub-relation: {x}" for x in w]

# ---------------- CORE ----------------
core_just = {}
def J(stem, pairs):
    for term, why in pairs:
        core_just[f"{stem}: {term}"] = why

J("principals-and-agents", [
 ("Principal", "The party every invariant protects (I1, I2, I8); every agent, home and key belongs to exactly one."),
 ("Person", "A single-node install's principal is a person; no one certifies its key, and only a person records a verdict."),
 ("Managed policy", "I1, I4 and I7 name it; its item also defines harness configuration, the thing I7 governs."),
 ("Agent", "What restore blocks and recall serve; the reason ParleyInn exists."),
 ("Run", "The unit the record is keyed by and a restore block is built for (I1, I3)."),
 ("Subagent", "The harness starts subagents; each has its own run and writer tied by a parent link, or their events are lost (I1)."),
 ("Ingested run", "`parinn ingest` recovers what capture missed (I1, I9 fail-open); such runs must be named and untrusted (REC-22)."),
 ("Author", "Every event and pin has one authoring seat; which pins qualify to restore depends on it (PIN-10, I3)."),
 ("Member", "Whether a seat is a member decides where a run's events go and which pins restore (hub Relations)."),
 ("Owner", "The principal owns its personal room; ownership gives its device seats every capability there."),
 ("Delegate", "A subagent is a delegate; recording its delegated task and report needs the term (OWN-24, OWN-25)."),
])
J("harness-facts", [
 ("Harness", "What ParleyInn records; I7 governs its configuration."),
 ("Harness session", "The harness unit that yields a transcript and keys a run."),
 ("Transcript", "The source ingest reads (REC-01, REC-03)."),
 ("Hook", "How the harness reports to the core's hook handlers; capture rides on it."),
 ("Turn", "User turns are an I2 trusted source in interactive mode; spans and turn triggers follow turns."),
 ("Compaction", "What I3 restores after; compaction guidance is an I2 closed path."),
])
J("places", [
 ("Home", "I8's unit of isolation, bound to one OS user."),
 ("Node", "I1 and I10 are stated per node: one home on one machine."),
 ("Device", "A node is a device; its device key certifies its seats (PRV-11)."),
 ("Room", "Every pin lives in exactly one room; the unit of seats and writers."),
 ("Personal room", "Where every run's events and roomless pins go; the one room a single-node install needs."),
 ("Foreign room", "Core Relations on restore and recall name it; it keeps rooms the principal has no seat in untrusted and out of extended recall (I8, RCL-10)."),
 ("Principal's rooms", "Bounds recall (hub Relations; I8 'nor extends their recall')."),
])
J("record", [
 ("Record", "The set of writer logs I1 and I10 speak of; holds the settings layers beside it."),
 ("Writer", "The append-only, hash-chained log I1's address names."),
 ("Event", "What I1 says is never lost."),
 ("seq", "Half of I1's stable address; gap-free and never reused."),
 ("Address", "I1's (writer, seq); what recall, restore blocks and gap markers point at."),
 ("Payload", "Large events are stored whole outside the event, or I1 fails for them (REC-09)."),
 ("Segment", "The sealed unit of a writer; seals are made where segments close (REC-18, REC-19)."),
 ("Seal", "A seat key's signature over the chain head: the record's tamper evidence."),
 ("Commitment", "The only way the chain refers to content, so redaction and erasure never break it (REC-17)."),
 ("Provenance", "The class the trust policy reads to decide I2's trusted sources."),
 ("Origin", "Witnessed, ingested or received: I2's 'this node's own' depends on it."),
 ("Structural field", "Restore blocks are built only from these (I2)."),
 ("Structural event", "One of I2's trusted sources."),
 ("Capture", "The hook handlers recording runs; I1 needs it, and turning it off is widening."),
 ("Ingest marker", "Lets a hook budget cut ingest short without loss (I1, I9)."),
 ("Derived artifact", "I10's subject."),
 ("Redaction", "I1's first exception; a security-first record keeps no secrets (SEC-08)."),
 ("Flag", "Marks injection-pattern text so derived text carries only its counts (PRV-07); supports I2."),
 ("Quarantine", "I5's subject."),
 ("Gap marker", "What stands where content is purged, quarantined, truncated or missing, so no loss is silent (I1, I6)."),
 ("Integrity status", "Shows a writer's chain and seal state, making tampering visible (VIEW-10, I10)."),
])
J("pins-and-context", [
 ("Pin", "What I3 restores verbatim."),
 ("Pin version", "The immutable text stored and restored verbatim; edits add versions (I1, I3)."),
 ("Configuration pin", "PIN-01's baseline: the principal's declared constraints, authored by its own device seat and trusted on its node."),
 ("Pin priority", "Orders pins into the pin budget (PIN-08); I3's omission rule needs an order."),
 ("Budget", "I3's pin budget and I9's defined budgets."),
 ("Pin type", "Decides which pins restore (constraint, preference, intent)."),
 ("Unpin", "Ends a pin's restoring while its versions stay (I1)."),
 ("Compaction summary", "The harness's summary is recorded untrusted and never restored (I2, I3)."),
 ("Envelope", "I2's untrusted-data envelope, the only way recalled content reaches a model."),
 ("Envelope warning", "The fixed sentence that heads every envelope (I2)."),
 ("Trusted text", "The only text the restore builder accepts (INJ-03); I2's mechanism."),
 ("Restore block", "I2's first closed path and I3's vehicle."),
 ("Recall hint", "The restore block's fixed pointer to the recall tools (INJ-01)."),
 ("Model", "What I2 protects."),
 ("Working view", "Names the context window, never authoritative; why the record is."),
 ("Qualified requests", "Naming rule: 'request' never stands alone; costs nothing."),
])
J("seats-and-keys", [
 ("Seat", "The unit of membership, signing and authorship; every writer belongs to one."),
 ("Run seat", "Where a run's events go (hub Relations); its key seals the run's writer."),
 ("Device seat", "The seat for acting without a run: principal acts and device-seat pins."),
 ("Room id", "Names a room; part of every seat id and address form."),
 ("Seat id", "Derived from room id and first key; never chosen."),
 ("Seat key", "Signs room acts and seals the writer; the writer id derives from it."),
 ("Device key", "Certifies the node's seats from the first release (PRV-11)."),
 ("Seat certificate", "Makes a seat's kind provable; defines the key set I10 reads."),
])
J("acts-and-roles", [
 ("Room act", "How pins and other room state change (LANE-31)."),
 ("Principal act", "How the principal acts (device-seat pins, quarantine, capture); I2's 'on a principal act'."),
 ("Cut:", "OWN-11's classes tell widening from narrowing; every act is classed."),
 ("Neutral:", "OWN-11's classes tell widening from narrowing; every act is classed."),
 ("Widening:", "OWN-11's classes tell widening from narrowing; every act is classed."),
 ("Role", "What a seat may do; a run's personal-room seat is a contributor."),
 ("Contributor:", "Carries the pin capability and work for run seats."),
 ("Capability", "The closed set roles grant (LANE-16)."),
 ("Work", "The capability event routing depends on (hub Relations)."),
 ("Active pin", "I10 lists active pins."),
 ("Room merge", "Derives room state deterministically (I10)."),
 ("Concurrent", "The causal order the room merge needs (I10)."),
 ("Room state", "I10 lists it; the pin list derives from it."),
])
J("git-and-forge", [
 ("Qualified links", "Naming rule; its parent link ties a subagent's run to its parent's."),
])
J("trust-and-flow", [
 ("Trust level", "I2's trusted/untrusted, per event."),
 ("Trusted sources", "I2's list, as PRV-02 applies it."),
 ("Deployment mode", "I2 trusts user events only in interactive mode."),
 ("Trust policy", "The rule deriving trust levels (PRV-02, I10)."),
 ("Trust mark", "Shows where an item came from and whether it is trusted (I8 attribution, VIEW-07)."),
 ("Taint", "Recall results and other outputs inherit untrusted (I2)."),
 ("Principal surface", "Where principal acts are taken: the CLI or TUI at a terminal."),
 ("Recall", "I2's pull-only path; 'exact recall on demand'."),
 ("Closed path", "I2's closed set of writes to an agent."),
 ("Recall scope", "Bounds recall (I8: nothing extends it on its own)."),
 ("Delegation", "Names the harness's subagent hand-off the record captures (OWN-24)."),
 ("Delegated task", "A subagent's task text is recorded under its own provenance."),
 ("Fixed template", "I2's only shape for a write on a principal act; defines principal-typed text."),
 ("Counter", "I6."),
 ("Canary", "Proves capture and recall end to end, so a silent break shows (I6, OPS-04)."),
 ("Run status", "I10 lists run statuses; the `unrecorded` freshness mark shows hooks stopped arriving (I6)."),
])
J("components-and-surfaces", [
 ("Component", "I4's closed set of parts."),
 ("Core (B0):", "I4's B0: the hook handlers, CLI, MCP server, TUI; the only builder of what reaches the model."),
 ("harness adapter", "The core's per-harness transcript and hook parsing."),
 ("Boundary", "I4's network boundaries."),
 ("Room view", "The client of the record that the CLI and TUI implement as reduced clients."),
 ("Health:", "I6: failures visible to the node's principal."),
 ("Setup:", "I7: every configuration change shown first."),
])
for h, why in [
 ("A principal owns any number", "Ownership and an agent's principal following from its node (I8)."),
 ("A run has its personal-room", "A run's seats, starting with its personal-room seat."),
 ("A restore block carries the", "PIN-10: which rooms' qualifying pins a restore block carries (I3)."),
 ("Ingest splits one transcript by", "Ingest per run (I1)."),
 ("Every seat belongs to one", "Seat-principal-room-writer structure behind every address."),
 ("Each event goes to exactly", "Event routing: every event in exactly one writer (I1, I10)."),
 ("A run's history spans its", "A run's history across writers; recall by address."),
 ("Principals and agents create rooms;", "ParleyInn never creates a room on its own; the personal room comes from install."),
 ("A pin naming no room", "Roomless pins land in the personal room."),
 ("Recall extends only to the", "Recall scope bound (I8)."),
]:
    core_just[f"hub-relation: {h}"] = why

core = {
 "id": "core",
 "name": "Core: lossless record, verbatim pins, enveloped recall on one node",
 "description": "One principal, one node, one personal room: hook capture into hash-chained sealed writers, provenance and trust policy, pins from the principal's device seat restored verbatim in restore blocks, pull-only enveloped recall, quarantine, counters and Health/Setup, the B0 core only.",
 "entries": list(core_just.keys()),
 "requires": [],
 "requires_any": [],
 "soft": [],
 "value": 5,
 "risk": 3,
 "notes": "States I1-I10 for a single principal on a single node: I1 (address, writer, seq, redaction), I2 (trusted sources, envelope, recall, restore blocks, compaction guidance, fixed templates), I3 (pin, budget), I4 (B0 core only; B1-B3 absent), I5 (quarantine), I6 (counter, Health), I7 (harness configuration, Setup, managed policy), I8 (home), I9 (budgets, fail-open), I10 (derived artifacts, room merge, key set). The Cut/Neutral/Widening and Room act lists enumerate acts of optional features; with a feature absent those list items are simply unused.",
 "justification": core_just,
}

# ---------------- FEATURES ----------------
F = []
def feat(id, name, desc, entries, requires, value, risk, notes, requires_any=None, soft=None):
    F.append({"id": id, "name": name, "description": desc, "entries": entries,
              "requires": requires, "requires_any": requires_any or [], "soft": soft or [],
              "value": value, "risk": risk, "notes": notes})

feat("git-tracking", "Git tracking",
 "Repository identity, branches, commits, worktrees and worktree checkpoints recorded as events.",
 E("git-and-forge", "Repository", "Branch", "Commit", "Worktree") + E("record", "Worktree checkpoint"),
 [], 4, 2,
 "No invariant names it directly; worktree checkpoints are events under I1, their diffs redacted (I1 redaction). Binding a repository identity by hand is a widening act.")

feat("work-rooms", "Work rooms",
 "Rooms beyond the personal room: create, join, leave, branch links routing a run's events by branch, visibility, title/labels/assignments/stakes, room trailers and landing links with proof classes.",
 E("acts-and-roles", "Join", "Viewer:") + E("places", "Visibility") +
 E("pins-and-context", "Title, labels", "Assignment", "Stake") +
 E("git-and-forge", "Landing", "Landing link", "Proof class", "Room trailer"),
 ["git-tracking"], 5, 3,
 "I10 lists room state and proof classes; I8 'nor extends their recall' bounds what joining does. Room trailers put room ids in commit messages (disclosure surface). Visibility changes are widening.",
 soft=["shared-rooms", "bridges", "results-and-verdicts", "git-carrier", "publishing", "blind-peer"])

feat("shared-rooms", "Shared rooms (multi-principal)",
 "Rooms with other principals' seats: admission, invites and invite links, bars by principal key.",
 E("acts-and-roles", "Admission", "Bar"),
 ["work-rooms", "principal-keys"], 4, 5,
 "I8 (content another principal wrote kept as theirs, attributed to its seat key, untrusted unless stamp or trust grant), I2 ('anything another node or principal produced'). Needs a transport for other principals' nodes to hold the room. Invite review step applies redaction (SEC-08).",
 requires_any=[["peer-sync", "git-carrier"]],
 soft=["access-tokens", "moderation", "ownership-transfer", "trust-grants"])

feat("moderation", "Moderation and appointments",
 "The moderator role, kick, mute, list removal, and appointing a run seat or another principal's device seat as moderator within the appointment rate.",
 E("acts-and-roles", "Moderator:", "Kick", "Mute", "Appointment") + E("pins-and-context", "List removal"),
 ["work-rooms"], 3, 2,
 "I10 room state (roles, appointments, mutes). Room acts are governance, not I2 trust. List removal never stops a stamped version restoring (LANE-32).",
 soft=["conversation", "stamps", "needs-you", "facilitator", "shared-rooms"])

feat("facilitator", "Facilitator and room summaries",
 "A service account's device seat appointed as the room's facilitator, writing untrusted room summaries that agents read only through room_summary_get.",
 E("principals-and-agents", "Facilitator") + E("pins-and-context", "Room summary"),
 ["moderation", "service-accounts", "conversation", "shared-rooms"], 2, 3,
 "I2: a room summary (`summary` provenance) is never trusted; the restore block carries only its pointer by id. Trust grants never cover room summaries (OWN-29).")

feat("ownership-transfer", "Handover and succession",
 "Transfer a room's ownership by offer and acceptance, or to a named successor once every owner seat has left.",
 E("acts-and-roles", "Successor", "Handover"),
 ["shared-rooms"], 2, 3,
 "No invariant names it; changes which pins restore (intent restores only to stampers' agents until the new owner revises it). Handover offer expiry uses expire acts (PRV-10).",
 soft=["principal-keys", "stamps", "intent-criteria"])

feat("stamps", "Stamps",
 "A principal stamps one pin version after seeing its exact text, author and key, so it restores verbatim to that principal's own agents (run-seat, token-key-only and other principals' pins).",
 E("acts-and-roles", "Stamp"),
 [], 4, 3,
 "I2 ('a pin version that principal stamped'; 'a pin version a principal stamped, for that principal's own agents'), I8 ('unless this principal's own stamp ... covers it'). A human-reviewed widening path from untrusted pin text into restore blocks.",
 soft=["needs-you", "shared-rooms", "access-tokens"])

feat("intent-criteria", "Intent and criteria",
 "A room's lead intent pin (goal, criteria, paths), changed only by the owner, restored to agents; criteria with stable ids.",
 E("pins-and-context", "Intent", "Criterion"),
 ["work-rooms"], 4, 2,
 "I3 (intent is a type that restores, verbatim). I2: restores as its newest version's author wrote it, only to that author's principal's agents, stampers' agents or trust-grant holders.",
 soft=["ownership-transfer", "trust-grants", "results-and-verdicts", "pin-candidates"])

feat("pin-candidates", "Pin candidates",
 "Agent-suggested or ParleyInn-detected proposed pin text that the principal confirms into a device-seat pin.",
 E("pins-and-context", "Pin candidate"),
 [], 3, 2,
 "I2: an agent's suggestion is an untrusted `assistant` event; only the principal's widening confirmation makes a trusted pin. I10: a detected candidate is a derived artifact.",
 soft=["intent-criteria"])

feat("conversation", "Conversation and presence",
 "Posts, comments on marked ranges, drafts and cross-room posts, plus ephemeral presence and typing hints.",
 E("pins-and-context", "Post") + E("trust-and-flow", "Presence hint"),
 [], 4, 2,
 "I2 and I8 name 'posts ... written from a device seat'; posts reach an agent only by recall, endorsement or a trust grant. Presence hints are never stored (outside I1).",
 soft=["work-rooms", "endorsement", "trust-grants", "peer-sync"])

feat("endorsement", "Directed posts and endorsement",
 "A post directed to an agent waits in Needs you; its principal endorses it, sending the exact confirmed text into the agent's input inside OWN-08's template.",
 E("pins-and-context", "Directed post") + E("trust-and-flow", "Endorsement"),
 ["conversation", "needs-you", "launcher"], 3, 4,
 "I2 ('a post a principal endorsed exactly as shown inside the template of OWN-08'). Delivered only through the harness's own input (OWN-03), hence the launcher.")

feat("notices", "Opt-in notices",
 "Trusted fixed-text notices that posts or a delegate report wait or a room changed, sent only while the owner's allowance and the principal's opt-in both stand.",
 E("pins-and-context", "Opt-in notice") + E("trust-and-flow", "Notice allowance, notice opt-in"),
 [], 2, 2,
 "I2 closed path 'opt-in notices (INJ-10)', built only from fixed text, structural fields and ids.",
 soft=["conversation", "delegation-grants", "work-rooms"])

feat("held-requests", "Held requests and hand-off",
 "Permission requests, questions and hand-offs held with stable ids, answerable from any principal surface within a hold window; permission grants; hand-back with a note and worktree checkpoint.",
 E("pins-and-context", "Held request") + E("trust-and-flow", "Permission grant", "Hand-off, hand-back"),
 [], 4, 3,
 "I2 closed path 'the fixed templates of OWN-04' (hook permission decisions) and principal-typed replies; I9 (the hold window is bounded; on failure the harness's own prompt answers). Replies and hand-back notes need the launcher (OWN-03), else shown unavailable (OWN-15).",
 soft=["launcher", "git-tracking", "needs-you", "away-policy", "paired-phone"])

feat("away-policy", "Away policy",
 "A principal's opt-in answer for unanswered held requests: keep going, pause or stop.",
 E("trust-and-flow", "Away policy"),
 ["held-requests"], 3, 4,
 "I2 closed path 'the fixed templates of ... OWN-07'. Never starts or resumes a turn. A later allow reaches the agent through OWN-03 (launcher).",
 soft=["launcher"])

feat("rule-levels", "Rule levels and recall taint",
 "Per action class: act without asking, act when told, ask first or hand off; a run that recalled untrusted content gets sensitive classes tightened.",
 E("trust-and-flow", "Rule level", "Recall taint"),
 ["held-requests"], 4, 3,
 "No invariant names it; it bounds what an agent does after untrusted content reached it by recall (I2's residual risk). Loosening is widening; repository configuration only tightens.",
 soft=["delegation-grants", "launcher"])

feat("launcher", "Launcher and run controls",
 "B1 `parinn launch`: starts, hosts and controls runs, steer/interrupt/pause/resume/stop, terminal takeover, corrections and retries, carrying in only text the core built.",
 E("components-and-surfaces", "Launcher (B1):") + E("trust-and-flow", "Run controls", "Correction, retry"),
 [], 4, 4,
 "I4 B1 ('the launcher carries into the harness's input only text the core built'), I2 ('through the harness's own input, only principal-typed text, a fixed template ..., or a post a principal endorsed'). The only component that starts programs. Corrections answer verdicts; retries start from worktree checkpoints.",
 soft=["results-and-verdicts", "git-tracking", "witness-check", "sandbox-risk", "browser-room-view"])

feat("sandbox-risk", "Sandbox state and risk acceptance",
 "Record what confines each run and which residual risks it leaves open; refuse widening acts until the principal accepts open risks.",
 E("places", "Sandbox") + E("trust-and-flow", "Residual risk", "Sandbox state", "Risk acceptance"),
 [], 3, 1,
 "No invariant names it; it is a gate on every widening act (OWN-22) and managed policy may forbid risk acceptance. Dropping it removes that gate. Sandbox state is `harness_meta` recorded from outside the sandbox (launcher, OQ-40).",
 soft=["launcher"])

feat("presence-proof", "Presence proofs",
 "Hardware-backed authenticators giving user-verified presence proofs bound to one widening act.",
 E("trust-and-flow", "Presence proof") + E("seats-and-keys", "Authenticator"),
 [], 2, 1,
 "No invariant names it; it hardens principal acts (OWN-11, OWN-12) and paired-phone allows (OWN-16). Mitigation, not surface.",
 soft=["browser-room-view", "paired-phone"])

feat("delegation-grants", "Delegation under grants",
 "Agents delegate tasks to other agents of the same principal under a delegation grant (targets, rule level, budget in spend, concurrency, depth, expiry); delegate reports return via delegation_get.",
 E("trust-and-flow", "Delegation grant", "Delegate report", "Spend"),
 ["launcher", "rule-levels"], 3, 4,
 "I2 ('Under a delegation grant its principal recorded (OWN-23) ... a delegated task inside the fixed template of OWN-24'). Delegates inherit recall taint and never get a looser rule level.",
 soft=["notices", "access-tokens", "acceptance-grants"])

feat("acceptance-grants", "Cross-principal delegation",
 "Delegation to another principal's agent, which needs that principal's acceptance grant on its own node.",
 E("trust-and-flow", "Acceptance grant"),
 ["delegation-grants", "principal-keys"], 2, 5,
 "I2 ('for an agent of another principal, an acceptance grant that agent's principal recorded (OWN-26)'). OWN-26 must pass a recorded I2 security review.",
 requires_any=[["peer-sync", "git-carrier"]])

feat("trust-grants", "Trust grants",
 "A principal trusts another principal's key for its own agents, in one room or everywhere: that principal's device-seat pins restore and its posts arrive in OWN-29's template.",
 E("trust-and-flow", "Trust grant"),
 ["principal-keys", "shared-rooms"], 3, 5,
 "I2 ('what a trust grant of that principal covers'; 'ParleyInn applies such grants and never grants trust itself'), I8 ('unless this principal's own stamp or trust grant covers it'). Never in a foreign room; refuses relaying service accounts.",
 soft=["conversation", "service-accounts", "launcher"])

feat("principal-keys", "Principal keys and expire acts (PRV-10)",
 "A principal key certifies device keys, so principal acts and device-seat pins from the principal's other devices are trusted within device scope; expire acts end bars, mutes, offers, tokens and grants.",
 E("seats-and-keys", "Principal key") + E("acts-and-roles", "Expire act"),
 [], 4, 3,
 "I2 and I8 ('acts signed by a device key it certified', 'once PRV-10 ships'), I10 (the node's key set). Enabler for multi-device trust, bundles and multi-principal features.",
 soft=["moderation", "ownership-transfer", "access-tokens", "delegation-grants", "acceptance-grants"])

feat("service-accounts", "Service accounts",
 "Non-human principals with their own principal key, certified by a person, another service account or managed policy, marked whether they relay others' text.",
 E("principals-and-agents", "Service account"),
 ["principal-keys"], 3, 3,
 "I8 (its own principal: its content is kept as its own). Managed policy may certify by listing; unmarked listings count as relaying.",
 soft=["trust-grants", "ci-attestation"])

feat("access-tokens", "Access tokens and ephemeral nodes",
 "Short-lived access tokens to enroll devices or peers, carry invite links, and certify token-key-only ephemeral nodes' seats through token keys.",
 E("seats-and-keys", "Access token", "Token key"),
 ["principal-keys"], 3, 4,
 "I10 key set (token certificates). A token-key-only node signs no principal acts; its pins restore only once stamped (I2). Expiry by expire act retires its writers.",
 requires_any=[["peer-sync", "git-carrier"]],
 soft=["stamps", "segment-exchange", "shared-rooms", "delegation-grants"])

feat("segment-exchange", "Segment exchange base",
 "Holding other nodes' writers: forks kept for forensics with the principal choosing one, and retiring a writer at a last accepted seq.",
 E("trust-and-flow", "Fork") + E("record", "Retire a writer"),
 [], 1, 2,
 "I1 (both forks kept), I10 ('independent of the order in which logs arrived'). Base gene shared by peer sync and the git carrier; no value alone.")

feat("peer-sync", "Peer sync",
 "B2 peer component: encrypted, mutually authenticated sync of sealed ranges with peers enrolled by key; the Peers surface; serves paired phones.",
 E("components-and-surfaces", "Peer component (B2):", "Peers:") + E("places", "Peer"),
 ["segment-exchange"], 5, 4,
 "I4 B2 ('nodes and paired phones the node's principal enrolled by key, off until that principal turns it on'), I1 ('on every node that holds its writer's log'), I5 ('a quarantine reaches another node only as a quarantine request'), I8 (another node's content kept as theirs), I10 (order independence).",
 soft=["principal-keys", "purge-retention", "paired-phone", "blind-peer", "store-protection"])

feat("blind-peer", "Blind peers",
 "A peer that stores and serves a room's segments encrypted to device keys it does not hold, reading none of them.",
 E("places", "Blind peer"),
 ["peer-sync", "work-rooms"], 2, 2,
 "I4 B2. Allowed only as the room's visibility allows; a room held only blind is never foreign there.")

feat("paired-phone", "Paired phone",
 "A homeless device over B2 that reads and allows or denies held permission requests within its device scope.",
 E("places", "Paired phone"),
 ["peer-sync", "held-requests"], 3, 4,
 "I4 B2 ('nodes and paired phones the node's principal enrolled by key'). Keys and writer location rest on OQ-39.",
 soft=["presence-proof", "access-tokens"])

feat("git-carrier", "Git carrier (B3 publish component)",
 "Segment exchange through a namespaced location of the principal's git remote, run by the B3 publish component, whose model item this gene carries.",
 E("components-and-surfaces", "Publish component (B3):"),
 ["segment-exchange"], 3, 3,
 "I4 B3 ('outbound exchange with hosts the node's principal names, off until turned on'). Segments encrypted to the room's principals' device keys (PEER-08). Also hosts read-only publishing (see publishing).",
 soft=["work-rooms", "publishing"])

feat("publishing", "Read-only publishing",
 "Serve a room's bundle read-only through the publish component, as its visibility allows; no list item of its own (defined in Export and Publish component).",
 [],
 ["git-carrier", "export-bundles", "work-rooms"], 2, 4,
 "I4 B3 ('read-only publishing'). Requires git-carrier only because that gene carries the shared Publish component item. Publishing is widening with a review step (SEC-26).")

feat("bridges", "Bridges (forge, CI, notification)",
 "B3 bridge component: the forge bridge reading pull requests, reviews and forge checks (asserted), the CI bridge fetching attestations, and the notification bridge.",
 E("components-and-surfaces", "Bridge component (B3):") + E("git-and-forge", "Forge", "Pull request"),
 ["git-tracking"], 3, 4,
 "I4 B3 ('outbound exchange with hosts the node's principal names'); I2: bridge events take provenance `web`, always untrusted.",
 soft=["ci-attestation", "needs-you", "work-rooms"])

feat("results-and-verdicts", "Results, comparisons and verdicts",
 "Checks and results with evidence classes, own checks, branch comparisons with exposure, outcomes, and a person's met/not met/needs changes verdict per criterion.",
 E("git-and-forge", "Check", "Result", "Evidence", "Evidence class", "Own check", "Comparison", "Outcome", "Verdict") + E("trust-and-flow", "Exposure"),
 ["intent-criteria", "work-rooms"], 4, 2,
 "I10 ('evidence and proof classes'). Verdicts are neutral principal acts by persons only; ParleyInn never derives one.",
 soft=["witness-check", "ci-attestation", "room-view-surfaces", "rule-levels", "launcher"])

feat("witness-check", "Witness checks",
 "Re-run a check through the launcher on a fresh checkout of the exact commit, network denied, by a node whose git identity authored no commit on the branch.",
 E("git-and-forge", "Witness check"),
 ["results-and-verdicts", "launcher"], 3, 3,
 "I10 evidence classes; I4 (runs through the launcher, network denied; refused where it cannot be denied). Starting one confirms a command from an untrusted event (OWN-18).")

feat("ci-attestation", "CI attestation",
 "CI keys the owner enrolls in a room sign a check's exit status for an exact commit; results resting on them are CI attested.",
 E("git-and-forge", "CI attestation") + E("seats-and-keys", "CI key"),
 ["results-and-verdicts", "bridges"], 3, 3,
 "I10 evidence classes. Enrolling a CI key is widening, revoking it cut.",
 soft=["service-accounts"])

feat("presentations", "Outcome window and presentations",
 "Seats present a dev server URL, build artifact, file or diff in the Room page's outcome window; moderators pick which shows.",
 E("acts-and-roles", "Present", "Pick") + E("git-and-forge", "Presentation") + E("components-and-surfaces", "Outcome window"),
 ["work-rooms", "room-view-surfaces"], 2, 2,
 "No invariant names it; a presentation's URL is inert text never opened by ParleyInn.",
 soft=["moderation", "facilitator"])

feat("needs-you", "Needs you queue",
 "One queue of items waiting on the principal, by queue class Q1-Q4, with a focus set ordering rooms and notifications carrying only petname or id, class and count.",
 E("components-and-surfaces", "Needs you:") + E("trust-and-flow", "Queue class", "Focus set") + E("pins-and-context", "Notification"),
 [], 4, 1,
 "I10 ('queues'). Notifications never reach a model nor accept answers.",
 soft=["bridges", "room-view-surfaces", "held-requests", "stamps"])

feat("room-view-surfaces", "Room view surfaces",
 "Fleet, Room page (Timeline, Review, Replay with context lens, verify/why, quarantine list and forensic view) and Catch up; petnames; the stuck? watchdog mark.",
 E("components-and-surfaces", "Fleet:", "Room page:", "Catch up:") + E("trust-and-flow", "Petname", "Watchdog observation"),
 [], 4, 1,
 "I5 forensics (quarantine list, forensic view), I6 (visible to the principal). Shown in the TUI (B0) or the browser room view.",
 soft=["results-and-verdicts", "needs-you", "browser-room-view"])

feat("room-status", "Room status and overlap",
 "Room status computed where shown (Running, Quiet, Ready for review ...), open rooms, and overlap of runs in two open rooms editing one file.",
 E("trust-and-flow", "Room status", "Open room", "Overlap"),
 ["work-rooms", "needs-you"], 3, 1,
 "Never recorded or derived (outside I10's list); ready and abandoned marks are neutral principal acts.",
 soft=["results-and-verdicts", "room-view-surfaces"])

feat("browser-room-view", "Browser room view",
 "B1 room-view component serving the browser room view on loopback, authenticated by launch/origin secrets, recording principal acts taken there; phone-scoped secret.",
 E("components-and-surfaces", "Room-view component (B1):"),
 [], 3, 3,
 "I4 B1 ('listen only on loopback ...'). A principal surface (SEC-20).",
 soft=["room-view-surfaces", "presence-proof"])

feat("landmarks", "Spans, landmarks and stats",
 "Spans over a run's events, structural landmarks rolled into tiers, the restore block's landmark index, and stats via stat_list.",
 E("record", "Span", "Landmark") + E("pins-and-context", "Landmark index") + E("trust-and-flow", "Stat"),
 [], 4, 1,
 "I10 ('landmarks', 'stats'), I5 (a span can be quarantined), I2 (landmark index in restore blocks only from structural fields; natural-language headlines never reach a restore block).")

feat("compute-kernel", "Compute kernel",
 "A hermetic kernel where an agent runs code over the record, with per-run kernel variables, step budget and output caps.",
 E("record", "Kernel"),
 [], 3, 3,
 "I4 (the kernel worker is B0: no socket), I9 (step budget, wall-clock and memory limits), I2 (kernel output is tainted and enveloped; built-ins leave summaries out).")

feat("purge-retention", "Purge and retention",
 "Delete content by principal act or under a retention policy, leaving a tombstone; purge and erasure requests.",
 E("record", "Purge", "Retention policy"),
 [], 4, 3,
 "I1 ('data the node's principal explicitly purges, or removes under a retention policy that principal or managed policy sets. Every exception is recorded'). The only way stored content is destroyed.",
 soft=["peer-sync", "store-protection"])

feat("store-protection", "Store protection",
 "Backups without keys, head and purge receipts kept apart from the home, encryption at rest, and storage/event quotas.",
 E("record", "Receipt", "Backup", "At-rest key") + E("trust-and-flow", "Quota"),
 [], 3, 2,
 "I8 (home bound with strict permissions; backups carry no keys), I6 (quota refusals counted), I9. Backup restore mints new seat keys (REC-24).",
 soft=["purge-retention", "peer-sync"])

feat("export-bundles", "Export and bundles",
 "Reviewed `parinn export` of a room as a signed bundle (file or git ref), a rendering, or a trusted-only export for downstream memory systems.",
 E("record", "Export", "Rendering", "Trusted-only export", "Bundle"),
 ["principal-keys"], 3, 4,
 "Data leaves the home as a file (outside I4's network channels); export is widening with a review step applying redaction (SEC-26, SEC-08). Outputs carry taint.",
 soft=["git-carrier", "publishing", "work-rooms"])

feat("bundle-import", "Bundle import",
 "Import another node's or principal's bundle into this record as a foreign room, matching pull-request authors through binding statements.",
 E("record", "Import") + E("principals-and-agents", "Pull-request author"),
 ["export-bundles", "git-tracking"], 3, 4,
 "I1 ('secrets removed by redaction ... on import'), I8 (content another principal wrote kept as theirs), I2 (origin `bundle`, foreign room: untrusted).",
 soft=["shared-rooms"])

out = {"core": core, "features": F}
json.dump(out, open(os.path.join(HERE, "features.json"), "w"), indent=1, ensure_ascii=False)
print(len(F), "features")
