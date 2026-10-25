# Multica: issue-centered agent workspace

Date: 20 October 2026. Primary sources: the Multica repository and its own
documentation, read on this date. This is a product and documentation review,
not a source-code audit or a live deployment test. A missing documented
guarantee is recorded as unknown unless a documented path contradicts it.

## What Multica ships

Multica's unit of work is an issue. Its description, discussion, status,
assignee, run history and linked pull requests stay together. Assigning an
agent or squad creates a run; a connected daemon invokes a local agent CLI and
sends progress back to the issue. The web, desktop and mobile clients show the
workspace; self-hosting uses a server and database. Supported CLIs include
Claude Code, Codex and Cursor. Sources:
<https://github.com/multica-ai/multica>,
<https://multica.ai/docs/issues>,
<https://multica.ai/docs/how-multica-works>.

The run view streams messages, tool calls and errors, and retains earlier run
records when an issue has several runs. People can steer a running agent.
Squads wake a leader first; the leader can mention members to create further
runs. Git worktree mode lets concurrent runs use separate checkouts. Pull
requests can link to an issue by id in the branch, title or body, and the issue
then shows the pull request's state, diff size, CI state and mergeability.
Sources: <https://multica.ai/docs/tasks>,
<https://multica.ai/docs/squads>,
<https://multica.ai/docs/project-resources>,
<https://multica.ai/docs/github-integration>.

The issue centralizes discussions written **in Multica**: members and agents
post comments and nested replies there, can resolve a discussion, and see run
results in the issue's activity and execution log. Linking a pull request adds
a card with its state, change size, CI and mergeability, which opens the pull
request on the forge. The documented GitHub integration subscribes to pull
request and check/status events, not review-comment events; it does not
document mirroring pull request review threads into the issue. Discussion on
the pull request therefore remains on the forge unless someone brings it into
the Multica issue. Submitting a pull request review is not a documented trigger
for a new run. A new top-level issue comment routes to the issue's agent
assignee; a reply to an agent's issue comment routes to that agent; and an
explicit mention selects an agent. A person can use any of these paths to ask
an agent to inspect the pull request review. Multica's tutorial instead shows
a reviewer agent posting findings as issue comments, then an engineer agent
responding there.
Sources: <https://multica.ai/docs/comments>,
<https://multica.ai/docs/issues>,
<https://multica.ai/docs/github-integration>,
<https://multica.ai/docs/mentioning-agents>,
<https://multica.ai/docs/tutorial>.

This is a stronger shipped assignment and review surface than ParleyInn currently
has. ParleyInn's room view is specified in M7, while the peer network and forge
review come later in M8 and M9. A ParleyInn room can hold an intent, conversation,
seats, pins and branches; it is not tied to an issue in a project.
Sources: [ParleyInn status](../../../README.md),
[milestones](../../../PLAN.md),
[domain model](../../../docs/domain-model/index.md).

## Record, context and trust

Multica's issue and run history helps a returning person reconstruct a piece
of work. Its documentation does not establish ParleyInn's stronger I1, I3 and I10
properties: stable addresses for every event, verbatim pin restoration after
compaction, or deterministic rebuilding of derived artifacts. The documented
transcript is a useful view; whether it captures every edit or is tamper-evident
remains unknown from the sources reviewed. Sources:
<https://multica.ai/docs/issues>,
<https://multica.ai/docs/tasks>,
[ParleyInn invariants](../../../docs/srs/invariants.md).

Multica adds workspace context to every agent in that workspace and project
description and resource data to runs in a project. Squad instructions are
appended to the leader's instructions, and an agent's mention can trigger a
member's run. Those documented paths do not offer ParleyInn's I2 rule that
untrusted stored content reaches a model only on explicit recall inside an
untrusted envelope, or through a principal act's closed path. This is a
comparison of documented behavior, not a claim that Multica lacks other access
controls. Sources: <https://multica.ai/docs/workspaces>,
<https://multica.ai/docs/projects>,
<https://multica.ai/docs/squads>,
[ParleyInn invariants](../../../docs/srs/invariants.md).

## Execution and network boundary

Multica's daemon invokes agent CLIs as child processes. By default each run
has the daemon OS user's file, credential and network access. Its security
documentation explicitly recommends a dedicated user, container or virtual
machine. It provides per-run working directories, run-scoped API tokens and
separate Codex state, but says these do not sandbox an agent process. Unattended
approval prompts are answered automatically on its default path. Source:
<https://multica.ai/docs/security-model>.

The server and daemon exchange run data over the network. In a self-hosted
deployment the API server sends a daily, aggregate deployment telemetry
snapshot by default; `DO_NOT_TRACK=1` disables it. This differs from ParleyInn's
I4 network-free core and no-telemetry contract. Multica can be self-hosted, but
that does not make its core socket-free or turn off telemetry by default.
Sources: <https://multica.ai/docs/how-multica-works>,
<https://github.com/multica-ai/multica/blob/main/SELF_HOSTING.md>,
[ParleyInn invariants](../../../docs/srs/invariants.md).

## Competitive reading

Multica is a direct competitor for the experience of assigning work, following
many agents, catching up on a run and handing its result to review. It is an
adjacent rather than exact match for ParleyInn's room: Multica's issue gathers
its own discussion and separate runs, and links to pull requests whose review
threads remain on the forge. ParleyInn aims to keep the full addressed record of
the work and the trust rules that govern recall. The existing
[pitch comparison](../live-pr-pitch-review/competitors.md) still places Zed
Delta closer to the live shared-branch claim. Multica puts more pressure on the
usefulness of ParleyInn's planned room view than on the novelty of its I2 and I4
contracts.

The review-thread split is also a product distinction, not just a missing
integration detail. ParleyInn's intended room keeps the conversation about an
intent with any branches it names and its results; a pull request, when used,
is the forge's review object for a branch rather than a second place to
continue that conversation. ParleyInn's M9 contract brings both linked issue
discussion and pull-request review items into the room, each only after
the forge read credential owner grants that exact issue or branch to the room, and lets a person
explicitly send a room response to the forge; the
[competitor review](../live-pr-pitch-review/competitors.md) records this
criterion.

The implementation gap is decisive for a user choosing software today:
Multica's assignment, run and review surfaces are available; ParleyInn's M7 to M9
surfaces are planned. ParleyInn can claim a differentiated contract, but the
comparison cannot establish a better delivered experience before those
milestones ship. Sources: <https://github.com/multica-ai/multica>,
[ParleyInn status](../../../README.md),
[milestones](../../../PLAN.md).

Multica is source-available under its own license. It incorporates Apache 2.0
with additional conditions covering third-party hosted services, commercial
embedding and branding. This matters if evaluating code reuse; it does not
change the product ranking. Source:
<https://github.com/multica-ai/multica/blob/main/LICENSE>.

## Open checks

- Whether the transcript captures every edit and can be checked for missing or
  changed events. The documentation reviewed does not establish this.
- Whether any pin or instruction survives harness compaction verbatim. The
  documentation reviewed does not establish this.
- Whether Multica offers any untrusted-content envelope or principal-controlled
  endorsement path outside the documented run context and squad flow. The
  documentation reviewed does not establish one.
- How much of the run transcript and linked code remains available after an
  issue or workspace is deleted. The docs say those deletions permanently
  remove linked records; they do not specify an independent forensic record.
  Source: <https://multica.ai/docs/issues>,
  <https://multica.ai/docs/workspaces>.
