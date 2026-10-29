---
id: ADR-2610290002
title: "Reviewer evaluation: cases as edits, graded from the report, gated in the review"
status: proposed
summary: >-
  Each reviewer component of the layered review has cases, each an
  edit to main's tree with the report it must give. The cases run
  through the review's own harness on the component's model, a
  deterministic grader reads the report, a changed component's cases
  run before the gate decides, and a published scorecard per component
  and model shows when a model stops serving the instructions.
---
# ADR-2610290002: Reviewer evaluation

## Context

ADR-2610262245 puts one orchestrating agent, six generalists and
about sixty specialists between a change and main, each a Markdown
file under `.claude/`. The review gate checks that every dispatched
agent was consulted and that each report ends with the outcome its
agent answered. It cannot check that the text guides its model as the
author meant: that a specialist finds the defect its checks name,
passes a clean change, and reports rather than obeys a sentence in
the diff that speaks to it. The aliases `haiku`, `sonnet` and `opus`
move to new models over time, so instructions that work today can stop
working with no change to the repository. And a stronger or cheaper
model may serve a component better than the one its front matter names,
with no measurement to say so.

## Decision

- **Cases.** A component's cases live under
  `tooling/review-eval/cases/<component>/`, one TOML file each: the
  edits that make the fixture (a file created, text replaced, a file
  removed), the reports to seed for a generalist, the stakeholder's
  consent when a case grants it, and what the report must say: its
  outcome, each finding by path, severity and words, each finding it
  must not make, and text it must not contain. Every component has at
  least a defect, a clean change and an instruction addressed to the
  reviewer. The cases of each SRS section and domain-model file are
  derived from the tree, so they never rot; the others are written.
- **The world.** A run builds the review's own input: a worktree of
  main as the orchestrator's checkout, the component under test copied
  in from the tree being evaluated, a second worktree with the case's
  edits as `pr-head/`, its instruction files moved out as the workflow
  moves them, `pr.diff`, and `review/` from `review-bundle`, so the
  dispatch must bring the component or the case fails before any model
  runs.
- **The harness.** `claude --agent <component>` runs the component as
  the session, with the review's tools, write rule and turn budget, and
  `--model` names another model for a comparison. Each run has a
  deadline; a run that does not answer is counted and failed. The
  `stream-json` log gives the answer, the resolved model, usage, cost,
  turns, the files written and the agents called.
- **Grading.** The report format is fixed, so a grader decides without
  a judge model: the outcome line, each finding line, the report
  written, nothing else written. Each failed assertion is named, so a
  failing case reads like a failing test. A departure from the shape
  that the review survives, such as a label line or an answer that is
  more than the last line and the path, is a slip: counted and shown,
  never a failed run, so a scorecard says how well a model holds the
  shape without that drowning what it found.
- **Repetitions.** A case runs up to three times and passes with two
  passes, stopping as soon as the result is decided. Every run's usage
  and cost are kept per model.
- **What runs.** A pull request runs the cases of each component
  whose file, or whose cases, differ from main's; the shared sections
  are expanded into each file, so a shared change reaches every
  component that includes it. The evaluate job of `review.yml` runs
  after the review job, from main's definition with the same token the
  review has, and the post job merges its findings into the gate's
  deterministic findings. A weekly run evaluates every component on
  its model; a dispatched run takes a candidate model and compares.
- **The scorecard.** `tooling/review-eval/scorecard-v1.txt` records,
  per component, the model its front matter names, the model that
  resolved to, the cases and runs passed, cost, turns and duration, and
  the hash of what was evaluated. The stakeholder owns it. A comparison
  lists, per component, whether a candidate passes more cases, the
  same at lower cost, or fewer.

## Alternatives

- **`claude plugin eval`.** Cases as prompts with graders and an LLM
  judge, for a plugin. It gives a case read-only fixture directories
  and a scaffold script, but no review bundle, no way to run an agent
  as the session itself, and no usage per model in its results, so the
  dispatch, the report files and the cost would stay unproven, and its
  format is marked experimental.
- **An LLM judge.** A judge reads a report for quality the grader
  cannot see, such as whether the one sentence is apt. It adds a model
  whose own drift would need evaluating; the fixed report shape makes
  the deterministic grader enough for the gate. A judge may join later
  as a metric, never as the gate.
- **Running the evaluation in `ci.yml`.** CI runs as the pull request
  defines it, so a branch could rewrite the job around the token the
  evaluation needs. The evaluate job runs beside the review, from
  main's definition.
- **Running every suite on every pull request.** About seventy
  components and several hundred runs per change; the hash selection
  runs what changed, and the weekly run covers the rest.

## Consequences

- A change to a component costs its suite: a few dozen runs on its
  model, minutes on haiku. A change to a shared section costs every
  suite that includes it, which is the point of the section.
- The weekly run spends about one review's worth of model tokens per
  ten components; it runs from the stakeholder's subscription like the
  review.
- The `claude` command line is pinned in `mise.toml` for the harness,
  beside the action that runs the review.
- A new tooling crate, `review-eval`, with no new dependency: it reads
  its cases through `toml` and `serde`, both justified, deriving in
  tooling what the core never derives.
- A specialist's shared report section gains the trust rule the
  orchestrator and the generalists already carry, so its injection case
  has a rule to check.
