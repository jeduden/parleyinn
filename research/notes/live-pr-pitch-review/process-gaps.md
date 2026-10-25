# Process gaps in the "live PR" pitch

Lens: a pull request is a process, not a UI. It is a gate, a permission
boundary, an audit artefact and a hand-off point. The pitch describes the
UI of a PR (chat, live edits, multiplayer, rich results) and the security
of the record. It says almost nothing about the process that a PR exists
to run. Evidence comes only from `research/` and `docs/srs/`. Paths are
relative to the repository root.

## Summary ranking

| #   | Gap                                                                                  | Severity  |
| --- | ------------------------------------------------------------------------------------ | --------- |
| 1   | No merge gate: approval, code owners, required checks, branch protection             | Crucial   |
| 2   | Owner-only trust breaks the review loop, and there is no permission model for a lane | Crucial   |
| 3   | No identity model for agents versus humans versus "the owner"                        | Crucial   |
| 4   | The landing path (squash, rebase, merge queue, conflicts, stacks) rewrites the code  | Crucial   |
| 5   | Coexistence with the forge where review and bots already live                        | Crucial   |
| 6   | Audit and compliance: legal hold, retention, who may shred a shared lane             | Important |
| 7   | Hand-off and ownership transfer; abandoning and forking lanes                        | Important |
| 8   | Attention management across a fleet of lanes                                         | Important |
| 9   | Cost and quota control when many people can wake many agents                         | Important |
| 10  | "One click away" for whom, and for how long, after merge                             | Important |
| 11  | Provenance of "interactive results": agent-asserted versus canonical CI              | Important |
| 12  | Open-source onboarding and contributor policy                                        | Important |
| 13  | Issue and task linkage; duplicate work across lanes                                  | Minor     |
| 14  | Offline and asynchronous work across time zones                                      | Minor     |
| 15  | Deployment and rollback after merge                                                  | Minor     |

## What the pitch already handles (dropped)

- **Prompt injection through collaboration.** "A message from anyone
  except the lane's owner reaches an agent as untrusted, and nothing
  stored is injected without being asked for." That is I2 applied to
  chat, and it matches the research recommendation for the chat view
  (`research/reports/custom-storage-git-and-chat-interfaces.md`, "The
  chat view"). Not a gap as a security claim. It does create gap 2.
- **Integrity of the audit trail.** "The lane's record is tamper-evident"
  covers tamper evidence. Gap 6 is about the rest of compliance.
- **Erasure as a mechanism.** "Sensitive content can be erased by key"
  covers the mechanism. Gap 6 covers who may use it and when it must be
  refused.

## Crucial gaps

### 1. No merge gate

The pitch: "When the lane is done, the code lands in git as an ordinary
commit." It never says who decides the lane is done. In a PR, that
decision is the product: required reviewers, code owners, required status
checks, branch protection, and separation of duties between author and
approver. A "live PR" without a gate is a shared editor with a commit
button.

Why it is crucial: every agent platform in the research kept or
strengthened the gate when agents arrived. The one product that removed
it shows what removal means.

- GitHub's Copilot cloud agent: "Draft PRs need a human to merge. The
  requester cannot approve. Actions workflows wait for 'Approve and run
  workflows'. An unattributed agent PR needs one extra approval." It
  "stays subject to branch protections"
  (`research/notes/agent-session-storage-sweep/sheets.json`, Copilot cloud
  agent; <https://docs.github.com/en/copilot/concepts/agents/cloud-agent/risks-and-mitigations>).
- GitLab Duo attributes agent MRs to the triggering human "for
  segregation-of-duties compliance", and recommends CODEOWNERS and branch
  protection on the agent config file
  (`sheets.json`, GitLab Duo Agent Platform;
  <https://docs.gitlab.com/user/duo_agent_platform/security>).
- Sourcegraph Agentic Batch Changes: "The agent cannot merge; a human
  merges," and "branch protection and required checks still apply"
  (`sheets.json`; <https://sourcegraph.com/docs/agentic-batch-changes/configuration>).
- Zed Delta, the closest "replace the PR" product, turned off pull
  requests on its own repository: "33 team members landed 570 changes to
  main since we turned off pull requests." Landing and CI stay in git;
  "submitting a verdict does not merge"
  (`research/notes/git-alternatives-for-agent-sessions/agent-vcs.md`;
  <https://zed.dev/blog/delta-public-beta>;
  <https://delta.dev/docs/agents/review-and-sync>).
- Block Buzz is the only design that puts the gate inside the room: a
  "review approval" is a signed event, and "the merge decision lands in
  the same room as the evidence"
  (`research/notes/custom-storage-git-and-chat/integrated-platforms.md`;
  <https://github.com/block/buzz>).
- ParleyInn's own SRS demands this gate for ParleyInn itself. ENG-21 requires
  approval by a reviewer other than the author, CODEOWNERS approval for
  requirement and gate changes, two approvals for security-sensitive
  packages, and labelled AI changes. ENG-28 requires an agent approval to
  be posted by an identity that authors no change, on the head reviewed,
  once CI passed there (`docs/srs/10-engineering-quality.md`). The pitch
  sells a process weaker than the one its authors use.

What the pitch would need: a gate model (who may approve, on which head,
after which checks) and the statement that the forge's branch protection
stays authoritative.

### 2. Owner-only trust breaks the review loop; no permission model

The pitch's one access rule is "a message from anyone except the lane's
owner reaches an agent as untrusted." Under I2 untrusted content reaches
the model only through pull-only, enveloped recall
(`docs/srs/01-introduction.md`). So a reviewer who writes "rename this and
add a test" cannot steer the agent. The agent sees it only if it chooses
to recall it, and then only as untrusted data. The core PR loop (review,
fix, re-review) runs only through the owner relaying every comment. That
contradicts "several agents and people work one lane together."

Why it is crucial: the shipped systems either draw the trust line at a
real permission, or forbid non-owners from writing at all. None has the
pitch's "everyone can talk, only one person counts" model.

- Copilot draws the line at write access: "comments from users without
  write access are never presented to the agent." Users with write access
  are heard (`sheets.json`, Copilot cloud agent).
- Coder Agents: one owner per chat, "no owner override," and sharing is
  read-only: viewers "cannot send or edit messages"
  (`sheets.json`; <https://github.com/coder/coder/blob/main/docs/ai-coder/agents/chat-sharing.md>).
- Sourcegraph ABC: "only the owner can send messages, approve actions,
  and publish"; others get a read link (`sheets.json`).
- Zed Delta: sharing a thread "grants repository-wide access to every
  Delta worktree history of the attached repos"; modes are invite,
  organization and "anyone with the link." Delta "has no agent permission
  system" (`sheets.json`; <https://delta.dev/docs/privacy-and-security/data-storage>).
- A Slack employee in the Buzz thread warns that "multiplayer agents" leak
  data across permission boundaries, and teams "end up having to write and
  maintain complex rulesets"
  (`research/notes/git-alternatives-for-agent-sessions/hacker-news.md`;
  <https://news.ycombinator.com/item?id=48996051>).
- Claude Code agent teams: teammates inherit the lead's permission mode,
  and plan approvals are auto-granted by the lead
  (`sheets.json`; <https://code.claude.com/docs/en/agent-teams>).

The pitch also names no rule for who can join a lane, see it, or message
its agents. Invariant tension: a lane holding several humans' content is
a multi-tenant object. I8 binds all state to one tenant's home and
refuses state it does not own. NG4 puts multi-host shared stores out of
v1, and OQ-07 defers "per-tenant row security, provenance-gated sharing"
(`docs/srs/01-introduction.md`; `docs/srs/13-open-questions-and-risks.md`).

What the pitch would need: lane roles (owner, maintainer, reviewer,
viewer) bound to forge permissions. It would also need an explicit,
audited act by which the owner adopts a reviewer's request as an
instruction. That keeps I2, because the trust comes from the owner's
act, not from the content.

### 3. No identity model for agents versus humans

"The lane's owner" is the pitch's only security boundary. It presupposes
authenticated identities for humans and agents, spanning machines. The
pitch does not say what an identity is, how an agent's identity binds to
a human, or who authors the commit that lands.

- Buzz gives every agent its own keypair "plus a second signature binding
  it to its human owner"
  (`research/reports/agent-session-storage-beyond-git.md`;
  <https://siliconangle.com/2026/07/21/block-launches-buzz-open-source-workspace-humans-ai-agents/>).
- GitLab Duo writes as a composite identity, a service account plus the
  human, with the intersection of their permissions (`sheets.json`).
- Copilot commits are authored by Copilot, co-authored by the requester,
  and signed (`sheets.json`).
- Goose retired Nostr session sharing within about 4.5 months because
  "imports could not be authenticated and bearer keys in URLs could not
  be revoked" (`research/reports/agent-session-storage-beyond-git.md`;
  <https://www.mankier.com/1/goose-session-export>).
- Attribution of the landed commit is contested. Open-source policy
  settled on a tool-level `Assisted-by` trailer naming no session
  (<https://docs.kernel.org/process/coding-assistants.html>). HN objects
  that session links mislead when humans edited the code, and argues
  about copyright in unlabelled AI commits
  (`hacker-news.md`; <https://news.ycombinator.com/item?id=49498349>;
  <https://news.ycombinator.com/item?id=49498379>).

What the pitch would need: per-agent and per-human keys, the binding
between them, device revocation, and a stated authorship rule for the
landed commit.

### 4. The landing path rewrites the code the record describes

The tagline is "the record never drifts from the code." But code reaches
main through squash, rebase or a merge queue. Each produces a commit no
lane participant ever saw. Parallel lanes conflict. Dependent lanes need
stacking. The pitch covers none of this, and this is exactly where
session-to-code links break in the field.

- Squash merges drop trailers and sever commit-to-session links
  (`research/notes/version-control-beyond-git/migration-and-debate.md`;
  <https://julien.danjou.info/blog/how-entire-works-under-the-hood/>).
  Only 3 of 24 Copilot agent commits carried their trailer. A squash
  built from the PR body drops the trailer and keeps the raw prompt
  (`sheets.json`, Copilot cloud agent;
  <https://github.com/dotnet/runtime/commit/1c7b5c49466a1a2ca6a3c2bba12dbfe80159fab4>).
- Merge queues are load-bearing at agent scale. A GitHub merge-queue
  regression hit 658 repositories and 2,092 PRs on 23 April 2026
  (`research/notes/version-control-beyond-git/scale-departures.md`;
  <https://github.blog/news-insights/company-news/an-update-on-github-availability/>).
  Gas Town runs a "Bors-style bisecting merge queue" (the Refinery) for
  its agents (`research/notes/git-as-agent-session-storage/beads-gastown.md`).
- Cursor's Origin is fronted by the Graphite team. Stacked PRs and
  agent-aware merge queues were demoed, but they are "not supported" in
  the August beta (`research/notes/git-alternatives-for-agent-sessions/cursor.md`;
  <https://appwrite.io/blog/post/cursor-origin-review-an-engineers-perspective>).
  Devin's coordinator merges through stacked PRs (`sheets.json`).
- Agents handle conflicts badly. "The agent just kept one or the other
  changes instead of merging it intelligently." "My agent rebased and
  forcepushed with conflicts." Cursor's agent "force-pushed despite
  explicit 'ask for permission' rules"
  (`hacker-news.md`; <https://news.ycombinator.com/item?id=46469974>;
  <https://news.ycombinator.com/item?id=48065840>;
  <https://news.ycombinator.com/item?id=46728766>).
- The research's own rule: derive session-to-commit links from evidence
  the record holds, and abstain with a typed reason when proof is
  incomplete (rule 11 in `research/reports/agent-session-storage-beyond-git.md`;
  lesson 9 in `research/notes/agent-session-storage-sweep/catalog.md`).

What the pitch would need: how a lane maps to the landed SHA after squash,
rebase or queue, and what the record shows when the link is unproven
(I6). It would also need how cross-lane conflicts and lane stacks are
represented, and who resolves them.

### 5. Coexistence with the forge where review and bots already live

"Code lands in git" implies the forge stays. Branch protection, required
reviews, CI, Copilot code review and the Claude Code GitHub Action all
live on the forge's PR. If the team keeps the forge PR (and gap 1 says it
must), there are two conversation surfaces: lane chat and forge review
comments. "Code and talk can never drift apart" fails the day a reviewer
comments on GitHub.

- Origin mirrors GitHub with "GitHub as the source of truth." It syncs
  PR comments both ways, but not Issues, Actions workflows or secrets
  (`cursor.md`; <https://cursor.com/docs/origin/mirror-github>).
- Forgejo's link to Matrix is only a webhook
  (`integrated-platforms.md`). GitHub webhooks must answer within 10
  seconds and failed deliveries are not retried
  (`research/reports/agent-session-storage-beyond-git.md`;
  <https://docs.github.com/en/webhooks/using-webhooks/handling-failed-webhook-deliveries>).
  A live bridge is lossy by default.
- "Nobody with real users has removed git from the host/CI boundary"
  (`migration-and-debate.md`). HN compares leaving the ecosystem to
  "replacing WordPress" (<https://news.ycombinator.com/item?id=48631726>).
- In Cursor's Slack pane, "every message spawns a new cloud agent, with
  no way to follow up on an existing one"
  (`research/reports/custom-storage-git-and-chat-interfaces.md`;
  <https://forum.cursor.com/t/slack-assistant-pane-every-message-spawns-a-new-cloud-agent-with-no-way-to-follow-up-on-an-existing-one/165947>).
  Bridging chat surfaces is where continuity breaks.
- Invariant tension: pulling forge comments into the lane needs network
  access (I4 forbids it in the core). Every imported comment is foreign,
  untrusted content (I2). The research puts any bridge in a separate,
  opt-in process whose copies cannot be shredded
  (`research/reports/custom-storage-git-and-chat-interfaces.md`, "Copies
  that leave custody").

What the pitch would need: a statement of which surface is authoritative
for review, and how forge comments and bot output enter the lane (as
foreign, untrusted, audited imports).

## Important gaps

### 6. Audit and compliance: legal hold, retention, shredding in a shared lane

"Erasable by key" without a hold check is a compliance defect. In a
shared lane it is also unclear whose key covers which content.

- GDPR Art. 17(3) exempts retention for legal claims. The research infers
  that "a legal hold must be able to block a shred, and the block must be
  audited." It also records that "legal holds were not researched"
  (`research/reports/custom-storage-git-and-chat-interfaces.md`).
- The EDPB (v2.0, 7 July 2026) does not accept key deletion alone as
  erasure (same report;
  <https://www.edpb.europa.eu/system/files/2026-07/edpb_guidelines_202502_blockchain_v2_en.pdf>).
- Subject keys for third parties' data are proposed, but "how subjects
  are recognised at capture is unsolved" (same report). A lane holds
  many people's words by design.
- Precedents set expectations a buyer will bring. The Anthropic
  Compliance API keeps activity for 6 years, uses scoped keys, and audits
  every API call as an activity
  (`sheets.json`; <https://platform.claude.com/docs/en/manage-claude/compliance-api>).
  GitLab Duo deletes sessions 30 days after last activity, so "the
  reasoning behind a merged change is usually gone long before the code
  is" (`sheets.json`). Copilot cloud sessions "can be archived but not
  deleted" (`sheets.json`).
- Atlassian says personal data in git history "is essential for auditing
  and providing a chain of license contribution and authorship"
  (`research/notes/git-as-agent-session-storage/git-limits-realtime.md`;
  <https://confluence.atlassian.com/bitbucketserver078/right-to-erasure-inbitbucket-server-and-data-center-1037995074.html>).
  Shredding a merged lane collides with that audit purpose.
- The SRS has a per-provenance retention knob, REC-15, whose default
  keeps everything (`docs/srs/05-functional-requirements.md`). It has no
  legal-hold requirement.

### 7. Hand-off, ownership transfer, abandoning and forking lanes

An owner-centred trust model makes the owner a single point of failure.
The pitch says nothing about transferring a lane, handing it between
agents, abandoning it, or forking it.

- Sourcegraph ABC: "There is no co-ownership and no transfer; if the
  owner is unavailable, someone starts a new agentic batch change"
  (`sheets.json`).
- Devin's `/handoff` "summarizes the current conversation", so the cloud
  session "starts from a summary, not the transcript". Moving machines
  "creates a NEW session rather than syncing a log" (`sheets.json`;
  <https://docs.devin.ai/cli/handoff.md>). That hand-off is lossy, the
  opposite of I1.
- Claude Code `--teleport` loads the history, and then the terminal copy
  diverges: "new work there stays local" (`sheets.json`;
  <https://code.claude.com/docs/en/claude-code-on-the-web>).
- The research rule for forks: deterministic fork IDs, never merged; a
  second successor from one origin is audited, never resolved silently
  (rule 3 in `research/reports/agent-session-storage-beyond-git.md`;
  lesson 15 in `catalog.md`). Agor models divergence as fork and spawn
  trees, one branch per session (`sheets.json`).
- Buzz: after merge "the channel becomes an archived record of why the
  change exists" (`integrated-platforms.md`). The pitch has no lane
  lifecycle states (open, blocked, abandoned, merged, archived).

### 8. Attention management across a fleet of lanes

"Watch a whole fleet of lanes at once" is a monitoring pitch, not an
attention model. Real time multiplies events. The scarce resource is a
human's notice of the lane that needs a decision.

- The Appwrite review of Origin found agent-created PRs "not immediately
  visible" because the PR list defaults to the user's own (`cursor.md`).
- Sourcegraph added `trailerOnly` mode "to cut down on notification
  noise" (`sheets.json`).
- Gas Town needed dedicated watchdog agents. A Witness runs per rig "to
  detect stuck polecats, triggering nudges or escalations", and a Deacon
  patrols every ~3 minutes (`sheets.json`, Gas Town).
- Notifications and webhooks were among the subsystems that buckled at
  GitHub under agent load (`scale-departures.md`). The research rule for
  the relay is that "notifications are hints" and a receiver re-reads the
  durable log (`research/reports/agent-session-storage-beyond-git.md`).
- I6 makes this a contract matter. A lane stuck waiting on an approval
  nobody saw is a silent failure from the operator's view.

### 9. Cost and quota control

Multiplayer means many people can wake agents that someone pays for. The
pitch is silent on who pays, how spend is capped, and whether a
non-owner's message can trigger a paid turn.

- Delta: a send "submits all trailing drafts from every author in one
  agent turn, charged to the sender" (`sheets.json`).
- Gas Town's `scheduler.max_polecats` "gates concurrent agent spawn to
  prevent API rate limit exhaustion; default is immediate dispatch"
  (`sheets.json`). The HN thread "Does Gas Town 'steal' usage from users'
  LLM credits to improve itself?" drew 253 points
  (`hacker-news.md`; <https://news.ycombinator.com/item?id=47785053>).
- Vendors meter per session. Devin records `acus_consumed`; Copilot
  records premium requests and caps a session at 59 minutes; Claude Code
  cloud sessions "share the account's rate limits" (`sheets.json`).
- If an untrusted message can wake an agent, it is a cost
  denial-of-service path even though I2 keeps it from steering the model.

### 10. "One click away" for whom, and for how long

"The whole story stays one click away" needs a host for that click. I4
keeps the core local, so a reviewer on another machine, or an auditor in
two years, has no record to open unless a carrier delivered it.

- Vendor death is a documented failure. Terragon's record died with its
  service; Warp shares expire after about a week; GitLab deletes after 30
  days (lesson 11 in `catalog.md`).
- HN: "Do any of us truly believe these links will work 30 years from
  now?… Git is supposed to be the durable storage medium"
  (`research/reports/agent-session-storage-beyond-git.md`;
  <https://news.ycombinator.com/item?id=49499194>).
- Cursor Blame fetches attribution from Cursor's servers on demand
  (`cursor.md`). This is the hosted pattern the pitch would need to avoid
  or adopt explicitly.
- The research's answer is sealed, encrypted segments carried by the
  user's git or a relay outside the binary. That is a separate product
  with its own review (`research/reports/agent-session-storage-beyond-git.md`).
  The pitch presents the click as given.

### 11. Provenance of "interactive results"

"Test results, benchmarks, diffs and rendered output arrive as live
views." Results produced in the agent's worktree are agent-asserted.
Results from the forge's CI are canonical but need network access to
fetch. The pitch does not distinguish them, and reviewers will treat a
green live view as a passed check.

- The Compliance API tags every message as verified, `client_asserted`
  or `synthetic_marker` (`sheets.json`). That is the precedent for
  labelling a result's origin.
- GitHub Agentic Workflows keeps the agent job read-only. Its write
  requests are buffered and checked by a separate detection job before
  scoped jobs apply them (`sheets.json`;
  <https://github.github.com/gh-aw/introduction/architecture/>).
- Delta shows "thread CI indicators" that follow commits pushed from the
  thread: CI stays external (`sheets.json`). Coder polls the forge API
  with the owner's OAuth token on a 120 s TTL to learn PR and check state
  (`sheets.json`). Both use network access, which I4 keeps out of the
  core.
- jj "runs no git hooks (pre-commit checks silently skipped)"
  (`migration-and-debate.md`). A new layer can silently bypass checks,
  and I6 forbids that.

### 12. Open-source onboarding and contributor policy

Open-source projects get a drive-by contributor's PR on a forge. They do
not get a lane. Their policies also lean against session links.

- Kernel, Fedora and LLVM settled on a tool-level `Assisted-by` trailer
  "with no session or prompt", and kernel maintainers strip trailers
  (`catalog.md`; <https://docs.kernel.org/process/coding-assistants.html>).
- Ghostty's "AI tooling must be disclosed" PR drew 729 points
  (`hacker-news.md`; <https://github.com/ghostty-org/ghostty/pull/8289>).
- The privacy camp: "I don't want my thoughts to be serialized, version
  controlled and publicly accessible" (87 replies), and DeltaDB "gives
  micro-managers the data they need to micro-manage you"
  (`hacker-news.md`; <https://news.ycombinator.com/item?id=48494076>;
  <https://news.ycombinator.com/item?id=49188128>).
- Origin repositories are private to a team; there are no public repos
  (`cursor.md`). The research rule: publish to open source "as
  projections, not raw segments" (rule 12 in
  `research/reports/agent-session-storage-beyond-git.md`).
- The pitch has no story for a contributor who does not run ParleyInn,
  or for how a maintainer finds and joins a public lane.

## Minor gaps

### 13. Issue and task linkage

Lanes start from issues. Without linkage, a fleet duplicates work.

- Kiro Web: when several users assign the same GitHub issue, "Kiro
  creates a separate task and sandbox for each, and the docs tell teams
  to coordinate by hand" (`sheets.json`; <https://kiro.dev/docs/web/github.md>).
- Agor recommends "1 branch = 1 issue = 1 PR" (`sheets.json`). Gas Town
  tracks work as beads apart from code, so work can be claimed
  independently of branches (`sheets.json`).

### 14. Offline and asynchronous work

The pitch is built around real time. Teams across time zones work
asynchronously, and sandboxes get killed.

- Delta keeps unsynced changes as an explicit "(local changes)" copy and
  never auto-merges (`sheets.json`). That is the I6-compatible pattern.
- Origin keeps a `/local` write path while GitHub is down, and does not
  say how the two copies reconcile afterwards (`cursor.md`).
- "Where to flush the record when a cloud sandbox is killed" is listed as
  an open angle (critic angle 17 in `catalog.md`).
- HN: "agentic development is all about throughput, not latency"
  (`migration-and-debate.md`; <https://news.ycombinator.com/item?id=48631726>).
  Combined with gap 7, an owner asleep in another time zone stalls every
  lane they own.

### 15. Deployment and rollback after merge

The research barely covers this, which is itself a finding. The pitch
ends at "lands in git". The HN use case it most needs to win is
debugging an old commit: "a lifesaver when you are trying to debug an old
commit" (`research/reports/agent-session-storage-beyond-git.md`;
<https://news.ycombinator.com/item?id=49498363>). That means tracing an
incident or a revert back to the lane, and opening a lane for the
revert. Delta's "Revert Conversation to Cursor" and Kiro Crew's
`git reset --hard HEAD~1` on failed review are in-lane rewinds, not
post-merge rollback (`sheets.json`). No evidence covers deployment.

## Evidence the research lacks

- Legal holds: explicitly not researched
  (`research/reports/custom-storage-git-and-chat-interfaces.md`).
- How branch protection and rulesets treat custom refs such as
  `refs/parinn/*`: "No source covers" it
  (`research/reports/git-as-agent-session-storage.md`). agentdiff notes
  that custom refs escape branch protection
  (`research/reports/interesting-ideas-digest.md`).
- Merge queues, stacked PRs, required checks and CODEOWNERS behaviour for
  agent-heavy repositories: only incidental mentions.
- Deployment and rollback: none.
