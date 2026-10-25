# T3 Code: a local GUI over coding agents

Scope: what T3 Code is and how it compares with ParleyInn, read from its
source at [pingdotgg/t3code](https://github.com/pingdotgg/t3code)
commit 00eb8f6 (MIT, T3 Tools Inc.) on 3 October 2026, after a first
pass over secondary sources. Paths are relative to that repository.

## What it is

- A web, desktop (Electron) and React Native mobile GUI that drives the
  coding agents a user already pays for, from Theo's ping.gg team,
  launched in March 2026 — [Stork](https://www.stork.ai/de/t3-code),
  [PyShine](https://pyshine.com/T3Code-Minimal-Web-GUI-Coding-Agents/).
- Threads per agent session, plan mode, git worktrees for parallel
  agents, a live timeline of tool calls and diffs, checkpoints with
  rollback, stacked commit, push and pull request actions, and merge.
- A local Node.js or Bun server with a WebSocket RPC to the clients.

## Local server and remote access

- Binds `127.0.0.1` by default on port 3773 (`server.ts:248`,
  `config.ts:23`); `--host` or `T3CODE_HOST` opens it, desktop
  "Network access" binds `0.0.0.0`, and the WSL backend always does.
- Every route needs a scoped session: an HMAC-signed `httpOnly`,
  `SameSite=Lax` cookie, a bearer token or a DPoP-bound token, from a
  bootstrap token or a one-time pairing token shown with a QR code.
  WebSocket upgrades take a short-lived ticket, and each RPC passes a
  per-method scope check (`auth/RpcAuthorization.ts`).
- No Origin or Host check on HTTP or WebSocket, no CSRF token, no
  explicit DNS-rebinding defence; packaged builds answer CORS with a
  wildcard origin, without credentials. A page on another localhost
  port is same-site and receives the Lax cookie.
- Remote access is a core feature: LAN pairing, Tailscale Serve, SSH
  with a port forward, and T3 Connect, which uses Clerk identity, a
  Cloudflare relay Worker and managed cloudflared tunnels
  (`docs/user/remote-access.md`, `docs/internals/t3-connect.md`).
- Telemetry goes to PostHog by default, off only with
  `T3CODE_TELEMETRY_ENABLED=false` (`docs/user/telemetry.md`). Other
  hosts: the T3 relay, `clerk.t3.codes`, `app.t3.codes`, OAuth
  providers, forge APIs and update feeds.

## Event store

- SQLite at `~/.t3/userdata/statev2.sqlite`, table
  `orchestration_events`: sequence, event id, aggregate, stream and
  stream version, type, time, command, causation and correlation ids,
  actor kind, payload and metadata JSON. Most payloads are full-state
  snapshots rather than deltas.
- One transaction commits the events, projections, command receipt and
  outbox effects (`orchestration-v2/ProjectionMaintenance.ts`).
  `rebuild` re-applies the log to empty projections; `verify` checks
  schema version, last sequence and thread set at start.
- No hash chain, signature or immutability guard. A compaction path
  deletes superseded snapshot events, called only from tests today.
  Deletion is a soft `thread.deleted` tombstone. No redaction of
  secrets in payloads was found.

## Harness control

- Claude through the Agent SDK's `query()` with a streaming prompt
  queue, no Claude Code hooks: steer by a mid-turn message, interrupt,
  switch permission mode, plan mode with `ExitPlanMode` captured as a
  plan. Approvals arrive through `canUseTool`, become events, and wait
  for the UI's accept, accept-for-session, decline or cancel. New
  threads default to full access (`docs/user/permission-modes.md`).
- Codex through `codex app-server` over JSON-RPC on stdio: start,
  steer, interrupt, fork, compact; per-turn approval and sandbox
  policy; command, file-change, permission and MCP elicitation
  approvals.

## Git

- Checkpoints as hidden refs `refs/t3/checkpoints/<thread>/turn/<n>`,
  built through a temporary index; worktrees under `~/.t3/worktrees`.
- Pull requests through the host CLIs (`gh`, `glab`, `fj`, `tea`,
  `az`) or Bitbucket's API; the GitHub token comes from `gh auth token`,
  pinned to its verified host. Merge and auto-merge via `gh pr merge`.

## Agents, threads and people

- One thread holds one agent conversation at a time. Switching
  provider mid-thread is a handoff: a budgeted selection of the history
  moves to the new provider, not a summary, and the docs say "for an
  important constraint, you can still repeat it in your next message"
  (`docs/user/portable-handoffs.md`).
- A fork or a subagent becomes its own thread (lineage `fork` or
  `subagent`); subagent threads cannot take messages, and `merge_back`
  pulls a fork's context back (`packages/contracts/src/orchestrationV2.ts`,
  `docs/user/thread-sidebar.md`).
- Parallel agents run as separate threads, optionally in separate
  worktrees, grouped by project in the sidebar. No channel holds several
  agents and several people together.
- No multiplayer: one person, many devices (web, desktop, mobile)
  paired to one environment. No presence, posts, roles or co-authors.

## Changes and review

- Each turn's diff opens in a right-hand panel, inline or side by side,
  with a changed-files tree in the chat; diffs come from per-turn
  checkpoints, can be rolled back, and open in the user's editor
  (`apps/web/src/components/DiffPanel.tsx`, `diffPanelStore.ts`). The
  change is not shown in place in a shared editor buffer.
- Pull requests are reviewed inside the app: a code tab with the
  forge's existing threads, line comments queued into a review, and
  approve or request changes sent to the forge
  (`components/pullRequest/PullRequestReviewForm.tsx`).

## Sandboxing

- No sandbox of its own. It passes each provider's own policy per turn:
  Codex's `sandboxPolicy` (read-only, workspace-write, full access),
  the Cursor SDK's sandbox, and for Claude a mapping that includes an
  `externalSandbox` kind (`orchestration-v2/RuntimePolicy.ts`,
  `Adapters/ClaudeAdapterV2.ts`). ACP agents run under their own
  sandbox; for Pi the docs say the policy "is not an operating-system
  sandbox".
- Permission modes are approval levels, not isolation: Supervised,
  Auto-accept edits, Auto, Full access. New threads default to Full
  access (`docs/user/permission-modes.md`).
- The T3 server runs as the user, holds forge tokens and its signing
  key under `~/.t3/secrets`, and starts the agents as its children, so
  an unsandboxed agent of the same user faces the same residual risks
  as with ParleyInn (ParleyInn's SRS §6.1, R1–R7).

## Untrusted content and history

- Chat markdown is sanitized (`rehype-sanitize`, DOMPurify for
  Mermaid); agent-produced HTML and SVG are served under a sandbox CSP.
- Prompt-injection handling is ad hoc: attachments are wrapped as
  untrusted data, and handoffs say history is context. No trust mark
  on tool or web output in events.
- The agent can search (plain `LIKE`) and read past threads through
  MCP tools scoped to its project, and the text returns raw, with no
  untrusted envelope. An MCP respond tool can answer questions but,
  by its schema, cannot approve a permission.
- It imports existing Claude Code and other agents' sessions.

## Against ParleyInn

| Question          | T3 Code                                                     | ParleyInn (2.0-draft)                                                          |
| ----------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------ |
| Where it sits     | A GUI that drives the harnesses                             | A record under the harnesses, and a lane view that steers them                 |
| People            | One person, several devices                                 | Co-authors, reviewers and watchers in one lane (P2)                            |
| Shared channel    | One agent per thread; parallel agents in separate threads   | One lane holds many agents and people; posts reach an agent by endorsement     |
| Changes           | Per-turn diff panel, checkpoints, rollback                  | Each edit as a diff in the timeline, attributed per hunk; replay               |
| Review            | In-app PR review sent to the forge                          | Review tab with evidence classes; approvals next (P2)                          |
| Sandboxing        | Passes each provider's own policy; default Full access      | Records each session's sandbox state; risks accepted when unsandboxed (OWN-22) |
| Record            | Event store in SQLite; no chain; snapshots may be compacted | One signed, hash-chained log per writer; nothing deleted silently              |
| Local guard       | Scoped tokens, no Origin or Host check                      | Per-launch token, Host and Origin checks, origin-scoped storage                |
| Network           | Remote access, relay and telemetry built in                 | Core opens no socket; peers opt-in; no central service or telemetry            |
| Untrusted content | Sanitized rendering; history recalled raw                   | Pull-only recall in an untrusted envelope (I2)                                 |
| Constraints       | None found                                                  | Pins restored verbatim after compaction (I3)                                   |
| Erasure           | Soft tombstone; no redaction                                | Redaction before storage; purge that leaves no confirming value                |
| Harness control   | Agent SDK and Codex app-server, rich steering               | Through the harness's input interfaces (OWN-03), in `parinn-run`               |

## Lessons for ParleyInn

- Worth reusing:
  - committing event, projection and receipt in one transaction;
  - `rebuild` and `verify` from the log alone (I10);
  - short-lived WebSocket tickets, and a scope check on every RPC;
  - pairing that narrows scope and never widens it;
  - pairing secrets in URL fragments;
  - a sandbox CSP on agent-produced HTML;
  - a forge token pinned to its verified host;
  - an MCP tool whose schema cannot approve a permission, which is
    OWN-04's rule expressed in a type.
- Its harness control is the shape ParleyInn's adapters need (ASM-19): the
  Agent SDK's streaming queue for steer, `canUseTool` for approvals,
  and Codex's app-server requests.
- It competes directly with ParleyInn's lane view, owner controls and
  review tab (VIEW, OWN, `parinn-run`): it ships today what plan
  2610022338 plans. Only ParleyInn's core, the record, sits beneath whatever
  drives the harnesses; the lane view and T3 Code are rival surfaces.
- Where ParleyInn differs is the lane as a shared channel: several agents
  and several people in one lane, peer to peer, with endorsement. T3
  Code keeps one agent per thread and one person per environment.
- Its handoff docs tell users to repeat important constraints by hand
  after a switch: the gap ParleyInn's pins close (I3).
- Its gaps are ParleyInn's pitch: no tamper evidence, no Origin or Host
  check on a port it also exposes remotely, telemetry and cloud
  services by default, no redaction, raw history recall, and full
  access as the default permission mode.
