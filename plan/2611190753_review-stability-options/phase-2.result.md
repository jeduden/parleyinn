---
n: 2
title: "A blocking finding stands until a dropped line withdraws it"
status: "✅"
result: true
summary: >-
  ENG-28 now has a generalist's blocking finding stand beside a nit it
  states later at the same place and source, until a dropped line
  withdraws it. In 582 recorded generalist runs, 4 stated such a nit,
  each a second point and none a downgrade, and three of them failed an
  evaluation case for the blocking finding the review had dropped. The
  ENG-28 scenario step, a fourth run in its weighing check and a drift
  case land with it; no instruction changes.
---
# Phase 2 handoff

## Done

- ENG-28's row states the rule, and its scenario step on the reading of
  a weighing carries it. No scenario was pending.
- The weighing keeps a `blocking` finding when a later line at its place
  and source is a `nit`. A `dropped` line still withdraws both, a later
  `blocking` line still replaces both, and a second nit replaces the
  first. The review and the evaluation's grader read the weighing by
  this one rule.
- [review-evaluation.md](../../docs/review-evaluation.md) says so where
  it describes how a generalist is graded.

## Verified

- Unit tests of the weighing, the review's decision and the grader: the
  tests of `review-eval`, `review-run` and `engineering` pass.
- The cucumber scenario `@ENG-28` passes.
- The ENG-28 record check of the weighing runs `review-run` on a
  generalist that states a blocking finding and then a nit at one
  place: the review requests changes and holds both.
- The drift suite catches the case that removes the rule, and every
  other case that injects a file this phase touches.

## Next

- The three generalists' `injection-in-the-diff` cases are where the
  lost finding showed. The weekly run will show whether they now pass
  every run.
- A generalist that means to weigh a finding down must drop it first.
  None on record did otherwise; if one does, its blocking finding
  stands and the author sees both lines.
- Phase 3: consent given after a review clears a line that only
  restated it.
