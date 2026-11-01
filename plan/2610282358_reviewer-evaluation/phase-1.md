---
n: 1
title: "The harness, the cases and ENG-32"
status: "✅"
result: false
---
# Phase 1: the harness, the cases and ENG-32

Requirements. This phase adds ENG-32 (P0, Ver T). Every reviewer
component has its cases. The cases run through the review's own harness
on the component's model. A changed component's cases run before the
gate decides. Every run records what each component cost on which
model. The SRS edit waits for the stakeholder's approval.

BDD coverage: `@ENG-32` lands in `features/engineering.feature`,
`@pending` until the workflow step of phase 2 is read. Its checkout
steps read the cases, apply each to the tree, and run the dispatch on
each; a decision table drives the grader and the two-of-three rule.

RED: `review-eval check` fails on a component without its cases. The
grader's unit tests fail on the missing crate. The gate test
`the_checkout_s_reviewer_components_each_have_their_cases` fails on
the empty case directory. GREEN sites:

- `tooling/review-eval`: `case` reads a TOML case with its edits and
  expectations; `component` lists what the review bundle can dispatch
  and hashes each file; `world` builds the fixture from two worktrees
  and the review bundle; `runner` starts `claude --agent` with the
  review's tools and a deadline; `log` reads the run; `report` and
  `grade` decide one run; `score` folds runs into cases, suites and a
  scorecard; `plan` names what a pull request changed; `compare` reads
  a scorecard against the published one.
- `tooling/review-eval/cases/<component>/*.toml`, and the derived cases
  for each SRS section and domain-model file.
- The trust rule in `docs/agents/specialist-report.md` and the other
  shared report sections, so every specialist has the rule its
  injection case checks.
- The gate test in `tooling/engineering`, its drift cases, and the
  step bindings in `tooling/scenario/tests/bdd/engineering_review.rs`.

Gate: `cargo test -p review-eval` and the gate test pass, and
`review-eval check` passes on the checkout. Live, `review-eval run
--component lang-rust` passes on haiku, and one orchestrator case
passes on sonnet.
