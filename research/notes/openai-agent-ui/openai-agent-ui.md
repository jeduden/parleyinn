# OpenAI agent harness UIs: live sessions, results, fleets and connections

Scope: how OpenAI shows agent harnesses in a UI as of 3 October 2026. It
covers the live session, its work results, several agents in one view, and
how harnesses connect to UIs, to each other and to remote machines. Each
item says where state lives, whether OpenAI's service is needed, and what
ParleyInn could reuse. Status labels are **[PREVIEW]**, **[EXPERIMENTAL]**,
**[BETA]**, **[DEPRECATED]** and **[REMOVED]**.

Method and source notes:

- Primary sources were read on 3 October 2026. The Codex docs moved:
  `developers.openai.com/codex/*` now answers 308 and redirects to
  `learn.chatgpt.com/docs/*`. Each page has a Markdown twin at
  `/docs/<slug>.md`, which is what was read — [docs index](https://learn.chatgpt.com/llms.txt).
- `openai/codex` was cloned at `f5a2272` (committed 3 October 2026,
  00:02 UTC). `openai/symphony` was cloned at `be10a1b` (15 September
  2026). `openai/openai-agents-python` was cloned at `81f0ccf`
  (2 October 2026). Code links below point at `main`.
- `openai.com/index/*` returned HTTP 403 to every fetch. Claims from the
  App Server engineering post rest on a third-party mirror of its text
  ([newton20 mirror](https://github.com/newton20/harness-engineering-kb/blob/master/raw/openai-com-index-unlocking-the-codex-harness.md))
  and on search snippets. Claims from the Symphony announcement rest on
  search snippets and press coverage. The DevDay 2026 notes were read on
  learn.chatgpt.com, not on openai.com.
- Product naming changed in 2026. The Codex app merged into the
  **ChatGPT desktop app** on 9 July 2026; "threads" are now called "chats"
  in the product docs but stay `thread/*` in the protocol — [What's new](https://learn.chatgpt.com/docs/whats-new.md).

## Key findings

1. One harness, many clients. Every Codex surface (CLI TUI, IDE
   extension, desktop app, web, mobile, Symphony, third-party IDEs) drives
   the same open-source agent loop through the **app-server** JSON-RPC
   protocol. The UI is a client of a long-lived process, not the process
   itself — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
2. The protocol's model is thread, turn, item. Items carry an explicit
   `item/started` → deltas → `item/completed` lifecycle, and the completed
   item is authoritative. Approvals are server-initiated JSON-RPC requests
   that pause the turn — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
3. The app-server command, its WebSocket transport and the remote modes
   are labelled experimental and "not supported for production
   workloads", even though OpenAI's own clients and Symphony depend on
   them — [Codex MCP server removal](https://learn.chatgpt.com/docs/mcp-server.md).
4. Codex does not speak the Agent Client Protocol (ACP). Zed and other
   ACP clients reach Codex through `codex-acp`, an adapter outside
   OpenAI's repo that wraps the app-server — [agentclientprotocol/codex-acp](https://github.com/agentclientprotocol/codex-acp).
5. Several agents in one view is done three ways: a sidebar of chats with
   live status (plus an **Activity** view since 30 July 2026), a
   subagent panel per chat, and Symphony's tracker board plus an optional
   dashboard. There is no first-party "fleet" console in the Codex app —
   [Notifications](https://learn.chatgpt.com/docs/notifications.md), [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents.md), [Symphony SPEC](https://github.com/openai/symphony/blob/main/SPEC.md).
6. Work results are shown as Git state, not as agent output: a review pane
   over unstaged, staged, commit, branch and last-turn diffs, inline
   review comments, an integrated terminal, project actions and a PR
   panel. The protocol streams `turn/diff/updated` and `turn/plan/updated`
   for this — [Code review](https://learn.chatgpt.com/docs/code-review.md), [Local environments](https://learn.chatgpt.com/docs/environments/local-environment.md).
7. Local-first for local work: local threads are JSONL rollouts plus a
   SQLite index under `~/.codex`, and SSH hosts are reached by starting
   the remote app-server over SSH. Phone and cross-device control goes
   through an OpenAI relay on `chatgpt.com`; the code accepts no other
   relay host except localhost — [Troubleshooting](https://learn.chatgpt.com/docs/reference/troubleshooting.md), [remote_control/protocol.rs](https://github.com/openai/codex/blob/main/codex-rs/app-server-transport/src/transport/remote_control/protocol.rs).
8. Multiplayer is thin. OpenAI ships read-only thread snapshots (20 August
   2026), Slack and Teams threads where only the requester controls the
   task, and ChatGPT Space for shared documents. Nothing documents two
   people steering one live agent session, and nothing works through a
   network partition — [changelog](https://learn.chatgpt.com/docs/changelog), [Use ChatGPT in Slack](https://learn.chatgpt.com/docs/third-party/slack.md).
9. Untrusted-content handling is mostly policy and sandboxing: web search
   defaults to a cached index, a separate "auto-review" agent judges
   boundary crossings, and Symphony keeps tracker credentials out of the
   agent. The newest code (the agent message board) delivers other
   agents' posts as attributed, size-capped previews that never start a
   turn, with the full post pull-only — [Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security.md), [agent_message_board.rs](https://github.com/openai/codex/blob/main/codex-rs/core/src/agent_message_board.rs).
10. The newest direction separates **harness** (agent loop, app-server)
    from **environment** (an `exec-server` that runs commands). The
    environment can be remote, reached directly over an authenticated
    WebSocket or through an OpenAI rendezvous relay carrying
    Noise-encrypted frames — [exec-server README](https://github.com/openai/codex/blob/main/codex-rs/exec-server/README.md).

## Three reusable ideas for ParleyInn's UI

1. **Thread/turn/item with server requests.** Model the live session as
   typed items with a started/delta/completed lifecycle, and model every
   permission prompt as a request with an id that any subscribed client
   may answer, followed by a broadcast `serverRequest/resolved`. This
   gives one view the prompt, tool calls, approvals and output, and lets
   a phone, a terminal and a browser share one session without a central
   service — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
2. **Results are views over durable state, not chat text.** Codex's review
   pane reads Git (unstaged, staged, commit, branch, last turn), and
   Symphony's dashboard reads only orchestrator state and "MUST NOT be
   REQUIRED for correctness". ParleyInn can render tests, diffs and benchmarks
   from its record, so the UI stays an optional client — [Code review](https://learn.chatgpt.com/docs/code-review.md), [Symphony SPEC §13.7](https://github.com/openai/symphony/blob/main/SPEC.md).
3. **A small, shared status vocabulary for the fleet.** Codex surfaces
   converge on four or five states: protocol `idle`, `active` with flags
   `waitingOnApproval` or `waitingOnUserInput`, `systemError`,
   `notLoaded`; Codex Micro shows Idle, Thinking, Complete, Requires
   input and Error. A lane or fleet view needs exactly this per harness,
   plus a parent/child tree for subagents — [ThreadStatus.ts](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/schema/typescript/v2/ThreadStatus.ts), [Codex Micro](https://learn.chatgpt.com/docs/features/codex-micro.md).

## 1. The Codex app (now Codex in the ChatGPT desktop app) and Codex web

**Takeaway.** The desktop app is a multi-chat command centre. A project
sidebar lists chats; each chat runs Local, in a Worktree, or in the
Cloud; a review pane, terminal and PR panel show results. Parallel agents
are separate chats in separate worktrees, plus subagents inside a chat.
The app is closed source and needs a ChatGPT sign-in.

Dates:

- Codex app for macOS launched 2 February 2026 "for running agent threads
  in parallel", with a project sidebar, thread list and review pane, and
  built-in worktrees, Git tooling, skills and automations — [changelog 2026-02-02](https://learn.chatgpt.com/docs/changelog).
- "Sync" was renamed "Handoff" on 3 February 2026; mid-turn steering
  arrived 5 February; parallel approvals 10 February — [changelog](https://learn.chatgpt.com/docs/changelog).
- Codex joined the ChatGPT desktop app on macOS and Windows on 9 July 2026
  (app 26.707), adding PR Chat, inline diff editing and multi-repository
  projects — [changelog 2026-07-09](https://learn.chatgpt.com/docs/changelog), [What's new](https://learn.chatgpt.com/docs/whats-new.md).
- Linux desktop app: week of 10–14 August 2026 — [What's new](https://learn.chatgpt.com/docs/whats-new.md).
  The first Windows date of the standalone Codex app could not be
  verified.

Cited findings:

- Run modes: **Local** works in the project directory, **Worktree**
  isolates changes in a Git worktree, **Cloud** starts a remote task from
  a published environment. Local and Worktree run on your computer —
  [Codex environments](https://learn.chatgpt.com/docs/environments/modes.md).
- Worktrees: "Worktrees let Codex run multiple independent chats in the
  same project without interfering with each other." Codex creates them
  in `$CODEX_HOME/worktrees` on a detached HEAD; Handoff moves a chat
  *and* its code between Local and Worktree; the app keeps the 15 most
  recent managed worktrees by default — [Worktrees](https://learn.chatgpt.com/docs/environments/git-worktrees.md).
- Parallelism guidance: "Each chat keeps its own context, messages,
  results, and goal. Run chats concurrently, but avoid letting two chats
  change the same files" — [Long-running work](https://learn.chatgpt.com/docs/long-running-work.md).
- Goal mode: `/goal` sets a goal that is "both the first prompt and the
  completion criteria"; a progress row above the composer pauses,
  resumes, edits or clears it — [Long-running work](https://learn.chatgpt.com/docs/long-running-work.md).
- Review pane: shows **Unstaged** by default, plus **Staged**, **Commit**,
  **Branch** and **Last turn**. It "reflects the state of your Git
  repository, not just what Codex edited". `/review` findings appear as
  inline comments; reviews run inline or **Detached** in a separate chat.
  Multi-repo projects get a repository selector and an **All repos** view
  for the last turn — [Code review](https://learn.chatgpt.com/docs/code-review.md).
- Test results: no dedicated test-result widget is documented. Tests run
  as agent commands, as project **Actions** in the top bar, or in the
  integrated terminal, whose output "ChatGPT can read" — [Local environments](https://learn.chatgpt.com/docs/environments/local-environment.md), [Integrated terminal](https://learn.chatgpt.com/docs/integrated-terminal.md).
- PR review: the Code Review plugin shows a PR's description, changed
  files, comments and checks; GitHub is generally available, GitLab is
  **[PREVIEW]** — [Code review](https://learn.chatgpt.com/docs/code-review.md).
- Multi-agent views: the **Activity** view (bell in the sidebar) lists
  chats that are unread, running or waiting for a response, filterable
  by Work, Chat, Pinned and Scheduled; a floating "pet" shows Running,
  Needs input, Ready or Blocked — [Notifications](https://learn.chatgpt.com/docs/notifications.md); Activity shipped 30 July 2026 — [What's new](https://learn.chatgpt.com/docs/whats-new.md).
- Codex Micro (15 July 2026, limited-run hardware with Work Louder): six
  Agent Keys follow six chats and light up by status — [Codex Micro](https://learn.chatgpt.com/docs/features/codex-micro.md).
- iOS added a **Priority** view of running tasks, unread updates and tasks
  awaiting a response (1.2026.237, 1 September 2026) — [changelog](https://learn.chatgpt.com/docs/changelog).
- Codex web: "Codex Web uses the Codex harness, but runs it in a
  container environment"; a worker launches the app-server inside the
  container and the browser talks to the Codex backend "over HTTP and
  SSE". "The web app cannot be the source of truth for long-running
  tasks" — App Server post, 4 February 2026, via [mirror](https://github.com/newton20/harness-engineering-kb/blob/master/raw/openai-com-index-unlocking-the-codex-harness.md).
- The desktop app and VS Code extension bundle a pinned app-server binary
  and talk to it as a child process over stdio — same post, via [mirror](https://github.com/newton20/harness-engineering-kb/blob/master/raw/openai-com-index-unlocking-the-codex-harness.md).
- Open source: the CLI, SDK and app-server are open; the IDE extension
  and Codex cloud are not. The desktop app is not listed as open source —
  [Open Source](https://learn.chatgpt.com/docs/open-source.md).

**Where state lives.** Local and Worktree chats: on the machine, as
rollout JSONL in `~/.codex/sessions` with archives in
`~/.codex/archived_sessions` ([Troubleshooting](https://learn.chatgpt.com/docs/reference/troubleshooting.md)), indexed in
SQLite ([thread-store README](https://github.com/openai/codex/blob/main/codex-rs/thread-store/README.md)). Pinned chats sync between
desktop and iOS since 20 August 2026, so some metadata reaches OpenAI
([changelog](https://learn.chatgpt.com/docs/changelog)). Cloud chats: OpenAI cloud. Model inference
goes to OpenAI unless a custom provider is configured.

**Needs OpenAI.** Yes for the app (ChatGPT sign-in, see
[Get started](https://learn.chatgpt.com/docs/app.md)). Local execution itself does not.

**For ParleyInn.** The run-location switch (Local, Worktree, remote host)
sitting in the chat footer is a good fit for "lanes". The review pane's
scopes map onto ParleyInn work results. The Activity view and the
four-colour status lights show that a fleet view can stay small.

## 2. The Codex app-server protocol

**Takeaway.** A bidirectional "JSON-RPC lite" protocol (no `"jsonrpc"`
field) over stdio, a Unix socket or WebSocket. Primitives are thread,
turn and item. The server streams notifications and sends requests for
approvals, user input and MCP elicitations. Several connections can
subscribe to one thread. It is open source, versioned by generated
schemas, and labelled experimental.

Cited findings:

- Purpose: "the interface Codex uses to power rich clients (for example,
  the Codex VS Code extension)" — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- History: OpenAI "first experimented with exposing Codex as an MCP
  server", found MCP semantics hard to fit to VS Code, and introduced a
  JSON-RPC protocol that "mirrored the TUI loop". Clients exist in Go,
  Python, TypeScript, Swift and Kotlin; the surface "is designed to be
  backward compatible" — App Server post via [mirror](https://github.com/newton20/harness-engineering-kb/blob/master/raw/openai-com-index-unlocking-the-codex-harness.md).
- Transports: `stdio://` (default, JSONL); `ws://IP:PORT`
  **[EXPERIMENTAL]**, "one JSON-RPC message per WebSocket text frame";
  `unix://` or `unix://PATH` (WebSocket over a Unix socket); `off`.
  Non-loopback WebSocket listeners "currently allow unauthenticated
  connections by default during rollout"; auth flags offer capability
  tokens or signed bearer tokens — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Remote TUI: `codex app-server --listen ws://127.0.0.1:4500` then
  `codex --remote ws://127.0.0.1:4500`; plain WebSockets only for
  localhost or an SSH port-forward — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Handshake: one `initialize` per connection, then `initialized`.
  Capabilities include `experimentalApi` and `optOutNotificationMethods`
  (exact method names to suppress, e.g. `item/agentMessage/delta`) —
  [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Streaming a session: `thread/start` "automatically subscribes you to
  turn/item events"; `turn/start` begins work; notifications include
  `turn/started`, `item/started`, `item/agentMessage/delta`,
  `item/commandExecution/outputDelta`, `item/reasoning/summaryTextDelta`,
  `item/completed` ("treat this as the authoritative state"),
  `turn/diff/updated` (aggregated unified diff), `turn/plan/updated`,
  `thread/tokenUsage/updated` and `turn/completed` — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Item types include `userMessage`, `agentMessage`, `plan`, `reasoning`,
  `commandExecution`, `fileChange`, `mcpToolCall`, `dynamicToolCall`,
  `webSearch`, `enteredReviewMode`, `exitedReviewMode` and
  `contextCompaction` — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Steering and stopping: `turn/steer` appends input to the in-flight turn
  and requires `expectedTurnId`; `turn/interrupt` ends the turn as
  `interrupted` — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Approvals: "The app-server sends a server-initiated JSON-RPC request to
  the client, and the client responds with a decision payload." Command
  decisions are `accept`, `acceptForSession`, `decline`, `cancel` or an
  exec-policy amendment; file-change decisions are `accept`,
  `acceptForSession`, `decline`, `cancel`. Order: `item/started`, then
  `item/commandExecution/requestApproval` (or
  `item/fileChange/requestApproval`), then the response, then
  `serverRequest/resolved`, then `item/completed` — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Permission requests: `item/permissions/requestApproval` asks for
  network or filesystem permissions; the client returns "only the granted
  subset", scoped to the `turn` or the `session`. MCP servers interrupt
  with `mcpServer/elicitation/request` (form or URL) — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Network approvals are grouped by host, protocol and port, so one prompt
  can unblock several queued requests — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Resuming: `thread/resume` reopens a stored thread; `thread/fork`
  branches it (optionally at `lastTurnId`, or `ephemeral`);
  `thread/read`, `thread/turns/list` and `thread/items/list` read history
  without resuming; `thread.sessionId` names the root of a live session
  tree — [App Server docs](https://learn.chatgpt.com/docs/app-server.md). `thread/items/list` accepts an item anchor
  as a cursor — [app-server README](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md).
- Multi-client: `thread/unsubscribe` removes one connection; "If this was
  the last subscriber", the server unloads the thread after 30 minutes
  without subscribers or activity and emits `thread/closed`;
  `thread/status/changed` reports `idle`, `active` (with
  `waitingOnApproval`) and so on — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Review: `review/start` with targets `uncommittedChanges`, `baseBranch`,
  `commit` or `custom`, delivered inline or detached into a new thread —
  [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Thread attachments: `thread/attachment/add|list|remove` attach durable
  resource references (e.g. a pull request) to a stored thread without
  loading it, with a reverse lookup `thread/attachmentOwner/list` —
  [app-server README](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md).
- Schemas: `codex app-server generate-ts` and `generate-json-schema` emit
  schemas specific to the running version — [App Server docs](https://learn.chatgpt.com/docs/app-server.md). The repo
  checks in 645 generated TypeScript v2 types
  ([schema/typescript/v2](https://github.com/openai/codex/tree/main/codex-rs/app-server-protocol/schema/typescript/v2)).
- Status: "The app-server command is experimental and isn't supported for
  production workloads" — [Codex MCP server removal](https://learn.chatgpt.com/docs/mcp-server.md). The SDKs are
  the recommended route for CI and automation — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- **[REMOVED]** `codex mcp-server`: deprecated 24 August 2026, removed
  5 September 2026; integrations must move to the app-server — [changelog](https://learn.chatgpt.com/docs/changelog).
- SDKs: the TypeScript SDK "spawns the CLI and exchanges JSONL events
  over stdin/stdout" with `run()` and `runStreamed()` — [sdk/typescript README](https://github.com/openai/codex/blob/main/sdk/typescript/README.md).

ACP comparison:

- ACP is JSON-RPC 2.0. Agent methods are `initialize`, `authenticate`,
  `session/new`, `session/prompt`, optional `session/load`,
  `session/set_mode` and `logout`, and the `session/cancel`
  notification. Client methods are `session/request_permission`, optional
  `fs/*`, `terminal/*` and `elicitation/create`. Progress arrives as
  `session/update` notifications — [ACP overview](https://agentclientprotocol.com/protocol/overview).
- Mapping: ACP session ≈ app-server thread; `session/prompt` ≈
  `turn/start`; `session/update` ≈ the `item/*` and `turn/*`
  notifications; `session/request_permission` ≈ the three
  `requestApproval` server requests; `session/load` ≈ `thread/resume`.
  App-server adds forks, steering, review, goals, attachments, thread
  listing and multi-subscriber fan-out, which ACP's base methods lack
  (inference from the two method lists above).
- Codex support: none in `openai/codex`; a repo-wide search for
  `agent-client-protocol` and `agentclientprotocol` found nothing at
  `f5a2272`. Zed's original adapter
  ([zed-industries/codex-acp](https://github.com/zed-industries/codex-acp)) says development moved to
  `agentclientprotocol/codex-acp`, "built on the new Codex App Server". That
  adapter "starts the Codex App Server, translates ACP requests into Codex
  operations, and maps Codex events back", including "native ACP subagent
  sessions" with "root-routed permissions" — [agentclientprotocol/codex-acp](https://github.com/agentclientprotocol/codex-acp).
- Zed lists Codex as an External Agent installed from the ACP Registry —
  [Zed external agents](https://zed.dev/docs/ai/external-agents).

**Where state lives.** In the app-server's `CODEX_HOME`: JSONL rollouts
plus SQLite metadata. The `ThreadStore` trait allows other stores
"outside this repository" — [thread-store README](https://github.com/openai/codex/blob/main/codex-rs/thread-store/README.md).

**Needs OpenAI.** Not for the protocol. A model provider is needed;
OpenAI is the default.

**For ParleyInn.** Copy the shape, not the dependency. Typed items with an
authoritative completed state, server requests for approvals, a
`resolved` broadcast so other clients clear their prompt, notification
opt-out for thin clients, and generated schemas per version. Note the
cost of "experimental" labels on a protocol that partners already embed.

## 3. Codex cloud tasks and local–cloud handoff

**Takeaway.** Cloud tasks run in OpenAI VMs from a published, reusable
environment. Results are a chat with a diff and test output, followed by
a commit, a PR, or a patch pulled down. Local to cloud is a one-way
delegation that copies context; cloud to local is a patch or a PR. The
desktop's own Handoff does not reach the cloud.

Cited findings:

- Reusable cloud environments launched at DevDay, 29 September 2026:
  "Describe your development setup, let Codex prepare and test it, then
  publish the environment … Continue cloud tasks from web, mobile, or
  desktop while your computer sleeps" — [DevDay 2026](https://learn.chatgpt.com/docs/whats-new/devday-2026.md).
- "Each new task gets its own isolated workspace from the published
  environment." Existing tasks keep their own files, "including
  uncommitted changes and installed tools". "Saved state doesn't replace
  source control" — [Cloud environments](https://learn.chatgpt.com/docs/environments/cloud-environments.md).
- Results: "Review changes and test results before committing or opening
  a pull request"; a task's VM state is recoverable for up to seven days
  — [Cloud environments](https://learn.chatgpt.com/docs/environments/cloud-environments.md).
- **[DEPRECATED soon]** Codex Cloud (Legacy) still backs Code Review and
  the Linear and GitHub integrations; "We plan to deprecate this
  experience." Legacy flow: container, setup script, agent loop, then
  "it shows its answer and a diff of any files it changed" — [Cloud environments](https://learn.chatgpt.com/docs/environments/cloud-environments.md), [Codex Cloud (Legacy)](https://learn.chatgpt.com/docs/environments/cloud-environment.md).
- Local to cloud from the IDE: pick a cloud environment under the
  composer, and "Codex creates a new chat in the cloud that carries over
  the existing chat context (including the plan and any local source
  changes)" — [Codex manual, "Delegate refactor to the cloud"](https://learn.chatgpt.com/docs/llms-full.txt).
- Cloud to local: "Create a PR directly from the cloud or pull changes
  locally"; `codex apply` applies "the most recent diff from a Codex
  cloud chat to your local repository"; `codex cloud list --json` returns
  task status for scripts — [Codex manual](https://learn.chatgpt.com/docs/llms-full.txt).
- Desktop Handoff moves chats between Local, Worktree and SSH hosts, but
  "handoff to a Codex cloud environment isn't supported" — [Remote connections](https://learn.chatgpt.com/docs/remote-connections.md).
- Web harness: the browser talks HTTP and SSE to the backend, which
  streams events from the app-server in the container, so "work continues
  even if the tab disappears" — App Server post via [mirror](https://github.com/newton20/harness-engineering-kb/blob/master/raw/openai-com-index-unlocking-the-codex-harness.md).
- Not yet supported in cloud environments: computer and browser use,
  GitLab and self-hosted GitHub Enterprise Server — [Cloud environments](https://learn.chatgpt.com/docs/environments/cloud-environments.md).
  (GitLab in Codex cloud is in beta via the legacy path since 19 August
  2026 — [changelog](https://learn.chatgpt.com/docs/changelog).)

**Where state lives.** OpenAI cloud: environment, task VM state, chat
history.

**Needs OpenAI.** Yes, entirely. Codex cloud is not open source.

**For ParleyInn.** The "published environment, isolated per-task workspace"
split is a clean model for remote lanes. Handing off by Git state
(worktree plus patch) rather than by moving a process is what lets
Codex move chats between hosts. ParleyInn would do it from its own record.

## 4. Remote, SSH and devbox modes

**Takeaway.** Three mechanisms. SSH hosts: the desktop app starts the
remote app-server over SSH. Codex Remote: a phone or another desktop
controls a host through an OpenAI relay. Remote environments: an
`exec-server` runs commands for a harness elsewhere, directly or through
a relay. Only the SSH path and the direct exec-server path avoid
OpenAI's service.

Cited findings:

- SSH: "The app starts the remote Codex app server through SSH, using the
  remote user's login shell." Hosts come from `~/.ssh/config`. "Don't
  expose app-server transports directly on a shared or public network";
  use a VPN or mesh instead — [Remote connections](https://learn.chatgpt.com/docs/remote-connections.md).
- Chat handoff between hosts (week of 15–19 June 2026): "Codex creates or
  reuses a worktree on the destination host, transfers the chat and Git
  state, and switches the chat to that host. If the chat is running,
  handoff interrupts the current response" — [Remote connections](https://learn.chatgpt.com/docs/remote-connections.md), [What's new](https://learn.chatgpt.com/docs/whats-new.md).
- Codex Remote reached general availability on 25 June 2026 with
  "authenticated one-to-one QR pairing between each iOS or Android device
  and each host"; Windows hosts since 29 May 2026 — [changelog](https://learn.chatgpt.com/docs/changelog).
- What a phone can do: start chats, steer, approve commands, and "Review
  outputs, diffs, test results, terminal output, and screenshots". "A
  secure relay layer keeps trusted machines reachable across your
  authorized ChatGPT devices without exposing them directly to the public
  internet." Both devices must use the same ChatGPT account and workspace
  — [Remote connections](https://learn.chatgpt.com/docs/remote-connections.md).
- Relay host in code: the remote-control URL must be HTTPS on
  `chatgpt.com` or `chatgpt-staging.com` (or subdomains), or HTTP/HTTPS on
  localhost; the server enrolls at `wham/remote/control/server/enroll` and
  opens a WebSocket at `wham/remote/control/server`. Envelopes carry
  `client_id`, `stream_id` and `seq_id` — [remote_control/protocol.rs](https://github.com/openai/codex/blob/main/codex-rs/app-server-transport/src/transport/remote_control/protocol.rs).
- Protocol methods: `remoteControl/enable|disable`,
  `remoteControl/pairing/start|status`, `remoteControl/client/list|revoke`
  and `remoteControl/status/changed` — [common.rs](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/common.rs).
- **[EXPERIMENTAL]** App-server daemon: `codex app-server daemon
  bootstrap --remote-control` prepares "Codex instances launched over
  SSH, including fresh developer machines"; state lives in
  `CODEX_HOME/app-server-daemon/` — [app-server-daemon README](https://github.com/openai/codex/blob/main/codex-rs/app-server-daemon/README.md).
  `codex remote-control pair` prints a short-lived pairing code —
  [Codex manual](https://learn.chatgpt.com/docs/llms-full.txt).
- Devbox provisioning: the DigitalOcean plugin can create a Droplet,
  configure SSH and connect it as a remote workspace (25 June 2026) —
  [changelog](https://learn.chatgpt.com/docs/changelog).
- Remote environments: `codex exec-server` listens on `ws://IP:PORT`
  with bearer-token auth, or registers with an environment registry and
  reconnects to "the service-provided rendezvous websocket". Relay frames
  carry stream ids, acks, resume and heartbeat, with "encrypted data"
  payloads under the Noise contract — [exec-server README](https://github.com/openai/codex/blob/main/codex-rs/exec-server/README.md).
  Clients add one with `environment/add`; `thread/environment/connected`
  and `disconnected` report it — [common.rs](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/common.rs).
- Work across devices (DevDay, 29 September 2026): for ChatGPT Work, "OpenAI's
  cloud coordinates the task, and an online, connected computer can
  execute steps that need its local resources". If the computer is
  offline when a turn starts, the task can continue in a cloud container
  without local files — [Remote connections](https://learn.chatgpt.com/docs/remote-connections.md), [Local computer access](https://learn.chatgpt.com/docs/enterprise/cloud-local-access.md).

**Where state lives.** SSH: on the remote host. Codex Remote: on the
host; the relay routes messages, and pinned chats and pairings sit with
OpenAI. Work across devices: OpenAI cloud orchestrator.

**Needs OpenAI.** SSH and direct exec-server: no. Codex Remote, pairing
and the Noise rendezvous registry: yes.

**For ParleyInn.** The SSH pattern (start the harness's server over SSH,
speak the same protocol) is self-hostable and fits requirement 3.
Codex Remote shows the cost of a vendor relay: one account, one
workspace, relay on `chatgpt.com`. ParleyInn would need a peer-to-peer or
self-hosted relay. Under I4 that relay cannot live in the shipped
binary.

## 5. Symphony's orchestration board

**Takeaway.** Symphony makes the issue tracker the fleet UI. Each active
issue gets a workspace and a Codex app-server session; agents run until
the issue reaches a handoff state such as Human Review. The tracker holds
the durable state. An optional loopback dashboard and JSON API show live
runs. It is an engineering preview for trusted environments.

Cited findings:

- Announced 27 April 2026 on openai.com ("An open-source spec for Codex
  orchestration: Symphony"), per search snippets of the post; press
  coverage dates it 28 April — [Help Net Security](https://www.helpnetsecurity.com/2026/04/28/openai-symphony-codex-orchestration-linear/). OpenAI reported a "500% increase in
  landed pull requests" on some teams (search snippet of the
  [openai.com post](https://openai.com/index/open-source-codex-orchestration-symphony/), not fetched).
- **[PREVIEW]** "Symphony is a low-key engineering preview for testing in
  trusted environments." The demo shows agents providing "proof of work:
  CI status, PR review feedback, complexity analysis, and walkthrough
  videos" — [Symphony README](https://github.com/openai/symphony/blob/main/README.md).
- The spec, not a product: "Implement Symphony according to the following
  spec"; the Elixir reference is "prototype software intended for
  evaluation only" — [Symphony README](https://github.com/openai/symphony/blob/main/README.md), [Elixir README](https://github.com/openai/symphony/blob/main/elixir/README.md).
- Trackers: Linear, GitHub Issues, Jira Cloud, Asana and GitLab adapters
  in the reference implementation — [Elixir README](https://github.com/openai/symphony/blob/main/elixir/README.md).
- State: "Support tracker/filesystem-driven restart recovery without
  requiring a persistent database; exact in-memory scheduler state is not
  restored." Non-goal: "Rich web UI or multi-tenant control plane" —
  [Symphony SPEC §2](https://github.com/openai/symphony/blob/main/SPEC.md).
- Live session record per run: `thread_id`, `turn_id`, last event,
  timestamp, summarised message, token counts and turn count —
  [Symphony SPEC §4.1.6](https://github.com/openai/symphony/blob/main/SPEC.md).
- Agent protocol: the Codex app-server is the source of truth for message
  shapes; approvals and user-input requests "MUST NOT leave a run stalled
  indefinitely" — [Symphony SPEC §10](https://github.com/openai/symphony/blob/main/SPEC.md). The reference keeps such
  issues claimed and shows them as blocked; blocked entries are in memory
  only — [Elixir README](https://github.com/openai/symphony/blob/main/elixir/README.md).
- Dashboard: optional HTTP server at `/` plus `GET /api/v1/state`,
  `GET /api/v1/<issue_identifier>` and `POST /api/v1/refresh`. It
  "SHOULD bind loopback by default" and "MUST NOT become REQUIRED for
  orchestrator correctness" — [Symphony SPEC §13.7](https://github.com/openai/symphony/blob/main/SPEC.md). The reference
  uses Phoenix LiveView — [Elixir README](https://github.com/openai/symphony/blob/main/elixir/README.md).
- Remote workers: an SSH extension launches the app-server "over SSH
  stdio instead of as a local subprocess"; "Once a run has already
  produced side effects, a transparent rerun on another host SHOULD be
  treated as a new attempt" — [Symphony SPEC Appendix A](https://github.com/openai/symphony/blob/main/SPEC.md).

**Where state lives.** Tracker (vendor cloud for Linear and the others),
per-issue workspaces on disk, orchestrator memory.

**Needs OpenAI.** Only for model inference through Codex. The tracker is
a third-party service.

**For ParleyInn.** The strongest pattern here: the status surface draws only
from orchestrator state and is never required. "Proof of work" as the
result of a run matches ParleyInn's tests, diffs and benchmarks. Symphony's
single orchestrator is a central point ParleyInn would replace with lanes.

## 6. Subagents and multi-agent features

**Takeaway.** Subagents are generally on in local Codex. Each subagent is
its own thread, linked to its parent; clients show them as an activity
panel and let you open each thread. A message board between agents is
under development. The API offers a separate Multi-agent beta.

Cited findings:

- "Current Codex releases enable subagent workflows by default. Subagent
  activity appears in the ChatGPT desktop app, Codex CLI, and the IDE
  extension" — [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents.md).
- UI per surface: the app "surfaces each subagent thread so you can
  inspect its work and the summary returned to the main chat"; the CLI
  uses `/agent` to switch threads; the IDE shows active subagents "above
  the composer" with stop-all; the web sidebar has read-only **Active**
  and **Done** lists with no per-subagent controls — [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents.md).
- Approvals from children: in the CLI, "approval requests can surface from
  inactive agent threads … The approval overlay shows the source thread
  label, and you can press `o` to open that thread" — [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents.md).
- Custom agents are TOML files in `~/.codex/agents/` or `.codex/agents/`
  with `name`, `description` and `developer_instructions`; built-ins are
  `default`, `worker` and `explorer` — [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents.md).
- Protocol: a `collabAgentToolCall` item records `spawnAgent`,
  `sendInput`, `resumeAgent`, `wait`, `closeAgent`, `sendMessage`,
  `followupTask`, `interruptAgent` and `listAgents`, with sender and
  receiver thread ids and agent states; a `subAgentActivity` item carries
  `agentThreadId` and `agentPath` — [item.rs](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/v2/item.rs).
  `thread/list` filters by `parentThreadId` or `ancestorThreadId`
  **[EXPERIMENTAL]** — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Feature flags: `multi_agent` stable and on by default;
  `multi_agent_v2` stable and off by default; `agent_message_board`
  under development — [features/src/lib.rs](https://github.com/openai/codex/blob/main/codex-rs/features/src/lib.rs).
- **[EXPERIMENTAL / under development]** Agent message board: "Shared agent
  discussions with interchangeable local, in-memory and remote backends"
  with channels, posts, threads and subscriptions; a remote board is a
  configured URL with a bearer token — [ext/agent-message-board](https://github.com/openai/codex/blob/main/codex-rs/ext/agent-message-board/src/lib.rs), [feature_configs.rs](https://github.com/openai/codex/blob/main/codex-rs/features/src/feature_configs.rs).
- **[BETA]** Responses API Multi-agent: "lets a model spin up and
  coordinate subagents in parallel"; enabled with `multi_agent.enabled`
  and the `responses_multi_agent=v1` beta header; "Item schemas may change
  while Multi-agent is in beta" — [Multi-agent guide](https://developers.openai.com/api/docs/guides/responses-multi-agent.md).
  Launch date not verified; DevDay notes GPT-6.1 Sol supports it — [DevDay 2026](https://learn.chatgpt.com/docs/whats-new/devday-2026.md).

**Where state lives.** Local subagents: child threads in the same
`CODEX_HOME`, with a parent/child graph store
([agent-graph-store](https://github.com/openai/codex/blob/main/codex-rs/agent-graph-store/src/lib.rs)). ChatGPT Work and API multi-agent: OpenAI
cloud.

**Needs OpenAI.** Local subagents: model only. API Multi-agent: yes.

**For ParleyInn.** Children are first-class threads with a parent link, and
approvals bubble up labelled with their source thread. That is the lane
view in miniature. ParleyInn can render a lane as a tree using the same
parent/ancestor queries.

## 7. Agents SDK sessions, tracing and the traces dashboard

**Takeaway.** The Agents SDK keeps conversation memory in pluggable
client-side sessions and exports traces to OpenAI's Traces dashboard by
default. Traces are a span tree, not a live interactive session; there
are no approvals in the dashboard.

Cited findings:

- Sessions maintain history "across multiple agent runs"; built-ins
  include SQLite, Redis, SQLAlchemy, Dapr, MongoDB, encrypted wrappers
  and OpenAI Conversations API sessions — [sessions docs](https://github.com/openai/openai-agents-python/blob/main/docs/sessions/index.md).
- Trust boundary: `SQLiteSession` "does not authenticate stored rows or
  detect external edits, deletions, reordering, or replay"; the encrypted
  wrapper "does not verify the completeness or order of conversation
  history" — [sessions docs](https://github.com/openai/openai-agents-python/blob/main/docs/sessions/index.md).
- Human-in-the-loop: paused runs serialize to `RunState`; "Treat tool
  names and arguments as untrusted display content and escape them when
  rendering HTML"; a snapshot sent through a client must be integrity
  checked and bound to the user and run — [HITL docs](https://github.com/openai/openai-agents-python/blob/main/docs/human_in_the_loop.md).
- Tracing: "enabled by default", recording generations, tool calls,
  handoffs, guardrails and custom events as traces of spans, viewable in
  the [Traces dashboard](https://platform.openai.com/traces); "unavailable
  for organizations that use OpenAI's APIs under a Zero Data Retention
  (ZDR) policy" — [tracing docs](https://github.com/openai/openai-agents-python/blob/main/docs/tracing.md).
- `set_trace_processors()` replaces the OpenAI exporter, so traces "will
  not be sent to the OpenAI backend" — [tracing docs](https://github.com/openai/openai-agents-python/blob/main/docs/tracing.md).
- **[DEPRECATED]** Agent Builder shuts down on 30 November 2026; ChatKit
  remains — [Safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety.md).
- The dashboard itself could not be inspected (needs a platform login).

**Where state lives.** Sessions: wherever the app puts them. Traces:
OpenAI cloud by default, or the app's own exporter.

**Needs OpenAI.** The dashboard does; the SDK does not.

**For ParleyInn.** The span tree (run, turn, agent, generation, tool,
handoff) is a good shape for an after-the-fact timeline. The SDK's own
warning about unauthenticated session rows is the problem ParleyInn's
append-only record already targets.

## 8. Collaboration, sharing and multiplayer

**Takeaway.** OpenAI's sharing is either read-only (snapshots) or
document-centric (Space). Team features route work through accounts and
service accounts, with the requester in control. No shared live agent
session is documented.

Cited findings:

- Shared thread snapshots (20 August 2026): "share a read-only snapshot
  of a local Codex thread" from the macOS desktop app; it "doesn't update
  when the original thread changes"; personal links are open to anyone
  with the link, workspace links to members; "Codex redacts known secret
  patterns, but review the shared content" — [changelog 2026-08-20](https://learn.chatgpt.com/docs/changelog).
- Slack and Teams: "When the **Follow along** and **Cancel** controls are
  available, they appear for the requester … the channel reply doesn't
  give every channel member access to the underlying task." "A request
  using another account may start a separate coding task" — [Use ChatGPT in Slack](https://learn.chatgpt.com/docs/third-party/slack.md).
- Cloud tasks from Slack or Teams: "another participant's request doesn't
  automatically continue your task" — [Cloud environments](https://learn.chatgpt.com/docs/environments/cloud-environments.md).
- ChatGPT Space (DevDay, 29 September 2026): pages and files with view,
  comment and edit roles; `@ChatGPT` or `@dot` in a page asks an agent to
  act — [Collaborate in Space](https://learn.chatgpt.com/docs/space/collaboration.md), [Work with agents in Space](https://learn.chatgpt.com/docs/space/agents.md).
- Team Tasks: scheduled or event-triggered work through a team's service
  account; "The team's Activity view keeps track of changes" —
  [DevDay 2026](https://learn.chatgpt.com/docs/whats-new/devday-2026.md), [Codex manual](https://learn.chatgpt.com/docs/llms-full.txt).
- Cloud environment caches are shared by all users of an environment in
  Business and Enterprise workspaces — [Codex Cloud (Legacy)](https://learn.chatgpt.com/docs/environments/cloud-environment.md).
- Partitions: no document describes offline or partitioned multiplayer.
  Remote control requires both devices online and on the same account;
  "Keep the host awake and connected" — [Remote connections](https://learn.chatgpt.com/docs/remote-connections.md).

**Where state lives.** OpenAI cloud for snapshots, Space, Slack tasks and
Team Tasks.

**Needs OpenAI.** Yes for every sharing feature.

**For ParleyInn.** Two takeaways. Requester-only controls in a shared channel
are close to ParleyInn's "only the lane owner instructs the agent". Read-only
snapshots that do not update are the floor; ParleyInn's lanes would need live
replication across partitions, which OpenAI does not attempt.

## 9. Self-hosting, offline and data residency

**Takeaway.** The harness, protocol, CLI and SDKs are open source and run
locally, with local or third-party models. The desktop app, Codex cloud,
remote control and every sharing feature need OpenAI. Residency options
are narrow and mostly US.

Cited findings:

- `--oss` uses "a local open source model provider", LM Studio or Ollama;
  custom providers set base URL, wire API and auth — [Codex manual](https://learn.chatgpt.com/docs/llms-full.txt).
- Amazon Bedrock is a built-in provider for local Work and Codex surfaces
  — [What's new](https://learn.chatgpt.com/docs/whats-new.md).
- "Local execution does not mean offline or device-only model inference"
  — [Codex manual](https://learn.chatgpt.com/docs/llms-full.txt). "Offline installation doesn't provide offline access to
  ChatGPT" — [Codex manual, Windows deployment](https://learn.chatgpt.com/docs/llms-full.txt).
- Managed `enforce_residency` "Currently accepts `us`"; with ChatGPT
  sign-in, "Codex respects workspace residency settings" — [Codex manual, config reference](https://learn.chatgpt.com/docs/llms-full.txt).
- Local computer access with Work Cloud "does not provide strict zero
  data retention"; if any cloud policy enables `enforce_residency`, local
  access is disabled — [Local computer access](https://learn.chatgpt.com/docs/enterprise/cloud-local-access.md).
- Cloud orchestration events "do not reach your existing OpenTelemetry
  collector"; use the Compliance API instead — [Local computer access](https://learn.chatgpt.com/docs/enterprise/cloud-local-access.md).
- The universal cloud image is public as a Dockerfile and pullable image
  ([openai/codex-universal](https://github.com/openai/codex-universal)), but the cloud service is not open source —
  [Open Source](https://learn.chatgpt.com/docs/open-source.md).
- `clientInfo.name` should be registered with OpenAI for the Compliance
  Logs Platform if a new client targets enterprises — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).

**For ParleyInn.** A fully self-hosted Codex stack exists only at the
CLI/app-server level, with SSH as the remote path. That is the layer
ParleyInn's design resembles.

## 10. How untrusted content is handled

**Takeaway.** OpenAI's main defences are the sandbox, approvals and a
reviewer agent. Guidance tells builders to keep untrusted text out of
developer messages and to constrain it with structured outputs. Codex's
newest code adds attribution, size caps and "never starts a turn" for
messages from other agents.

Cited findings:

- Web search defaults to a cached, OpenAI-maintained index, which
  "reduces exposure to prompt injection from arbitrary live content.
  Treat web results as untrusted." "Prompt injection can cause the agent
  to fetch and follow untrusted instructions" — [Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security.md).
- **[REMOVED]** `approval_policy = "untrusted"` is retired; the stricter
  rule moves to a per-project `trust_level = "untrusted"` — [Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security.md).
- Protected paths: `.git`, `.agents` and `.codex` stay read-only inside
  writable roots — [Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security.md).
- Auto-review: "a separate reviewer agent" decides boundary crossings; it
  "is a reviewer swap, not a permission grant", and it targets
  exfiltration, credential probing, security weakening and destructive
  actions. Hidden reasoning is not shown to the reviewer — [Auto-review](https://learn.chatgpt.com/docs/sandboxing/auto-review.md).
- Reviewer context in code: "Assistant messages are untrusted context,
  not authorization or verified questions" — [guardian_sender_messages.rs](https://github.com/openai/codex/blob/main/codex-rs/core/src/context/guardian_sender_messages.rs).
- Agent message board: the adapter "never starts or restores recipients
  and never queues a notification for an idle agent"; notices are built
  with `trigger_turn` false; "Remote metadata is untrusted"; notices over
  1024 bytes are refused and "the post remains available through the
  board tools" — [agent_message_board.rs](https://github.com/openai/codex/blob/main/codex-rs/core/src/agent_message_board.rs). A preview "must never
  start a new turn" — [message board protocol.rs](https://github.com/openai/codex/blob/main/codex-rs/agent-message-board-client/src/protocol.rs). Notices are
  rendered with role `assistant` and a `Sender:` field, not as user input
  — [agent_message_board notification](https://github.com/openai/codex/blob/main/codex-rs/core/src/context/agent_message_board_notification.rs).
- Builder guidance: "Don't use untrusted variables in developer
  messages"; "Use structured outputs to constrain data flow"; "Design
  workflows so untrusted data never directly drives agent behavior" —
  [Safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety.md).
- Symphony: tracker tools run host-side so the agent never sees tracker
  tokens; implementations "SHOULD NOT assume that tracker data,
  repository contents, prompt inputs, or tool arguments are fully
  trustworthy" — [Symphony SPEC §10.5, §15.5](https://github.com/openai/symphony/blob/main/SPEC.md).
- Other users' input: Slack and cloud tasks keep control with the
  requester's account (section 8). No document describes enveloping
  another human's message as data for the agent.
- `thread/inject_items` lets any connected client append raw Responses
  items, including `assistant` messages, to model-visible history without
  a turn — [App Server docs](https://learn.chatgpt.com/docs/app-server.md). Any client on the socket is fully trusted.

**For ParleyInn.** The message-board rules are close to I2: attribute the
sender, cap what is pushed, never auto-start a turn, keep the rest
pull-only. `thread/inject_items` is the counter-example: a protocol that
gives every client write access to model context cannot meet
requirement 5 without a trust tier per client.

## 11. Fit against ParleyInn's five UI requirements

- **R1, one view of live harness, results and lane or fleet.** Codex
  splits this across a chat view, the review pane, the Activity view and
  the subagent panel. Symphony splits it between tracker and dashboard.
  No single first-party view combines all three.
- **R2, harnesses connect without a UI.** Partly. App-server clients can
  be other programs (Symphony, SDKs, ACP adapters). Agent-to-agent links
  are subagents inside one app-server, or the message board **[under
  development]**.
- **R3, no central service.** Met for CLI, app-server, SSH and direct
  exec-server. Not met for the desktop app, Remote, cloud, sharing,
  pairing or traces.
- **R4, multiplayer through partitions.** Not addressed anywhere.
- **R5, non-owner messages are untrusted data.** Partly: requester-only
  controls in Slack, `trigger_turn` false for board posts. Any app-server
  client can still inject model-visible items.

## 12. What OpenAI requires a central service for

- Model inference by default (avoidable with `--oss`, custom providers or
  Bedrock).
- Identity: ChatGPT sign-in for the desktop app, mobile and cloud;
  service accounts for headless enterprise use.
- Codex Remote and cross-device control: relay and enrollment on
  `chatgpt.com`, one-to-one QR pairing, same account and workspace.
- Remote environments over the Noise rendezvous registry (direct
  WebSocket mode avoids it).
- Codex cloud: environments, task VMs, saved state, Slack, Teams,
  GitHub, GitLab and Linear triggers.
- Work across devices and dots: the OpenAI cloud orchestrator.
- Sharing: thread snapshots, Space, Team Tasks, pinned-chat sync.
- Observability: the Traces dashboard, the Compliance API and analytics.

## 13. Conflicts between sources

- `thread/rollback`: the docs call it deprecated; the repo README says
  it "has been removed from the API" and points to `thread/revert` —
  [App Server docs](https://learn.chatgpt.com/docs/app-server.md), [app-server README](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md).
- Collab item name: the docs list `collabToolCall` with
  `receiverThreadId`, `newThreadId` and `agentStatus`; the code and
  generated schema use `collabAgentToolCall` with `receiverThreadIds` and
  `agentsStates`, plus an undocumented `subAgentActivity` item —
  [App Server docs](https://learn.chatgpt.com/docs/app-server.md), [ThreadItem.ts](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/schema/typescript/v2/ThreadItem.ts).
- Stability: the February post promises a surface partners "could safely
  depend on" that is "backward compatible"; the September docs call the
  app-server command experimental and not for production — [mirror](https://github.com/newton20/harness-engineering-kb/blob/master/raw/openai-com-index-unlocking-the-codex-harness.md), [Codex MCP server removal](https://learn.chatgpt.com/docs/mcp-server.md).
- The February post presents `codex mcp-server` as an integration option;
  it was removed on 5 September 2026 — [changelog](https://learn.chatgpt.com/docs/changelog).
- The February post says all surfaces use stdio; current docs add
  WebSocket, Unix socket and remote-control transports — [App Server docs](https://learn.chatgpt.com/docs/app-server.md).
- Symphony launch date: 27 April 2026 (openai.com, by search snippet)
  versus 28 April 2026 (press) — [Help Net Security](https://www.helpnetsecurity.com/2026/04/28/openai-symphony-codex-orchestration-linear/).
- Symphony's Elixir README still lists `untrusted` as an approval-policy
  value, which Codex has retired — [Elixir README](https://github.com/openai/symphony/blob/main/elixir/README.md), [Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security.md).
- The IDE page links to `cloud#delegate-from-the-ide-extension`, an anchor
  the current Codex Cloud page no longer has — [Codex IDE extension](https://learn.chatgpt.com/docs/codex/ide.md), [Codex Cloud](https://learn.chatgpt.com/docs/cloud.md).
- The agent-safety guide still recommends "GPT-5 or GPT-5-mini" while the
  same docs set announces GPT-5.5's retirement — [Safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety.md), [What's new](https://learn.chatgpt.com/docs/whats-new.md).
