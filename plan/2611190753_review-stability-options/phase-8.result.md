---
n: 8
title: "A generalist gives its whole weighing in its final answer"
status: "✅"
result: true
summary: >-
  A generalist is now asked for every line of its weighing in its final
  answer and none before it, after the model's API refused the code
  generalist of one pull request twice as asking for its reasoning in
  its text. ENG-28 and ENG-32 keep their rows: the review still reads
  the whole run log, and the evaluation counts a line stated early and
  left out of the answer as a slip. The generalists' 26 cases passed in
  52 of 52 runs with no slip. The refusal did not occur outside CI, so
  the fix is not proven against it. One drift case lands with it.
---
# Phase 8 handoff

## Done

- The report section a generalist's definition includes asks for the
  whole report in the final answer: every line, one line a place, then
  a scope sentence, and none of it before then.
- The six generalist definitions are regenerated from it.
- The grader shows a weighing line a generalist stated before its
  final answer, which the answer does not hold, as a slip. It fails no
  run.
- [docs/review-evaluation.md](../../docs/review-evaluation.md) lists
  that slip.

## Verified

- Unit tests of `review-eval` pass, among them the five of the new
  slip and the one through the grader for a generalist and a
  consultant.
- Evaluation run on the reviewers' models: `review-code`,
  `review-spec`, `review-security` and their deep instances passed 26
  of 26 cases in 52 of 52 runs, with no slip, for 4.74 USD. One run
  wrote a text before its final answer, a working note.
- `review-eval check` applies all 235 cases for 74 components.
- The drift suite catches the new case, a grader that no longer counts
  such a line, beside the eight on the changed files.

## Not done

- The refusal is not reproduced. The review of pull request 152 ran
  three times on this machine and its code generalist answered each
  time. Whether the new wording clears the refusal shows only in CI.
- The specialists' prompt still asks for each finding the moment it is
  found. No specialist was refused, and their model's documentation
  does not list the category.
- A run the API refused is still read as an answer, and the review's
  summary still names a turn budget as its cause. That fix is on its
  own branch.
- No scorecard was published. That is the stakeholder's to accept
  after a run of every component.

## Next

- Watch the first reviews in CI after this lands, pull request 152's
  among them, for a refused generalist.
