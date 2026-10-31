---
n: 1
title: "The review gate, the workflow and ENG-28"
status: "✅"
result: false
---
# Phase 1: the review gate, the workflow and ENG-28

Requirements. This phase adds ENG-28 (P0, Ver T). An agent's approval
is posted by the reviewer app, from a workflow main defines. The
reviewing agent holds no credential that can approve or write. A
separate step approves only an approving verdict with no blocking
finding, on the head the agent read, once CI passed there. It
requests changes otherwise, and fails without posting on a missing
or malformed verdict or credential. The SRS edit waits for the
stakeholder's approval.

BDD coverage: `@ENG-28` lands off `@pending`. Its steps read
[review.yml](../../.github/workflows/review.yml): `workflow_run` on CI
as the only trigger; no write permission and no app key in the agent's
job; the key in one job that runs no agent, needs the agent's job and
runs the gate. A decision table drives the gate itself.

RED: the scenario fails on the missing workflow, the gate's unit tests
on the missing package. GREEN sites:

- `internal/review` decides the
  review, and `cmd/review-gate` wraps
  it for the workflow. Both sit at the 100% coverage floor.
- [review.yml](../../.github/workflows/review.yml) runs the agent and
  the gate, and the review skill, `.claude/skills/review/SKILL.md`
  (retired by plan 2610292344), is the agent's protocol.
- Five drift cases guard ENG-28 in
  `internal/drift`.

Gate: `@ENG-28` passes and the drift suite catches all five drifts.
