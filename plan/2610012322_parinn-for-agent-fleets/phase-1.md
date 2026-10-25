---
n: 1
title: "Scope proposal and the record/index proving slice"
status: "✅"
result: false
---
# Phase 1: scope proposal and the record/index proving slice

Requirements. No SRS id closes, and no scenario leaves `@pending`.
The phase writes evidence and a proposal; the SRS changes follow in
phase 2, for the stakeholder.

BDD coverage: none yet. The proposal names the scenarios each new
requirement would add, so phase 2 can land them `@pending`.

The proposal, `proposal.md` beside this plan, covers, per fleet
setting (worktrees, cloud sandboxes, machines, open source, real
time):

- what ParleyInn does there, and what stays out of scope; - the invariant
  that governs it, restated for many nodes where needed (I1 stable
  addresses, I2 foreign trust, I4 core versus the network side, I8
  tenant across nodes); - the identity model: writer (node and
  session), the address (writer, seq), segment hash, and how a project
  is keyed so worktrees of one clone share it; - the lane and the
  process a pull request runs, as plan task 1 lists them.

RED: a test in a separate module beside this plan, reusing S2's
corpus, writes events as segments for two writers, A and B. It builds
the SQLite index from A then B, and from B then A. It fails until both
indexes are byte-identical, every event keeps its writer and trust,
and B's events read as untrusted on A's node. It also has seats A and
B each link one branch to the room, and fails until both orders derive
one branch link (LANE-31; SRS 2.0 renamed the lane to the room, whose
branches branch links name). It purges one of A's events and fails
until no stored or replicated value matches a guess at its content
(REC-17, the per-event commitment).

GREEN sites: the segment writer and reader, and the index rebuild,
inside that module only. Nothing enters parinn's module graph.

Gate: the test passes on both orders and after the purge. The
proposal covers every fleet setting above. A persona review of the
proposal reports no blocker.
