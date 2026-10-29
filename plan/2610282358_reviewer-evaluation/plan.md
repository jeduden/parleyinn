---
id: 2610282358
title: "Reviewer evaluation: every reviewer component against its cases"
status: "🔳"
summary: >-
  Each reviewer component of the layered review gets evaluation cases
  that prove its instructions find a planted defect, pass a clean
  change and resist text addressed to the reviewer, on the model its
  front matter names. A pull request that changes a component runs its
  cases before the gate decides; a scheduled run and a model comparison
  show when a model stops serving the instructions or another serves
  them better (ENG-32).
model: opus
depends-on: [2609301942]
---
# Reviewer evaluation: every reviewer component against its cases

## Goal

Every reviewer component of the review (ENG-28) is held to cases the
way code is held to tests. A case is a defect it must find, a clean
change it must pass, or an instruction addressed to it that it must
report and not obey. Each case runs through the review's own harness on
the model the component names. So a change to the instructions, or a
change of the model behind them, shows up as a failing case before it
reaches main.

## Context

The layered review of plan
[2609301942](../2609301942_agent-review-app/plan.md) and
[ADR-2610262245](../../docs/adr/ADR-2610262245-layered-agent-review.md)
puts about seventy agents between a change and main: an orchestrating
skill, three generalists with their deep instances, and a specialist
per language, SRS section, domain-model file and test layer. Each is
a Markdown file under `.claude/`. Nothing today proves that a file
guides its model as the text intends, or keeps doing so when the
model behind `model: haiku` moves. The review gate checks the
routing, never the judgement.

What was searched, and what is reused:

- `review-bundle` already computes the dispatch and writes the bundle
  from two trees and a diff; the harness calls it on the fixture, so a
  case also proves the dispatch brings its component.
- `review-gate` already weighs deterministic findings; the evaluation
  reports its failures in that shape, so the gate needs no change.
- The drift registry's injection model (replace, create, remove) is
  the shape a case's edits take; the crate's static types are not
  reused, since cases are read from files.
- `parinn-eval` (§11.1) sets the conventions: repetitions, Wilson
  intervals, a published result the stakeholder owns. It links the
  product crates, so the reviewer evaluation is its own tooling crate.
- The `claude` command line runs one agent as the session with
  `--agent`, and `--model` overrides the agent's front matter; its
  `stream-json` log holds what the review workflow already reads: each
  call's usage, the agents consulted and the result's cost.

Not reused: `claude plugin eval`, which evaluates a plugin's cases with
graders and an LLM judge. It has no fixture world, no review bundle,
and its format is marked experimental. The reports' fixed shape lets a
deterministic grader decide, so no judge model is in the loop. The
design is recorded in
[ADR-2610290002](../../docs/adr/ADR-2610290002-reviewer-evaluation.md).

## Tasks

1. ENG-32 in the SRS with its scenario, the ADR and this plan.
2. The `review-eval` crate: the case format, the fixture world, the
   harness, the grader, the scorecard, the plan of what to run and the
   comparison with the published scorecard.
3. Cases for every component: written for the fixed specialists, the
   generalists and the orchestrator; derived from the tree for each SRS
   section and domain-model file. The trust rule the specialists lack
   joins their shared report section.
4. The gate test with its drift cases; `@ENG-32` off `@pending`.
5. CI: the evaluate job of `review.yml`, the scheduled run and the
   model comparison in `review-eval.yml`, the `claude` pin.
6. The first full run on every component's model, the published
   scorecard, and the fix of each case or instruction that fails.
7. Cases for every other agent under `.claude/agents`, which the bundle
   never dispatches and a skill consults: the persona agents,
   `domain-model`, `test-engineer` and `reviewer-engineer`, graded from
   their text answer, with the one-line finding shape and the trust
   rule in their instructions.

## Execution

| Phase | Model  | Gate                                                                                                     |
| ----- | ------ | -------------------------------------------------------------------------------------------------------- |
| 1     | opus   | `review-eval check` passes on the checkout; one suite and one orchestrator case pass live on their model |
| 2     | sonnet | `@ENG-32` passes; a pull request changing an agent gets its evaluation findings in the posted review     |
| 3     | sonnet | The scheduled run passes on every component and matches the published scorecard                          |
| 4     | opus   | A candidate model's scorecard compares with the published one, and the decision is recorded              |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                                                                                                                                                                                                                                                                              |
| --- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | ✅     | [The harness, the cases and ENG-32](phase-1.md)                                                                                                                                                                                                                                                                    |
|     | ↳      | ENG-32 added and off @pending. The review-eval crate builds the review's own world for a case, runs the component through claude on its model, grades the report deterministically and scores the runs; 185 cases cover 63 components, 59 written and the rest derived; two gate tests with drift cases hold them. |
| 2   | ✅     | [The evaluate job, the scheduled run and the comparison](phase-2.md)                                                                                                                                                                                                                                               |
|     | ↳      | review.yml plans what a pull request changed and runs those cases in an evaluate job whose findings the gate weighs; review-eval.yml runs every component weekly against the published scorecard and compares a candidate model on demand. The live run on a pull request waits for the workflow to reach main.    |
<?/catalog?>

## Acceptance Criteria

- [x] Scenario `@ENG-32` passes, off `@pending`
- [x] Every component the review bundle can dispatch, the orchestrator
      and each deep instance has its three kinds of case, and
      `review-eval check` applies every case to the checkout
- [ ] A pull request that changes a component gets that component's
      cases run from main's workflow, and a failing case blocks it
- [ ] The scheduled run reports every component on its model against
      the published scorecard, and a model comparison names what a
      candidate does better or worse
- [ ] The drift suite catches a case that no longer applies, a
      component without its cases, and a workflow that skips the
      evaluation
- [ ] All tests pass: `mise run test`
- [ ] `mise run lint` is clean
