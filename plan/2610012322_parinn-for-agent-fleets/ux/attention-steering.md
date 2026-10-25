# Attention and steering across a fleet of lanes

Design for two areas of the lane experience: (2) where the owner's
attention goes when 20 to 50 lanes run at once, and (3) how the owner
steers an agent mid-run. Both sit on the lane record. Neither may open
a path from untrusted content to an agent (I2), and neither may need a
network or a vendor push service in standalone (I4, B1).

Inputs: the [pitch and decisions](../pitch.md), the network boundaries
in [plan 2610022338](../../2610022338_parinn-network-side/plan.md) and
its [phase 1](../../2610022338_parinn-network-side/phase-1.md), the
invariants in [§1.3](../../../docs/srs/invariants.md), and the
proposed VIEW, PRV-09 and SEC-19 changes in
[trace-backward.md](../trace-backward.md).

## Summary

- Watching is not the job. The job is finding the one lane that needs
  a decision. The home screen is a **Needs you** queue. The fleet
  board comes second.
- One status vocabulary covers every harness: seven badges, drawn from
  Codex, Claude Code's agent view, Linear and the orchestrators. A lane's
  status is derived from its harnesses' statuses and its checks.
- Every request for the owner is a **held request** with an id. Any
  of the owner's surfaces can answer it: the UI, the terminal, the
  harness itself, or another of the owner's machines. The first valid
  answer wins, and a `resolved` event clears it everywhere.
- A request nobody answers in time is **parked**, never auto-allowed.
  The agent gets "held for the owner" and carries on with other work.
  A later answer arrives as a signed owner instruction.
- Notifications stay on the owner's machines: badges and toasts in the
  page, desktop alerts from the open loopback tab, and terminal bells
  and status lines. A phone needs either an enrolled peer or an
  opt-in bridge that runs outside the binary and carries hints only.
- Steering is a capability matrix per harness adapter. A control the
  harness cannot honour is shown greyed out with the reason, never
  faked.
- Owner input enters as `operator` only after the owner is
  authenticated. A co-author's words never steer. The owner can
  **Adopt** them, and the adoption is a signed act that names its
  source.

## Area 2: attention across a fleet of lanes

### Attention: the user's goal

"Tell me which lane needs me, why, and what happens if I wait. Let me
decide in one keystroke where I safely can. Leave me alone otherwise."
The owner runs many lanes. The scarce resource is their notice, not
screen space ([process gap 8](../../../research/notes/live-pr-pitch-review/process-gaps.md)).
A lane stuck on an approval nobody saw is a silent failure as far as
the owner can tell, so I6 makes attention a contract matter, not only
a convenience.

### Attention journey

Maya owns 31 lanes across her laptop, a desktop under the desk and two
cloud sandboxes that dial out to the desktop. At 09:10 she opens
`parinn ui`. The page opens on **Needs you (4)**:

1. `auth-refactor`, agent `claude-2`: wants to run
   `npm install jose@5`. It is blocked now and has waited 40 s. The
   card shows the command, the rule that sent it here ("package
   install: ask first") and the agent's last message. She presses `a`.
   The card leaves. The agent's terminal on the desktop shows the
   prompt cleared, "approved from UI on laptop".
2. `flaky-ci`: required check `go test ./...` failed on CI, and the
   agent claims it is fixed. The evidence badge reads **claim**, not
   **CI**. She presses `o` to open the lane, sees the failing test,
   and types a steering message.
3. `docs-catalog`: finished. All agents are idle, the local run is
   green and the review is waiting. She snoozes it until after lunch
   with `e`, then `2h`.
4. `peer-sync`: a co-author, Jon, wrote "use iroh-gossip, not
   polling". The card is marked **co-author · not an instruction**.
   She presses `Shift+A` (Adopt). A sheet shows the text she will
   send as her own instruction, with "adopted from Jon, event e7f3" at
   the top. She edits one word and sends it.

The other 27 lanes sit in the board below, grouped and collapsed:
**Running 19 · Ready for review 3 · Quiet 5**. Her browser tab title
reads `(0) ParleyInn`. The favicon dot is grey.

### Status vocabulary

The research converges on four to seven states per agent. ParleyInn takes
the union's shape, not any one product's names.

| Source                                          | States                                                                                                   |
| ----------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Codex app-server `ThreadStatus`                 | `notLoaded`, `idle`, `active` with flags `waitingOnApproval` and `waitingOnUserInput`, `systemError`     |
| Codex Micro keys                                | Idle, Thinking, Complete, Requires input, Error                                                          |
| Codex desktop pet                               | Running, Needs input, Ready, Blocked                                                                     |
| Claude Code agent view (`claude agents --json`) | state `working`, `blocked`, `done`, `failed`, `stopped`; status `busy`, `waiting`, `idle`; `waitingFor`  |
| Linear agent sessions                           | `pending`, `active`, `error`, `awaitingInput`, `complete`, `stale` (no activity within 10 s of creation) |
| Google Jules                                    | QUEUED, PLANNING, AWAITING_PLAN_APPROVAL, AWAITING_USER_FEEDBACK, IN_PROGRESS, PAUSED, FAILED, COMPLETED |
| Claude Squad                                    | Running, Ready, Loading, Paused                                                                          |
| workmux                                         | working, waiting, done; "interrupted" after 10 s of unchanged output while working                       |
| Agent Deck                                      | running, waiting, idle, error                                                                            |
| OpenAI dots Activity view                       | progress, files, results, requests for input                                                             |

Sources: [openai-agent-ui.md](../../../research/notes/openai-agent-ui/openai-agent-ui.md)
(Codex, Micro, pet, Activity), [dots.md](../../../research/notes/openai-agent-ui/dots.md),
and the sheets for Claude Code agent view, Jules, Claude Squad,
workmux and Agent Deck in
[sheets.json](../../../research/notes/agent-session-storage-sweep/sheets.json).
Linear's states are from
<https://linear.app/developers/agent-interaction>.

#### Harness status

One badge per harness (an agent session). It has two parts: an
**activity** and a **liveness** mark.

| Badge         | Colour token   | Meaning                                                                                           | Fed by (Claude Code / Codex / ACP)                                                                                        |
| ------------- | -------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| **Working**   | `--st-working` | A turn is running: model output or a tool                                                         | `UserPromptSubmit` to `Stop` / `active` with no flags / `session/prompt` in flight                                        |
| **Needs you** | `--st-needs`   | Blocked on the owner: approval, question or hand-off. Sub-label says which                        | `PermissionRequest`, `Notification:permission_prompt`, `elicitation_dialog` / `waitingOn*` / `session/request_permission` |
| **Idle**      | `--st-idle`    | Turn ended, waiting for a next prompt, nothing pending                                            | `Stop`, `Notification:idle_prompt` / `idle` / prompt response                                                             |
| **Paused**    | `--st-paused`  | Held by the owner at a safe point                                                                 | ParleyInn pause event, acknowledged by the adapter                                                                        |
| **Blocked**   | `--st-blocked` | Cannot proceed without something other than a decision: rate limit, error, crash, suspected stuck | `StopFailure`, `quota_*` notifications / `systemError` / error stop reason; watchdog events                               |
| **Ended**     | `--st-ended`   | Session closed, by the agent, the owner or a crash; resumable if the harness allows               | `SessionEnd` / `thread/closed` / process exit                                                                             |
| **Starting**  | `--st-working` | Launched, no first event yet                                                                      | `SessionStart` / `thread/started`                                                                                         |

Liveness is a separate, quiet mark beside the badge:

- **live**: the harness's machine heartbeated recently and its log is
  current here;
- **behind**: the harness is on a peer whose log has not reached this
  node lately (partition or sleep). The badge shows the last known
  activity and "as of 14:02 on desktop";
- **not proven**: ParleyInn cannot show the state, for example a harness
  observed only through transcripts with no hook events. The badge is
  outlined, not filled.

The sub-labels of **Needs you** are `approval`, `question`,
`hand-off`, `review` (lane level only) and `adopt` (a co-author's
message waiting). The sub-labels of **Blocked** are `rate limit`
(with "resumes 15:00" when the harness reports it), `error`, `crash`
and `stuck?`. The question mark matters: stuck is a suspicion, never
a fact.

Derived state reads no clock (I10). Two statuses depend on time:
"stuck?" and an expired hold. Neither is computed in the projection.
A watchdog writes an observation event (`watch.no_progress` with the
interval it saw), and the holder writes `request.expired`. The
projection only folds events, so a rebuild reproduces the same badges.

#### Lane status

A lane's badge is derived from its harnesses, its checks and its
lifecycle, in this order. The first match wins:

| Lane badge           | Rule                                                                          |
| -------------------- | ----------------------------------------------------------------------------- |
| **Needs you**        | any harness is Needs you, or an inbox item of class P1 or P2 is open          |
| **Failing**          | a required check failed on the lane's head (evidence class local run or CI)   |
| **Blocked**          | any harness is Blocked, and none is Working                                   |
| **Running**          | any harness is Working                                                        |
| **Ready for review** | every harness is Idle or Ended, the head has results, and no review is signed |
| **Quiet**            | everything else that is open                                                  |
| **Landed**           | the derived link to a landed commit is proven                                 |
| **Abandoned**        | the owner closed it without landing                                           |

Beside the badge, the lane row always shows the **evidence class** of
its newest result (claim, local run, CI), derived from structural
events only, never from text an agent wrote. This follows dots'
"completed is not verified" and proposed REC-19 in
[trace-backward.md](../trace-backward.md).

### The Needs you queue

The queue is the home screen (`g i`). It lists open **inbox items**,
one per thing the owner must decide or acknowledge. An item is an
event in the lane record, not a notification. Notifications are hints
that point at items, as the research's relay rule says ("notifications
are hints").

#### Item kinds and priority classes

| Class                    | Kinds                                                                                                                                                | Why this rank                                       |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| **P1 Blocking now**      | permission request; question or elicitation; hand-off ("you do this"); a held request about to park                                                  | an agent is idle and burning wall time on the owner |
| **P2 Blocking the lane** | failed required check; stuck agent (watchdog); crash or error stop; rate limit with no auto-resume; refused segment from a peer (I6)                 | progress stopped, but no agent is waiting on a yes  |
| **P3 Waiting on you**    | lane ready for review; co-author message awaiting the owner; parked request (the agent moved on); budget soft cap reached; partition fork to look at | nothing burns while it waits                        |
| **P4 For your record**   | turn finished, lane landed, rule changed elsewhere, digest ready                                                                                     | never notifies by default                           |

Ordering inside the queue is deterministic, so two of the owner's
machines show the same order for the same record:

1. class, P1 first;
2. inside P1, the number of agents blocked on the same answer, then
   the order in which the requests were raised;
3. inside P2 and P3, lanes the owner pinned first, then order of
   arrival.

"How long it has waited" is shown in every card, but the projection
sorts by causal order, not by age, so it never reads the clock.

#### What a card shows

```text
┌ P1  auth-refactor › claude-2 (desktop)                 waiting 40 s ┐
│ Run shell command                                                    │
│   npm install jose@5                                                 │
│ Rule: package install → Ask first        recall-taint: no            │
│ Last message: "Switching to jose for JWT verification."              │
│ [a] Allow once  [A] Allow for session  [d] Deny  [r] Reply  [o] Open │
└──────────────────────────────────────────────────────────────────────┘
```

- Source line: lane, harness, machine. The harness's own label shows
  as untrusted text in a muted face, because an agent picks it.
- The action, rendered from the structured tool input, escaped. Tool
  names and arguments are untrusted display content, as the OpenAI
  Agents SDK warns about its own approvals.
- The rule that routed it here, and the session's recall-taint flag
  (SEC-13). When taint is set the card says "this agent has read
  untrusted data" in one quiet line.
- The agent's last message, collapsed to one line, marked untrusted.
- The buttons, with their keys.

#### Grouping and batching

- **Same answer, many agents.** Codex groups network approvals by
  host, protocol and port so one prompt unblocks several queued
  requests. ParleyInn groups requests whose normalised action and rule
  are identical: "3 agents want `npm install`". Pressing `a` on the
  group opens a checklist, all ticked, so the owner sees every lane
  before it goes.
- **Never batch** actions at the hand-off level, actions from a
  tainted session, or anything in the "always ask" set (push to a
  protected branch, deleting outside the worktree, touching secrets).
- **Collapse per lane.** A lane with more than three open items shows
  one row, "auth-refactor · 5 items", which expands in place.

#### Keyboard

| Key                   | Action                                                  |
| --------------------- | ------------------------------------------------------- |
| `j` / `k`             | next / previous item                                    |
| `a` / `A`             | allow once / allow for this session                     |
| `d`                   | deny (asks for an optional reason the agent will see)   |
| `r`                   | reply: answer a question or deny with an instruction    |
| `Shift+A`             | adopt a co-author's message as your instruction         |
| `o`                   | open the lane at this item                              |
| `e`                   | snooze (`30m`, `2h`, `tomorrow`, `until it changes`)    |
| `x`                   | dismiss a P3 or P4 item (P1 and P2 cannot be dismissed) |
| `space`               | select, for batch actions                               |
| `g i` / `g b` / `g d` | go to inbox / board / digest                            |
| `/`                   | filter by lane, repo, machine, kind                     |

The verbs match Codex's approval decisions (`accept`,
`acceptForSession`, `decline`, `cancel`) so an adapter maps them one
to one.

### The fleet board

Below the queue, or full screen with `g b`. It is built for 20 to 50
lanes and a few hundred harnesses.

- **Groups**, collapsible, in this order: Needs you, Failing, Blocked,
  Running, Ready for review, Quiet. Landed and Abandoned are behind a
  filter. Alternative groupings: repository, machine, label.
- **Row columns:** status badge · lane name and branch · harness dots
  (one per agent, coloured by harness status, children nested as
  smaller dots) · open items · last activity, as "3 events ago" by
  default with relative time on hover · evidence badge · tokens and
  estimated cost today · machine(s) · co-authors' faces.
- **Focus set:** up to six pinned lanes render as live tiles above the
  list, each streaming its harness. Six is Codex Micro's key count and
  the most tiles that stay readable at 1440 px. Everything else is a
  28 px row.
- **Density:** row heights stay fixed while statuses change, so the
  list does not jump under the cursor. New rows enter at the bottom of
  their group with a short highlight.

### Notifications that stay on your machines

Channels, from nearest to farthest. All of them read the same inbox
items, and none needs a vendor push service.

| Channel               | How                                                                                                                          | Boundary     |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------- | ------------ |
| In page               | count in the tab title `(3) ParleyInn`, a favicon dot coloured by the top class, the queue badge, at most two toasts at once | B1           |
| Desktop               | the browser's Notification API, raised by the open loopback tab (loopback counts as a secure context); click opens the item  | B1           |
| Terminal: harness     | Claude Code's `Notification` hook can return a `terminalSequence`; ParleyInn's hook emits BEL or OSC 9 or OSC 777 there      | B0           |
| Terminal: any shell   | `parinn inbox --follow` prints one line per new item and rings the bell; `parinn status --line` feeds tmux or a prompt       | B0           |
| Terminal: status line | Claude Code's status line runs `parinn status --line`, so every harness terminal shows "⚑ 2 other lanes need you"            | B0           |
| Other owned machines  | with peering on, inbox items replicate as lane events; each machine alerts locally                                           | B2           |
| Phone or chat         | only through an opt-in bridge or an enrolled phone peer, below                                                               | B2 or bridge |

Rules that keep this quiet:

- Desktop and sound alerts fire only for P1 and P2 by default, and
  only while the page is hidden. Zed's settings give the shape:
  `notify_when_agent_waiting` and `play_sound_when_agent_done` with
  `when_hidden`. ChatGPT desktop offers never, in the background, or
  always ([Zed commit](https://git.secluded.site/zed/commit/f8f36d0c1716a5ce60bb2c5edf2ec096bf5fcede),
  [Codex notifications](https://learn.chatgpt.com/docs/notifications.md)).
- At most one desktop alert per lane per five minutes. Further items
  in that window coalesce into "auth-refactor: 3 more".
- An item answered on any surface raises `request.resolved`. Every
  machine clears its toast, its desktop alert (via the notification
  tag) and its terminal line. This copies Codex's
  `serverRequest/resolved` broadcast.
- Alerts follow presence. When peering is on, the machine where the
  owner last typed or clicked alerts first. The others wait 60 s and
  alert only if the item is still open.
- Web Push is not used. It routes through the browser vendor's push
  service, which is a central service under decision 3.

#### What an opt-in bridge would need

A bridge carries "something needs you" to a phone, chat or email when
no owned machine is in front of the owner. It is a separate process,
never in the core or the UI server (I4). Requirements:

1. Off by default. It is enabled per destination, by an explicit
   owner act, recorded as an event.
2. Outbound only. It holds no listening socket.
3. **Hints, not content.** The default payload is a lane alias of the
   owner's choosing, the item class and a count, for example
   `lane-7: 1 approval`. Commands, file names, messages and branch
   names stay home unless the owner widens it per lane.
4. **No answers through the bridge.** A reply typed in a chat app is
   untrusted text from a third party's server. To answer from a phone,
   the phone must be an enrolled owner device: a peer holding a device
   key the owner signed. Its answer is a signed event that arrives over
   B2, not through the bridge.
5. Rate-limited, and every send and failure counted and audited (I6).
   Its silence must never read as "nothing happened".
6. Self-hostable targets first: a self-hosted ntfy, Matrix or the
   owner's own mail server. A hosted target is the owner's choice and
   is labelled in the settings as "leaves your machines".

Claude Code Remote Control and Codex Remote show the cost of a vendor
relay: one account, one workspace, a relay on the vendor's domain
([openai-agent-ui.md §4](../../../research/notes/openai-agent-ui/openai-agent-ui.md)).
The bridge avoids that because it carries no authority.

### Quiet hours and digests

- **Quiet hours** are a weekly schedule per owner, set in the UI and
  stored as owner-signed config. During them, nothing alerts except
  items the owner marked **break through**. The defaults are P1 items
  in lanes labelled `urgent`, and security items (a refused segment, a
  signature failure).
- **Away mode** is the same switch, turned on by hand (`Shift+Q`) or
  by the owner's machine sleeping. It also applies the lane's **away
  policy** (below).
- **The digest** (`g d`) is a projection over a window the owner
  picks: "since I left", "today", "since my last review of this
  lane". The window is an input to the projection, so the projection
  still reads no clock. Sections, in order:
  1. Decisions waiting, in queue order.
  2. Finished, with the evidence class of each result.
  3. Failed or blocked, with the first error.
  4. What agents did on their own in sensitive categories: actions
     allowed by an "act without asking" rule that touched pushes,
     deletes, installs or config.
  5. Parked requests, and what each agent did instead.
  6. Spend per lane and per agent, against budgets.
  7. Security marks: refused segments, flagged untrusted content
     (PRV-07), taint changes.
- Every line links to the event in the record. The digest is built
  from structured events only. It is not a model's summary of the
  lanes, since a summary is lossy and a poisoning channel (NG7).

### Cost and quota

Process gap 9 notes that "If an untrusted message can wake an agent,
it is a cost denial-of-service path". The view shows spend where the
decision is made.

- **Per agent**, in the harness header: input, output and cache
  tokens, turns, wall time and an estimated cost. Token counts are
  `harness_meta`, structural and trusted (PRV-02). Codex streams
  `thread/tokenUsage/updated`. Claude Code transcripts carry usage per
  message.
- **The estimate is labelled an estimate.** Prices come from a local
  table that ships with the release and that the owner can edit.
  ParleyInn never fetches prices.
- **Quota**: the harness's own limit signals, such as Claude Code's
  `quota_auto_resume_*` notifications or a rate-limit stop reason,
  set **Blocked · rate limit** with "resumes 15:00" when known.
  Agents on one account share one quota, so the board shows the
  account's state once, above the groups, not 30 times.
- **Per lane and fleet**: today, this week and the lane's lifetime,
  as columns on the board and a strip in the lane header.
- **Budgets** per lane, set by the owner:
  - soft cap: a P3 item "auth-refactor passed $20 today";
  - hard cap: the lane pauses at the next safe point (see Pause
    under Area 3), with a P2 item. ParleyInn records the pause and its
    reason.
- **Nobody but the owner wakes an agent.** A co-author's message
  never starts a turn. Codex's agent message board makes the same
  rule for posts from other agents: a notice "must never start a new
  turn" ([openai-agent-ui.md §10](../../../research/notes/openai-agent-ui/openai-agent-ui.md)).
  So a co-author cannot spend the owner's quota.

### When the owner is away for hours

Each lane has an **away policy**. The owner sets a fleet default and
can override it per lane.

| Policy                     | When a request would block                                                                               |
| -------------------------- | -------------------------------------------------------------------------------------------------------- |
| **Keep going** (default)   | Hold for the hold window, then park. The agent is told "held for the owner as R-41; continue other work" |
| **Pause at the first ask** | Hold, then pause the agent at that point. Nothing else runs in the lane until the owner returns          |
| **Stop at the first ask**  | Hold, then stop the session cleanly. The lane goes to Blocked with a P2 item                             |

A night, for Maya's 31 lanes, with **Keep going**:

- 23:40: `auth-refactor` asks to push. Quiet hours are on, so nothing
  alerts. After the hold window the request parks. The agent writes
  tests instead and goes Idle.
- 01:15: `flaky-ci` hits the account rate limit. Every agent on that
  account shows Blocked · rate limit, "resumes 05:00". Claude Code
  auto-resumes at 05:00 and the badges go back to Working.
- 02:30: the watchdog on the desktop sees `peer-sync` working with no
  new event for 20 minutes and writes `watch.no_progress`. The owner's
  configured nudge, a fixed text "If you are stuck, write what blocks
  you and stop", is sent once. It is static owner config, like PIN-07's
  fixed compaction guidance, so it carries no record content. Gas
  Town's Witness does the same job with nudges and escalations
  ([process gap 8](../../../research/notes/live-pr-pitch-review/process-gaps.md)).
- 04:00: the desktop sleeps. The laptop is closed. The sandboxes keep
  running and keep their logs. Their events reach the desktop when it
  wakes, and every badge shows **behind** until then.
- 08:50: Maya opens the laptop. The page opens on the digest, "since
  23:12", with 2 parked requests, 6 finished lanes (4 with CI
  evidence), 1 stuck, $41 spent. Pressing `a` on a parked push sends
  an owner instruction to that agent: "R-41 approved: push
  `auth-refactor`". The agent's next turn starts from it.

### Attention edge cases

- **UI not running.** Harnesses keep working and hooks keep recording
  items. Requests fall back to the harness's own prompt in its
  terminal. When the UI starts, the queue shows everything, including
  items answered in terminals meanwhile.
- **Partition.** Each side shows its own items. An item raised on the
  far side appears when its log arrives, marked **behind**. Answers
  made on both sides of a split are covered under steering edge cases.
- **Fifty lanes erupting at once**, for example after a shared
  dependency breaks. The queue groups identical P2 items ("31 lanes:
  `go test` failed on the same package") and alerts once.
- **A noisy agent** that raises 40 questions an hour. The lane gets a
  per-lane alert cap and a P3 item suggesting a stricter rule or a
  pause, never a silent drop. Every suppressed alert is counted (I6).
- **A forged status.** An agent prints "All checks passed" or
  "WAITING FOR APPROVAL" in its output. Badges come only from
  structural events (hook payloads, exit statuses, signed check
  results), never from text, so the badge does not move.
- **Clock skew between machines.** Ordering is causal. Relative times
  are shown with the machine they came from.

### What attention borrows

| From                   | What                                                                                      | Source                                                                                                 |
| ---------------------- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Codex app-server       | `ThreadStatus` with waiting flags; `serverRequest/resolved`; grouped network approvals    | [openai-agent-ui.md §2](../../../research/notes/openai-agent-ui/openai-agent-ui.md)                    |
| Codex desktop and iOS  | Activity view of unread, running and waiting chats; iOS Priority view; pet states         | [openai-agent-ui.md §1](../../../research/notes/openai-agent-ui/openai-agent-ui.md)                    |
| Codex Micro            | six keys that light by status, hence a focus set of six                                   | [Codex Micro](https://learn.chatgpt.com/docs/features/codex-micro.md)                                  |
| OpenAI dots            | requests for input at the top of the Activity view; completed is not verified             | [dots.md](../../../research/notes/openai-agent-ui/dots.md)                                             |
| Claude Code agent view | `state` plus `status` plus `waitingFor`; `agent_needs_input` and `agent_completed` events | [sheets.json](../../../research/notes/agent-session-storage-sweep/sheets.json), Claude Code agent view |
| Warp                   | a notification mailbox with All, Unread and Errors; at most two toasts                    | [Warp agent notifications](https://docs.warp.dev/agents/capabilities/agent-notifications/)             |
| Zed                    | notify when waiting, sound only when hidden                                               | [Zed commit](https://git.secluded.site/zed/commit/f8f36d0c1716a5ce60bb2c5edf2ec096bf5fcede)            |
| Linear                 | `awaitingInput` and `stale` as first-class states                                         | <https://linear.app/developers/agent-interaction>                                                      |
| Gas Town, workmux      | watchdogs for stuck agents; "interrupted" after unchanged output                          | [sheets.json](../../../research/notes/agent-session-storage-sweep/sheets.json)                         |
| Symphony               | approvals "MUST NOT leave a run stalled indefinitely"; the dashboard is never required    | [openai-agent-ui.md §5](../../../research/notes/openai-agent-ui/openai-agent-ui.md)                    |

### What ParleyInn does differently in attention

- **The queue is in the record, not in a notification service.** An
  item is an event, so it survives restarts, crosses partitions with
  the lane and is answered once across every surface. Codex's
  Activity view and Claude Code's notifications are per client.
- **No vendor push.** Every product above with phone alerts routes
  them through its own cloud. ParleyInn needs either an owned peer or a
  bridge that carries no authority.
- **Status cannot be forged by text.** Badges and evidence come from
  structural events. That matters more here than elsewhere, because a
  wrong badge steers the owner's attention, which is itself an attack
  surface.
- **Stuck is an observation, not a computation.** Watchdog events keep
  the projection free of clocks (I10).
- **Co-author messages are queue items, not prompts.** They wait for
  the owner and never start a paid turn.

### Requirements implied by attention

- The view MUST show one status per harness from a closed set
  (Starting, Working, Needs you, Idle, Paused, Blocked, Ended) and a
  liveness mark (live, behind, not proven).
- Harness and lane status MUST be derived only from structural events
  and MUST NOT change because of text an agent or a co-author wrote.
- Status projections MUST read no clock. Time-dependent states (stuck,
  expired) MUST come from recorded observation events.
- Every request for the owner MUST be recorded as an inbox item with
  a stable id, a class (P1 to P4) and its source lane, harness and
  machine.
- The inbox order MUST be a deterministic function of the record, so
  two nodes holding the same events show the same order.
- Answering an item on any surface MUST record a resolution event,
  and every other surface MUST clear the item when it receives it.
- P1 and P2 items MUST NOT be dismissible. They leave the queue only
  when resolved.
- Notifications in standalone MUST use only in-page, browser
  Notification API, terminal and hook channels, and MUST NOT use a
  vendor push service.
- Every suppressed, coalesced or failed notification MUST be counted
  and visible in `parinn status` (I6).
- A bridge to phones or chat, if offered, MUST run outside the core
  and the UI server, MUST be off by default, MUST send no lane content
  by default, and MUST NOT accept answers.
- A message from anyone but the owner MUST NOT start an agent turn.
- Each harness MUST show token use and, when a price table is present,
  a cost labelled as an estimate. ParleyInn MUST NOT fetch prices.
- A lane with a hard budget cap MUST pause at the next safe point once
  the cap is passed, and MUST record why.
- The digest MUST be built from structured events with links to them,
  and MUST NOT be a model-written summary.

## Area 3: steering agents

### Steering: the user's goal

"When an agent goes the wrong way, let me correct it without losing
its work. When it asks, let me answer from wherever I am. When I say
stop, tell me exactly what has already happened. And nobody but me
gets to do any of this."

### Steering journey

At 10:20 Maya watches `auth-refactor` in the big pane. The agent is
rewriting the token cache she wanted left alone.

1. She types into the composer at the bottom of the harness pane,
   "Leave `cache.go` alone; only change the verifier", and presses
   `Enter`. The button reads **Steer**: the message joins the running
   turn at the next tool boundary. A chip "steer · delivered at tool
   12" appears in the timeline, marked with her key.
2. The agent keeps editing `cache.go`. She presses `Esc` twice:
   **Interrupt**. The turn ends as interrupted. The timeline shows
   what the turn had finished: two files edited, one test run.
3. She opens **Revert…** on the turn and picks "restore `cache.go`
   from before turn 7". That is a worktree change she can undo, and
   it is recorded as her edit.
4. She sends a new instruction. It is a new turn.
5. At 10:45 the agent asks to `git push --force-with-lease`. Her rule
   for force pushes is **Hand off**. The agent is told "the owner does
   this", and a P1 card "You do this: force-push `auth-refactor`"
   appears with the agent's reason.
6. She presses `t` to **take over** the harness's terminal in the UI,
   runs the push herself, then presses **Hand back** with the note
   "pushed; continue with the README". The agent resumes from her
   note.

That evening she watches from the couch on her laptop, while the
harness runs on the desktop. A permission card appears on both
machines. She answers on the laptop. The desktop's terminal prompt
clears with "answered from laptop".

### Steering controls

Each harness pane has a control strip. Each child agent has the same
controls in its row of the agent tree.

| Control       | Key           | What it does                                                           | Recorded as                     |
| ------------- | ------------- | ---------------------------------------------------------------------- | ------------------------------- |
| **Steer**     | `Enter`       | adds owner input to the running turn, read at the next tool boundary   | `steer` with the target turn id |
| **Queue**     | `Ctrl+Enter`  | holds owner input for the start of the next turn                       | `steer` marked `next_turn`      |
| **Interrupt** | `Esc Esc`     | ends the current turn as interrupted; the session stays                | `interrupt`, then the ack       |
| **Redirect**  | `Shift+Enter` | interrupt plus a new instruction, in one act                           | `interrupt` then `prompt`       |
| **Pause**     | `p`           | holds the agent at the next safe point; queued input waits             | `pause`, then the ack           |
| **Resume**    | `p`           | releases a pause, optionally with a note                               | `resume`                        |
| **Stop**      | `Shift+S`     | ends the session after the stop sheet (below)                          | `stop`, then the ack            |
| **Fork**      | `f`           | starts a new lane or worktree from turn N, to try another direction    | `fork` naming the parent turn   |
| **Take over** | `t`           | gives the owner the harness's terminal input (if the adapter hosts it) | `takeover` and `release`        |
| **Hand back** | `h`           | ends a hand-off; the agent resumes with the owner's note               | `handback`                      |

Two keys guard the destructive end. Interrupt needs a double `Esc`,
because Claude Code users already press `Esc` once to cancel input.
Stop always opens a sheet first.

A steer names the turn it was written for, like Codex's `turn/steer`
with `expectedTurnId`. If that turn ended before the steer arrived,
the steer is not dropped and not slipped into a different turn. The
composer turns it into a queued message and says "turn 7 ended before
this arrived; send at the start of turn 8?".

#### Pause is a ParleyInn concept

Most harnesses have interrupt and resume, but no pause. ParleyInn builds
pause from the parts each adapter offers:

1. The adapter refuses the next tool call with "paused by the owner"
   (a `PreToolUse` deny for Claude Code, a declined approval for
   Codex, a refused permission for ACP).
2. Then it interrupts the turn.
3. Queued input stays queued. **Resume** sends the owner's note, or a
   fixed "resume" text, as the next prompt.

A paused agent shows **Paused**, not Idle, so the owner can tell "I
stopped it" from "it finished".

### Held requests: answer from anywhere, now or later

Every permission prompt, question and hand-off becomes a **held
request** in the lane record. This is Codex's server request (a
request with an id that any subscribed client may answer, followed by
`serverRequest/resolved`), carried in a log that crosses machines.

```text
requested ──▶ answered (allow | allow_session | deny | reply)
    │                └─▶ delivered ──▶ resolved
    ├──▶ answered in the harness (native prompt) ──▶ resolved
    └──▶ expired ──▶ parked ──▶ answered later ──▶ delivered as an
                                                   owner instruction
```

- **Id.** `R-<lane>-<n>`, bound to the harness's own id where one
  exists (`tool_use_id` in Claude Code, the JSON-RPC request id in
  Codex and ACP), so the record and the harness agree on which prompt
  was answered.
- **Any surface answers.** The UI on any of the owner's machines, the
  harness's own prompt in its terminal, `parinn answer R-41 allow` in
  any shell, or an enrolled owner phone. The first valid answer in
  causal order wins. Later answers are recorded as superseded and
  shown as "also answered from laptop".
- **The hold window** is set per rule level and capped by what the
  adapter can hold. Claude Code's `PermissionRequest` hook can wait up
  to its timeout (600 s by default for command hooks); the Agent SDK's
  permission callback, an ACP client and a Codex client can wait as
  long as they like. The default window is 3 minutes.
- **Expiry parks; it never allows.** When the window ends, the holder
  writes `request.expired` and answers the harness with a deny whose
  reason is fixed text: "Held for the owner as R-41. Continue with
  other work or end your turn." The request becomes **parked**: a P3
  item.
- **Late answers.** Approving a parked request sends an owner
  instruction to the agent, "R-41 approved: run `npm install jose@5`",
  at its next turn or on resume. It is delivered through the steering
  path, so it is signed and `operator`-class.
- **ParleyInn failure falls back to the harness, never to allow.** If
  ParleyInn crashes while holding, the hook times out or exits without a
  decision. The harness then shows its own prompt. That is fail-open
  for the agent (I9) without granting anything.

### Rule levels per action

Dots' four levels give the owner's control a familiar shape. ParleyInn
applies them per action class, beside I2's rule on what an agent
reads.

| Level                  | Effect in ParleyInn                                                                                 | Claude Code mapping                            |
| ---------------------- | --------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| **Act without asking** | allowed; listed in the digest when the class is sensitive                                           | `PreToolUse` → `allow`                         |
| **Act when told**      | allowed only under a live grant the owner gave: "Do it" on a card, or `/allow push once` in a steer | `allow` if a matching grant exists, else `ask` |
| **Ask first**          | a held request, P1                                                                                  | `ask` or `PermissionRequest` held              |
| **Hand off**           | denied with "the owner does this"; a P1 hand-off card with the agent's reason and context           | `deny` with a fixed reason                     |

- **Action classes**, each with patterns the owner can edit: shell
  commands by prefix; writes outside the worktree; package installs;
  `git push`, force pushes and pushes to protected branches; network
  fetches; deleting files; edits to CI, hooks and agent config;
  reading secrets; landing or merging.
- **Scope**: all lanes, one repository or one lane. The narrowest
  scope wins.
- **Repository files can only tighten.** A checked-in rule file may
  raise a class from Act to Ask first, never lower it. Claude Code's
  settings precedence makes the same rule for repository config
  ([sheets.json](../../../research/notes/agent-session-storage-sweep/sheets.json),
  cross-session messaging).
- **Taint raises the level.** Once a session has recalled untrusted
  content (SEC-13's recall-taint flag), every sensitive class moves up
  one level for that session: Act becomes Ask first. The card says
  why.
- **Remembering a choice.** "Allow for session" grants until the
  session ends. "Always in this lane" opens the rule editor
  pre-filled, so a standing rule is a deliberate edit, not a
  side effect.
- **The rule editor** is a table: class · pattern · level · scope ·
  source (you, or a repository file marked "tighten only"). A test box
  shows which rule a sample command would hit. Every rule change is an
  owner-signed event, so the record shows what an agent was allowed to
  do at each moment.

### Handing work to the human and back

1. **The agent hands off.** A hand-off rule fires, or the agent asks
   for help: Claude Code's elicitation, an ACP permission it cannot
   get, or the agent saying it is blocked. The harness goes to
   **Needs you · hand-off** and the lane to Needs you.
2. **The card** says what the owner should do, why, and what the
   agent will do after, all marked as the agent's untrusted words.
   Buttons: **Take over** (`t`), **Open worktree** (copies the path),
   **Decline** (`d`, with a reason the agent sees).
3. **The owner works.** In the taken-over terminal, in their editor,
   or anywhere. Edits made outside the agent's tool calls reach the
   record through a worktree checkpoint (proposed REC-18) when the
   owner presses Hand back.
4. **Hand back** (`h`) asks for an optional note. The agent resumes
   with the note as an owner instruction and a pointer to the
   checkpoint. The checkpoint's content is `file` provenance and
   untrusted (PRV-01). Only the note carries the owner's authority.

The owner can also take work from an agent unasked: **Interrupt**,
then **Take over**. Hand back works the same way.

### Taking over a harness's terminal

The core holds no `os/exec` (I4, B0), so it cannot host a terminal.
Two cases:

- **Hosted harness.** The owner starts the harness through a launcher,
  `parinn run -- claude …`. The launcher is a separate B1 process that
  owns the pseudo-terminal, the way Claude Squad, Agent Deck and
  Superset host theirs. The UI then offers:
  - **Watch** (default): a read-only stream of the terminal;
  - **Take over** (`t`): an exclusive input lock. The local terminal
    shows a banner, "controlled from the ParleyInn UI on laptop · press
    `Ctrl+]` to take back". One writer at a time, as Claude Code's
    Remote Control does with "Another connection took over this
    session";
  - **Release** (`t` again, or `Ctrl+]` locally).
- **Unhosted harness** (started directly in the owner's own terminal).
  The UI shows **Go to terminal**, which names the machine and, when
  known, the tmux pane, plus a hint for next time: "start with
  `parinn run` to take over from here".

Keystrokes typed during a takeover are owner input and are recorded.
Input typed at a no-echo prompt, such as a password, is recorded only
as "input withheld (no echo)" (redaction before storage, I1's
exception). Taking over a terminal on another machine works only
between the owner's own enrolled machines, over B2.

### Steering background and child agents

- **The agent tree.** Each harness pane lists its children: subagents,
  background agents and teammates. Each has its own badge, cost and
  controls. This copies Codex's parent and child threads and dots'
  "separate, visible threads".
- **Requests bubble up labelled.** A child's request appears in the
  queue as "auth-refactor › claude-2 › test-writer". `o` opens that
  child, like the Codex CLI's `o` on an approval from an inactive
  thread.
- **Steer a child directly** when the adapter can address it; else
  steer the parent with "tell test-writer to …", and the tree marks
  that the instruction went through the parent.
- **Stop all children** (`Shift+S` on the tree header) and stop one
  child. A parent stop asks whether to stop its children too, and
  defaults to yes.
- **Children cannot widen rules.** A child inherits its parent's
  rule levels and taint, and may only tighten them. Claude Code's
  agent teams let teammates inherit the lead's permission mode, and
  let the lead auto-grant plan approvals; ParleyInn keeps every approval
  with the owner ([process gap 2](../../../research/notes/live-pr-pitch-review/process-gaps.md)).
- **A child's result is `subagent_result`**, untrusted. A message from
  one agent to another is never the owner's consent, as Claude Code
  already marks peer messages.

### What stop means

Dots' docs say it: "Stopping work doesn't undo completed actions."
GitHub Copilot's stop "ends the Actions run and keeps commits already
pushed" ([sheets.json](../../../research/notes/agent-session-storage-sweep/sheets.json),
Copilot cloud agent). ParleyInn makes that visible before and after the
stop, through the **stop sheet**:

```text
Stop claude-2 in auth-refactor?

Already done (stop will not undo these)
  ✓ 14 files edited in the worktree          [Revert worktree to…]
  ✓ 3 commits on auth-refactor               [Revert commits…]
  ✓ pushed to origin/auth-refactor 10:31     cannot be undone here
  ✓ ran `npm install jose@5`                 lockfile changed
  ✓ 2 network fetches                        cannot be undone here

In flight
  ▸ `go test ./...` running for 40 s         (•) let it finish  ( ) kill it

Waiting
  2 queued messages from you                 will be kept, not sent
  R-44 held request                          will be denied

[Stop]  [Stop and revert worktree]  [Cancel]
```

- **Done** lists effects the record can show: edits, commits, pushes,
  commands with side effects, outbound fetches. Each says whether
  ParleyInn can offer an undo, and of what kind. Revert in the worktree
  is local and reversible. Reverting commits makes new revert commits.
  A push, an external call or a sent message "cannot be undone here".
- **In flight** lets a running tool finish by default. Killing it may
  leave a half-written state, so it is the explicit choice.
- **Stop is idempotent** and needs the harness's acknowledgement. Until
  the ack event arrives, the badge says "stop sent · not confirmed".
  Over a partition it stays that way until the far side's log arrives.
  The view never shows Ended without the ack (I6).
- **A resumed or restarted session after a stop is a new attempt**,
  as Symphony's spec says of reruns after side effects. The timeline
  shows the boundary.

### Owner authentication for steering

Steering is the one place ParleyInn writes text that reaches a model.
Under I2 only `TrustedText` may do that, so steering input must be
proven to come from the owner. Under PRV-04, in the default
automation mode, even the harness's own `user` turns are untrusted,
so the owner's authority cannot rest on "it was typed into the
harness".

Who counts as the owner on each surface:

| Surface                        | Proof                                                                                                                                                                       | Class                       |
| ------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------- |
| UI on this machine             | a one-time launch code exchanged for an HttpOnly, SameSite=Strict cookie; Host and Origin checks (SEC-19); the answer signed with this machine's owner device key           | `operator`                  |
| UI, sensitive actions          | the above plus user presence: a WebAuthn assertion with user verification (a fingerprint or a device PIN) for Hand off, force pushes, rule changes that loosen, and landing | `operator`, marked presence |
| `parinn answer` in a shell     | an interactive terminal plus a short code the UI or the prompt shows ("type 7K2Q to confirm"); refused without a terminal                                                   | `operator`                  |
| The harness's own prompt       | whatever the harness checks; ParleyInn records the outcome as the harness reported it                                                                                       | `harness_meta`              |
| Another owned machine (B2)     | an event signed by an owner device key the owner enrolled                                                                                                                   | `operator`, imported        |
| An enrolled phone              | the same: a device key, signed answers over B2                                                                                                                              | `operator`, imported        |
| A co-author                    | their own key. Never steering                                                                                                                                               | untrusted (PRV-09)          |
| A bridge message, a chat reply | none                                                                                                                                                                        | ignored, audited            |

The threat that shapes this table: **an agent approving its own
request.** An agent runs as the same OS user as the owner. Without a
sandbox it can read files, call the loopback port and run
`parinn answer`. The defences, in layers:

1. Nothing an agent can read holds the UI credential. The launch code
   is single-use and is shown only once, to the owner, in their
   terminal or browser.
2. `parinn answer` refuses without an interactive terminal and a code
   that only the owner's screen shows. An agent's shell tool has no
   such terminal.
3. User presence for sensitive actions. An agent cannot press a
   fingerprint reader.
4. `parinn doctor` checks the harness's sandbox, if one is set: is the
   UI port reachable from inside? Is the key store readable? An
   agent running without a sandbox gets a quiet mark on its tile:
   "unsandboxed · owner checks are best effort".
5. Every answer records its surface and its proof, so a self-approval
   that slipped through is visible in the record afterwards.

**Adopting a co-author's words.** A co-author's message reaches the
owner's queue, never the agent. The owner may **Adopt** it (`Shift+A`).
The composer opens with the text, which the owner may edit, and a
header naming the source event. Sending signs an endorsement: "owner
instruction, adopted from Jon's event e7f3". The trust comes from the
owner's act, not from Jon's text. This is option (b) of the proposed
OQ-14, and process gap 2's "explicit, audited act by which the owner
adopts a reviewer's request as an instruction".

### Harness adapters and what they can honour

ParleyInn can only steer through what each harness exposes. The view greys
out a control the adapter lacks and says why on hover.

| Capability             | Claude Code via hooks                            | Claude Code via `parinn run` or the Agent SDK | ACP agent                    | Codex app-server                        |
| ---------------------- | ------------------------------------------------ | --------------------------------------------- | ---------------------------- | --------------------------------------- |
| Observe live           | hooks and transcript                             | stream                                        | `session/update`             | `item/*`, `turn/*`                      |
| Answer approval        | `PermissionRequest` hook, held up to its timeout | permission callback, no time limit            | `session/request_permission` | `requestApproval` server requests       |
| Answer a question      | native prompt only, or via the terminal takeover | yes                                           | `elicitation/create`         | `waitingOnUserInput` requests           |
| Steer mid-turn         | no; queue for the next turn                      | yes, at a tool boundary                       | no; queue                    | `turn/steer` with `expectedTurnId`      |
| Interrupt              | no (only from the terminal)                      | yes                                           | `session/cancel`             | `turn/interrupt`                        |
| Pause, resume          | deny at `PreToolUse`; resume via the terminal    | yes                                           | refuse plus cancel; prompt   | decline plus interrupt; `turn/start`    |
| Stop                   | no (only from the terminal)                      | yes                                           | `session/cancel`, close      | `turn/interrupt`, unsubscribe           |
| Take over the terminal | no                                               | yes, with the launcher                        | no                           | no (the TUI can attach with `--remote`) |
| Child agents           | `SubagentStart` and `SubagentStop`, observe only | observe; steer through the parent             | per adapter                  | `collabAgentToolCall`, child threads    |

Sources: Claude Code hooks at <https://code.claude.com/docs/en/hooks>
(the `PermissionRequest` decision object, `PreToolUse` decisions
`allow`, `deny`, `ask` and `defer`, `Stop` with `decision: block`, the
`Notification` types and the hook timeouts); Codex and ACP in
[openai-agent-ui.md §2](../../../research/notes/openai-agent-ui/openai-agent-ui.md).

The plain-hooks column is weak by nature. That is the honest reason
to offer `parinn run`: it is how Claude Code gets the full strip.
Neither path changes the agent's configuration without an explicit
install step that shows the diff (I7).

### Steering edge cases

- **Two answers across a partition.** The laptop allows R-41 while
  the desktop, cut off, denies it. When the logs meet, the earlier
  answer in causal order stands if one exists. If the two are
  concurrent, deny wins: the safer outcome. A P3 item says "answered
  twice; deny applied", and both events stay in the record.
- **A steer to a harness that is behind.** It is signed and appended
  here at once, and shows "not yet delivered" until the far side's ack
  arrives. It is never re-sent as a new message.
- **Stop while the agent is mid-push.** The stop sheet lists the push
  as in flight. "Let it finish" is the default.
- **The agent asks the same thing again after a deny.** The card shows
  "asked again · denied 2 min ago" and offers "deny for this session".
- **The owner steers with pasted text from a co-author** without
  using Adopt. It is the owner's act and goes as an owner instruction.
  The composer notices pasted text that matches an untrusted event and
  offers to record it as an adoption, so the provenance is not lost.
- **The UI is open on two machines and both take over one terminal.**
  The second takeover asks to steal the lock, and the first side shows
  "taken over from desktop".
- **Automation mode with no owner present.** Nothing in this area runs
  without an owner. Held requests park, and the away policy applies.

### What steering borrows

| From                     | What                                                                                                      | Source                                                                                                                          |
| ------------------------ | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Codex app-server         | server requests with ids; `turn/steer` with `expectedTurnId`; `turn/interrupt`; decisions                 | [openai-agent-ui.md §2](../../../research/notes/openai-agent-ui/openai-agent-ui.md)                                             |
| Codex subagents          | approvals from inactive threads labelled with their source; `o` to open                                   | [openai-agent-ui.md §6](../../../research/notes/openai-agent-ui/openai-agent-ui.md)                                             |
| OpenAI dots              | four rule levels; "Stopping work doesn't undo completed actions"; visible child threads                   | [dots.md](../../../research/notes/openai-agent-ui/dots.md), [Control your dot](https://learn.chatgpt.com/docs/dots/controls.md) |
| Claude Code              | `PermissionRequest` and `PreToolUse` decisions; peer messages are never consent; one host holds a session | <https://code.claude.com/docs/en/hooks>, [competitors.md](../../../research/notes/live-pr-pitch-review/competitors.md)          |
| GitHub Copilot           | stop keeps commits already pushed                                                                         | [sheets.json](../../../research/notes/agent-session-storage-sweep/sheets.json)                                                  |
| Symphony                 | a rerun after side effects is a new attempt                                                               | [openai-agent-ui.md §5](../../../research/notes/openai-agent-ui/openai-agent-ui.md)                                             |
| Claude Squad, Agent Deck | a hosted terminal per agent                                                                               | [sheets.json](../../../research/notes/agent-session-storage-sweep/sheets.json)                                                  |
| Conductor                | messages sent mid-turn are queued or "steered into the running turn"                                      | [sheets.json](../../../research/notes/agent-session-storage-sweep/sheets.json)                                                  |

### What ParleyInn does differently in steering

- **The owner is a key, not a login.** Codex Remote and Claude Code
  Remote Control prove the owner through a vendor account. ParleyInn
  proves them with a device key and, for sensitive actions, presence.
  It works offline and across partitions.
- **Any client is not any writer.** Codex's `thread/inject_items` lets
  every connected client append model-visible items. ParleyInn's UI is a
  client of the record, and only owner-authenticated acts become
  `operator` events.
- **Unanswered never means yes.** Holds park and deny. A ParleyInn failure
  falls back to the harness's own prompt.
- **Co-authors are heard and never obeyed.** Adopt is the one path,
  and it names its source. No shipping multiplayer lane product does
  this ([competitors.md](../../../research/notes/live-pr-pitch-review/competitors.md),
  claim C8).
- **Stop shows its limits.** The sheet lists what is done, in flight
  and waiting, and which undo applies to each.

### Requirements implied by steering

- Every permission request, question and hand-off MUST be recorded as
  a held request with a stable id bound to the harness's own request
  id where one exists.
- Any owner surface MUST be able to answer a held request. The first
  valid answer in causal order MUST win, and later answers MUST be
  recorded as superseded.
- A held request that expires MUST be denied to the harness with fixed
  text and parked. An expired or failed hold MUST NOT allow the action.
- If ParleyInn fails while holding a request, the harness MUST fall back to
  its own permission prompt.
- An approval of a parked request MUST reach the agent only as a
  signed owner instruction.
- Steering input MUST enter the record as `operator` only when the
  surface authenticated the owner. Otherwise it MUST be untrusted and
  MUST NOT be delivered to an agent.
- A message from a co-author MUST NOT reach an agent through steering.
  An owner adoption MUST be a signed event that names the source event.
- Loosening a rule, a hand-off answer, a force push and landing MUST
  require user presence when a presence authenticator is enrolled.
- `parinn answer` MUST refuse to run without an interactive terminal
  and a confirmation code shown only on an owner surface.
- A steer MUST name its target turn. A steer that arrives after its
  turn ended MUST NOT be applied to another turn without the owner's
  confirmation.
- Rule levels MUST be one of: act without asking, act when told, ask
  first, hand off. A repository file MUST only be able to tighten
  them. A recall-tainted session MUST have its sensitive classes
  raised one level.
- Every rule change MUST be an owner-signed event.
- A child agent MUST NOT hold a looser rule level than its parent.
- Before a stop, the view MUST list the completed effects the record
  can show, what is in flight, and what is waiting, and MUST say for
  each whether and how it can be undone.
- The view MUST NOT show a harness as stopped, paused or interrupted
  until the harness's acknowledgement is recorded.
- Concurrent conflicting owner answers MUST resolve to deny, and the
  conflict MUST be recorded and shown.
- A control an adapter cannot honour MUST be shown as unavailable with
  its reason, and MUST NOT be simulated.
- Terminal hosting MUST live outside the core. Input typed at a
  no-echo prompt during a takeover MUST NOT be stored.

## Open decisions for the stakeholder

1. **The hold window and expiry.** Default 3 minutes, then park and
   deny, as proposed? Or hold up to the adapter's limit (10 minutes
   for Claude Code hooks), at the cost of a longer-blocked agent?
2. **The launcher.** Ship `parinn run` as a separate B1 process that
   hosts harness terminals and drives the Agent SDK, so Claude Code
   gets the full steering strip? It brings back `os/exec` outside the
   core, and needs its own review. Without it, plain-hook Claude Code
   cannot be interrupted, stopped or taken over from the UI.
3. **User presence.** Require WebAuthn presence for sensitive steering
   acts (loosening rules, hand-off answers, force pushes, landing)?
   It is the only defence that holds against an unsandboxed agent, at
   the cost of a touch per sensitive act.
4. **Unsandboxed agents.** Should the owner-authenticated controls
   work at all for an agent that runs without a sandbox, or only with
   the "best effort" mark?
5. **Who may adopt.** Adoption by the owner only (OQ-14 option b), or
   also a granted role (option c), which changes PRV-05 and needs an
   I2 review?
6. **Concurrent answers.** Is "deny wins" the right rule for owner
   answers made concurrently on two sides of a partition, or should
   the second answer be held for the owner to confirm?
7. **The bridge.** Offer an opt-in notification bridge at all in v1,
   and if so, which first target: self-hosted ntfy, Matrix or email?
   Or require an enrolled phone peer for anything off the desk?
8. **Cost estimates.** Ship a price table in the release (stale the
   day prices change), or show tokens only unless the owner supplies
   prices?
9. **Automatic nudges.** Allow an owner-configured fixed nudge to a
   stuck agent? It is static owner text, like PIN-07, but it is ParleyInn
   writing to a model without a fresh owner act.
10. **Requirement placement.** Do these land in the proposed VIEW
    family, or in a new family for held requests and steering (owner
    acts as events), given that both touch I2 and need a security
    review?
