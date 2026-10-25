# Agent work UX: T3 Code, Amp orbs and Claude Code on the web

Scope: the screen layout and interaction design of three products that
show coding agents at work, read from their documentation, changelogs
and (for T3 Code) source on 4 October 2026. The question is what
ParleyInn's screen must match and where it can lead. Claims a source did
not support are marked unverified.

## T3 Code

A desktop, web and mobile control surface over Claude Code, Codex,
Cursor, OpenCode and others; "very very early"
([README](https://github.com/pingdotgg/t3code)).

- **Left: an inbox of threads**, not a project tree: Pinned, Active,
  Snoozed, Settled; threads settle after 3 idle days or a merged pull
  request ([thread-sidebar](https://github.com/pingdotgg/t3code/blob/main/docs/user/thread-sidebar.md)).
- **Centre: chat and composer**, with context attached as chips:
  terminal excerpts, diff comments, preview annotations, pull requests,
  other threads ([composer](https://github.com/pingdotgg/t3code/blob/main/docs/user/composer.md)).
- **Right: tabs** for Browser, Terminal, Files, Diff, Pull request and
  Device ([RightPanelTabs.tsx](https://github.com/pingdotgg/t3code/blob/main/apps/web/src/components/RightPanelTabs.tsx)).
  On mobile the panel becomes a sheet.
- **Outcome:** per-turn diffs from checkpoints, stacked or split; a
  pull-request file list with viewed marks synced to GitHub; an
  embedded browser preview where you annotate an element into the
  composer and see the agent's browser cursor; a live iOS or Android
  emulator the agent drives ([devices](https://github.com/pingdotgg/t3code/blob/main/docs/user/devices.md)).
  The activity log groups tool calls with exit codes and warns that
  "waiting on a thread does not mean it finished"
  ([activity-log](https://github.com/pingdotgg/t3code/blob/main/docs/user/activity-log.md)).
- **Steering:** a mid-run message steers or queues, per setting;
  queued messages can be edited or promoted; "Edit from here" rewinds
  with or without file changes; permission modes default to full
  access; plan mode ends in Refine or Implement.
- **Many at once:** status pills in priority order (pending approval,
  awaiting input, working, plan ready, completed unseen); a working
  section hides running threads until they come back to you; one
  prompt fans out to several models, each in its own worktree; mobile
  push on finish, fail or needs-you.
- **Hand-off:** commit, push and open a pull request; switching
  provider passes a budgeted slice of history. No multiplayer found.

## Amp orbs

A fresh remote machine per thread that keeps working with the laptop
closed ([orbs](https://ampcode.com/docs/orbs)).

- **Layout:** the conversation on the left; panes on the right for
  Changes (Ship, Review, Sync), Portals, Files, a terminal shared with
  the agent, and Space for voice and video
  ([threads](https://ampcode.com/docs/threads)). The sidebar groups
  threads by project; `/feed` lists every visible thread with filters.
- **Outcome:** the Changes pane puts "the files that best explain the
  change" first. Portals are authenticated URLs to a dev server in the
  orb, shown as a browser tab, live-reloading, with element annotation
  sent to the agent, and optionally public for 1 hour to 7 days
  ([portals](https://ampcode.com/docs/orbs/portals)).
- **Steering:** messages steer by default
  ([steer, don't queue](https://ampcode.com/news/steer-dont-queue));
  no approvals by default ([tools](https://ampcode.com/docs/tools)).
  Plan mode and rewind: none found.
- **Many at once:** agents spawn agents, each with its own orb, and
  message each other ([agent to agent](https://ampcode.com/docs/orbs/agent-to-agent));
  notifications when a thread is ready for your next message.
- **Hand-off and multiplayer:** Ship runs a configurable prompt
  (commit, rebase, test, push). Workspace members join a thread to
  prompt the agent and use its portal, terminal and files, with
  presence shown ([multiplayer](https://ampcode.com/docs/collaborate/multiplayer));
  Space shows which thread each person is viewing.

## Claude Code on the web

Cloud sessions from claude.ai/code, the mobile Code tab, the desktop
app or `claude --cloud`
([docs](https://code.claude.com/docs/en/claude-code-on-the-web)).

- **Layout:** a sessions sidebar; a prompt box with repository, branch
  and mode; a diff view with a file list and changes
  ([quickstart](https://code.claude.com/docs/en/web-quickstart)).
  Mobile is a client only; its layout is unverified.
- **Outcome:** a cumulative diff against the base branch, with inline
  line comments bundled into the next message. A live preview is
  documented for the desktop app, not for cloud sessions (unverified).
- **Steering:** mid-run messages queue and can be pulled back; modes
  Auto, Accept edits and Plan. Rewind in cloud sessions is unverified.
- **Many at once:** one session and branch per task. Projects (beta)
  group threads as Ready for review, Waiting on you, Working, Idle and
  Resolved, with a dot for waiting; a permission prompt must be
  answered inside its thread
  ([projects](https://code.claude.com/docs/en/claude-projects)).
- **Intent:** a project holds an optional one-line Goal and
  instructions on "how a thread checks its own work".
- **Hand-off:** Create PR; Auto-fix on CI failures and review
  comments; teleport the session to the terminal; shared views are
  read-only snapshots. No multiplayer.

## Synthesis for ParleyInn

| Pattern                                  | T3 Code                                 | Amp orbs                  | Claude Code web             | ParleyInn                                        |
| ---------------------------------------- | --------------------------------------- | ------------------------- | --------------------------- | ------------------------------------------------ |
| Sessions left, chat centre, review right | yes                                     | yes                       | yes                         | intents left, room centre, work right            |
| Live app preview                         | embedded, annotatable                   | portals, annotatable      | desktop only                | per agent's dev server                           |
| Follow an agent's edits live             | browser cursor, devices                 | shared terminal           | per-file diffs as it edits  | follow any player in the file view               |
| Several agents in one conversation       | no; a thread each                       | no; agents message        | no; a session each          | a room per intent                                |
| Several people in one conversation       | no                                      | yes, server-hosted        | read-only snapshots         | next, peer to peer                               |
| Needs-you signal                         | priority pills, inbox                   | notifications, feed       | Waiting on you bucket       | one ranked queue across intents                  |
| Goal stated up front                     | plan mode                               | none found                | project Goal                | intent with criteria                             |
| Agent presents the outcome per criterion | no; a plan, then a summary              | no                        | pasted summary (prose)      | agent links results to criteria with UI tools    |
| Evidence: recorded run against claim     | exit codes in a log                     | Ship runs tests, unshown  | pasted test summary (claim) | each criterion shows recorded run or claim       |
| Link to a marked passage or image region | quote chips; annotate a preview element | annotate a portal element | inline diff comments        | a link to any marked text, lines or image region |

**Table stakes:** the three-pane layout, background runs watched from a
phone, a ship or pull-request action, mid-run messages, a needs-you
signal, and a live preview with element annotation (two of three).

**Worth borrowing:**

- T3 Code's inbox that re-sorts by when a thread came back to you,
  with priority-ordered states.
- Annotating an element in the preview, or a line in the diff, so the
  feedback carries its anchor.
- Per-turn diffs tied to checkpoints.
- Amp's Ship as a configurable project prompt.

**Where none of them goes:**

- Tools for the agent to present its outcome against each criterion of
  a stated intent, pointing at the preview, the diff and the run, for
  a person to judge.
- A result linked to the recorded run that verified it.
- One queue ranked across many agents.
- A link to exactly what someone marked, text, lines or a region of an
  image, that people and agents can pass around the discussion. Pull
  requests link lines; none of the three links a region of an image.
- A room where several agents and, later, several people share one
  conversation without one agent instructing another.
