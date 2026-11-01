---
n: 1
title: "The harness, the cases and ENG-32"
status: "✅"
result: true
summary: >-
  ENG-32 added and off @pending. The review-eval crate builds the
  review's own world for a case, runs the component through claude on
  its model, grades the report deterministically and scores the runs;
  185 cases cover 63 components, 59 written and the rest derived; two
  gate tests with drift cases hold them.
---
# Phase 1 handoff

## Done

- ENG-32 (P0) is in the SRS, and its scenario passes: the checkout
  steps read the cases, the decision tables drive the grader and the
  two-of-three rule.
- `review-eval` reads a case, builds the world from two worktrees and
  `review-bundle`, runs `claude --agent` with the review's tools and a
  deadline, reads the log, grades the report and folds the runs into a
  scorecard, the gate's findings and a report. Its unit tests cover
  every line; an end-to-end test drives the executable through real
  worktrees with a stand-in `claude`.
- 59 written cases and 126 derived ones; `review-eval check` applies
  all 185 to the checkout. The specialists' shared report sections
  gained the trust rule the injection cases check.

- The live gate passed: `lang-rust` on haiku, 3 of 3 cases in 6 runs
  for three cents, and the orchestrator on sonnet, 5 of 5 cases in 12
  runs for four dollars. The first runs found three things, each fixed
  with a test. The review's `Write(./review/reports/**)` rule granted
  nothing, since Claude Code grants a file write through an
  `Edit(path)` rule; the harness, the workflow and its check now use
  that form. Haiku found every defect and refused every injection
  while departing from the report's shape in most runs, so the grader
  now counts such slips per run without failing it. And a Claude Code
  session running the evaluation made each agent its child, with its
  subagents in the background, so the harness clears what such a
  session sets.
- A grader false positive, the orchestrator's summary read as a
  finding, is fixed: only its findings are findings.

## Next

- The published scorecard holds no component until the first full run
  of phase 3 publishes one; the live scorecards of the two suites are
  in the pull request.
