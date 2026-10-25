# Pitch review: inconsistencies and contradictions

Lens: places where the pitch contradicts itself, the SRS contract
(I1–I10, non-goals, constraints, requirements), the plans, or the
research evidence. Line numbers refer to the files as read on
2026-10-02. "Pitch L" means the line in `pitch.md`.

Severity scale:

- **fatal**: the claim cannot be true unless the contract changes (an
  invariant, a non-goal or a P0 requirement), so it needs a security
  review and a new major version;
- **serious**: the claim is false or misleading as worded, or depends
  on a design decision nobody has made yet;
- **wording**: the claim can be made true by rewording alone.

Findings are ordered by severity.

---

## F1. "The core that feeds the model never touches the network" quietly narrows I4. Live, multiplayer sharing breaks I4 as written (fatal)

**Pitch:** "The core that feeds the model never touches the network"
(L26). The pitch also offers "Real time: everyone sees edits … as they
happen" (L13), "Multiplayer" (L14) and "a live, multiplayer PR" (L30).

**Conflicting sources:**

- `docs/srs/01-introduction.md:38`: I4 says "**ParleyInn** never talks to
  the network". Its meaning column says "No listening sockets, no
  outbound connections … Data leaves the machine **only** when Claude
  receives recalled content through a tool call."
- `docs/srs/06-security.md:55`: SEC-01 says "**Shipped binaries** MUST
  NOT open listening sockets or make outbound network connections."
- `docs/srs/02-context.md:57`: CON-04 forbids network access "in any
  binary that ships P0 features".
- `docs/srs/01-introduction.md:65`: NG4 puts "Multi-host shared stores"
  out of v1, because sharing "Multiplies blast radius; requires
  provenance-gated sharing design first (OQ-07)".
- `docs/srs/04-reference-architecture.md:98`: ADR-01, "Daemonless". It
  removes "the port, token, stale-daemon, and wedged-daemon failure
  classes". `parinn-sync` is a resident process on every host, and the
  relay and the lane server are servers.
- `plan/2610022338_parinn-network-side/plan.md:31`: "I4 is about the
  core binary, not the project". The plan rewrites the invariant rather
  than meeting it.

**Conflict.** As written, I4 is a property of ParleyInn and of the machine:
data leaves only through recall to the model provider. A live,
multiplayer lane means the record leaves the machine continuously,
through `parinn-sync` to a relay and on to other people's nodes. The
pitch's sentence is literally true of the planned design, because the
core reads an inbox and never opens a socket. But it gets there by
moving the boundary from "ParleyInn" to "the core that feeds the model".
The pitch then offers that narrower boundary as if it were the
contract. If the project ships `parinn-relay`, `parinn-sync` and a lane
server, SEC-01 is broken, since those are shipped binaries. The
network plan does admit the SRS change must come first
(`plan/2610022338_parinn-network-side/plan.md:65-67`). The pitch does
not admit it.

**Minimal fix.** Pick one:

- **Design decision.** Restate I4 as a boundary on the core, with its
  own invariants for the network components, and narrow SEC-01 and
  CON-04 to "the core binary". Lift NG4, and settle ADR-01 for the
  sync agent. All of this goes through security review and a major
  version, as CLAUDE.md requires.
- **Rewording.** Say "ParleyInn's core never touches the network; optional,
  separately reviewed companions move sealed, encrypted segments
  between machines", and mark live sharing as post-v1.

---

## F2. "A message from anyone except the lane's owner reaches an agent as untrusted" implies the owner is trusted. The contract forbids that by default, and forbids it entirely for messages that arrive over the network (fatal)

**Pitch:** "A message from anyone except the lane's owner reaches an
agent as untrusted" (L24-25). "only the owner is trusted" is the
pitch's implied trust rule.

**Conflicting sources:**

- `docs/srs/05-functional-requirements.md:40`: PRV-02 says the only
  trusted classes are `operator`, `harness_meta` and "`user` when the
  deployment mode is `interactive`".
- `docs/srs/05-functional-requirements.md:42`: PRV-04 makes deployment
  mode "`automation` by default, in which `user` is untrusted (prompts
  on runners are often generated from external content such as issue
  text)".
- `docs/srs/05-functional-requirements.md:43`: PRV-05 says
  "Configuration MUST NOT be able to classify any provenance class as
  trusted beyond PRV-02."
- `plan/2610012322_parinn-for-agent-fleets/plan.md:55-56` and
  `plan/2610022338_parinn-network-side/plan.md:83-84` both adopt the
  rule "only the lane owner's messages are trusted".
- `plan/2610022338_parinn-network-side/plan.md:44-46`: "A compromised
  network component can only deliver bytes … treats every foreign
  event as untrusted".

**Conflict.** There are three problems.

1. Fleets run on runners and cloud sandboxes, which use the default
   `automation` mode. In that mode even the owner's prompts are
   untrusted. The pitch states the opposite as the general rule.
2. An owner message typed into the lane server's web chat reaches the
   agent through network components. It is not the harness's own
   `user` provenance. To trust it, the design needs a new trusted
   class whose origin is the network, and PRV-05 forbids that.
3. If such messages are trusted, the claim that "a compromised network
   component can only deliver bytes" fails. A compromised lane server
   or relay can deliver bytes that claim to be from the owner. Unless
   the owner signs each message end to end, with a key the lane server
   never holds, a compromised server can forge trusted input. That
   breaks I2.

**Minimal fix.** Make a design decision and write it into the SRS
change:

- The owner's messages are trusted only when they enter through the
  harness's own local prompt path, in `interactive` mode.
- Anything typed into the lane server, including by the owner, is
  untrusted, or is trusted only when signed by an owner key that the
  core verifies and the server never sees.

Then reword the pitch: "Only what the owner types into their own
session is trusted; every message that crosses the network reaches an
agent as untrusted data."

---

## F3. "Chat: people talk with the agents … in the branch" and "Multiplayer" collide with pull-only recall. Under I2 an agent never notices a collaborator's message (serious)

**Pitch:** "people talk with the agents working on the branch" (L12).
"several agents and people work one lane together" (L14-15). "nothing
stored is injected without being asked for" (L25).

**Conflicting sources:**

- `docs/srs/01-introduction.md:36`: I2 says untrusted content "reaches
  the model only when Claude explicitly calls a recall tool".
- `docs/srs/05-functional-requirements.md:91`: INJ-04 says injection on
  `UserPromptSubmit` "MUST be disabled by default", and that if
  enabled it carries only `TrustedText`.
- `docs/srs/04-reference-architecture.md:100`: ADR-03 rejected
  "Per-prompt memory hints".
- `docs/srs/05-functional-requirements.md:41`: PRV-03 makes `assistant`
  output untrusted. Agent A's output therefore reaches agent B only by
  pull, too.
- `docs/srs/06-security.md:67`: SEC-13 sets a recall-taint flag once
  untrusted content has been returned. The example policy hook then
  requires approval for sensitive actions.

**Conflict.** Chat means the other side hears you. Under I2, a message
from a collaborator, or from another agent in the lane, reaches the
model only if the agent decides on its own to call recall. The agent
has no reason to call recall, because telling it "you have a new
message" mid-session is exactly the per-prompt injection that INJ-04
and ADR-03 rule out. So either:

- agents never notice collaborators, and "chat" and "work together"
  become comments the agent may or may not read; or
- ParleyInn adds a push notification. That is an automatic path, and even
  a sanitized one is a new injection surface that needs an SRS change.

And when the agent does read a collaborator's message, SEC-13 taints
the session, and further sensitive actions wait for approval. That is
collaboration that slows the agent down, which sits badly with I9.

The pitch's own sentence "nothing stored is injected without being
asked for" is what makes its "Chat" bullet hollow. The suspected
contradiction holds.

**Minimal fix.** A design decision, plus rewording:

- Decide whether a structural, `TrustedText`-only "N unread lane
  messages; call `parinn.lane_messages`" notice is allowed. If it is,
  it needs its own requirement, a scenario and an audit trail, along
  the lines of INJ-04.
- Reword the bullet: "Chat: people post to the lane; agents read posts
  when they choose to, always as untrusted data; only the owner
  directs them."

---

## F4. "Code and talk can never drift apart" and "the record never drifts from the code" contradict "the code lands in git as an ordinary commit" (serious)

**Pitch:** "so code and talk can never drift apart" (L6-7). "the code
lands in git as an ordinary commit" (L19). "the record never drifts
from the code" (L30-31).

**Conflicting sources:**

- `plan/2610012322_parinn-for-agent-fleets/plan.md:24-27`: "the
  conversation and the code changes in one order, **never two
  histories joined by a link**. Git stays where code lands." The plan
  contradicts itself: once code lands in git, git's history and the
  record are two histories, joined by landed commit ids (`plan.md:53`).
- `plan/2610012322_parinn-for-agent-fleets/plan.md:50-52`: the plan
  itself cites "links broken by squash".
- `research/notes/agent-session-storage-sweep/catalog.md:367-371`,
  Lesson 9: attribution is "a derived index with **typed
  abstention** … say 'not proven' rather than match on timing".
- `research/reports/agent-session-storage-beyond-git.md:154`, rule 11:
  link from evidence the record already holds, and never depend on a
  trailer.

**Conflict.** After landing, git is a second history that ParleyInn does
not control. Many things change code without the record seeing them:

- a squash merge or rebase on the forge, which creates a SHA the
  record never observed;
- a revert, a cherry-pick, or a conflict resolved in the forge's UI;
- a human edit made after the lane closed;
- a later force-push.

From then on, the link is an inference, and the research says it
should be allowed to say "not proven". "Never drifts" is therefore
false by the project's own evidence. Crypto-shredding (F5) adds a
second kind of drift: the code stays in git while the story of how it
was made gets a hole.

**Minimal fix.** Reword: "The record holds the edits and the commits
it saw, and links later git history to them from evidence, saying
'not proven' when it cannot." Also drop "never two histories" from the
fleets plan (`plan.md:26`), or define the lane as ending at the landed
commit id.

---

## F5. "sensitive content can be erased by key" is in no SRS requirement or plan, and the current SRS leaves guessable hashes behind. The EDPB does not accept key deletion as erasure (serious)

**Pitch:** "The lane's record is tamper-evident, and sensitive content
can be erased by key" (L26-27).

**Conflicting sources:**

- `research/reports/custom-storage-git-and-chat-interfaces.md:220`:
  "Today's purge leaves unsalted content hashes behind". REC-10's
  chain includes the payload hash, REC-09 keys payloads by SHA-256,
  and the §8.2 tombstone keeps a "hash of removed content". "anyone
  holding the record can test a guess at short content".
- `docs/srs/04-reference-architecture.md:63-66` and
  `docs/srs/08-data-and-storage.md:42-45`: a tombstone carries "a hash
  of the removed content", and purge keeps each row's `hash`.
- `docs/srs/06-security.md:63`: SEC-09, encryption at rest, is only P1
  (M5). `docs/srs/13-open-questions-and-risks.md:16`: OQ-04, key
  management, is open. No requirement mentions per-session keys or
  shredding.
- `research/reports/custom-storage-git-and-chat-interfaces.md:159` and
  `:163-169`: the EDPB guidelines v2.0 (7 July 2026) say "encrypted
  personal data is still personal data" (para 51). Personal data
  should be kept off an immutable ledger even in encrypted or hashed
  form (para 104). Unsalted hashes are "generally insufficient" (para
  52). The report's advice: "describe it as a strong technical
  measure. It should not claim GDPR-compliant erasure." See also
  `:253`.
- `research/reports/custom-storage-git-and-chat-interfaces.md:261-263`:
  "Copies that leave custody cannot be shredded", including the model
  provider, git objects pushed to a forge and other people's clones.
- `research/reports/custom-storage-git-and-chat-interfaces.md:226`
  and `:280`: data keys are wrapped to every authorised device, and "a
  revoked device keeps what it had". Key loss leaves an unresolved
  choice between I1 and I9.
- Neither `plan/2610012322_parinn-for-agent-fleets/plan.md` nor
  `plan/2610022338_parinn-network-side/plan.md` has a task for a
  keystore or for shredding.

**Conflict.**

1. **As specified today**, purge is deletion plus an unsalted
   commitment. So "erased" content stays confirmable by guessing, which
   is the Matrix `hashes` leak.
2. **"Erased" is a legal word** that the July 2026 EDPB text declines
   to grant to key deletion.
3. **Multiplayer makes this worse**:

  - every participant's node holds wrapped keys and plaintext
     projections;
  - the model provider got whatever was recalled;
  - anything that became code is in git.

   Within one lane, "erased by key" means "erased where ParleyInn has
   custody, on devices not yet revoked".
4. **No plan builds the feature.**

The suspected conflict between "tamper-evident" and "erased" does not
hold in principle. A chain that commits to keyed hashes of ciphertext
keeps verifying after a shred (`custom-storage…md:214-218`). It holds
only against the current SRS hash choices. So that part of the
suspicion is dropped, and the unsalted-hash part stands.

**Minimal fix.**

- **Design decision.** Adopt the research's rules as an SRS change:
  keyed payload addresses and tombstone hashes, per-session data keys
  kept outside the record, and a shred that deletes the ciphertext and
  appends a signed tombstone. Add a plan task for it.
- **Rewording.** "Sensitive content can be crypto-shredded wherever
  ParleyInn holds it; copies already sent to the model, pushed to git or
  held by other participants cannot be recalled."

---

## F6. "every edit, every tool run and every result" is not what the record can capture (serious)

**Pitch:** "That record holds the conversation, every edit, every tool
run and every result" (L5-6).

**Conflicting sources:**

- `docs/srs/05-functional-requirements.md:18`: REC-01 makes the record
  an ingest of Claude Code transcripts. No requirement captures edits
  made outside a transcript. Worktree checkpoints appear only in the
  fleets plan (`plan.md:52-53`), with no SRS id.
- `research/notes/agent-session-storage-sweep/catalog.md:324-329`,
  Lesson 2: Claude Code checkpointing's "file history is
  **tool-scoped**".
- `research/notes/agent-session-storage-sweep/catalog.md:61-65` and
  `:316-323`: native transcripts "are not durable records". Claude
  Code deletes them after 30 days, CCHV rewrites them in place, and
  resumed sessions copy messages into new files.
- `docs/srs/02-context.md:43-44`: ASM-04 (transcripts can be truncated
  or rewritten) and ASM-05 (they can be deleted after 30 days).
- `plan/2610012322_parinn-for-agent-fleets/plan.md:39-40`: sandboxes
  are reclaimed, but §2.2 assumes durable volumes.
- `docs/srs/09-interfaces.md:24`: `SessionEnd` gets about 1 s ("ingests
  what fits"). `docs/srs/04-reference-architecture.md:106`, ADR-09:
  the record is lossless only "with respect to post-redaction content".

**Conflict.** "Every edit" leaves out several kinds of change:

- edits a human makes in an editor. The pitch's own "humans" in the
  lane do not work through tool calls.
- edits made through Bash: `sed -i`, code generators, formatters,
  `git checkout` and `git stash`. These show up as commands, not as
  patches.
- changes from other processes, such as watchers and builds.
- the tail of a session in a cloud sandbox that is reclaimed before
  sync.

I1 and I6 do not promise "every". They promise "recoverable, or an
audited, counted gap" (REC-07, REC-08). The pitch drops that
qualifier.

**Minimal fix.** Reword: "every message, tool call and result the
agents' transcripts contain, plus worktree checkpoints, with every gap
audited". Separately, decide whether worktree snapshots, which would
catch edits made outside the agent, are in scope, and give them a
requirement id.

---

## F7. "Real time: everyone sees edits, test runs and messages as they happen" is more than the capture path delivers (serious)

**Pitch:** "Real time: everyone sees edits, test runs and messages as
they happen" (L13).

**Conflicting sources:**

- `docs/srs/09-interfaces.md:22` and
  `docs/srs/05-functional-requirements.md:30`: incremental ingestion
  happens on `PostToolUse` and `Stop`, within a 100 ms budget, and
  leaves a work marker for the rest. It is REC-13, priority **P1**
  ("SHOULD").
- `plan/2609292012_m1-record-and-recall.md:40-42`: M1 batches a hook's
  events into one commit for throughput.
- `plan/2610022338_parinn-network-side/plan.md:90`: the gate is "in
  under 5 s", for a **sealed** segment.
- `research/notes/agent-session-storage-sweep/catalog.md:113-115`: a
  file watcher is "real time on one machine and nowhere else".

**Conflict.** A test run becomes visible only after the tool call
finishes, because `PostToolUse` fires once the tool returns. Long runs
therefore do not stream "as they happen". A segment must also be
sealed before it can be synced, and the planned target is seconds,
not live. Human edits are not captured at all (F6).

**Minimal fix.** Reword: "within seconds of each agent step". Or
decide to add a separate streaming channel. Keep that channel out of
the record, as the research recommends
(`agent-session-storage-beyond-git.md:138`: "Live chat stays out of
the durable record").

---

## F8. The headline product, a live PR with a web interface, is a v1 non-goal, and the research recommended against building it (serious)

**Pitch:** "ParleyInn is the live pull request for the agent era" (L3).
"Interactive results … live views you can explore" (L16-17). "one
click away" (L20).

**Conflicting sources:**

- `docs/srs/01-introduction.md:66`: NG5, "Graphical UI — CLI and MCP
  only".
- `docs/srs/01-introduction.md:18-19`: the Intent is "unbounded, exact
  recall without creating a new attack surface". Pins (I3) and
  recovery after compaction are the core loop
  (`04-reference-architecture.md:70-83`). The pitch mentions neither.
- `research/reports/custom-storage-git-and-chat-interfaces.md:3`:
  "ParleyInn should own its storage and its keys, but not the network, and
  it should not become a forge or a chat server". `:301`: "Owning the
  network is not ParleyInn's job; I4 forbids it". `:282` lists "Scope
  creep" as a risk, noting that Buzz needs Postgres, Redis and object
  storage.
- `research/reports/agent-session-storage-beyond-git.md:167`:
  "Real-time sharing is a separate product, the relay".
- `research/notes/agent-session-storage-sweep/catalog.md:377-384`,
  Lesson 11: "The local record must outlive every vendor and every
  share link". `plan/2610022338_parinn-network-side/plan.md:19-20`:
  components "built and run by this project".

**Conflict.** The pitch leads with the very things that:

- the SRS puts out of v1 (NG4, NG5);
- the research advised against (a chat server, owning the network);
- the network plan reaches only in task 6 of 6, with no phase written
  (`plan/2610022338…/plan.md:81-84`).

Meanwhile it leaves out what the SRS actually specifies (recall,
pins, recovery after compaction).

**Minimal fix.** Either:

- **Design decision.** The stakeholder accepts the change of product
  identity, lifts NG5 for a separate binary, and records why the
  research's "not a chat server" advice is overridden.
- **Rewording.** Lead with the context layer, and offer the live lane
  as a planned companion: "ParleyInn keeps every lane's exact history safe
  to recall; a separately built lane server can show it live."

---

## F9. The pitch says "is", but nothing it describes has a requirement id, and every plan behind it is 🔲 (serious)

**Pitch:** "ParleyInn is the live pull request" (L3). "A ParleyInn lane is
live" (L9-10). "It is built so that" (L23).

**Conflicting sources:**

- `CLAUDE.md`, Project: "Status: pre-implementation".
- `plan/2610012322_parinn-for-agent-fleets/phase-1.md:9`: "No SRS id
  closes, and no scenario leaves `@pending`". The lane, its identity
  and the owner trust model are still only a proposal
  (`plan.md:98-101`).
- `plan/2610022338_parinn-network-side/phase-1.md:9-11`: "None closes
  yet". Phase 1 proves only that a relay can exchange segments.
  Chat, presence, result views and the public host are unphased
  tasks 4–6.
- `plan/2609292012_m1-record-and-recall.md:9`: M1 itself blocks on the
  fleets plan.

**Conflict.** Every feature bullet in the pitch is unscoped by the
SRS. Under CLAUDE.md's reporting rule, a capability exists only when
its scenario is off `@pending`. None of these even has a scenario.

**Minimal fix.** Reword into the future tense, or as a design target
("ParleyInn is designed to …"), and name the release that carries each
feature.

---

## F10. "every edit and tool run" shown to "everyone" contradicts private-by-default and projection-only sharing (serious)

**Pitch:** "everyone sees edits, test runs and messages" (L13). The
record holds "every tool run and every result" (L5-6). "watch a whole
fleet of lanes at once" (L15).

**Conflicting sources:**

- `research/reports/agent-session-storage-beyond-git.md:99`: the HN
  consensus wants full sessions kept "out of band, linked by ID,
  private and redacted". `:101`: "I don't want my thoughts to be
  serialized, version controlled and publicly accessible".
- `research/reports/agent-session-storage-beyond-git.md:152` and
  `:155`, rules 9 and 12: "Shared and public remotes get ciphertext or
  hashes only". "Publish to open source as projections, not raw
  segments".
- `research/notes/agent-session-storage-sweep/catalog.md:354-359`,
  Lesson 7: "make export a separate fail-closed step".
  `:239-243`: "Sync-on-by-default is the most common privacy gap".
- `docs/srs/06-security.md:62`: SEC-08 redacts secret patterns, not
  personal data in general (see also `custom-storage…md:220`).

**Conflict.** The lossless record is safe because it stays private.
The live lane is attractive because it is shared. The pitch states
both without saying which events reach whom. Broadcasting every tool
result to every participant is the sync-by-default pattern the
evidence warns against.

**Minimal fix.** A design decision: a lane's visibility defaults to
the owner only. Sharing is an explicit, fail-closed step, carried as
ciphertext, and public lanes get projections, not raw results. Reword
"everyone" to "everyone the owner admits".

---

## F11. "It is built so that collaboration can't become an attack on the agents" is absolute, while the lane UI routes untrusted content to the one trusted principal (serious)

**Pitch:** "collaboration can't become an attack on the agents" (L23).
"rendered output arrive as live views you can explore" (L16-17).

**Conflicting sources:**

- `research/notes/agent-session-storage-sweep/catalog.md:347-353`,
  Lesson 6: "Anything that reads stored history runs with zero
  authority and treats bytes as data". Rendering tool output as
  interactive views does the opposite.
- `docs/srs/01-introduction.md:36`: I2 covers automatic paths only. A
  human relaying content is outside its reach.

**Conflict.** The lane server shows the only trusted principal, the
owner, a stream of untrusted chat and rendered tool output, with a
reply box right next to it. Typing "do what bob suggested" turns
untrusted content into trusted input. No invariant can stop that, so
"can't" overclaims. Rendering untrusted output, such as HTML or
benchmark pages, in a browser also attacks the humans, which the
pitch's agent-only promise does not cover.

**Minimal fix.** Reword: "nothing a collaborator writes reaches an
agent automatically or as trusted input". Add a design rule: result
views render untrusted output inert (sandboxed, no script), and say so.

---

## F12. "verifiable" and "tamper-evident" promise more than a per-origin chain proves (wording)

**Pitch:** "verifiable and searchable" (L20-21). "The lane's record is
tamper-evident" (L26-27).

**Conflicting sources:**

- `docs/srs/06-security.md:31`: a "Process running as the same OS
  user" is "equivalent to the tenant; out of scope". The owner can
  re-chain their own log before anyone saves a checkpoint.
- `research/reports/agent-session-storage-beyond-git.md:126`, quoting
  Anthropic's transparency log docs: the saved checkpoint "is what
  turns 'the log is consistent today' into 'the log has been
  consistent since you started watching'". `:148`, rule 5: keys must
  be "pinned out of band".
- `plan/2610022338_parinn-network-side/plan.md:42`: the public host
  serves bundles read-only. Nothing in the plans makes them searchable
  by third parties.

**Conflict.** "Verifiable" proves integrity relative to a checkpoint
the verifier already holds, from keys pinned out of band. It does not
prove that the record is complete: an origin or the relay can
withhold segments, and edits made outside the agent are not captured
(F6). Shredded spans (F5) cannot be verified at all. "One click away"
verification in a hosted UI means trusting that UI. "Searchable" is
local FTS over the segments a node holds, not a public search.

**Minimal fix.** Reword: "tamper-evident against anyone who holds an
earlier checkpoint; missing or shredded parts are shown as gaps, never
hidden".

---

## F13. "in the order they happened" and "held as one record" do not match the per-origin design (wording)

**Pitch:** "held as one record … in the order they happened" (L4-6).

**Conflicting sources:**

- `research/reports/agent-session-storage-beyond-git.md:150`, rule 7:
  "Global order lives only in the index, built from causal links, then
  origin ID and segment hash. **Wall-clock time is for display only.**"
- `docs/srs/05-functional-requirements.md:29`: REC-12 says "`seq`
  alone defines order" within a source.
- `research/notes/agent-session-storage-sweep/catalog.md:330-334`,
  Lesson 3: "One writer per chain … merge only at read time into a
  rebuildable index".
- `plan/2610012322_parinn-for-agent-fleets/plan.md:65-71`: the record is
  per-origin log segments, and the index is a local SQLite database.

**Conflict.** A multi-writer lane is many records, one per origin,
merged in a derived index. Concurrent events from two origins have no
"happened" order, only a deterministic tie-break. I10 forbids reading
the clock in projections, so the order cannot come from timestamps.
Within one origin the claim holds.

**Minimal fix.** Reword: "in one causal order every node rebuilds
identically".

---

## F14. "watch a whole fleet of lanes at once" and lanes that span worktrees run against I8 and MEM-01 (wording, pending the fleets SRS change)

**Pitch:** "watch a whole fleet of lanes at once" (L15). "a branch with
its worktree, its agents and its humans, held as one record" (L3-5).

**Conflicting sources:**

- `docs/srs/01-introduction.md:42`: I8 says "All state is bound to one
  tenant's home … ParleyInn refuses to operate on state it does not own."
  Imported segments from other people are state ParleyInn does not own.
- `docs/srs/08-data-and-storage.md:22`: a project is keyed by
  "SHA-256(canonical project path)", so worktrees hash to separate
  stores (acknowledged in `plan/2610012322…/plan.md:37-38`).
- `docs/srs/05-functional-requirements.md:133`: MEM-01, "MUST NOT
  promote record content into any … cross-project store". A fleet
  view across projects comes close to that.

**Conflict.** The fleets plan already lists "I8 tenant across nodes"
as an invariant to restate (`phase-1.md:21-23`). Until that is done,
the pitch describes behavior the current contract refuses.

**Minimal fix.** Tie the claim to the pending SRS change, or reword it
as "your own lanes, across your worktrees".

---

## Suspected conflicts checked and dropped

- **"Lossless" against "erased"**: dropped. I1 itself allows "data an
  operator explicitly purges … recorded"
  (`docs/srs/01-introduction.md:35`). The problems that do exist are
  covered in F5 and F6.
- **"Tamper-evident" against "erased by key", in principle**: dropped.
  A chain over keyed hashes of ciphertext survives a shred
  (`custom-storage…md:214-218`, Matrix's reference hash). The conflict
  is only with the current SRS hashes (F5).
- **"The core never touches the network", as a literal sentence**:
  dropped. The planned core reads an inbox and keeps its import
  closure (`plan/2610022338…/plan.md:46-49`). What remains is the
  conflict with I4, SEC-01 and NG4 (F1).
- **"Nothing stored is injected" against the owner's own messages**:
  dropped, provided the owner types into the harness locally. That is
  live input, not stored content re-injected. It turns into F2 when
  the owner types into the lane server.
- **"Real time" against I10 (no clock in projections)**: dropped. Live
  views are presentation, not projections, as long as the index
  ignores wall-clock time (F13 covers ordering).
- **Order within one agent session**: the claim holds (REC-12).
