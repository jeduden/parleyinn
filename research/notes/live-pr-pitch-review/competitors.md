# Competitor review of the ParleyInn pitch

Lens: does an existing product already match the pitch, or come close?
Reviewer stance: adversarial. Date: 2 October 2026. Multica, LobeHub and an
updated Warp comparison were added on 20 October 2026; the original ranking
and verdict below remain a dated snapshot.

Provenance tags:

- **[R]**: the fact comes from the gathered research in `/home/user/cairn/research/`
  (the fact sheets in `notes/agent-session-storage-sweep/sheets.json`, the
  notes in `notes/custom-storage-git-and-chat/`, and the reports). The URL
  given is the primary source the research cited.
- **[W]**: the fact comes from a fresh web search or fetch made on
  2 October 2026 for the original review, or on 20 October 2026 for the
  dated Multica, LobeHub and Warp additions.

Where the research and the web disagree, both are noted.

## Pitch claims under test

| #   | Claim                                                                           |
| --- | ------------------------------------------------------------------------------- |
| C1  | One lane record: conversation, edits, tool runs and results in order            |
| C2  | Chat with the agents in the branch or PR                                        |
| C3  | Real-time visibility of edits, test runs and messages                           |
| C4  | Multiplayer: several agents and humans in one lane, plus a fleet view           |
| C5  | Interactive result views (tests, benchmarks, diffs, rendered output)            |
| C6  | Lands in git as an ordinary commit, with the story one click away               |
| C7  | Verifiable, tamper-evident record                                               |
| C8  | Messages from non-owners reach the agent as untrusted; nothing injected unasked |
| C9  | Network-free core                                                               |
| C10 | Erasure by key (crypto-shredding)                                               |

C2 and its scores describe the 2 October "live pull request" pitch. They do
not test whether one room holds discussion around an intent across linked
issues and pull requests. The
[20 October criterion below](#conversation-around-the-intent-a-missing-comparison-criterion)
separately tests where issue discussion, review feedback and replies happen.

## 1. Feature matrix

Legend: **Y** yes, **P** partial, **N** no, **?** unknown or undocumented.
Each row's sources follow the table.

| Product                                     | C1  | C2  | C3  | C4  | C5  | C6  | C7  | C8  | C9  | C10 |
| ------------------------------------------- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Zed Delta / DeltaDB                         | Y   | Y   | Y   | Y   | P   | P   | N   | ?   | N   | N   |
| Block Buzz                                  | P   | Y   | Y   | Y   | P   | Y   | P   | N   | N   | N   |
| GitHub Copilot (Agents tab, Agent HQ, app)  | P   | P   | Y   | P   | P   | Y   | P   | P   | N   | N   |
| Cursor (cloud agents, Graphite, Origin)     | P   | Y   | Y   | P   | Y   | P   | N   | N   | N   | N   |
| Conductor Cloud                             | Y   | Y   | Y   | Y   | P   | P   | N   | N   | N   | N   |
| Multica (added 20 October 2026)             | P   | P   | P   | P   | P   | P   | ?   | N   | N   | ?   |
| LobeHub (added 20 October 2026)             | P   | P   | P   | P   | P   | ?   | ?   | ?   | N   | ?   |
| Agor (preset-io)                            | P   | Y   | Y   | Y   | P   | P   | N   | N   | N   | N   |
| Warp (Agents and Factories; updated 20 Oct) | P   | P   | Y   | P   | P   | P   | ?   | N   | N   | ?   |
| Devin (Cognition)                           | P   | Y   | Y   | P   | P   | P   | N   | ?   | N   | N   |
| Augment Cosmos                              | P   | P   | Y   | P   | P   | P   | N   | N   | N   | P   |
| Coder Agents                                | P   | P   | Y   | P   | P   | P   | N   | N   | N   | N   |
| Replit Agent                                | Y   | P   | Y   | P   | Y   | P   | N   | N   | N   | N   |
| Entire CLI                                  | P   | N   | N   | N   | N   | Y   | P   | N   | N   | N   |
| Claude Code (teams, cross-session, web)     | P   | N   | P   | P   | P   | Y   | N   | Y   | N   | N   |
| Radicle                                     | N   | P   | N   | P   | N   | Y   | Y   | N   | P   | N   |
| Tangled                                     | N   | P   | P   | P   | N   | Y   | P   | N   | N   | N   |
| GitLab Duo Agent Platform                   | P   | P   | P   | P   | N   | P   | N   | N   | N   | N   |
| Linear agent sessions                       | P   | P   | P   | P   | P   | P   | N   | N   | N   | N   |
| GrayCodeAI/trace (alpha forge)              | P   | N   | N   | N   | N   | Y   | Y   | N   | N   | N   |
| Happy                                       | P   | N   | Y   | N   | N   | N   | N   | N   | N   | N   |
| Vibe Kanban, Sculptor (local)               | P   | N   | P   | P   | P   | P   | N   | N   | P   | N   |
| Diversion (+ Claude Code plugin)            | P   | N   | Y   | P   | N   | P   | N   | N   | N   | N   |

### Row sources and cell notes

**LobeHub** (added 20 October 2026; [full note](../lobehub/lobehub.md))

- C1 P, C2 P. LobeHub keeps agent conversations as Topics, with Tasks, Agent
  Documents and Pages around them. Connected coding agents run through their
  own runtimes; LobeHub shows their responses and may show tools, status and
  diffs. The docs do not establish one event-addressed conversation tied to an
  intent across several branches and forge objects. [W]
  <https://github.com/lobehub/lobehub/blob/canary/docs/usage/start.mdx>,
  <https://github.com/lobehub/lobehub/blob/canary/docs/usage/agent/connect-external-agents.mdx>
- C3 P, C4 P, C5 P. Agent Groups, Workspaces, connected execution devices,
  cross-device conversations and Pages overlap with ParleyInn's planned live and
  fleet views. Progress details depend on the connected agent; the sources do
  not establish live shared-branch editing or ParleyInn's result-evidence view. [W]
  <https://github.com/lobehub/lobehub>,
  <https://github.com/lobehub/lobehub/blob/canary/docs/usage/agent/connect-external-agents.mdx>
- C6 ?, C7 ?, C10 ?. The docs reviewed do not establish a commit-to-story
  back-link, a verifiable full record or erasure by key. These remain unknown.
  [W] <https://github.com/lobehub/lobehub/blob/canary/docs/usage/start.mdx>,
  <https://github.com/lobehub/lobehub/blob/canary/docs/self-hosting/start.mdx>
- C8 ?. LobeHub documents its own connector permissions and delegates native
  file, command and approval permissions to the connected agent. The docs do
  not establish ParleyInn's pull-only untrusted-data rule for all content that can
  reach an agent. [W]
  <https://github.com/lobehub/lobehub/blob/canary/docs/usage/agent/connect-external-agents.mdx>
- C9 N. Its self-hosted application uses a networked Next.js service and
  PostgreSQL. Self-hosting gives deployment control, but not ParleyInn's
  socket-free core. [W]
  <https://github.com/lobehub/lobehub/blob/canary/docs/self-hosting/start.mdx>

**Multica** (added 20 October 2026; [full note](../multica/multica.md))

- C1 P. An issue keeps its description, discussion, status, linked pull
  requests and run history together. Each run has a transcript with messages,
  tool calls and errors. Linked pull requests show metadata and open on the
  forge; their review threads are not documented as mirrored into the issue.
  The sources do not establish an event-addressed record of every edit. [W]
  <https://multica.ai/docs/comments>,
  <https://multica.ai/docs/issues>,
  <https://multica.ai/docs/tasks>,
  <https://multica.ai/docs/github-integration>
- C2 P, C3 P. People can reply to a running agent, and the transcript streams
  while a run is active. The conversation is attached to an issue, and the
  sources do not establish live file-edit replication inside a shared branch.
  [W] <https://github.com/multica-ai/multica>,
  <https://multica.ai/docs/tasks>
- C4 P. People and agents share a workspace and issues; squads route work
  through a leader, whose mentions can start member runs. This is coordinated
  routing rather than several agents editing one shared branch in real time.
  [W] <https://multica.ai/docs/squads>,
  <https://multica.ai/docs/project-resources>
- C5 P. Run transcripts, diffs, linked pull requests and CI status are visible.
  No interactive result explorer is established by the sources reviewed. [W]
  <https://multica.ai/docs/tasks>,
  <https://multica.ai/docs/github-integration>
- C6 P. Ordinary git worktrees and pull requests are supported. Multica
  attaches a pull request to an issue when its title, branch name or body
  carries the issue id, or a person links it manually. No commit-to-run
  back-link is documented.
  [W] <https://multica.ai/docs/project-resources>,
  <https://multica.ai/docs/github-integration>
- C7 ?, C10 ?. The sources reviewed do not establish a tamper-evident full
  transcript or erasure by key. This is unknown, not proof of absence. [W]
  <https://multica.ai/docs/tasks>,
  <https://multica.ai/docs/security-model>
- C8 N. Workspace and project context is supplied to runs; a squad leader can
  cause another agent's run through a mention. No principal-controlled,
  pull-only, untrusted envelope is documented for these paths. This rates
  ParleyInn's specific C8 claim, not Multica's access controls. [W]
  <https://multica.ai/docs/workspaces>,
  <https://multica.ai/docs/project-resources>,
  <https://multica.ai/docs/squads>
- C9 N. A networked server and connected daemon coordinate runs. Self-hosted
  deployments send a daily telemetry snapshot by default, which can be disabled.
  [W] <https://multica.ai/docs/how-multica-works>,
  <https://github.com/multica-ai/multica/blob/main/SELF_HOSTING.md>

**Zed Delta / DeltaDB**

- C1 Y. "A message and the edit it produced are recorded side by side"; every
  operation has a stable identity. Agent turns, tool calls, tool output and
  terminal output are part of the thread. [R]
  <https://zed.dev/blog/introducing-deltadb>,
  <https://delta.dev/docs/agents/threads>
- C2 Y. "You invite teammates directly into your conversations with agents."
  The thread replaces the PR; Zed disabled PRs on Delta's own repository. [R]
  <https://zed.dev/blog/delta-public-beta>. 33 team members have landed 570
  changes that way. [W]
  <https://alphasignal.ai/news/zed-opens-delta-to-the-public-replacing-pull-requests-with-ai-threads>
- C3 Y. It "replicates the conversation and the worktree together, in real
  time, for everyone in a thread". [R] <https://zed.dev/blog/introducing-delta>
- C4 Y. CRDT worktrees let "many people and agents edit the same files at
  once". Agents in any thread can message agents in any other thread
  (0.18.0). The thread sidebar groups subthreads and subagents. [R]
  <https://zed.dev/blog/introducing-deltadb>; [W]
  <https://delta.dev/docs/whats-in-the-latest>. The fleet view is a sidebar,
  not a dashboard, so this is close to Y.
- C5 P. Land Changes results render as cards with branch, commit and CI status.
  Comments anchor to transcript, code and diff spans. These are structured
  cards, not explorable test or benchmark views. [W]
  <https://delta.dev/docs/whats-in-the-latest>; [R]
  <https://delta.dev/docs/agents/comments>
- C6 P. "A commit remains the checkpoint you push, pull, and build from." Non-Delta
  users "see a normal git repo". The story stays in DeltaDB. No trailer or
  back-link into the commit is documented. [W]
  <https://zed.dev/blog/delta-public-beta>,
  <https://delta.dev/docs/agents/review-and-sync>
- C7 N. No signing, hash chain or verification is documented. [W]
  <https://delta.dev/docs/agents/review-and-sync>
- C8 ?. No collaborator trust model is documented. A send submits every
  author's drafts in one turn, charged to the sender. That design treats
  co-authors as peers, not as untrusted. [R]
  <https://delta.dev/docs/collaboration/collaborate-thread>
- C9 N. Live sync runs over TLS to a Cloudflare Durable Objects backend. [R]
  <https://delta.dev/docs/privacy-and-security/security>
- C10 N. TLS plus Cloudflare-managed keys at rest. No E2E, no customer keys.
  Server-side deletion means emailing privacy@zed. [R]
  <https://delta.dev/docs/privacy-and-security/security>,
  <https://delta.dev/docs/privacy-and-security/data-storage>

**Block Buzz** (Apache-2.0, launched 21 July 2026)

- C1 P. "Every message, reaction, workflow step, review approval, and git event
  is a signed event in one log." The log holds patches as NIP-34 events, not
  every keystroke edit or tool run. [W]
  <https://raw.githubusercontent.com/block/buzz/main/README.md>
- C2 Y. "Branch as room": a feature branch creates a channel. Patches, CI, agent
  review and the merge decision land "in the same room as the evidence". [R] [W]
  <https://github.com/block/buzz>
- C3 Y. A WebSocket relay with Redis presence and typing. [R]
  <https://github.com/block/buzz>
- C4 Y. Agents are members with their own keypairs. Channels, threads and
  workflows serve as the fleet view. [W]
  <https://siliconangle.com/2026/07/21/block-launches-buzz-open-source-workspace-humans-ai-agents/>
- C5 P. It has canvases and CI posts, but no documented live test or benchmark
  explorer. [W] <https://raw.githubusercontent.com/block/buzz/main/README.md>
- C6 Y. The git hosting backend and NIP-34 git events are listed under "Works
  today" (verified on the raw README on 2 October 2026; a WebFetch summary
  claimed otherwise and was wrong). Smart-HTTP serves ordinary `git clone` and
  `git push`. After merge, the channel "becomes an archived record of why the
  change exists". [W]
  <https://raw.githubusercontent.com/block/buzz/main/README.md>; [R]
  <https://raw.githubusercontent.com/block/buzz/main/VISION.md>
- C7 P. Signed Nostr events, plus `buzz-audit`, a hash-chained log with
  `verify_chain()`. The audit write is fire-and-forget, so a failed write is
  silent. [R] <https://github.com/block/buzz/blob/main/ARCHITECTURE.md>
- C8 N. Agents are scoped by identity and channel membership. Nothing is
  documented on treating message content as untrusted input. [W]
  <https://raw.githubusercontent.com/block/buzz/main/README.md>
- C9 N. A relay with Postgres, Redis and S3. [R] <https://github.com/block/buzz>
- C10 N. "TLS in transit. At-rest encryption delegated to the storage layer."
  NIP-09 relay-side deletion only. [R]
  <https://raw.githubusercontent.com/block/buzz/main/VISION.md>,
  <https://raw.githubusercontent.com/block/buzz/main/NOSTR.md>

**GitHub Copilot: Agents tab, Agent HQ, Copilot app**

- C1 P. Session logs group tool calls and show inline diffs. They are separate
  from PR review comments. [R]
  <https://github.blog/changelog/2026-03-19-more-visibility-into-copilot-coding-agent-sessions/>
- C2 P. You steer by follow-up prompts and @copilot on the PR. "Steering is not
  available for third-party coding agents." [R]
  <https://github.blog/news-insights/company-news/welcome-home-agents/>
- C3 Y. "Follow progress in real time" from the Agents tab, VS Code, mobile and
  the CLI. [R]
  <https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/track-copilot-sessions>
- C4 P. One writer per session, one branch per task. The Agents tab ("mission
  control") and the Copilot app's "My Work" view are fleet views. Copilot,
  Claude and Codex can run on the same task. [R]
  <https://github.blog/changelog/2026-01-26-introducing-the-agents-tab-in-your-repository>;
  [W]
  <https://letsdatascience.com/news/github-launches-copilot-app-as-desktop-home-for-ai-agents-287628bd>
- C5 P. The Copilot app has inspectable "canvases" for plans and terminal output,
  plus Agent Merge. [W]
  <https://letsdatascience.com/news/github-launches-copilot-app-as-desktop-home-for-ai-agents-287628bd>
- C6 Y. "The commit message for each agent-authored commit includes a link to the
  agent session logs" (the Agent-Logs-Url trailer). [R]
  <https://github.blog/changelog/2026-03-20-trace-any-copilot-coding-agent-commit-to-its-session-logs>
- C7 P. The cloud agent signs every commit (Verified badge). That covers the
  commit, not the session record. [W]
  <https://github.blog/changelog/2026-04-03-copilot-cloud-agent-signs-its-commits/>
- C8 P. Hidden characters, such as HTML comments in issues, are filtered before
  user input reaches the agent. Only users with write access can trigger it.
  [R] <https://docs.github.com/en/copilot/concepts/agents/about-third-party-agents>
- C9 N. Hosted. C10 N. Cloud sessions can be archived, not deleted. No
  encryption detail. [R] (same Agent HQ sheet sources)

**Cursor: cloud agents, shared transcripts, Graphite, Origin**

- C1 P. The transcript holds "prompts, model responses, tool calls, diff context,
  and demo artifacts". It is a server-owned linear log, separate from the PR.
  [R] <https://cursor.com/docs/cloud-agent/security>
- C2 Y. Graphite Agent puts "chat in your PR page for follow-up questions and
  conversational edits". Cursor Cloud Agents run inside Graphite. [W]
  <https://graphite.com/blog/introducing-graphite-agent-and-pricing>. Team
  follow-ups let several humans write into one agent conversation. [R]
  <https://cursor.com/docs/cloud-agent/settings>
- C3 Y. SSE streams with resume. Teammates open the run URL to watch. [R]
  <https://cursor.com/docs/cloud-agent/api/endpoints>
- C4 P. Many parallel agents, each on its own VM and branch. Team follow-ups are
  multi-human but serialized. No shared lane with several agents. [R]
  <https://cursor.com/docs/cloud-agent>
- C5 Y. Agents post videos, screenshots and logs as PR artifacts, plus remote
  desktop takeover. MCP Apps render interactive UIs. [R]
  <https://cursor.com/help/ai-features/background-agents>; [W]
  <https://www.havoptic.com/blog/deep-dive-cursor-q1-2026>
- C6 P. The PR body carries "Open in Web" links back to the agent. The commit
  message carries no run-ID trailer. [R]
  <https://github.com/ketheridge7/sideline/pull/78>
- C7 N. C8 N. Cursor itself warns that team follow-ups allow "lateral movement and
  secret exposure"; it offers no untrusted marking. [R]
  <https://cursor.com/docs/cloud-agent/settings>. C9 N. C10 N. [R]

**Conductor Cloud** (GA 30 July 2026)

- C1 Y. Workspace = branch = worktree. "Chats, diffs, PR state, and archive
  actions stay attached to the same workspace." There are per-turn checkpoint
  refs. [R] <https://www.conductor.build/docs/concepts/git-worktrees>,
  <https://www.conductor.build/docs/reference/checkpoints>
- C2 Y. You chat with agents inside the branch's workspace. [R]
- C3 Y. "The transcript and new agent output update live for everyone", with
  presence avatars and typing indicators. [R] [W]
  <https://www.conductor.build/docs/cloud/collaboration>
- C4 Y. Teammates "prompt agents together". Home shows the organization's
  workspaces, a fleet view. It runs Claude Code, Codex, Cursor and OpenCode. [W]
  <https://runtimewire.com/article/conductor-launches-multiplayer-cloud-workspaces-coding-agents>,
  <https://www.conductor.build/docs/cloud/collaboration>
- C5 P. A diff viewer and per-response file lists. [R]
- C6 P. The branch is turned into an ordinary GitHub PR. No trailer or session
  ID goes into the commit. [R] <https://api.conductor.build/v0/openapi.json>
- C7 N. C8 N. Its only guard against secrets is "don't ask an agent to print
  secrets into chat". [R] <https://www.conductor.build/docs/reference/privacy>.
  C9 N for Cloud. C10 N: stored data "is encrypted", with no detail. [R]

**Agor** (preset-io, open source, npm 0.26.8 of 29 September 2026)

- C1 P. Each task records `sha_at_start` and `sha_at_end` against its message
  range. Every session belongs to a branch. [R]
  <https://raw.githubusercontent.com/preset-io/agor/main/packages/core/src/types/task.ts>
- C2 Y, C3 Y, C4 Y. A real-time multiplayer board with cursors, facepiles,
  shared terminals and comments pinned to sessions. It runs many Claude Code,
  Codex and Gemini sessions, one card per worktree. That is a fleet view. [R]
  <https://agor.live/guide/architecture>; [W]
  <https://jimmysong.io/ai/agor>
- C5 P. Shared terminals and boards. C6 P. "1 branch = 1 issue = 1 PR". Nothing
  is written into git. [R] <https://agor.live/llms-full.txt>
- C7 N. C8 N. C9 N: it is self-hostable, but the daemon is networked. C10 N:
  only secrets are AES-GCM encrypted. [R]

**Warp Agents and Factories**

The row now covers Warp Agents and Factories, including product documentation
published after the original review. See the [full note](../warp/warp.md) and
the [20 October update](#update-warp-20-october-2026). [W]
<https://github.com/warpdotdev/warp>,
<https://docs.warp.dev/factories/how-factories-work/>,
<https://docs.warp.dev/factories/integrations/github/>.

**Devin**

- C2 Y. A `/devin` comment on a PR with an active session "is sent to that
  session". Devin auto-responds to PR comments. [R]
  <https://docs.devin.ai/integrations/gh>
- C3 Y, hosted only. C4 P. A coordinator session spawns child sessions. C5 P.
  IDE and browser iframes in the session. C6 P. The PR body links the session;
  commits carry only `Co-Authored-By`. [R]
  <https://docs.devin.ai/integrations/gh>,
  <https://github.com/flightcontrolhq/modules/pull/84>
- C7 N, C9 N, C10 N. C8 ?. [R]

**Augment Cosmos**

- C1 P, C3 Y, C4 P. Sessions own every message, turn and tool call. Others
  "watch it run and follow every turn". Shared sessions allow several
  contributors. [R] <https://docs.augmentcode.com/cosmos/sessions-overview.md>
- C6 P. PR and branch artifacts link sessions by reference; nothing is written
  into git. [R] <https://docs.augmentcode.com/cosmos/artifacts.md>
- C10 P. Company-level customer-managed encryption keys are claimed. That is
  tenant-wide revocation, not per-item shredding. [R]
  <https://www.augmentcode.com/security>

**Coder Agents** (self-hosted)

- C3 Y. A WebSocket chat stream. C4 P. Sharing is read-only: viewers "cannot send
  or edit messages". C6 P. Chats link to remote, branch and PR, not to commits.
  Messages sit in plaintext Postgres. [R]
  <https://github.com/coder/coder/blob/main/docs/ai-coder/agents/chat-sharing.md>,
  <https://github.com/coder/coder/blob/main/coderd/database/dump.sql>

**Replit Agent**

- C1 Y. Checkpoints tie a git commit to a point in the conversation and a
  database snapshot. "Collaborators see checkpoint creation in real-time." C5 Y.
  The live running app is the result view. C6 P. Commits pushed to GitHub carry
  no link back. [R]
  <https://docs.replit.com/features/version-control/checkpoints-and-rollbacks>

**Entire CLI** (Go, MIT)

- C6 Y. Commits carry an `Entire-Checkpoint:` trailer. Transcripts live in
  `refs/entire/checkpoints/...` on the user's own remote. That is "an ordinary
  commit with the story attached" in plain git. [R]
  <https://github.com/entireio/cli>
- C7 P. Signing is best effort, nothing verifies signatures on read, and there is
  no hash chain beyond git's object graph. [R]
  `research/reports/agent-session-storage-beyond-git.md`, citing
  <https://github.com/entireio/cli/blob/89c26160877a27a5017dc0bd614768c463e793bd/cmd/entire/cli/checkpoint/persistent.go>

**Claude Code** (agent teams, cross-session messaging, cloud sessions)

- C8 Y. A message from another agent is marked as coming from another session,
  "never counts as your consent", and cannot approve prompts. In auto mode a
  classifier reviews every inter-agent message. [R]
  <https://code.claude.com/docs/en/agent-teams>,
  <https://code.claude.com/docs/en/cross-session-messaging>
- C6 Y. Cloud sessions add a `Claude-Session:` URL trailer to commits. This very
  review session is told to sign commits that way. [R]
  <https://code.claude.com/docs/en/claude-code-on-the-web>
- C3 P, C4 P. Remote Control, agent view and teams. No shared multi-human lane.
  [R] <https://code.claude.com/docs/en/agent-view>

**Radicle** (Rust)

- C7 Y. All refs are signed into `refs/rad/sigrefs`. COBs (issues, patches,
  reviews) are signed CRDT commit DAGs. C9 P. It is local-first: COBs can be
  created offline and sync by gossip. C2 P. Discussion is asynchronous patch
  comments, not live chat. Data is "not encrypted at rest". [R]
  <https://radicle.dev/guides/protocol>

**Tangled** (AT Protocol, alpha)

- C6 Y. Plain git on knots. C3 P. A WebSocket `/events` stream. C7 P. DID-signed
  records; Knot 2 seals per-repository signing keys. No agent linkage, and it is
  "public by design". [R] <https://docs.tangled.org/knot-self-hosting-guide.html>,
  <https://blog.tangled.org/intro>

**GitLab Duo Agent Platform**

- C3 P. Live checkpoint stream to the owner only. C6 P. Linking to the MR is weak
  and implicit. Sessions auto-delete 30 days after last activity. [R]
  <https://docs.gitlab.com/user/duo_agent_platform/sessions/>

**Linear agent sessions**

- C2 P, C5 P. Agent activities appear on the issue. Coding Sessions (June 2026)
  create PRs, with screenshots and recordings as verification artifacts.
  Webhook-driven, with no live stream. [R]
  <https://linear.app/developers/agents>

**GrayCodeAI/trace** (Go, alpha 0.0.1, 0 stars)

- C6 Y, C7 Y. A self-hosted forge in which every checkpoint links to a commit SHA
  and is signed by an Ed25519 node identity. Imports are append-only and reject
  changed signatures. No real-time features, minimal data. [R]
  <https://github.com/GrayCodeAI/trace>

**Happy, Vibe Kanban, Sculptor, Diversion, OpenHands, Ona**

- Happy: real-time E2E-encrypted relay of one person's session to their own
  devices. No link to git. [R] <https://github.com/slopus/happy>
- Vibe Kanban and Sculptor: local-first boards with per-task worktrees. They are
  local, but they call model APIs and store plaintext. [R]
  <https://github.com/BloopAI/vibe-kanban>, <https://imbue.com/sculptor/>
- Diversion: a cloud VCS that syncs work in progress in real time. Its Claude Code
  plugin links trajectories to code. [R]
  <https://docs.diversion.dev/agent-quickstart>,
  <https://www.diversion.dev/diversion-claude-code-plugin>
- OpenHands: event-sourced conversation log with a single writer and weak code
  linkage. [R] <https://docs.openhands.dev/sdk/arch/conversation.md>
- Ona: branch-tied environments and PRs that link back to the run. No real-time
  multi-user collaboration. [R] <https://ona.com>

### Recently surfaced names (not in the matrix)

- **Forklane.ai** advertises a "multiplayer coding environment" for humans and
  agents with "agent-aware version control". The only sources are AI-directory
  pages; no primary site was verified. Treat it as unconfirmed. [W]
  <https://www.stork.ai/tools/forklane>
- **Harness** announced an agent-ready repository with AI code review. [W]
  <https://letsdatascience.com/news/harness-launches-agent-ready-repository-and-ai-code-review-2826d9ea>
- **Microsoft AGT** emits Merkle-chained, HMAC-signed tool-call audit logs. These
  are tamper-evident within one deployment. [W]
  <https://microsoft.github.io/agent-governance-toolkit/adr/0032-agt-emits-trace-v01-trust-records/>

## 2. Closest overall match

**Zed Delta is the closest match.** It hits C1–C4 fully and C5–C6 partly. Its own
launch line is "Replacing pull requests is our first step toward replacing
GitHub" [R] <https://zed.dev/blog>. Its threads hold conversation, edits, tool
output, reviews and commits, replicated live to every participant [R] [W].
Several humans and agents work one thread together. Agents in any thread can
message agents in any other [W] <https://delta.dev/docs/whats-in-the-latest>.
Changes land as ordinary git commits through a Land subthread [W]
<https://delta.dev/docs/agents/review-and-sync>. Zed dogfoods it with PRs turned
off, and has landed 570 changes that way [W]
<https://alphasignal.ai/news/zed-opens-delta-to-the-public-replacing-pull-requests-with-ai-threads>.
It is free in public beta on macOS, Linux, Windows and the web.

By pitch paragraph, Delta covers the first three paragraphs almost word for
word: "one record", "code and talk can never drift", "a PR today is a frozen
diff", chat, real time, multiplayer. It covers none of the fourth paragraph
(C7–C10). By count, Delta is about **60–65 %** of the pitch, and it covers the
headline half.

**Block Buzz comes second**, and it is closer on C6 and C7. Branch-as-room, a
signed event log, an audit hash chain and git hosting that works today put it
ahead of Delta on "verifiable" and "lands in git". It is weaker on C1: its record
holds patches, not every edit. It is self-hostable and open source (Apache-2.0)
[W] <https://raw.githubusercontent.com/block/buzz/main/README.md>.

**Conductor Cloud is third** on the product surface: a workspace per branch,
live shared transcripts, prompting together, and a fleet Home view [W]. It has
nothing on C6–C10.

Together, Delta (live thread), Buzz (signed branch room) and Entire (story in
git refs plus a commit trailer) cover every pitch claim except C8–C10.

## 3. Unoccupied claims and commodity claims

### Already commodity (at least three shipping products each)

- **C3 real time**: Delta, Buzz, Conductor, Agor, Warp, Cursor, Copilot, Devin,
  Augment, Coder, Replit. It is table stakes.
- **C2 chat with the agent on the branch or PR**: Delta, Buzz, Graphite Agent,
  Devin `/devin`, Conductor, Agor, @copilot.
- **C4 fleet view**: Copilot Agents tab and Copilot app, Conductor Home, Agor
  board, Delta sidebar, Cursor agents list. Multiplayer within one lane is
  rarer (Delta, Conductor, Buzz, Agor, Warp), but it is shipped.
- **C6 lands as an ordinary commit with a back-link**: Copilot's Agent-Logs-Url
  trailer, Claude Code's Claude-Session trailer, Entire's Entire-Checkpoint
  trailer, Buzz's archived room, Devin and Cursor's PR-body links. The
  "one click away" part is commodity. Only "verifiable" is not.
- **C5 interactive results**: Cursor artifacts and MCP Apps, Replit's live app,
  Copilot canvases, Buzz canvases, Delta land cards. These are partial
  everywhere, but no one would call them new.

### Partly occupied

- **C1 one record of conversation plus every edit**: Delta does exactly this at
  operation granularity. Replit and Conductor do it at checkpoint granularity.
  It is not new. Delta owns it.
- **C7 tamper-evident record**: Radicle (signed COBs), Buzz (signed events and an
  audit hash chain), GrayCodeAI/trace (signed checkpoints, alpha), Copilot
  (signed commits only). None of them covers a full agent transcript with a
  verifiable chain that does not fail silently. The research reports reach the
  same conclusion: "No surveyed tool combines full verbatim transcripts, many
  writers, a tamper-evident append-only record and treating recalled content as
  untrusted" (`research/reports/agent-session-storage-beyond-git.md`).
- **C8 untrusted handling of non-owner messages**: Claude Code already does this
  for agent-to-agent messages, with a classifier in auto mode. GitHub filters
  hidden characters. No *multiplayer lane product* marks human co-author
  messages as untrusted. Cursor and Augment document the opposite risk, where a
  collaborator steers an agent holding someone else's secrets.

### Genuinely unoccupied (no shipping lane or PR product found)

- **C8 within a multiplayer lane**: "a message from anyone except the owner
  reaches the agent as untrusted", and "nothing stored is injected unasked".
- **C9 network-free core**: every live, multiplayer competitor is a hosted or
  self-hosted networked service. Local tools (Vibe Kanban, Sculptor, Radicle)
  are not lane products.
- **C10 erasure by key**: no candidate crypto-shreds. The nearest precedents are
  Keybase exploding messages (dormant) and Augment's tenant-level customer keys.
  [R] `notes/custom-storage-git-and-chat/integrated-platforms.md`
- **The combination** of C1–C6 with C7–C10. That is the only defensible
  novelty. The research names this quadrant as unoccupied:
  "ParleyInn's I2 and crypto-shredding goals would put it in a quadrant nobody
  currently occupies" [R].

The unoccupied claims are all in the pitch's fourth paragraph, the security
paragraph. The headline ("the live pull request for the agent era") and the
one-liner are the occupied part, and Zed shipped them first.

## 4. What a competitor would say in rebuttal

**Zed (Delta):** "That's our launch post. 'Replacing pull requests' is our
tagline. 'A message and the edit it produced are recorded side by side' is our
DeltaDB design. Our public beta runs on four platforms, including the web, and
we have landed 570 changes with PRs turned off. ParleyInn is pre-implementation. Our
CRDT worktrees let people and agents edit the same files live. A lane that
'never touches the network' cannot be real-time multiplayer without a relay,
and that relay is the product. Redaction runs before anything leaves the
device. Tamper evidence and key erasure are roadmap items we can add to a
working product faster than you can build the product."

**Block (Buzz):** "Branch-as-room, every event signed, agents with their own
keys, a hash-chained audit log, git hosting that works today, Apache-2.0 and
self-hostable. We already ship 'verifiable'. We ship the open, self-hostable
version of your multiplayer story, and Claude Code, Codex and goose already
plug in through ACP."

**GitHub:** "Every agent commit is signed and carries a link to its session
log. The Agents tab and the Copilot app are the fleet view. Copilot, Claude and
Codex run side by side on the forge where the code already lives. We
filter hidden-character injection before it reaches the agent. Teams will not
adopt a new 'lane' system to get what their PR already shows them."

**Conductor and Agor:** "Workspace = branch = worktree, a live shared
transcript, presence, prompting together and a board of every lane. We run the
agents your team already uses: Claude Code, Codex, Cursor, OpenCode. We shipped
in July."

**Anthropic (Claude Code):** "Peer-agent messages already arrive marked as
untrusted and can never count as consent. Commits already carry a
Claude-Session trailer. ParleyInn's C8 is a policy our harness already applies."

**Common rebuttal to the security paragraph:** "Nobody buys a PR replacement for
crypto-shredding. Tamper evidence over a transcript proves only that the log was
not edited after the fact, not that the agent behaved. Marking teammates'
messages as untrusted breaks the collaboration you are selling: if a reviewer's
'please fix line 40' arrives as untrusted data, the agent must refuse it or ask
the owner, so multiplayer collapses into single-player with spectators. And a
network-free core contradicts 'real time, multiplayer'. Either the relay is
part of the product and the claim is hollow, or it is not and the product is
not live."

## Verdict

The headline and one-liner ("the live pull request for the agent era",
"code and talk never drift") are not new. Zed Delta ships them in public beta,
Block Buzz ships the branch-as-room version, and Conductor and Agor ship the
multiplayer lane and fleet view. What is unoccupied is the security paragraph:
untrusted non-owner messages, a network-free core, tamper evidence over the full
record, and erasure by key. Lead with that paragraph and treat Delta as the
benchmark. Then answer two objections head-on: the tension between C8 and
collaboration, and the tension between C9 and real time.

### Update: Multica, 20 October 2026

Multica adds a strong **issue-centered agent-workspace** competitor to the
headline and product-surface claims. It already connects assignment, live run
transcripts, issue comments, worktrees, pull request cards and CI status,
with several agent CLIs and a self-hosting option [W]
<https://github.com/multica-ai/multica>,
<https://multica.ai/docs/project-resources>,
<https://multica.ai/docs/github-integration>. Pull request review discussion
stays on the forge; the issue card links to it [W]
<https://multica.ai/docs/github-integration>. It is less direct than Delta as a
live shared-branch record: its durable unit is an issue, and the documented
collaboration is routing between runs, not replicated editing in one shared
branch. The 2 October Delta ranking is therefore unchanged by this addition.

The distinction on the security paragraph remains material but must be stated
as a difference in documented contracts, not a claim that Multica has no
security. Multica documents run-scoped API tokens and per-run working
directories, while warning that its daemon's OS user is the execution boundary:
by default agent processes can use that user's files, credentials and network,
and unattended approval prompts are answered automatically [W]
<https://multica.ai/docs/security-model>. Its workspace and project context
enters runs automatically [W] <https://multica.ai/docs/workspaces>,
<https://multica.ai/docs/project-resources>. No source reviewed establishes
ParleyInn's pull-only untrusted-content rule, a tamper-evident full record, or
erasure by key. ParleyInn's comparable room view, record and trust guarantees must
ship before they can be tested against Multica's working experience.

Multica's license is source-available with additional conditions on hosted
services, commercial embedding and branding; it is not Apache-2.0 alone [W]
<https://github.com/multica-ai/multica/blob/main/LICENSE>. This affects reuse,
not the feature comparison.

### Update: LobeHub, 20 October 2026

LobeHub adds a broad **agent workspace and control-surface** competitor. It
connects external coding agents, including Claude Code and Codex, to Topics;
adds Tasks, Agent Groups, Pages, memory and shared Workspaces; and lets a
conversation use an execution device other than the one showing it [W]
<https://github.com/lobehub/lobehub>,
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/start.mdx>,
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/agent/connect-external-agents.mdx>.
This makes it a closer competitor for the fleet and collaboration experience
than for ParleyInn's full-record and trust guarantees. LobeHub documents that a
connected runtime keeps its own session, memory and permissions, and that
Topic history does not transfer that runtime state to another device [W]
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/agent/connect-external-agents.mdx>.
Its sources do not establish an intent-centered room that includes every
linked issue and review discussion. That is an unknown to test, not proof it
cannot do it. The 2 October Delta ranking is unchanged by this addition.

### Update: Warp, 20 October 2026

Warp is now a stronger **agent workflow and review** comparison than the
original shared-session row captured. Its open-source client hosts native and
third-party agents, shared sessions, branch diffs and inline review feedback
sent to an agent. Factories, currently marked early access, preserve one work
item through triage, planning, implementation and review. GitHub issues, PRs
and submitted reviews can start or continue Factory work; results return to
the originating forge thread [W]
<https://github.com/warpdotdev/warp>,
<https://docs.warp.dev/agents/local-agents/interactive-code-review/>,
<https://docs.warp.dev/factories/how-factories-work/>,
<https://docs.warp.dev/factories/integrations/github/>.

This narrows the distinction on review handling. Warp already reacts to a PR
review and lets people review agent changes inline. The documented human PR
path shares agent context by a link in the PR and keeps GitHub's diff as the
merge authority [W]
<https://docs.warp.dev/guides/agent-workflows/how-to-attach-agent-session-context-to-github-prs/>.
The sources reviewed do not establish a single conversation that imports
discussion from both a linked issue and its PR, with a person replying to
either source comment from that conversation. The [full note](../warp/warp.md)
sets out that question and the separate record, trust and network boundaries.

### Conversation around the intent: a missing comparison criterion

The original C2 asks whether people can chat with an agent on a branch or pull
request. That is weaker than ParleyInn's intended experience: one room holds the
conversation among people and agents around an intent, and may name branches
and link forge issues; results, evidence and review appear in that room. An
issue link does not set the intent. A pull request, when used,
is the forge's review object for a branch, connected by a pull-request link;
the ParleyInn conversation stays in the room. Future comparisons need to ask
whether issue discussion, review feedback and the responses to both stay in
that conversation without a person carrying words between threads.

Multica does not meet this stronger criterion on the documented path. Its issue
centralizes its own comments and runs, but a linked pull request opens on the
forge and its review discussion is not mirrored into the issue [W]
<https://multica.ai/docs/comments>,
<https://multica.ai/docs/github-integration>. Delta is the closer comparison:
its shared thread holds code and conversation and replaces the pull request in
its own workflow [R] <https://zed.dev/blog/delta-public-beta>.

LobeHub's Topic and Group conversations bring multiple agents into one
workspace, but the sources reviewed do not establish that a Topic imports both
linked issue comments and pull-request review threads and can send a reply to
either forge discussion from there [W]
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/start.mdx>,
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/agent/connect-external-agents.mdx>.

Warp's Factory resumes work on new activity in the originating issue, pull
request or review thread and can post back there. Its documented path for a
human reviewer still places the agent session beside the PR by link; it does
not establish that the linked issue and PR become one discussion in a Warp
session [W] <https://docs.warp.dev/factories/connect-your-factory/>,
<https://docs.warp.dev/factories/integrations/github/>,
<https://docs.warp.dev/guides/agent-workflows/how-to-attach-agent-session-context-to-github-prs/>.

ParleyInn now specifies the stronger path for linked issues and pull requests: the
forge bridge records review and issue items as untrusted events shown with
the room's conversation after the forge read credential owner authorizes
sharing that exact linked branch or issue with the room. A person can
explicitly send a response written as a room post back to the forge. The
bridge cannot send approvals or land code.
This is an M9 requirement, not a shipped capability. The comparison therefore
tests whether a competitor centralizes issue and review discussion and can
answer either externally originated comment without leaving that conversation
([LANE-08](../../../docs/srs/05b-lane-requirements.md),
[LANE-34](../../../docs/srs/05b-room-collaboration-requirements.md),
[SEC-28](../../../docs/srs/06-security.md)).

### Lessons taken into ParleyInn's contract

1. **Start from an issue without adopting its intent.** Multica's issue-first
   path makes entry easy. ParleyInn's M9 issue flow lets a person choose or create a
   room and add the issue link as a separate act; imported text never sets the
   intent ([LANE-34](../../../docs/srs/05b-room-collaboration-requirements.md)).
2. **Finish external discussions in the room.** The room shows issue and
   review items, accepts a room post in response, and sends it only on that
   person's explicit forge reply act. The M9 exit fixture covers the import,
   send, echo and failure path; exact room grants guard credentialed reads
   ([M9](../../../plan/2610112059_m9-review-publishing-bridges.md)).
3. **Make a return visit actionable.** Catch up states what changed, what
   needs the principal and which action can answer it; Needs you opens the
   item with its evidence ([VIEW-05 and VIEW-08](../../../docs/srs/05b-room-view-requirements.md)).
4. **Keep the source behind every short account.** Catch up lines and room
   summary statements link exact event addresses. A room summary stays
   untrusted, pull-only for agents, and cannot replace a pin or the record
   ([VIEW-08](../../../docs/srs/05b-room-view-requirements.md),
   [LANE-33](../../../docs/srs/05b-room-collaboration-requirements.md)).

These are specified or planned behaviors, not shipping claims. The comparison
should score competitors on these user paths and mark ParleyInn's delivery status.
