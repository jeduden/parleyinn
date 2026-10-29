---
summary: >-
  How the review's agents are held to evaluation cases (ENG-32): the
  cases each reviewer component has, the world and harness a run builds,
  the deterministic grader, what the review of a pull request runs, the
  weekly run against the published scorecard, and how a candidate model
  is compared before a component moves to it.
---
# Evaluating the reviewers

The review's agents are instructions in Markdown, and ENG-32 holds each
of them to cases the way code is held to tests. A case is one change to
main's tree with the report the component must give for it. Every
component the review bundle can dispatch has at least three: a defect it
must find, a clean change it must pass, and a comment addressed to the
reviewer, which it must report and not obey. The orchestrator and each
deep instance are among them. The `review-eval` crate under
[tooling/review-eval](../tooling/review-eval) runs them.
[ADR-2610290002](adr/ADR-2610290002-reviewer-evaluation.md) records the
design.

## The cases

- **Written cases** live under `tooling/review-eval/cases/<component>/`,
  one TOML file each. A case names the files it creates, the text it
  replaces, the reports it seeds for a generalist, the stakeholder's
  consent when it grants one, and what the report must say. A finding
  is matched by its path and the words, severity, ids and source the
  case names; `none` forbids one; `absent` forbids text. Cases belong to
  the normal instance of a generalist; its deep instance runs the same
  ones.
- **Derived cases** come from the tree for each SRS section and
  domain-model file: a requirement row changed without its scenario, a
  cited id no section defines, a true sentence under the title, and the
  same with a comment addressed to the reviewer. They never rot.

## A run

- **The world** of a run is the review's own input: a worktree of main
  with the component under test copied in, the case's edits in a second
  worktree as `pr-head/`, its instruction files moved out, `pr.diff`,
  and `review/` from `review-bundle`. The dispatch must bring the
  component, or the case fails before any model runs.
- **The harness** starts `claude --agent <component>` with the review's
  tools, write rule and turn budget, on the component's model or the one
  `--model` names; the orchestrator runs as the workflow runs it. It
  refuses a component whose front matter names a key outside name,
  description, tools, model, effort, maxTurns and generalist, or a tool
  its kind may not use, so a definition under test cannot start a
  process or widen its reach through the run (I4).
- **The grader** is deterministic. It reads the report: its outcome,
  each finding expected or forbidden, nothing else written, and, for
  the orchestrator, every dispatched agent consulted. A departure from
  the report's shape that the review survives is a slip: a label line
  between the findings, an answer that is more than the last line and
  the path, an eleventh finding. Slips are counted and shown per run
  and fail none. A case passes with two of three runs, stopping once
  decided. A run that does not answer, or runs past its deadline, is
  counted and failed.
- **The outputs** of a run go under `--out`: `scorecard.json`,
  `findings.json` in the shape the review gate reads, `report.md`, and
  each run's log, report and grade under `runs/`.

## The commands

`cargo run -p review-eval -- check` applies every case to the checkout and
confirms the dispatch, without a model. The engineering crate's gate tests run
the same check on every pull request, each with its drift case (ENG-27). `cargo
run -p review-eval -- run --component lang-rust --out DIR` runs one component's
cases on its model; it needs the `claude` command line and its login, and clears
what a Claude Code session sets for its children, so an agent may run it too.
`--model` names another model for every component, `--repetitions` and
`--passes` change the rule, and `--parallel` the cases run at once. `compare`
reads a run's scorecard against the published one, and `publish` writes it into
it.

## In the review of a pull request

The `evaluate` job of [review.yml](../.github/workflows/review.yml)
runs the cases of each reviewer component the pull request changes, on
the model its front matter names, from the pull request's definition of
it. The review job plans what changed with `review-eval plan`. The
evaluate job runs after the agent's job, holds the agent's token and no
write permission, and ends before the job that holds the reviewer app's
key. A case that fails becomes a blocking finding, which the gate weighs
beside the bundle's. With nothing to evaluate the job ends in seconds.
A component main already has is judged by main's cases; the pull
request's cases judge only the components it adds, so a change cannot
soften the cases that judge it.

## The weekly run and a candidate model

[review-eval.yml](../.github/workflows/review-eval.yml) runs every
component's cases on its model weekly. It compares the result with the
scorecard the repository publishes,
`tooling/review-eval/scorecard-v1.txt`, so a model that stops serving
its instructions shows up as failed cases. Run from the Actions "Run
workflow" button with a `model`, it runs the cases on that model instead
and writes the comparison into its summary. That is how a candidate
model is weighed before a component moves to it. Both run from main's
definition. The Claude Code command line both start is an npm package,
which the mise rule for released binaries refuses, so each workflow pins
its version in `CLAUDE_CODE_VERSION` (ENG-01).

## Changing a component or a case

A new or changed component lands with its cases in the same change. The
pull request's review runs them before the gate decides. A case that
keeps failing is a finding about the instructions or about the case,
never a reason to loosen it. The cases and the published scorecard are
stakeholder paths ([CODEOWNERS](../.github/CODEOWNERS)). The
`reviewer-shape` skill drives such a change, a failing case first, and
the `reviewer-engineer` agent reviews it.
