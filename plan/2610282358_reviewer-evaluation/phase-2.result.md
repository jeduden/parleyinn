---
n: 2
title: "The evaluate job, the scheduled run and the comparison"
status: "✅"
result: true
summary: >-
  review.yml plans what a pull request changed and runs those cases in
  an evaluate job whose findings the gate weighs; review-eval.yml runs
  every component weekly against the published scorecard and compares
  a candidate model on demand. The live run on a pull request waits for
  the workflow to reach main.
---
# Phase 2 handoff

## Done

- The review job plans the evaluation; the `evaluate` job runs the
  changed components' cases from the pull request's tree with the
  agent's token and no write permission; the post job merges its
  findings into the gate's deterministic findings.
- `review-eval.yml` runs weekly on every component and on demand on a
  candidate model, writing the comparison into its summary.
- CODEOWNERS names the stakeholder on the cases and the scorecard.

## Next

- The workflow runs as main defines it, so this change cannot evaluate
  itself: the first live evaluate job runs on the next pull request
  that changes a component after this one merges.
- The weekly run needs `CLAUDE_CODE_OAUTH_TOKEN`, which the review
  already has. Phase 3 publishes the first scorecard from its artifact
  with `review-eval publish`.
