---
id: 2610022338
title: "ParleyInn's lane experience and peer network: no central service"
status: "🔲"
summary: >-
  Builds the live lane — a pull request with chat, real time,
  multiplayer and result views — over a peer network that needs no
  central service. Every node is a full peer, self-hosting is normal,
  and a partitioned lane keeps working and merges on reconnect. The
  experience comes first; making sync seamless follows.
model: opus
depends-on: [2610012322]
---
# ParleyInn's lane experience and peer network: no central service

## Goal

Standalone comes first. One machine runs with no network at all.
Several harnesses work their lanes there, and one view shows them all.
It shows each agent's harness as it runs, its work results, and the
other harnesses.

Peer to peer extends that, opt-in. People and agents work a lane
together live, from any machine or sandbox, with no central service.
Like git, every node holds the whole lane and can serve it; a
self-hosted node is the normal case, not an upgrade. A lane split by
a network partition keeps working on every side and merges cleanly
when the sides meet again.

## Context

The stakeholder's decisions (3 October 2026):

1. ParleyInn must not depend on a central service. Self-hosting is normal,
   as it is for git.
2. Multiplayer must work without a central server, because networks
   partition. CRDTs and other conflict-resolution methods handle the
   merge.
3. Design the user experience first. Making it seamless is the second
   step. Security must be good enough, measured against Zed Delta,
   which syncs through a central Cloudflare backend.
4. There is a UI. It shows a harness (an agent session such as Claude
   Code, live: prompts, tool calls, permission requests, output), the
   work results, and other harnesses, in one view. NG5 must be lifted
   in the SRS change. The UI sits next to harnesses that connect
   directly: a harness joins a lane through its local peer, reads
   and writes the lane from inside the session, and reaches other
   harnesses with no UI in between. The UI is one more client of the
   same lanes, never a required hop.
5. Self-hosted standalone works with no network at all: one machine,
   its worktrees, its harnesses and the UI. Peer to peer is an
   extension the user turns on, built on top of standalone.

Standalone means ParleyInn sends nothing off the machine; it does not mean
giving up the interface people expect. The UI is a web page in the
browser, served from a port bound to loopback only, as local tools
such as Jupyter do. It guards against the known local attacks: a token
per launch, Host and Origin checks against DNS rebinding, and no
cross-origin requests. SEC-01 as worded bans every listening socket,
so the SRS change rewords it: no socket reachable from off the machine
and no outbound connection in standalone, with peering as the opt-in
step. Local harnesses connect directly through the local store.

Network boundaries. Four boundaries, from strictest to widest. Each
component sits behind exactly one, and a wider boundary is never on
by default:

| Boundary   | Inside                                                  | What may cross                                                                           | Default | Enforced by                                                                                                            |
| ---------- | ------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------- |
| B0 process | the core: record, hooks, recall, what reaches the model | nothing; it talks only to the local store                                                | always  | build-time evidence shows no code the core can execute opens a socket or starts a program beyond its own kernel worker |
| B1 machine | the UI server and local harnesses                       | loopback only: the browser to the UI, with a token per launch and Host and Origin checks | on      | binds to loopback only; a test fails on any other address or any outbound connection                                   |
| B2 peer    | the peer process                                        | signed segments, to and from peers the user enrolled by key                              | off     | peer allow-list, signature and chain checks, imported as untrusted (I2)                                                |
| B3 public  | the public host                                         | reviewed, signed lane bundles, read-only, to anyone                                      | off     | export review step, stricter redaction, no write path                                                                  |

What each piece needs:

| Piece                                | No network | Loopback | Local network (LAN)                        | Remote network (internet)                                     |
| ------------------------------------ | ---------- | -------- | ------------------------------------------ | ------------------------------------------------------------- |
| Core (hooks, recall, MCP over stdio) | yes        | no       | no                                         | no                                                            |
| Local store                          | yes        | no       | no                                         | no                                                            |
| UI server and browser                | no         | yes      | no                                         | no                                                            |
| Peer: machines on one network        | no         | no       | yes, to enrolled peers; optional discovery | no                                                            |
| Peer: other sites, cloud sandboxes   | no         | no       | no                                         | yes: outbound to a self-hosted peer; sandboxes only dial out  |
| Public host                          | no         | no       | no                                         | yes: inbound, read-only                                       |
| Git carrier                          | no         | no       | optional                                   | yes: to the user's own git remote                             |
| The harness itself (Claude Code)     | no         | no       | no                                         | yes, to its model provider, as today; not ParleyInn's traffic |

Standalone uses only the first three rows. Peering adds the peer rows,
LAN first, then remote. The self-hosted peer that remote peers and
sandboxes dial accepts inbound connections from the internet, from
enrolled peers only; that listener sits behind B2. Publishing and git
are separate opt-ins.

Across all four, ParleyInn sends no telemetry and calls no third-party
service. Data reaches the model provider only the way it does today:
through the harness, when the agent recalls it (I4). The SRS change
turns this table into the reworded I4 and SEC-01.

A harness view must not be tied to one agent. The Agent Client
Protocol (ACP), which Zed uses, streams a session as updates and can
replay it with `session/load`; Claude Code's hooks and transcripts
feed the same view for Claude first. Orchestrators in the research
show several agents side by side (Conductor, Agor, Vibe Kanban, Claude
Squad, Superset); none pairs that with a trust model or a peer
network.

What merges without conflict already: plan 2610012322 gives each
writer its own append-only, signed log. The set of all writers' logs
is a grow-only set, so any two peers that exchange logs converge, in
any order, after any partition. Messages, tool runs, results and
verdicts are events in those logs; their order across writers is
causal (each event names the heads it saw), not wall-clock.

What needs a CRDT: state that several participants edit at once. Two
cases are known:

- lane metadata (title, status, labels, assignment): small registers
  and sets;
- files edited live by several people and agents in one worktree, as
  Delta's CRDT worktrees do. Agents in separate worktrees, the common
  case, need none: their edits are events, and git merges code at
  landing.

The contract rules a network side out today. I4 says ParleyInn never talks
to the network. SEC-01 and CON-04 ban sockets in shipped binaries. NG4
and NG5 put shared stores and a graphical interface out of v1. Plan
2610012322's SRS change must decide first: narrow I4 to the core that
feeds the model, through a security review and a new major version,
or ship this as a separate product. Either way the core
keeps no socket, and everything a peer delivers reaches an agent only
as untrusted, pull-only data (I2).

What was searched, from [the research](../../research/README.md):

- Zed Delta: the benchmark for the experience; central, Cloudflare
  Durable Objects, no signing, no trust model for co-authors.
- Block Buzz: branch as room over a signed event log, but one relay
  holds the community.
- Radicle, NIP-34 and Tangled: peer-to-peer git collaboration with
  signed refs; Radicle's per-peer namespaces fit one log per writer.
- iroh, Hypercore and Willow: peer-to-peer transport and range-based
  reconciliation; candidates for the sync layer.
- Automerge, Loro and Yjs: CRDTs for the metadata and live co-editing
  cases; the tech re-evaluation (OQ-32) decides between them (ENG-18).
- torchwood and C2SP tlog-tiles: the segment and checkpoint format.

A cloud sandbox can only dial out, so it needs some reachable peer.
That is any self-hosted node, including the user's own machine; no
node is special.

## Tasks

1. Proving slice: the standalone lane experience on one machine, with
   no network: several harnesses, their results and one view
2. Peer to peer: two peers, no server, split by a partition and
   merged on reconnect
3. Seamless sync: peer discovery, sandboxes behind outbound-only
   networks, background sync and offline queues, and a blind peer
   another entity can host without reading the lanes (PEER-12)
4. Live co-editing in one worktree, with a CRDT chosen by ADR
5. Landing through git and the forge: approvals and checks stay with
   the forge, shown as asserted, and each room links its branches to
   their pull requests, working across partitions
6. Public lanes: export with stricter redaction and review, signed,
   served by any peer, imported as untrusted
7. Git as a carrier: one ref per writer for open-source projects

## Execution

| Phase | Model | Gate                                                                                                                                        |
| ----- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus  | With networking denied, two local harnesses on separate worktrees show in one view with their results; the stakeholder signs off the design |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                         |
| --- | ------ | ------------------------------------------------------------- |
| 1   | 🔲     | [The standalone lane experience, with no network](phase-1.md) |
<?/catalog?>

## Acceptance Criteria

- [ ] Standalone works with no network: several local harnesses, their
  results and one browser view, served on loopback only, with no
  traffic off the machine
- [ ] The stakeholder signs off the lane experience before peering is
  built
- [ ] Two peers that never reach a server, split by a partition, both
  keep working and converge to the same lane after they reconnect
- [ ] A segment with a broken signature or chain is refused and
  audited, never imported (I6)
- [ ] Build-time evidence shows that no code the core can execute
  opens a socket or starts a program beyond its own kernel worker
- [ ] Every new direct dependency has an accepted ADR (ENG-18)
- [ ] All tests pass: `go test ./...`
- [ ] `mdsmith check .` is clean
