---
n: 3
title: "Retire the orchestrator"
status: "✅"
result: true
summary: >-
  The review skill, its ENG-32 component and cases, and what only the
  orchestrator used are gone, and ENG-32's row, scenario and the docs
  name no orchestrator. The grader and review-run read a dropped line
  by one rule, and review-run fails closed on a silent generalist and a
  blocking specialist report. The live run waits for consent.
---
# Phase 3 handoff

## Done

- One dropped rule: `report::dropped` in `review-eval` (first word
  `dropped`, whole word, after any list marker, numbered ones
  included, or a `dropped` severity) is what the grader and
  `review-run` both read. The grader no longer takes any line holding
  `drop`; `review-run` now skips a numbered dropped line in a closing
  sentence. The whole-word fix of c802458 lives there with its test.
- Fail closed in `review-run`: a generalist's answer with no finding,
  no dropped line and no `none — nothing to add — name` line
  (`report::nothing`) requests changes with a blocking finding at
  `review/` naming the generalist, and the summary names it; a
  specialist report the gate takes that ends `outcome: blocking` is a
  blocking finding on its report file whatever its generalist answers.
  `docs/agents/generalist-report.md` gives the none shape.
- Retired: `.claude/skills/review/SKILL.md`,
  `tooling/review-eval/cases/review/` (five cases), the `Orchestrator`
  kind, its model, effort, turn budget, tools, prompt and outcome
  schema, the grader's review-outcome branch, the run log's agent
  calls, structured output and main-thread write list, the world's
  dispatch set and consult count, a case's `consulted` key and the
  verdicts among a case's outcomes. A skill is no reviewer component:
  the names, the evaluation plan and the case check leave it out.
- ENG-32's row names the generalists with their deep instances, the
  specialists and every other agent under `.claude/agents`; an agent
  naming no model runs on the evaluation's default. Its scenario's
  component step drops "the orchestrator" and keeps the rest, with its
  binding. New drift case: a case left under `cases/review/` names no
  component.
- Reworded: the generalists' input section ("the review runs you
  once"), the bundle's "Agents to consult" text, the dispatch module,
  the reviewer-engineer agent, the reviewer-shape skill,
  `docs/review-evaluation.md`, `docs/development.md`, ADR-2610300031's
  decision and consequences, and the changelog. A plan link to the
  removed skill is plain text now.
- Checks: `review-eval check` (213 cases, 73 components), mdsmith,
  rustfmt, workspace clippy, and the tests of review-eval, review-run,
  review-gate, review-bundle, engineering, drift and scenario pass,
  apart from `the_checkout_s_languages_each_have_their_specialist`
  (a worktree's `.git` is a file) and the hook-budget scenarios, which
  need release builds of `parinn` and `calibrate`. `review-run` keeps
  100% line coverage; review-eval's lines are covered by its unit
  tests. The ENG-32 scenario passes; ENG-28's passes but for that
  worktree step.
- Evaluation run after the generalists' input and report sections
  changed (`review-eval run --repetitions 3`, Claude Code 2.1.314,
  mdsmith on PATH): every case passed. review-code 4 of 4 cases (8 of
  9 runs, one injection run missed), review-security, review-spec and
  the three deep instances 4 of 4 with 8 of 8 runs each, and
  reviewer-engineer 3 of 3 (6 of 7 runs); no slips. Sonnet $0.55 to
  $0.70 per component, Opus $1.18 to $1.52.

## Decisions

- The accepted ADRs that describe the orchestrator (ADR-2609301941,
  ADR-2610262245, ADR-2610290002) keep their text as the record of
  what was decided; ADR-2610300031 says so. The old plans keep theirs.
- A case's `consent` key and the world's consent file stay: the
  bundle reads consent for every component's world.
- The grader does not demand the none line of a generalist's case:
  the cases stay as they are, and `review-run` holds the rule.
- `features/lane.feature`'s "orchestrating agent" is a lane's own
  agent, not the review's, and stays.

## Next

- The live check, one review decided by the gate on its first run,
  needs this branch on main, after the stakeholder's consent to the
  `.github/`, SRS, scenario, agent-instruction and ADR paths.
- ADR-2610300031 stays `proposed` until the stakeholder accepts it;
  then ADR-2610262245 is marked superseded.
- Waiting on the stakeholder, untouched: the empty dispatch, the
  consent finding, retries, the parallel bound and the since-last
  step.

## Follow-up: remove the orchestrator skill after this merges

The `evaluate` job runs main's harness against this pull request's tree, and
main's harness still reads `.claude/skills/review/SKILL.md` for the `review`
component. With the file removed, the job stops with "read
.claude/skills/review/SKILL.md: No such file". The file is kept, unchanged from
main, until main runs `review-run`. A follow-up pull request then deletes it.
No review reads it any more: the review job runs `review-run`.
