---
n: 4
title: "A review a cost cap stopped says what to do"
status: "✅"
result: true
summary: >-
  ENG-28 now has a step that fails without deciding post the check
  failed with its own error in the summary. The error names each
  dispatched agent that did not run with the reason the review outcome
  gives. Where a cost cap kept one from starting, that holds what the
  pull request's reviews cost and the cap, and the error adds how what
  they spent is reset. The gate's rule is unchanged: it decides no
  review that left an agent out. A scenario step, a record check and
  four drift cases land with it.
---
# Phase 4 handoff

## Done

- ENG-28's row states the rule, and its scenario has one more step.
  No scenario was pending.
- The gate's error repeats, for each dispatched agent that did not
  run, the reason the review outcome gives. One form names such an
  agent in the outcome, and the gate's crate writes and reads it.
- For a review a cost cap kept agents out of, the error ends with how
  what the pull request's reviews spent is reset.
- The decide script keeps the gate's errors, prints them to the log as
  before, and puts the last line in the summary of the check it fails.
  [ci.md](../../docs/ci.md) says so.

## Verified

- Unit tests of the gate, and of `review-run`, `review-eval` and
  `engineering`, pass.
- The cucumber scenario `@ENG-28` passes with its new step. The step
  runs `review-run` under a cap its first run reaches, then the gate
  on what it wrote: the error names all four agents not started, each
  with its reason, and the reset. It also reads the job that holds the
  reviewer app's key for the three things the script does.
- The drift suite catches the four new cases and the 23 others that
  inject a file this phase touches.
- The decide script was run by hand with stand-ins for `cargo`, `gh`
  and `jq`: the check's summary held the gate's line in a code span,
  its backticks dropped. No test runs the script itself; the record
  check reads its text.

## Next

- The summary names the artifacts by their pattern,
  `review-spend-pr-<N>-a*`, not by the pull request's number. The
  script could fill the number in.
- Phase 5: a generalist starts once its reserve is left of the pull
  request's cap, capped at what is left.
