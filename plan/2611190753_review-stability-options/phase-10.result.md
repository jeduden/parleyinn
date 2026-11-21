---
n: 10
title: "A place spelled in the pull request's tree is the same place"
status: "✅"
result: true
summary: >-
  ENG-28 gains one sentence: the review reads a place a generalist
  spelled in the pull request's tree, `pr-head/<path>`, as the file at
  that path. The review of pull request 154 had blocked on a weighing
  that named all four of its places that way, a spelling the
  evaluation's grader already took. The findings such lines keep are
  posted and tracked at the repository's path. A fifth run of the
  ENG-28 weighing check, named in its scenario step, and two drift
  cases land with it; no instruction changes.
---
# Phase 10 handoff

## Done

- A finding or dropped line at a place spelled under `pr-head/`, or a
  line naming such a place, weighs the specialist's finding at the
  repository's path.
- A finding a generalist keeps at such a place is kept at the
  repository's path: it is posted there, tracked across reviews as the
  same finding, and kept a nit when a specialist's requirement request
  is at that place.
- ENG-28's row states the rule. Its weighing check runs `review-run` a
  fifth time, on a generalist that spells its place in the pull
  request's tree, and its scenario step names that run.

## Verified

- Unit tests of `review-run` and `engineering` pass, with clippy
  denying warnings. Without the fix the weighing check names both
  rules.
- The cucumber scenario `@ENG-28` passes.
- The drift suite catches the two new cases and every other case that
  injects the file this phase touches.

## Not done

- A specialist's place spelled under `pr-head/` is not read this way.
  None was seen, and a generalist's line at the repository's path
  still weighs it.

## Next

- Merge main into pull request 154, whose review runs main's tools,
  and see its weighings decided whole.
