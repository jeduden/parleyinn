---
title: "1.5 Non-goals for v1"
summary: >-
  What ParleyInn v1 deliberately does not do, NG1 onward, each with its
  reason.
---
# 1.5 Non-goals for v1

NG5 to NG8 were retired by the stakeholder on 6 October 2026, and their ids are
not reused: the room view reaches the principal's enrolled devices, and any
wider reach is open ([§1.4](01b-scope.md)), ParleyInn serves Claude first rather
than ruling other harnesses out (§1.4), room summaries are in scope (LANE-33),
and whether ParleyInn may run, push or merge code is open (OQ-37).

| #   | Non-goal                                                           | Reason                                                                                                                                                                 |
| --- | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| NG1 | Replacing or disabling Claude Code's native compaction             | Behaviour of the harness is outside our control; ParleyInn makes compaction recoverable instead. OQ-01 kept this after S6; steering when compaction happens is UC-006. |
| NG2 | Long-term semantic memory, fact extraction, or automatic promotion | Largest poisoning surface; belongs to a separate layer-5 system with its own policy                                                                                    |
| NG3 | Embedding or vector search                                         | BM25 is deterministic, needs no model calls at ingest, and is sufficient for v1 (Scroll uses BM25)                                                                     |
| NG4 | Sync in v1.0, and a shared mutable store at any version            | Sealed segments of each writer replicate between enrolled peers as P2 (PEER) after their own security review. OQ-07 closes with PEER.                                  |
| NG9 | Merging several seats' edits to one worktree in v1.0               | A run and its subagents sharing a worktree are in scope (LANE-19); a conflict-free method of merging them is chosen by ADR after PEER ships                            |
