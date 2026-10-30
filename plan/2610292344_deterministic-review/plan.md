---
id: 2610292344
title: "Deterministic review orchestration: code routes, Opus reviews"
status: "🔳"
summary: >-
  The review's orchestrating agent is replaced by code. A tool reads the
  review bundle's dispatch, runs each specialist and then each
  generalist as its own agent, and assembles the review outcome, the
  consulted list and the answers from what the run produced, so no
  model decides whether an agent is called or waited for. The
  generalists run on Opus and do the weighing (ENG-28).
model: opus
depends-on: [2610282358]
---
# Deterministic review orchestration: code routes, Opus reviews

## Goal

No model routes the review. Code reads `dispatch.json`, runs every
specialist, waits for every report, runs every generalist on the
reports, and writes the review outcome, so the gate's check that every
dispatched agent was consulted holds by construction. Opus, as the
generalists, does the weighing and the judging that code cannot.

## Context

The orchestrating agent (Sonnet, low effort, a skill) routes and
assembles. It failed to decide the review of three pull requests: it
took the reply "runs in the background" of a specialist call for an
answer, answered before the generalists were called, and the gate
refused its outcome. Wording fixes (jeduden/parleyinn#99) did not remove
the failure; the polling they added spends turns of a fixed budget. The
root cause is that a model does what a program does exactly: call every
name in a list and wait.

What was searched, and what is reused:

- `review-bundle` already computes the dispatch (each generalist with
  its specialists), each specialist's scoped diff and the class, and
  writes them as files. It is reused unchanged.
- `review-eval` already starts `claude --agent` through a host trait
  with the review's tools, write rule and turn budget, reads the
  stream-json log (answer, writes, cost, turns, agents called) and
  parses a report into findings. Its host, runner and log modules are
  reused by the new tool, not copied.
- `review-gate` already holds every input it needs: the outcome, the
  consulted list, `answers.json`, the reports and the dispatch. Its
  inputs stay as they are; only the producer changes.
- The orchestrator's two judgements move: consent and stakeholder paths
  are already deterministic findings of `review-bundle`, which the gate
  posts, so the orchestrator's copies of them go; the weighing of
  findings stays with the generalists.
- `claude-code-action` is not reused for the review: it runs one
  agent session. The evaluate job already installs a pinned `claude`
  command line for the same reason.

Not reused: asking a model to call a list of agents, which is the
failure this plan removes.

## Tasks

1. A `review-run` tooling crate: read the bundle's dispatch and
   scopes, run the specialists in parallel, then the generalists, each
   as `claude --agent` through the host trait with its own deadline;
   assemble `review/outcome.json`, `review/consulted.json` and
   `review/answers.json`; record each agent's usage.
2. The verdict is a pure function: `approve` exactly when every
   dispatched agent answered with a well-formed result and no
   generalist's kept finding is blocking. Findings are the generalists'
   kept lines, parsed with the report reader the evaluation already
   uses. The summary is composed from counts and each generalist's
   closing sentence.
3. `review.yml`: the Review step installs the pinned command line and
   runs `review-run`; the consulted-list step reads `review-run`'s
   output; the usage step reads its logs; the gate's inputs are
   unchanged.
4. The generalists, normal and deep, run on Opus; the specialists stay
   on Haiku at low effort.
5. Retire the orchestrator: its skill, its component in the reviewer
   evaluation (ENG-32) with its cases, its prompt and schema in the
   runner, and every mention in the SRS, the ADRs, CLAUDE.md and the
   docs.
6. ENG-28 is reworded: the review MUST be run by code that consults
   every dispatched specialist and generalist, not by an orchestrating
   agent. A new ADR supersedes ADR-2610262245's orchestrating-agent
   decision.

## Execution

| Phase | Model  | Gate                                                                                                 |
| ----- | ------ | ---------------------------------------------------------------------------------------------------- |
| 1     | opus   | `review-run` against a fake host produces the outcome, consulted list and answers; unit tests at 99% |
| 2     | sonnet | `review.yml` runs it on a pull request and the gate posts a decision from its output                 |
| 3     | sonnet | The orchestrator, its cases and its mentions are gone; ENG-28 and ENG-32 scenarios pass              |

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

| #   | Status | Phase                                                                                                                                                                                                                                                                                                                   |
| --- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | ✅     | [The review-run crate against a fake host](phase-1.md)                                                                                                                                                                                                                                                                  |
|     | ↳      | The review-run crate runs every dispatched specialist, then every generalist, as claude --agent through the evaluation's runner and process host, and writes the outcome, consulted list, answers and usage the review gate reads; unit tests cover every line, and an end-to-end test has the gate approve its output. |
<?/catalog?>

## Acceptance Criteria

- [ ] `review-run` runs every dispatched agent exactly once and writes
      the outcome, the consulted list and the answers the gate reads
- [ ] A review of a pull request is decided by the gate on its first
      run, with no orchestrating agent
- [ ] The generalists run on Opus
- [ ] The orchestrator skill, its evaluation component and its cases
      are gone, and `docs/`, the SRS and CLAUDE.md name none
