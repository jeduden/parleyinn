---
n: 2
title: "The evaluate job, the scheduled run and the comparison"
status: "✅"
result: false
---
# Phase 2: the evaluate job, the scheduled run and the comparison

Requirements. The workflow half of ENG-32. A pull request that changes
a component or its cases runs those components' cases, from the
workflow the default branch defines, before the review gate decides. A
scheduled run runs every component's cases on its model. A run on
another model compares with the published scorecard.

BDD coverage: the `@ENG-32` step that reads
[review.yml](../../.github/workflows/review.yml) for the `evaluate`
job, after the agent's job and before the key's, with a drift case
that removes the job.

RED: the step fails on a workflow without the job. GREEN sites: the
`evaluate` job and the plan step in `review.yml`, the post job merging
the findings; [review-eval.yml](../../.github/workflows/review-eval.yml)
for the weekly run and the dispatched comparison; CODEOWNERS on the
cases and the published scorecard; the docs.

Gate: `@ENG-32` passes with the workflow step, and zizmor is clean.
Once the workflow is on main, a pull request that changes an agent gets
its evaluation findings in the posted review.
