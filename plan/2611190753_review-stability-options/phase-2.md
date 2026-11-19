---
n: 2
title: "A blocking finding stands until a dropped line withdraws it"
status: "✅"
result: false
---
# Phase 2: a blocking finding stands until a dropped line withdraws it

Requirements. ENG-28: its row says how the review reads a generalist's
weighing from its whole run log. No scenario comes off `@pending`:
`@ENG-28` runs already, and its step on the reading of a weighing gains
the new clause with the row.

What the record shows. The review read a later line at a place and
source as replacing what stood there. Of 582 recorded generalist runs,
4 stated a `nit` after a `blocking` finding at one place and source.
In each the nit was a second, lesser point: the comment claims
approval, blocking, and the comment should go, a nit. None was a
finding weighed down. The review kept the nit alone, and three of the
four runs failed their evaluation case for the lost blocking finding.

The rule. A `nit` never replaces a `blocking` finding at its place and
source. The blocking finding stands beside it until a `dropped` line
naming its source withdraws it, or a later `blocking` line restates it.
Every other later line replaces as before. The generalists'
instructions already say a later `dropped` withdraws a finding, so no
instruction changes and no reviewer component runs its cases for this.

RED. A unit test of the weighing in `review-eval`, with a blocking
finding and a nit in the final answer, in a message, and one in each.
A decision test in `review-run`, a grader test that replays the
recorded shape, and a fourth run in the ENG-28 record check of the
weighing.

GREEN. The one function that settles a later line in
[report.rs](../../tooling/review-eval/src/report.rs) keeps a blocking
finding when the later line is a nit. The review and the grader both
read the weighing through it.

Gate. The record check in `review_weighings` runs `review-run` on a
generalist that states a blocking finding and then a nit at one place:
the review must request changes and hold the blocking finding. One
drift case removes the rule and fails that check.
