---
n: 3
title: "Retire the orchestrator"
status: "✅"
result: false
---
# Phase 3: retire the orchestrator

Requirements. ENG-32 no longer names the orchestrator's skill among
the reviewer components: they are each generalist with its deep
instance, each specialist the review bundle can dispatch, and every
other agent under `.claude/agents`. An agent that names no model runs
on the evaluation's default model. This phase leaves the ENG-28 row as
phase 1 rewrote it; its review run by code now also fails closed on a
generalist whose answer says nothing readable and on a specialist
report ending blocking.

BDD coverage. The ENG-32 scenario's component step is reworded to drop
the orchestrator and kept, with its binding. A drift case proves the
case check refuses a case left for the retired `review` component. No
scenario is deleted or put back on `@pending`.

RED. Write tests for each of these, in order:

- The grader and `review-run` read a dropped line by one rule, the
  whole first word `dropped` or a `dropped` severity, after any list
  marker, a numbered one included.
- A skill is no reviewer component: the component names, the plan of a
  pull request's evaluation and the case check leave it out.
- A generalist answer with no finding, no dropped line and no
  `none — nothing to add — name` line requests changes; a specialist
  report ending `outcome: blocking` requests changes whatever its
  generalist answers.

GREEN.

- `report::dropped` and `report::nothing` in `review-eval`, used by the
  grader and `review-run`.
- Remove `.claude/skills/review`, `tooling/review-eval/cases/review/`,
  the `Orchestrator` kind, its prompt, schema, tools and turn budget,
  the grader's review-outcome branch, and what only it read: the run
  log's agent calls, structured output and main-thread writes, the
  world's dispatch and consult count, a case's `consulted` and the
  verdicts among a case's outcomes.
- Reword the generalists' input section, the bundle's dispatch text,
  ENG-32's row, scenario and binding, the reviewer-engineer agent, the
  reviewer-shape skill, `docs/review-evaluation.md`,
  `docs/development.md`, ADR-2610300031 and the changelog.

Gate. The ENG-28 and ENG-32 scenarios pass; `review-eval check`,
mdsmith, rustfmt, clippy and the tests of the changed crates are
clean; `review-run` keeps 100% line coverage. The generalists' cases
run again on their models.
