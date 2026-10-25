---
id: 2610012322
title: "Scope ParleyInn for agent fleets: shared, real-time, public sessions"
status: "✅"
summary: >-
  Widen ParleyInn from one agent's local memory to secure session memory
  shared across worktrees, cloud sandboxes, machines and the public,
  in real time. Splits the record (per-origin log segments) from the
  index (SQLite), and ends in proposed SRS changes M1 builds on.
model: opus
depends-on: [2609292004]
---
# Scope ParleyInn for agent fleets: shared, real-time, public sessions

## Goal

Decide what ParleyInn is for when many agents share a repository: in
separate worktrees, in ephemeral cloud sandboxes, on several machines,
in open source, and in real time. Turn the decision into SRS changes
the stakeholder approves before M1 builds the store.

The stakeholder's answer is the lane. A lane is a branch, its
worktrees, its agents and humans, and their conversations and results.
It holds what a pull request's conversation holds, plus the runs and
results behind it, so ParleyInn holds the lane as one record: the
conversation and the code changes in one order. Git stays where code
lands, and landing makes a second history. Squash, rebase and edits on
the forge then break any stored link, so the lane derives its link to
landed commits from the record, and says "not proven" when it cannot.
The record is what a live, multiplayer pull request reads. The working
pitch and the stakeholder's decisions are in [pitch.md](pitch.md).

## Context

For one agent on one machine, Claude can already grep its own
transcripts. There, ParleyInn mostly adds security: the envelope,
provenance, quarantine and redaction (I2, I5). It also keeps
constraints verbatim after compaction (I3). The harder problem is a
fleet, which is the stakeholder's own workflow:

- many agents on separate worktrees of one clone, which today hash to
  separate stores, because §8.1 keys a project by its path;
- Claude Code on the web, whose sandboxes are reclaimed, while §2.2
  assumes `PARINN_HOME` on durable volumes;
- two or three more machines, which share nothing (OQ-07, post-v1);
- open-source contributors, where another person's history is
  untrusted by definition;
- all of it live, which no git-based exchange gives.

Shared memory between agents is a prompt-injection network unless
provenance and trust travel with every event. That makes the security
model the product rather than a feature.

Linking sessions to commits drifts. The research found trailers lost
(3 of 24 Copilot agent commits kept theirs), notes refs clobbered and
links broken by squash. So the record carries the edits themselves:
patches from tool calls, checkpoints of the worktree and landed commit
ids, in the same order as the messages. Edits made outside the
agent's tool calls, by a person, a formatter or a shell, reach the
record only through worktree checkpoints. A lane also admits humans
and other agents. Their messages reach an agent enveloped and
untrusted (I2). Even the owner is trusted only through their own local
session, or a key no other node ever holds; in automation mode
(PRV-04) not even the owner's prompts are trusted.

Spike S2 ([plan 2609292004](../2609292004_spike-s2-sqlite-driver/plan.md))
chose the SQLite driver for a single-node store. It also showed
SQLite as the record cannot be merged, streamed or published: one
mutable file, a gap-free `seq` and one hash chain per store. The
candidate this plan tests keeps ADR-02's event sourcing but splits
it in two:

| Layer  | Form                                                                                     |
| ------ | ---------------------------------------------------------------------------------------- |
| Record | append-only, content-addressed, hash-chained log segments, one writer per origin         |
| Index  | local SQLite with FTS5, rebuilt from the segments a node holds (I10), on S2's `rusqlite` |

Each origin (node and session) writes only its own log, so the
record has no write conflicts and needs no conflict-free merge
method. Only state several participants edit at once, lane metadata
and live co-editing, needs one
([plan 2610022338](../2610022338_parinn-network-side/plan.md)).
Sync, publishing and sandbox persistence all become moving segments.

The evidence for this plan is in [research](../../research/README.md).
Its merged report argues that ParleyInn should own the record and let git
carry copies, and that code should stay on git.

What was searched and weighed:

- Replicated SQLite. Litestream ships one writer to replicas only.
  libSQL embedded replicas put the network in the core (I4).
  cr-sqlite and dqlite or rqlite need CGO (CON-02) or a server
  (ADR-01). None was reused.
- Git as the record. It is a natural transport for reviewed segments
  and open source, but too slow and too coarse as the live store.
- The S2 harness under
  [bench](../2609292004_spike-s2-sqlite-driver/bench-go/main.go) is
  reused for the index side: its corpus, its FTS5 probes and its
  measurements. Spike S12's proving slice ([report](slice/report.md))
  reads the corpus of S2's Rust port where it lives.

The changes touch §1, §2, §4.5, §6, §8 and the invariants, so they wait
for the stakeholder. A change that weakens I1–I10 needs a security
review and a new major version (CLAUDE.md). M1 blocks on this plan
because it builds the store's identity model.

## Tasks

1. Proving slice: a scope proposal, and a prototype that rebuilds the
   same index from two origins' segments in any order. The proposal
   defines the lane: its identity, edits as events, and the owner's
   trust model for messages from other participants. It also defines
   the process a pull request runs: the merge gate and who approves,
   identities and permissions for agents and humans, the landing path
   through squash, rebase and merge queues, and how a lane coexists
   with the forge's review, checks and bots. It decides whether the
   network side changes I4 or ships as a separate product. From OpenAI
   dots it weighs rule levels per agent action (act, act when told,
   ask first, hand off) beside I2's rule on what an agent reads, and
   one identity per harness across terminal, UI and other harnesses
2. The proposed SRS changes, in pull request #16 at the stakeholder's
   choice: goals and non-goals, deployment context, invariants restated
   for many nodes, the identity model, and new ADRs for the record/index
   split. Landed as SRS 2.0-draft, with the security review in
   ADR-2610032155, accepted on 4 October 2026
3. Re-plan M1 against the approved identity model
4. Name the lane and its peer network in the SRS change: no central
   service, every node a full peer, partitions merged on reconnect,
   their trust boundary and the invariants that govern them. Plan
   2610022338 builds them, experience first. The change also lifts
   NG5 (no graphical interface): the lane needs a UI that shows each
   agent's harness, its work results and other harnesses in one view
5. Re-evaluate the implementation stack against the complete SRS
   (OQ-32) before any implementation plan starts

## Execution

| Phase | Model | Gate                                                                                  |
| ----- | ----- | ------------------------------------------------------------------------------------- |
| 1     | opus  | The prototype's test rebuilds a byte-identical index from two origins in either order |

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

| #   | Status | Phase                                                                                                                                                                                                                                                   |
| --- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | ✅     | [Scope proposal and the record/index proving slice](phase-1.md)                                                                                                                                                                                         |
|     | ↳      | Spike S12 passes: on each node, two writers' segments in either order rebuild a byte-identical index file, foreign events stay untrusted, one branch link stands, a purge leaves nothing confirming a guess. Byte-identity per append needs a decision. |
<?/catalog?>

## Acceptance Criteria

- [x] The scope proposal names, for each fleet setting, what ParleyInn
  does, which invariant governs it, and what stays out of scope
- [x] Two origins' segments, merged in either order, rebuild a
  byte-identical index, and foreign events stay untrusted (spike S12,
  [report](slice/report.md))
- [x] The SRS changes are proposed, in pull request #16 (SRS
  2.0-draft); the invariant changes are approved in ADR-2610032155,
  accepted on 4 October 2026
- [x] M1 is re-planned on the approved identity model, in plan
  [2609292012](../2609292012_m1-record-and-recall/plan.md); the
  language and shell records the stack rests on (OQ-32) were accepted
  on 14 October 2026, leaving only the packaging open
- [x] All tests pass: `cargo test --workspace` at the root and
  `cargo test` in the proving slice's own crate
- [x] `mdsmith check .` is clean
