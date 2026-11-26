---
id: 2611261842
title: "The review reads a pull request as landing it would leave main"
status: "🔳"
summary: >-
  The review, its evaluation and the local review read a pull request's
  head merged into main, never the head alone, so a stacked or
  out-of-date branch is reviewed on what it changes, and a head that
  does not merge with main is not reviewed. ENG-28 says so; its
  scenario runs the merge on four histories.
model: opus
depends-on: []
---
# The review reads a pull request as landing it would leave main

## Goal

The review judges what landing a pull request changes in main, whatever
the branch's history. Its diff, its consent check, its evaluation plan,
its agents and its cases all read the head merged into main.

## Context

What happened. Phase 14 of plan 2611190753 merged phase 16 of plan
2611211544 and main apart, and so did the tree it was reviewed against.
`git diff BASE...HEAD` then has two merge bases and diffs from the
first, main, so the local review read phase 16's change as phase 14's
own. It ran 17 agents for 1.09 USD where phase 14's own diff runs 5 for
0.30, and it asked consent for `tooling/review-bundle/`, which phase 14
does not touch. The review workflow diffs the same way, from
`origin/main...$HEAD_SHA`, so a stacked pull request pushed after its
parent merged, before it merges main, is reviewed so in CI.

The evaluation plan compares main's tree with the head's. A branch
behind main so plans each reviewer component main changed since as one
it reverts: phase 14 against a main holding phase 17 of plan 2611211544
plans 42 components and 141 cases, about 11.88 USD, where against its
own base it plans none.

CI's checks already run on GitHub's test merge of the head into main;
the review read the head alone.

Reuse. `git merge-tree --write-tree` (git 2.38) merges two commits with
no work tree and names the files that conflict. `git commit-tree` with
a fixed author and date makes the merge commit a function of its tree
and parents, so the evaluation's job makes the same one. The stability
measurement's snapshots each hold their base, so their merge is their
head, and it stays as it is. `scripts/ci-local.sh` picks its gates from
the files changed since one merge base, which at worst runs more of
them.

Requirements. ENG-28 gains the sentence; its scenario gains two steps,
the workflow's wiring and the merge on four histories. No scenario
comes off `@pending`.

## Tasks

1. RED: ENG-28's scenario holds a step that runs the merge on a branch
   that merged main and its parent apart after main merged the parent,
   a branch behind main, a head that holds main and a head that
   conflicts, and one that finds the review job, the evaluation's job
   and the local review running it before they read the head. Both
   fail on the checkout as it is.
2. `scripts/review-merge-head.sh` moves `pr-head` to its head merged
   into the commit main's checkout holds and writes `pr.diff` from main
   to that merge; a head that holds main is its own merge, and one that
   does not merge fails, naming the files that conflict.
3. The review job runs it in place of the three-dot diff, the
   evaluation's job before its cases from a checkout with history, and
   `scripts/review-local.sh` once it has added its two worktrees.
4. ENG-28 says the review reads the head merged into main and does not
   run on a head that does not merge; `.mdsmith.yml` lists ENG-28 at its
   new length. Each step's check lands with a drift case.
5. [docs/ci.md](../docs/ci.md) says what the review reads.

## Acceptance Criteria

- [ ] A branch that merged main and another pull request apart is
      reviewed on its own change alone
- [ ] A branch behind main plans no reviewer component it does not
      change
- [ ] A head that does not merge with main is not reviewed, and the run
      names the files that conflict
- [ ] ENG-28's scenario passes with both steps, and a drift case fails
      each
