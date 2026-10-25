# Warp: shared agent sessions and source-centered factory work

Date: 20 October 2026. Sources: Warp's public repository and current product
documentation, read on this date. This is a documentation review, not a source
audit or deployed-product test. Warp labels Factories early access; the
availability of a documented feature may depend on access, plan or deployment.
A guarantee not found in these sources is unknown, not absent.

## Where it overlaps

Warp is an agentic development environment with its own coding agent and
support for third-party CLI agents such as Claude Code and Codex. Its public
repository contains the client codebase; the README says the UI framework is
MIT-licensed and the rest of that repository is AGPLv3. Its Factories are
separately presented as teams of cloud agents that triage, plan, implement,
review and hand work to people. They can use different models and harnesses,
keep one work item through those stages, and show runs in a dashboard and
inbox. Sources: <https://github.com/warpdotdev/warp>,
<https://docs.warp.dev/factories/how-factories-work/>.

Warp shares local or cloud agent sessions so people can inspect prompts,
commands, logs, output and follow-up context. Shared sessions and third-party
agent remote control allow permitted teammates to observe or steer work. Its
Code Review panel compares branches or uncommitted changes, and Interactive
Code Review lets a person batch comments on lines and send them to the native
agent or a supported CLI agent. This is meaningful overlap with ParleyInn's
planned live room and review experience, including a way to respond to code
feedback inside Warp before a pull request. Sources:
<https://docs.warp.dev/agents/>,
<https://docs.warp.dev/agents/cli-agents/remote-control>,
<https://docs.warp.dev/code/code-review/>,
<https://docs.warp.dev/agents/local-agents/interactive-code-review/>.

## Issues, pull requests and the conversation

A Factory can receive work from a GitHub issue, pull request, submitted review
or CI event. Its GitHub integration posts progress and links back to the
originating issue, pull request or review thread; new activity on that thread
continues the same work item. A submitted review on a labeled pull request is
a default trigger. The Factory can open a branch and pull request and later
complete linked tracker work on merge. Thus Warp already reacts to external
review feedback and preserves a work item's identity across multiple agent
stages. Sources: <https://docs.warp.dev/factories/integrations/github/>,
<https://docs.warp.dev/factories/connect-your-factory/>,
<https://docs.warp.dev/factories/how-factories-work/>.

For human PR review, Warp recommends placing a shared session or cloud run
link in the PR description or a comment. The reviewer uses that link to see
the agent's context alongside the GitHub diff, which remains the source of
truth for the merge. The documented Factory conversation continues in the
source issue, PR or review thread. The sources reviewed do not establish one
shared conversation that imports comments from a linked issue **and** linked
PR review threads, then lets a person answer either external comment from
that same conversation. That is a specific comparison question, not a claim
that Warp cannot do it. Sources:
<https://docs.warp.dev/guides/agent-workflows/how-to-attach-agent-session-context-to-github-prs/>,
<https://docs.warp.dev/factories/connect-your-factory/>,
<https://docs.warp.dev/factories/integrations/github/>.

ParleyInn's room is organized around an intent. Branches and forge issues link
to it, and a pull request is a forge artifact for a branch. ParleyInn specifies
that, after the read credential owner records an active forge import grant for
the exact linked issue or branch and room, issue comments and PR review items
appear in that room as untrusted forge content. A person may separately send
a room response back to the corresponding forge discussion.
That is an M9 requirement, not shipped behavior. Sources:
[ParleyInn room](../../../docs/domain-model/places.md),
[LANE-08](../../../docs/srs/05b-lane-requirements.md),
[LANE-34](../../../docs/srs/05b-room-collaboration-requirements.md),
[SEC-28](../../../docs/srs/06-security.md).

## Record, trust and network boundary

Warp's session and work item history supports inspection and collaboration.
The sources reviewed do not establish ParleyInn's I1/I3/I10 contract: events an
agent saw or produced remain recoverable by stable address subject to I1's
recorded redaction, purge and retention exceptions; qualifying pins restore
verbatim or are named by id and count when budget or run ambiguity omits them;
and derived room state rebuilds deterministically from writer logs. Factory
source context reaches agents as part of event-driven work. The reviewed
documentation does not establish ParleyInn's I2 closed paths: outside content
reaches an agent through explicit enveloped recall or the specified
principal-authorized paths. These are differences in stated contracts, not
claims that Warp lacks record or security controls. Sources:
<https://docs.warp.dev/factories/how-factories-work/>,
<https://docs.warp.dev/factories/connect-your-factory/>,
[ParleyInn invariants](../../../docs/srs/invariants.md).

Warp's shared sessions and Automation Platform use network services; the
platform offers hosted execution and managed self-hosted workers. ParleyInn's I4
instead specifies a socket-free core with optional peer and public components
that the principal turns on. This is a deployment and trust-boundary
difference, not a claim that self-hosted Warp work leaves the customer's
network. Sources: <https://docs.warp.dev/platform/overview>,
<https://docs.warp.dev/platform/>,
[ParleyInn invariants](../../../docs/srs/invariants.md).

## Competitive reading

Warp is a direct competitor for agent supervision, code review and work
following from issues through PRs. Its strongest lesson for ParleyInn is that
review comments, stage handoffs and merge status must remain connected to the
original work; a room view that only shows a linked PR card would fall short.
The documented Warp workflow routes conversation through the source thread and
shares agent context by link. ParleyInn's proposed difference is to let people
collaborate in one room around the intent, see both issue and PR discussion
there, and answer either from that room under explicit sharing authority.
This distinction will be credible only after the M7 room and M9 forge bridge
ship and are tested in a real review cycle.
