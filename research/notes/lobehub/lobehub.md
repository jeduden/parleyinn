# LobeHub: agent workspace and control surface

Date: 20 October 2026. Sources: LobeHub's repository and documentation on its
`canary` branch, read on this date. This is a documentation review, not a source
audit or a deployed-product test. A guarantee not found in these sources is
unknown, not absent. Availability may vary by release, edition and device.

## Where it overlaps

LobeHub presents itself as a place to organize agents, scheduled work and
reports. A person chooses an agent and opens a **Topic** for a conversation,
uses a **Task** for follow-up work, and can save material in Pages or Agent
Documents. Agent Groups coordinate several agents; a Workspace lets a team
collaborate. Projects are marked Alpha. The product also has a browser, desktop
and mobile experience, with conversations and memory synchronized across a
user's devices. Sources:
<https://github.com/lobehub/lobehub>,
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/start.mdx>,
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/feature-status.mdx>.

LobeHub can connect external coding agents, including Claude Code and Codex,
through its desktop app or a connected execution device. Its documentation
says the external agent keeps its own tool choice, permissions, memory, session
state and working directory; LobeHub shows the conversation and, depending on
the integration, tool calls, status, diffs, plans and approval requests. A
visible Topic does not guarantee that the external runtime session or project
files moved with it to another device or directory. Sources:
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/agent/connect-external-agents.mdx>,
<https://github.com/lobehub/lobehub/discussions/13927>.

This is real overlap with ParleyInn's planned fleet and room view: people can
organize agents, watch work and keep a conversation around it. LobeHub's
documented organizing unit is an agent and its Topic, with Tasks, Groups and
Projects around it. ParleyInn's room is organized around one intent and its
conversation, seats, pins, linked branches and linked forge issues. That
difference matters when several agents, people and branches serve the same
intent. Sources: [ParleyInn intent](../../../docs/srs/01-introduction.md),
[ParleyInn room](../../../docs/domain-model/places.md),
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/start.mdx>.

## Issue and pull-request discussion

LobeHub documents a GitHub connector that can read issues in an agent
conversation. Its external-agent guide says available progress and diff views
depend on the integration. The sources reviewed do not establish a room-like
link from one work intent to an issue and pull request, automatic import of
their discussion into one conversation, or an exact reply written there and
sent back to a forge issue comment or pull-request review thread. These are
open comparison questions rather than proven product gaps. Sources:
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/start.mdx>,
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/agent/connect-external-agents.mdx>.

ParleyInn specifies that stronger conversation path for M9: issue and review items
are shown as untrusted forge content in the room only after the credential
owner grants sharing of the exact issue or branch, and a person's separate act
can send their room response to the forge. The issue does not set the room's
intent; the pull request remains a forge review object for a branch. This is
specified, not shipped. Source: [LANE-08](../../../docs/srs/05b-lane-requirements.md) and [LANE-34](../../../docs/srs/05b-room-collaboration-requirements.md).

## Memory, record and trust

LobeHub describes Personal Memory as structured and editable, and its CLI
documents categories such as identity, activity, context, experience and
preference, plus extraction from chat history. It also offers Agent Documents
and a Resource Library for material an agent can use. For a connected external
agent, the runtime's own memory and session state remain separate from the
LobeHub Topic. Sources:
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/start.mdx>,
<https://github.com/lobehub/lobehub/blob/canary/.agents/skills/cli/references/memory.md>,
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/agent/connect-external-agents.mdx>.

Those sources do not establish ParleyInn's I1/I3/I10 contract: every observed
event recoverable by stable address, qualifying pins restored verbatim after
compaction, and derived room state reproducible from writer logs. Nor do they
establish I2's closed paths for untrusted material entering an agent. LobeHub
documents connector permissions and says a connected agent's native commands
follow that agent's own permission system; a working directory is not a
sandbox. This is a difference in stated contracts, not a claim that LobeHub
has no security controls. Sources:
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/agent/connect-external-agents.mdx>,
[ParleyInn invariants](../../../docs/srs/invariants.md).

## Network and deployment

LobeHub can be self-hosted. Its self-hosting architecture uses a Next.js
application and PostgreSQL, with optional Redis and S3-compatible storage;
agents may use connected devices and cloud services. ParleyInn instead specifies
a socket-free core, optional peer, publish and bridge components, and no central
or third-party service dependency. Self-hosting is a useful LobeHub deployment
choice, but it is a different network boundary from ParleyInn's I4. Sources:
<https://github.com/lobehub/lobehub/blob/canary/docs/self-hosting/start.mdx>,
<https://github.com/lobehub/lobehub/blob/canary/docs/usage/feature-status.mdx>,
[ParleyInn invariants](../../../docs/srs/invariants.md).

## Competitive reading

LobeHub is a direct competitor for the **agent workspace and control surface**:
multi-agent coordination, connected coding agents, cross-device access,
memory, tasks and shared work. Its documented path does not yet answer the
more specific ParleyInn question: can one conversation follow an intent across
several agents and branches, import the issue and review discussion around it,
and let a person reply from that conversation? That is the next workflow to
test, not a documented absence. ParleyInn's record, trust and network guarantees
also remain specification claims until they ship and are tested.
