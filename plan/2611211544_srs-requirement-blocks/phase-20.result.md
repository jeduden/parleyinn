---
n: 20
title: "What a review keeps for the next in a requirement of its own"
status: "✅"
result: true
summary: >-
  What a review keeps for the next review of its pull request is ENG-37,
  word for word, after ENG-32: the findings the next carries, what it
  spent, which the next sums, and the outcome a comment's review
  reuses. ENG-28 falls from 202 lines to 166. Their citations, 31 drift
  cases and three scenario steps follow. No scenario comes off @pending.
---
# Phase 20 handoff

## Done

- ENG-37 holds ENG-28's last five sentences, word for word, P0 and
  verified by test: each review keeps its tracked findings and what it
  spent for the next, which carries the findings and sums the spend
  from main's review workflow's artifacts alone, and a review a
  stakeholder's comment brings reuses an earlier outcome of the same
  head, since a comment changes nothing an agent reads. Where the first
  sentence passed the findings to the tool, it names the tool that runs
  the review (ENG-28). ENG-28 keeps the rest in 166 lines.
- ENG-37's scenario takes ENG-28's three steps on the findings carried,
  the spend summed and the outcome reused, bound as before; it starts
  from the repository checkout, which is all their bindings read.
- Four scripts, the review crates' comments on the previous findings,
  the spend file, the kept verdict and the consent the bundle leaves
  out, four engineering checks, the CI page and the findings' decision
  record cite ENG-37. Of the drift cases, 31 move to ENG-37 and the
  spend's other 13 stay ENG-28's.
- Appendix B counts 33 P0 engineering requirements and Appendix C gives
  ENG-37 ENG-28's personas in SEC-33's row. The parse gate counts 35
  engineering requirements. The srs kind lists ENG-28 at 166 lines and
  SEC-31 no longer.

## Verified

- `mdsmith check .` passes.
- The srs gates and the scenario gate pass, and the cucumber scenarios
  `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32`, `@ENG-34` and `@ENG-37`:
  6 scenarios, 81 steps. The workspace's 4,727 tests, clippy denying
  warnings and rustfmt pass.
- `review-eval check` finds every case applies: 242 cases, 75
  components. Against main, the evaluation plans no component.
- The drift suite catches the 36 cases that inject a file this
  changes, 21 of them ENG-37's.
- `scripts/review-local.sh` against main approves: 17 of 17 agents
  consulted, no finding, 0.97 USD. It blocks only on the consent this
  change asks for.

## Next

- ENG-28's other parts, each its own pull request: the specialist's
  report and how the tool reads it, the generalist's weighing, the
  runs and what the review posts when one did not run, and the cost
  cap with the answer store. Each fits in 50 lines, and ENG-28 keeps
  the approval, the run by code and the token's isolation in 43.
- Two evaluation cases' comments, lang-markdown's
  `clean-shared-section` and reviewer-engineer's
  `clean-instruction-with-its-case`, cite ENG-28 for the consent ENG-37
  now holds. They change with their suites' next change, which runs
  those cases anyway.
- LANE-01's split waits for room in `features/lane.feature`; its entry
  leaves the srs kind's list with a later change to `.mdsmith.yml`.
