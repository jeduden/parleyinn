---
n: 5
title: "A generalist starts on what is left of the pull request's cap"
status: "✅"
result: false
---
# Phase 5: a generalist starts on what is left of the cap

Requirements. ENG-28: its row holds the cost rule. A run is passed
its tier's cost cap and starts only once that cap fits in the pull
request's cap of 5 USD. No scenario comes off `@pending`: `@ENG-28`
runs already, and its step on the cost caps gains the new clause.

What the record shows. A generalist's cap is 1.50 USD. Of 596
generalist runs on record, 582 in the evaluation and 14 in reviews,
one cost 0.70 and no other more than 0.25. A run held its whole cap,
so a review was refused its generalists once 3.50 of the 5 USD were
spent. On pull request 136 the reviews had spent 4.10. All three
generalists were refused with 0.90 unused.

The option as it was put had a generalist hold less than its cap and
keep the cap. Then a run could spend past what it held, and the pull
request's cap would no longer be hard. This phase keeps it hard.

The rule. A run holds its tier's cap while that fits, and waits for
runs still running while it can still fit once they end. Once a
generalist's cap can never fit again, it starts all the same when half
its cap is left. It then holds what is left, and is passed that as its
own cost cap. A run paid for that left no result is charged the cap it
was passed. A specialist's rule is unchanged.

What it does not do. In a local run of pull request 136's review the
three generalists cost 0.70, 0.11 and 0.09: all of the 0.90 that was
left. Under this rule the first would have run and the others not. The
rule helps the usual review, whose generalists cost about 0.10 each.

RED. Unit tests of the budget: with 4.10 spent a generalist holds 0.90,
the next 0.78 once it ended at 0.12, and a third is refused with 0.66
left. A run waits while its whole cap can still fit.

GREEN. A need, a tier's cap and its reserve, replaces the one number a
run asked for. The reservation says what it holds, the command is
capped at it, and the charge of a run that left no result is what it
held.

Evaluation. No reviewer's behaviour changes, so no evaluation case
runs.

Gate. The record check in `review_reserve` runs `review-run` three
times after earlier reviews that spent 4.10 or 4.30. Three drift cases
fail it: a generalist held to its whole cap, one not capped at what it
holds, and one killed in flight charged its tier's cap.
