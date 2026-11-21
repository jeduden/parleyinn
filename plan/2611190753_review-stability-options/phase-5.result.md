---
n: 5
title: "A generalist starts on what is left of the pull request's cap"
status: "✅"
result: true
summary: >-
  ENG-28 now has a generalist whose tier's cap can no longer fit in the
  pull request's cap start all the same once half that cap is left. It
  is passed what is left as its own cost cap, so no run spends past the
  pull request's cap. A run paid for that left no result is charged the
  cap it was passed. Of 596 generalist runs on record one cost 0.70 USD
  and no other more than 0.25. The ENG-28 scenario step, a record check
  and three drift cases land with it; no instruction changes.
---
# Phase 5 handoff

## Done

- ENG-28's row states the rule, and its scenario's step on the cost
  caps holds the new clause. No scenario was pending.
- A run asks the budget for its tier's cap and its reserve. A
  specialist's reserve is its cap. A generalist's is half its cap,
  0.75 USD.
- A run holds its whole cap while that fits, and waits for the runs
  still running while it can fit once they end. Only then does a
  generalist start on what is left, in whole cents.
- The run's command carries what it holds as its cost cap, and a run
  that left no result is charged that.
- A refusal names the reserve too.
  [ci.md](../../docs/ci.md) says what a review does near the cap.

## Verified

- Unit tests of `review-run`, `review-eval` and `engineering` pass.
- The cucumber scenario `@ENG-28` passes. Its step on the cost caps
  runs `review-run` three times after earlier reviews. With 4.10 USD
  spent every agent starts, each generalist capped at what is left.
  With 4.30 spent no generalist starts. A generalist killed in flight
  is charged the 0.90 it was passed.
- The drift suite catches the three new cases and the 22 others that
  inject a file this phase touches.
- No reviewer's behaviour changes, so no evaluation case ran.

## Not done

- Pull request 136's review would still not have completed. In a
  local run of it the three generalists cost 0.70, 0.11 and 0.09 USD,
  all of the 0.90 that was left.
- The option as it was put kept a run's cap at its tier's. This phase
  caps the run at what it holds, so the pull request's cap stays hard.

## Next

- Phase 6: the evaluation holds a finding's ids to the known
  requirements, as the review does.
