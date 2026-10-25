# ParleyInn UX: users, onboarding and surfaces

This file designs three areas of the ParleyInn lane experience: who the
users are and their core journeys (area 1), onboarding (area 7), and
the surfaces ParleyInn runs on (area 8). It is direction for the SRS change
in [plan 2610012322](../plan.md) and for phase 1 of
[plan 2610022338](../../2610022338_parinn-network-side/plan.md). Nothing
here is normative. Draft MUST sentences are proposals and carry no ids.

Read first: [pitch.md](../pitch.md), the network boundaries B0 to B3 in
[plan 2610022338](../../2610022338_parinn-network-side/plan.md), and the
invariants in [§1.3](../../../docs/srs/invariants.md).

## Shared vocabulary

The journeys name these screens. Other UX files may refine them; the
names here are the ones this file assumes.

| Screen or surface | What is on it                                                                                                                      |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| **Fleet**         | Every lane on reachable nodes, one row each; the **Needs you** strip on top; filters by repo, node, state                          |
| **Lane**          | One lane: the live harness pane, the **Results** pane, tiles for the other harnesses in the lane, and the lane timeline underneath |
| **Needs you**     | The queue of permission requests, questions and gate decisions waiting on the owner, oldest first, across all lanes                |
| **Catch up**      | What changed since a chosen time: per lane, what was done, what verified it, what is waiting, what failed                          |
| **Land**          | The merge gate for one lane: approvals, required checks with their evidence level, the landing path, and the landed-commit link    |
| **Foreign lane**  | A lane imported from someone else (bundle, git carrier or peer), shown read-only and untrusted until the owner adopts parts of it  |
| **Setup**         | First-run and onboarding: import, hooks, mode, health                                                                              |
| **Peers**         | Peering on or off, enrolled nodes and devices, invites, revocation, sync state                                                     |
| **Health**        | `parinn status` in the browser: store locations, sizes, counters, gaps, refusals (I6)                                              |
| **TUI**           | `parinn lanes`: Fleet, Lane and Needs you in the terminal, read straight from the local store                                      |
| **Phone view**    | A reduced Fleet, Lane and Needs you, read and approve only, reached through a node the user owns                                   |
| **Harness strip** | What a Claude Code session shows its human about its lane: a status line and banners the model never sees                          |

Status words for a harness, shared by every surface. They follow the
states Claude Code's agent view and Codex already use, so a user who
knows either reads ParleyInn without learning new words
([agent view](https://code.claude.com/docs/en/agent-view),
[Codex ThreadStatus](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/schema/typescript/v2/ThreadStatus.ts)):

| Word            | Meaning                                                               | Mark                   |
| --------------- | --------------------------------------------------------------------- | ---------------------- |
| **Working**     | running tools or generating                                           | animated dot           |
| **Needs you**   | waiting on a permission, a question, a sandbox request or a dialog    | amber dot, with reason |
| **Idle**        | alive, ready for the next prompt                                      | hollow dot             |
| **Done**        | the turn ended and the agent claims the task is complete              | green check, "claimed" |
| **Failed**      | the session ended with an error                                       | red cross              |
| **Stopped**     | stopped by a person or a signal                                       | grey square            |
| **Unreachable** | on a peer ParleyInn cannot reach now; shows "last seen 14:02"         | dashed outline         |
| **Unrecorded**  | the harness runs but ParleyInn sees no hook events (gap since a time) | striped outline        |

The last two are ParleyInn's own. Agent view cannot know a session on
another machine, and it cannot know a recorder is missing. ParleyInn must
say both, because I6 forbids silent gaps.

Quiet marks for trust, used everywhere:

- No mark on the owner's own local events. Signed and verified is the
  silent default.
- A small hollow ring and the author's name on anything from another
  person, another agent, a peer or a bundle: "untrusted, from Bob".
- "Claimed", "ran here", "CI" on every result: the evidence level.
- "Not proven" on a landed-commit link the record cannot show.
- A loud banner only for refusals: a broken signature or chain, a
  refused segment, a gap. These are rare, and each links to Health.

## Area 1: users and core journeys

### Who uses ParleyInn

| #   | User                  | Setting                                                                   | Core question                                     | Needs peering             |
| --- | --------------------- | ------------------------------------------------------------------------- | ------------------------------------------------- | ------------------------- |
| U1  | **Solo, one machine** | Maya runs five Claude Code agents in worktrees of two repos on one laptop | "Which agent needs me, and is its work real?"     | No                        |
| U2  | **Solo, many nodes**  | Ravi has a laptop, a home server and Claude Code cloud sandboxes          | "One view of all my agents, wherever they run"    | Yes, own nodes only       |
| U3  | **Reviewer**          | Lena decides whether a lane may land, her own or a teammate's             | "Is this verified, and by what?"                  | Own lane: no. Other: yes  |
| U4  | **OSS maintainer**    | Tomás receives an outside pull request and the lane bundle behind it      | "What did this contributor's agent really do?"    | No live peering; a bundle |
| U5  | **Teammate, live**    | Bob joins Maya's lane to help, from his own machine                       | "Can I see and help without hijacking her agent?" | Yes                       |
| U6  | **Returning owner**   | Maya, Monday 09:00, after agents ran all weekend                          | "What did my agents do?"                          | No; more with peers       |

U1 and U6 are the same person on different days. They are the
standalone core, and phase 1 of plan 2610022338 must serve them fully
with no network. U2 and U5 need peering (B2). U4 needs a lane bundle
carried by git or a file, and the public host (B3) only when the lane
is published. U3 spans both.

### J1: Maya runs five agents on one machine (standalone)

**Goal.** Keep five agents moving without reading five terminals.

**Story.** Maya opens a terminal and types `parinn ui`. Her browser opens
**Fleet** on `http://127.0.0.1:47321/#t=…`. Five rows: three lanes in
`api`, two in `web`. Two rows show an amber **Needs you** dot. The
strip on top reads "2 need you · oldest 6 min". She presses `n` to jump
to the oldest. **Lane** opens on `api/rate-limit`. The live harness pane
shows the agent paused on `Bash: go test ./... -race`, waiting for
permission. She presses `a` to allow once. The request clears in
every surface at once, including the terminal the agent runs in. The
**Results** pane on the right updates: "tests: 214 passed · ran here ·
3 min ago". She presses `]` to go to the next lane needing her. That
agent asks a question: "Keep the v1 endpoint?" She answers in the
composer. Her answer goes to the agent as her own message, because
she is the owner on her own machine.

Later she clicks a tile in the **other harnesses** strip, sees `web`
agent B has been **Idle** for 40 minutes after claiming "Done", and
opens its Results: "claimed · no run recorded". She types "run the
e2e suite and show me" and goes back to work.

**Screens.** Fleet, Needs you strip, Lane (harness pane, Results,
other-harness tiles), composer.

**Edge cases.**

- An agent is stuck: Working for 25 minutes with no new event. The row
  shows "no events for 25 min" in grey, never a guess at the cause.
- The UI is closed: the agents keep working and recording through the
  hooks. Nothing waits on the UI (decision 8).
- One agent's hooks fail: the row turns **Unrecorded** with "since
  14:10", and Health counts the failures (I6). The agent itself keeps
  running (I9).

### J2: Ravi follows agents on three kinds of node (peering)

**Goal.** One Fleet for the laptop, the home server and the sandboxes,
with no service in the middle.

**Story.** Ravi enrolled his home server once (see onboarding, 7.6).
On Tuesday he starts two Claude Code cloud sessions from his phone's
browser on claude.ai. Each sandbox runs ParleyInn's plugin, which dials out
to his home server with a scoped, short-lived enrollment token held as
an environment secret. In his laptop's **Fleet**, two new rows appear
with a small cloud glyph and the node name `sandbox-7f3a`. Their
events arrive as signed segments through the home server. Every
event from a sandbox carries the hollow ring until he opens the lane
from his own machine: a sandbox is his, but it is still another
origin, and it never instructs his laptop's agents.

On the train the laptop goes offline. **Fleet** keeps all local lanes
live and shows the sandbox rows as **Unreachable** with "last seen
08:41". The sandboxes keep working and keep pushing to the home
server. When the laptop reconnects, the rows fill in. The timeline
shows a thin "synced 34 events from home-server" divider, not a
replay animation.

One sandbox is reclaimed before its last segment synced. Its row reads
"ended · last 3 min not received". Health records the gap (I6).

**Screens.** Fleet (node column, cloud glyph), Lane, Peers, Health.

**Edge cases.**

- Partition: both sides keep writing their own logs. On reconnect the
  per-writer logs merge in any order (plan 2610012322). Lane metadata
  edited on both sides, such as the title, shows "changed on two
  nodes" with both values until resolved; the CRDT choice is open.
- A sandbox's token expires mid-run: the sandbox keeps recording
  locally. Its row shows "not syncing: enrollment expired" and a
  **Renew** button that opens Peers.
- Many sandboxes: Fleet groups ended sandbox lanes under "Ended today
  (12)", collapsed.

### J3: Lena decides whether a lane may land (standalone or peering)

**Goal.** Land only what is verified, and know by what.

**Story.** Lena opens **Land** on `api/rate-limit`, from Fleet's
"Ready for review" group. The sheet has four blocks:

1. **What changed**: the diff against the base, grouped by the turn
   that made each hunk. Hunks no agent tool call made, by a person, a
   formatter or a shell, carry "from checkpoint".
2. **What verified it**: each required check with its evidence level.
   "unit tests · ran here · on head `a1b2c3`" and "CI · not seen yet".
   A check that ran on an older head says "stale: 2 commits behind".
3. **Approvals**: who approved which head. Her own approval would be
   the second.
4. **Landing**: the target branch, the method the forge will use
   (squash), and what ParleyInn will be able to prove afterwards.

She clicks a hunk and the timeline scrolls to the turn that made it:
the prompt, the tool call and the test run after it. She writes one
review comment on a hunk. If she is not the owner, the comment lands
in the lane as an untrusted post. The agent sees it only if it pulls
lane posts, and it acts on it only if the owner adopts it (see U5).
She presses **Approve head `a1b2c3`**. The approval is a signed event
in the lane. After the forge squashes and merges, Land shows "Landed
as `9f8e7d` · proven: patch identical" or "Landed as `9f8e7d` · not
proven: squash rewrote 2 files". It never shows a bare green tick.

**Screens.** Fleet (Ready for review), Land, Lane timeline.

**Standalone or peering.** Reviewing her own lane works standalone.
Reviewing a teammate's lane needs that lane on her node, by peering or
by a bundle.

**Edge cases.**

- The forge's branch protection disagrees with the lane gate: Land
  shows both and says the forge decides. ParleyInn's gate never overrides
  the forge.
- CI results need the network, which the core never touches. Land
  shows "CI · not fetched" until a separate, opt-in carrier brings
  them in as untrusted, audited imports.

### J4: Tomás receives a contribution with its lane (bundle, no peering)

**Goal.** See what an outside contributor's agent did before reviewing
the code.

**Story.** A pull request on the forge links a lane bundle, published
by the contributor's own node or attached as a git ref. Tomás runs
`parinn import https://…/lane.bundle` or drops the file on Fleet. A
**Foreign lane** opens, read-only, with a banner that stays quiet but
visible: "From @kai · imported · untrusted · signature valid for key
`kai:7Q4…`". Every event carries the hollow ring. Rich results render
inert: test output as text, HTML as source, no script.

He filters the timeline to **Tool runs** and sees the tests the
contributor's agent ran, all "claimed" or "ran there", none "ran here".
He clicks **Re-run here** on the test command. ParleyInn copies the command
into his own terminal; it does not run it, because ParleyInn runs nothing
(I4). He decides to let his own agent continue the work. He clicks
**Start my lane from this**. ParleyInn creates a new lane he owns, with a
link to the foreign lane. His agent can recall the foreign events, but
only as enveloped, untrusted data (I2).

**Screens.** Fleet (drop target), Foreign lane, Lane (new).

**Edge cases.**

- Broken signature or chain: the import is refused, nothing is shown
  as content, and Health records the refusal (I6).
- The contributor does not run ParleyInn: there is no bundle. The forge
  pull request is all Tomás has, and ParleyInn says nothing about it.
- The bundle holds redacted spans: they show as "redacted by author"
  with sizes, never silently missing.

### J5: Bob joins Maya's lane live (peering)

**Goal.** Watch Maya's agent, point at problems, and help, without
being able to steer her agent.

**Story.** Maya opens **Lane**, clicks **Invite**, and picks Bob, whose
node key she enrolled last week. Bob's Fleet shows the lane under
"Shared with me" with Maya's avatar. He opens it and sees the live
harness pane, the timeline and Results, as Maya does. His composer
reads "Post to lane" instead of "Message agent", and a hollow ring sits
beside it. He writes "line 40 drops the error". The post appears in
Maya's timeline with his name and the ring.

Maya's agent does not see the post. It is not injected (I2). Maya
hovers the post and clicks **Adopt as instruction**. A sheet shows the
exact text that will go to her agent, signed as hers, with "adopted
from Bob's post". She edits one word and sends it. The lane records
both events: Bob's post and Maya's adoption.

Bob's own agent works in another worktree on a sub-task. It shows as a
tile in Maya's other-harness strip, with Bob's name and the ring. She
can watch it; she cannot message it, and it cannot message hers except
as an untrusted post.

**Screens.** Lane (Invite, Post to lane, Adopt as instruction),
Fleet (Shared with me), Peers.

**Edge cases.**

- Maya is asleep and her agent needs a permission: Bob sees "Needs
  Maya" on the tile. He cannot answer it. He can **Nudge**, which
  queues a hint in Maya's Needs you, not in the agent.
- Partition between Bob and Maya: each keeps working. Bob's posts
  queue as "not delivered yet" in his timeline and deliver on
  reconnect, in causal order.
- Bob is removed: Maya clicks **Remove from lane**. His later segments
  are refused. His earlier posts stay in the record (I1), marked
  "from a removed participant".

### J6: Maya returns on Monday (standalone; wider with peers)

**Goal.** Answer "what did my agents do?" in two minutes.

**Story.** Maya opens `parinn ui` at 09:00. Fleet opens on **Catch up**
because she has been away for more than 12 hours. It reads "Since
Friday 18:20". Per lane, one line each:

- `api/rate-limit` · Done (claimed) · 3 commits · tests ran here,
  passed · waiting for review
- `api/pagination` · Needs you since Saturday 02:14 · permission:
  `rm -rf build/`
- `web/i18n` · Failed Sunday 11:03 · out of credit
- `web/a11y` · Done (claimed) · no test run recorded

Each line expands into the turns that matter, chosen by structure,
never by a model's summary: lane state changes, permission requests,
commits, test runs and failures. A summary from the harness, such as
agent view's status line, may appear, but it is marked as the agent's
own claim. Catch up never writes a summary itself (NG7). She presses
`Enter` on the pagination row and answers the 55-hour-old permission.
She presses `x` on the a11y row to mark it "seen". It moves to
"Reviewed".

**Screens.** Catch up, Needs you, Lane, Land.

**Edge cases.**

- A peer was offline all weekend: Catch up says "home-server: last
  synced Friday 23:10" at the top, so missing lanes are not mistaken
  for idle ones.
- Claude Code deleted old transcripts (it does after 30 days by
  default): the lane still holds them, because the record is ParleyInn's
  ([agent view state](https://code.claude.com/docs/en/agent-view),
  [catalog lesson 11](../../../research/notes/agent-session-storage-sweep/catalog.md)).

### What the journeys borrow

- The Needs you strip and the "oldest first" order come from Codex's
  Activity view and iOS Priority view, and from dots' "requests for
  input"
  ([OpenAI agent UI notes](../../../research/notes/openai-agent-ui/openai-agent-ui.md),
  [dots notes](../../../research/notes/openai-agent-ui/dots.md)).
- Approvals that clear on every client come from the app-server's
  `serverRequest/resolved` broadcast.
- Evidence levels on results come from dots' "a completed run is not a
  verified one" and the Compliance API's verified versus
  client-asserted tags
  ([process gaps 11](../../../research/notes/live-pr-pitch-review/process-gaps.md)).
- "Ready for review" and grouping by state come from Claude Code's
  agent view.
- Invite into a live lane comes from Zed Delta and Conductor
  ([competitors](../../../research/notes/live-pr-pitch-review/competitors.md)).

### What the journeys do differently

- **Post versus instruct.** Delta's send submits every author's drafts
  as one turn. ParleyInn splits the composer: a non-owner posts, the owner
  adopts. This answers the review-loop gap without breaking I2
  ([process gaps 2](../../../research/notes/live-pr-pitch-review/process-gaps.md),
  [inconsistencies F3](../../../research/notes/live-pr-pitch-review/inconsistencies.md)).
- **Catch up is structural.** Agent view's row summaries come from a
  model. ParleyInn's Catch up comes from the record, so it is rebuildable
  (I10) and is not a poisoning channel (NG7).
- **Unreachable and Unrecorded are states.** No competitor shows a
  missing recorder or a partitioned node as a state.

### Requirements implied by the journeys

- The UI MUST show every harness with exactly one status word from the
  shared set, and every surface MUST use the same words.
- The Needs you queue MUST order requests oldest first across all
  lanes and nodes, and an answer from any surface MUST clear the
  request on every other surface.
- Every result MUST show its evidence level: claimed, ran here, ran on
  another named node, or CI, with the head it ran on.
- A non-owner's text MUST reach the lane only as a post. A post MUST
  reach an agent only through the owner's explicit adoption, recorded
  as its own event, or through the agent's own pull, enveloped as
  untrusted.
- Catch up MUST be computed from the record alone, with no model call,
  and MUST name every node that has not synced since the chosen time.
- The landed-commit link MUST read "proven" or "not proven" with a
  reason, never a bare success mark.

## Area 7: onboarding

**Goal.** A working setup in five minutes with zero required
configuration (NFR-12), no configuration changed without being shown
first (I7), and nothing leaving the machine (decision 9).

### 7.1 First install

**Story.** Maya installs Claude Code's plugin from its marketplace:
`/plugin install parinn@…`. The plugin bundles the hooks, the MCP server
and the static binary (ADM-01, ADR-06). Claude Code's own install
dialog is the "show first" step: the harness lists what the plugin
adds, and she confirms. ParleyInn writes nothing to `settings.json` on
this path.

People who prefer the terminal download the signed binary and verify
it as [SECURITY.md](../../../SECURITY.md) describes. They then run
`parinn install --dry-run`, read the diff, and run `parinn install`
(ADM-02).

When Claude Code starts its next session, the plugin's first hook
prints one line to Maya through the hook's user-facing message, not to
the model:

```text
parinn: recording this session · open the lane view with `parinn ui`
```

### 7.2 First run with zero lanes

**Story.** Maya types `parinn ui`. ParleyInn starts the loopback server,
prints the URL with a fresh token, and opens the browser. **Setup**
shows, not an empty Fleet. It has three cards, in this order:

1. **Bring in what you already have.** "Found 41 Claude Code sessions
   in 3 repos under `~/.claude/projects`. 9 will be deleted by Claude
   Code within 7 days." Button: **Review and import**.
2. **Record new sessions.** Either "Hooks installed through the plugin
   ✓" or "Hooks not installed" with **Show the change**.
3. **Take the tour.** **Open a sample lane**: a recorded lane bundled
   with the binary, read-only, marked "sample". It is the same fixture
   phase 1 uses to judge the view before live harnesses are wired.

Under the cards, one quiet line: "Standalone · nothing leaves this
machine · peering off". It links to Peers.

A fourth, smaller card asks one question: "Do you type the prompts
on this machine yourself?" **Yes, I'm interactive** writes `mode =
"interactive"` to ParleyInn's own tenant config, shown as a diff first.
**Leave automation mode** keeps the default (PRV-04). The card explains
in one sentence: "In automation mode ParleyInn treats your prompts as
untrusted too, which is right for runners fed by issue text."

When the first lane arrives, Setup folds into a "Setup: 2 of 3 done"
chip in the Fleet header. It never blocks Fleet.

### 7.3 Importing sessions and attaching worktrees

**Story.** **Review and import** opens a table, one row per proposed
lane:

| Lane (proposed)   | Repo  | Worktrees                             | Sessions | Oldest | Expires soon | Import |
| ----------------- | ----- | ------------------------------------- | -------- | ------ | ------------ | ------ |
| `api/rate-limit`  | `api` | `api`, `.claude/worktrees/rate-limit` | 7        | 12 Sep | 2            | ☑      |
| `api/main`        | `api` | `api`                                 | 18       | 4 Sep  | 6            | ☑      |
| `web/(no branch)` | `web` | unknown: worktree removed             | 3        | 2 Sep  | 1            | ☐      |

ParleyInn groups sessions into lanes by clone and branch, so the worktrees
of one clone share one project (plan 2610012322's identity model). It
finds worktrees by reading the `.git` directory, never by running git:
the core holds no `os/exec` (I4). Claude Code's own worktrees under
`.claude/worktrees/<name>` on branch `worktree-<name>` are recognised
by name ([worktrees](https://code.claude.com/docs/en/worktrees)).
Sessions whose worktree is gone land in a lane marked "(no branch)",
unticked by default.

**Import 25 sessions** ingests the transcripts as untrusted, mutable
input. Every imported event carries the mark "imported 3 Oct from
transcript". It is not marked "witnessed": ParleyInn cannot prove what
happened before its hooks watched
([catalog lesson 1](../../../research/notes/agent-session-storage-sweep/catalog.md)).
A transcript that shrank or was rewritten is quarantined, not merged,
and counted (I5, I6).

**Attach a worktree** (Fleet, `+` menu) takes a path. ParleyInn shows the
lane it would join and any running session it found there. Attaching
records nothing new until the hooks fire there.

Other harnesses: Setup lists "Codex sessions found in `~/.codex`" as
**Import read-only**, and live Codex or ACP harnesses as "adapter not
installed". Claude Code comes first; the design must not preclude the
others (NG6).

### 7.4 Installing the hooks, shown first

**Story.** On a machine where the plugin path is unavailable, the
Record card reads "Hooks not installed". **Show the change** opens a
sheet with the exact diff to `~/.claude/settings.json`, the scope
(user or project), and the command:

```text
parinn install --scope user
```

The sheet has a **Copy command** button and no **Apply** button. Maya
runs the command in her terminal, which shows the same diff and asks
for confirmation (ADM-02). The browser updates to "Hooks installed ✓"
when the first hook event arrives.

The UI server never writes agent configuration. That keeps the B1
component, the one a browser can reach, unable to change what the
agent loads, even if its token leaks.

If managed settings are present, the sheet says "Your organisation
manages Claude Code settings. ParleyInn will not change them," and links
to the runner-image instructions (ADM-03). **Uninstall** sits in the
same sheet and shows the reverse diff, plus the purge offer (ADM-07).

### 7.5 Opening the UI on localhost

**Story.** `parinn ui` binds `127.0.0.1` on a free port, prints the URL,
and opens the default browser. The token travels in the URL fragment
(`#t=…`), which browsers do not send to servers or in referrers. The
page exchanges it once for a session cookie scoped to that origin, with
`HttpOnly` and `SameSite=Strict`, and strips the fragment from the
address bar. Requests with a wrong `Host` or `Origin` are refused and
counted.

Each launch makes a new token, so a bookmark goes stale. A stale tab
shows "This ParleyInn session ended. Run `parinn ui` to open a new one," not
a login form. `parinn ui --print` prints the URL without opening a
browser, for people who keep a pinned tab and paste the new token.

The UI is optional. Closing it stops nothing. Starting it while agents
run shows their live state at once, because it reads the same store.

### 7.6 Turning peering on

**Story, LAN.** Ravi opens **Peers** and clicks **Turn on peering**. A
sheet lists what changes: "Starts the peer process. It accepts
connections only from nodes you enroll by key. It listens on your
local network on port 47400." He confirms. Peers now shows his node's
key as a fingerprint and a QR code.

On the home server he runs `parinn peer invite`, which prints a
one-time code valid for 10 minutes. On the laptop he clicks **Enroll a
node** and pastes the code. Both screens then show the same six words,
like a Signal safety number. He checks they match on both and clicks
**They match** on each. The home server appears in Peers as "Trusted ·
LAN · synced". With **Find nearby nodes** on, local discovery lists
unenrolled nodes as "nearby, not enrolled". Discovery never enrolls a
node by itself.

**Story, remote.** For a node outside the LAN, the invite code also
carries the address the inviting node can be reached at. That is
Ravi's home server, behind a port he forwarded or on a VPN he runs.
Peers shows the link as "Remote · direct". ParleyInn names no relay of its
own, and there is no default rendezvous host. The table in plan
2610022338 allows remote peering only outbound to a self-hosted peer.

**Story, cloud sandbox.** A sandbox can only dial out. In Peers, Ravi
clicks **Allow sandboxes** for repo `api`. ParleyInn makes an enrollment
token scoped to that repo, to writing new lanes only, and to 7 days.
He pastes it into the cloud environment's secrets. He also adds his
home server's host to the environment's network allow-list. Each
sandbox makes its own key at start, enrolls with the token, and pushes
signed segments outbound. It pulls only the lanes it was started on.
Its row in Fleet reads "sandbox · via home-server".

**States of a peer.** Invited, Waiting for match, Trusted, Syncing,
Synced, Unreachable since a time, Revoked. A sandbox adds Ended, and
Ended with tail lost.

**Edge cases.**

- The words do not match: **They don't match** cancels both sides and
  records the attempt (I6).
- A laptop is stolen: **Revoke** on any other node refuses its key from
  then on. Segments it already wrote stay in the record, marked
  "from a revoked node".
- Peering is turned off: the peer process stops, and the lanes already
  synced stay readable. Fleet shows "peering off · last synced" on
  remote rows.

### What onboarding borrows

- A harness-native plugin install, from ADR-06 and Claude Code's plugin
  dialog, instead of rewriting settings (the lcm failure in SRS T10).
- Loopback server with a token per launch, from Jupyter; Symphony's
  dashboard rule that it "MUST NOT become REQUIRED"
  ([OpenAI agent UI notes](../../../research/notes/openai-agent-ui/openai-agent-ui.md)).
- A sample to tour before real data, from Delta's built-in introduction
  ([Delta getting started](https://delta.dev/docs/getting-started)).
- Pairing by code and matching words, from Codex Remote's QR pairing,
  without its relay on `chatgpt.com`.

### What onboarding does differently

- **Import shows expiry.** Claude Code deletes transcripts after 30
  days by default; ParleyInn says how many are about to go, which turns
  import from a chore into a rescue.
- **No apply button for agent config.** The browser shows the change;
  only the terminal makes it.
- **Imported history is marked as imported.** Competitors that ingest
  transcripts present them as the record. ParleyInn says it did not watch
  them happen.
- **Peering has no default host.** Codex Remote, Remote Control and
  Happy's hosted relay all start from a vendor host. ParleyInn starts from
  none.

### Requirements implied by onboarding

- With zero lanes, the UI MUST show Setup with import, recording status
  and a sample lane, and MUST state that nothing leaves the machine.
- Import MUST group sessions by clone and branch, MUST show how many
  source transcripts the harness will delete within 7 days, and MUST
  mark every imported event as imported, with its source.
- ParleyInn MUST discover worktrees by reading git's files, never by
  running a process.
- The UI server MUST NOT write agent configuration. It MAY show the
  diff and the command that would.
- The deployment mode MUST stay `automation` until the user opts in,
  and the opt-in MUST be shown as a diff of ParleyInn's tenant config.
- The UI token MUST NOT appear in server logs or referrers, and a stale
  token MUST lead to a page that names the command to reopen.
- Turning peering on MUST show what will listen and where, before it
  starts. Enrollment MUST need a key check on both nodes. Discovery
  MUST NOT enroll.
- A sandbox enrollment token MUST be scoped to a repository, a write
  scope and an expiry.

## Area 8: surfaces

**Goal.** The lane is the same object wherever it is seen. Each
surface shows as much of it as that surface can show safely.

### 8.1 The browser UI

The full experience: Fleet, Lane, Needs you, Catch up, Land, Foreign
lane, Setup, Peers and Health. Keyboard first, with the shortcuts
shared with the TUI:

| Key       | Action                                        |
| --------- | --------------------------------------------- |
| `g f`     | go to Fleet                                   |
| `n`       | open the oldest item that needs you           |
| `]` / `[` | next / previous lane that needs you           |
| `a` / `d` | allow once / deny the selected request        |
| `Enter`   | open the selected lane                        |
| `Space`   | peek at the selected row                      |
| `/`       | filter (`s:needs`, `r:api`, `n:sandbox-7f3a`) |
| `x`       | mark seen                                     |
| `?`       | all shortcuts                                 |

Untrusted results render inert: in a sandboxed frame with no script and
no network, or as text
([inconsistencies F11](../../../research/notes/live-pr-pitch-review/inconsistencies.md)).

### 8.2 Terminal only: `parinn lanes` and the CLI

**Story.** Ravi works over SSH on the home server. He runs `parinn
lanes`. A full-screen TUI shows Fleet: one row per lane, the same
status words and the same marks, grouped by state with Needs you on
top. `Space` peeks at the waiting question. `Enter` opens a Lane view:
the timeline as text, diffs as unified diffs, results with their
evidence level. `a` allows a request. `o` prints the browser URL for a
rich result he wants to see later.

The TUI reads the local store directly. It opens no socket, so
terminal-only use keeps even the B1 boundary closed. It lives with the
core under B0.

The CLI mirrors the same views for scripts, each with `--json` and the
documented exit codes (SRS 9.5):

```text
parinn lanes [--needs] [--json]          # Fleet
parinn lane show <lane> [--since 2d]     # Lane timeline
parinn needs                             # Needs you queue
parinn approve <request-id> | deny <id>  # answer a request
parinn catchup [--since friday]          # Catch up
parinn land check <lane>                 # Land, read-only
```

Keys copy agent view's where they overlap (`↑`/`↓`, `Space`, `Enter`,
`Ctrl+T` to pin, `?`), so Claude Code users need nothing new
([agent view](https://code.claude.com/docs/en/agent-view)).

### 8.3 Phone: read and approve only, through a node the user owns

**Story.** Saturday, 02:14. Maya is out. Her phone shows nothing,
because ParleyInn sends no push: a push would go through Apple's or
Google's servers. At breakfast she opens the ParleyInn page on her phone.
It reaches her home server over the peer link, using a device key her
laptop enrolled. The phone view shows Needs you first: "api/pagination
· permission: `rm -rf build/` · 7 h". She taps **Deny**. The denial is
a signed event from her device key, which her owner key certified, so
the agent's node accepts it as hers. Fleet and Catch up are there,
read-only, as text.

What the phone cannot do: type a message to an agent, adopt a post,
approve a lane to land, or change settings. It shows rich results as
text and diffs as unified text. It never renders untrusted HTML.

How it connects, in two stages:

- **v0, a tunnel she owns.** An SSH port-forward from the phone to the
  laptop's loopback UI. The browser sees `localhost`, so Host and
  Origin checks pass, and the token still applies. It needs nothing
  new from ParleyInn.
- **v1, a paired device.** Peers on the laptop shows **Pair a phone**
  and a QR code. The phone makes a device key; the owner key certifies
  it with the scope "read, approve or deny permission requests". The
  home server serves the phone view over the peer transport, only to
  certified device keys. **Revoke** works as for any node.

Never a vendor relay: unlike Codex Remote (relay on `chatgpt.com`) or
Remote Control (transcript held on Anthropic's servers while
connected), no third party carries or holds the lane
([OpenAI agent UI notes](../../../research/notes/openai-agent-ui/openai-agent-ui.md)).

### 8.4 Inside the harness

What a Claude Code session shows about its lane splits in two: what the
human at the terminal sees, and what the model sees.

**What the human sees (the Harness strip).** The status line, if the
user installed it (it is agent configuration, so I7 applies):

```text
⛰ api/rate-limit · you own · 2 agents · Bob watching · 1 post
```

At session start, a one-line banner through the hook's user-facing
message: "parinn: lane api/rate-limit · recording · `parinn ui` to open".
When a permission is answered elsewhere, the prompt in the terminal
clears with "answered in the lane view". None of this reaches the
model's context. The status line can therefore name Bob and count
posts without touching I2.

**What the model sees.** Nothing unless it asks. The MCP tools give it:

- `lane_status`: trusted structural fields only. Lane id, branch,
  owner, its own role, the number of participants and unread posts.
- `lane_posts`: the posts, always in the untrusted envelope, with
  authors.
- the existing recall tools, scoped to the lane.

Whether a structural "1 unread post; call `lane_posts`" line may be
injected is open (inconsistencies F3). This file proposes "no" by
default and opt-in per lane, matching INJ-04.

### Which surfaces must feel the same

| Element                            | Browser | TUI and CLI     | Phone            | Harness strip   |
| ---------------------------------- | ------- | --------------- | ---------------- | --------------- |
| Status words and marks             | full    | same            | same             | same            |
| Lane names, ids, colours           | full    | same            | same             | same            |
| Needs you order and content        | full    | same            | same             | count only      |
| Allow or deny a permission request | yes     | yes             | yes              | yes, in place   |
| Message the agent as owner         | yes     | yes (composer)  | no               | yes, it's there |
| Adopt a post as instruction        | yes     | yes             | no               | no              |
| Approve a lane to land             | yes     | yes             | no               | no              |
| Rich results (rendered)            | inert   | text, link out  | text             | no              |
| Catch up                           | full    | full            | read-only        | no              |
| Setup, Peers, Health               | full    | full (`parinn`) | Health read-only | no              |
| Opens a socket                     | B1      | none            | via B2 peer      | none            |

**Must feel the same:** status words, marks, lane identity, the Needs
you order, and what an approval means. An approval is one signed event
whatever surface made it, and every surface clears it at once.

**May be reduced:** the phone (no free text, no landing, no rich
render), the TUI (no rendered results), the harness strip (status and
count only). A reduced surface says what it left out: "Open in the
lane view to see the rendered result", never a blank.

### What the surfaces borrow

- One harness, many clients, from the Codex app-server, where every
  surface drives the same process.
- One identity across surfaces, from dots: the same agent in every
  channel
  ([dots notes](../../../research/notes/openai-agent-ui/dots.md)).
- The TUI's keys and states, from Claude Code's agent view.
- An E2E path to a phone over a relay you can host, from Happy
  (sheets.json in
  [the storage sweep](../../../research/notes/agent-session-storage-sweep/catalog.md)).
  ParleyInn goes one step further and needs no relay at all.

### What the surfaces do differently

- **Approvals are signed events, not RPC answers.** In the app-server,
  any client on the socket is fully trusted. In ParleyInn, an approval
  counts because of the key that signed it, so a phone over the peer
  link and a terminal on the machine carry the same weight.
- **The harness strip talks to the human, not the model.** Competitors
  that show peer activity in the session feed it to the model. ParleyInn
  keeps it on the user's side of the screen.
- **No push notifications.** Every competitor's phone story runs
  through a vendor push or relay service.

### Requirements implied by the surfaces

- Every surface MUST use the same status words, marks and lane ids,
  and MUST show a request answered elsewhere as answered at once.
- The TUI and CLI MUST read the local store directly and MUST NOT open
  a socket.
- Every CLI view MUST support `--json` and the documented exit codes.
- The phone view MUST be limited to reading and to allowing or denying
  permission requests. It MUST reach the lane only through a node the
  user enrolled, never through a third-party service.
- An approval from a paired device MUST be signed by a device key that
  the owner's key certified, with a scope, and MUST be revocable.
- ParleyInn MUST NOT send push notifications through a third-party service.
- What the harness shows its human about the lane MUST NOT enter the
  model's context. What the model learns about the lane MUST come
  through a tool call, as trusted structural fields or as enveloped
  untrusted content.
- A reduced surface MUST say what it left out and where to see it.

## Open decisions for the stakeholder

1. **Who counts as the owner on a phone.** Is a device key the owner
   certified the owner for approvals? This file says yes for allow and
   deny, no for free text and landing. The brief's rule ("their own
   local session or a signed message") allows either reading.
2. **Automation mode on a workstation.** PRV-04 makes the owner's own
   prompts untrusted by default. Should Setup ask the mode question,
   or should a workstation default to interactive? The question card
   is proposed; the default stays automation.
3. **Adopt as instruction.** Is the owner's explicit, recorded adoption
   of a post the way a co-author's words reach an agent, or does the
   lane also need granted roles (pitch "Open before the SRS change")?
4. **Unread-post notice in the model's context.** Allow a trusted,
   structural "N unread posts" line (opt-in per lane), or keep it
   pull-only with no hint at all (F3)?
5. **Phone v0 versus v1.** Ship the SSH-tunnel phone path first, or
   wait for paired devices over the peer link? v1 puts a UI on the B2
   side, which the boundary table does not yet allow.
6. **No push.** Accept that a phone learns of a waiting request only
   when opened, or allow a content-free "something needs you" ping via
   a self-hosted service the user names?
7. **The browser never applies agent config.** Accept the copy-command
   step in 7.4 as a deliberate bump, or allow an Apply button behind a
   typed confirmation?
8. **Sandbox enrollment by token.** A repo-scoped, expiring token held
   as an environment secret is the only way an outbound-only sandbox
   can enroll unattended. Is that secret on a vendor's sandbox
   acceptable, given that it grants write to new lanes only?
9. **Import of old transcripts.** Offer import on first run, given that
   imported history cannot be shown as witnessed, or record only from
   install onwards?
