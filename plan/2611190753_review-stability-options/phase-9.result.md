---
n: 9
title: "A run that ended on an error says so"
status: "✅"
result: true
summary: >-
  ENG-28 keeps its row: the review now keeps the harness's text apart
  from an agent's answer when a run ended on an error, a usage limit
  or a call the model's API refused. The finding on a generalist so
  ended quotes the error, and the summary names an error among the
  ways a run ends without a successful result, where it named only a
  turn budget, a deadline or a cost cap. Two drift cases land with it.
---
# Phase 9 handoff

## Done

- The run log keeps a result that is an error apart from the answer:
  the answer is empty, the error holds the harness's text whole.
- The blocking finding on a generalist that stated no weighing quotes
  the error its run ended on, on one line. One whose run ended on no
  error keeps its finding as before.
- The summary says a run ended without a successful result on a turn
  budget, a deadline, a cost cap or an error.
- [ci.md](../../docs/ci.md) says what the review makes of such a run.

## Verified

- Unit tests of the run log in `review-eval` and of the review outcome
  in `review-run` pass, among them the two of the log and the three of
  the finding.
- The drift suite catches the two new cases, the error read as the
  agent's answer and the finding left with no word of why, and the ten
  other cases that inject a file this phase touches.

## Not done

- No record check runs `review-run` end to end on a run that ended on
  an error. The unit tests and the drift cases hold the rule.
- A specialist whose run ended on an error is read from its log as
  before, with no quote of the error in a finding.

## Next

- The weekly evaluation run and the reviews of the coming pull
  requests show whether the reworded generalists are refused again.
