---
id: ADR-2610300031
title: "The review is run by code, not by an orchestrating agent"
status: accepted
summary: >-
  The review-run tool runs every specialist the review bundle
  dispatches, then every generalist, each as its own agent, and writes
  the review outcome from what they answered, so every dispatched agent
  is consulted by construction. It replaces the orchestrating agent of
  ADR-2610262245; the layers, reports, gate and required check stay.
---
# ADR-2610300031: The review is run by code

Accepted on the stakeholder's consent (see Approval). It supersedes
ADR-2610262245, which is marked superseded and names this record; the
layers, reports, gate and required check that record decided stay as
this record's "What stays" says.

## Context

ADR-2610262245 has one orchestrating agent, on Sonnet at low effort,
call the specialists the review bundle dispatches and then the
generalists, and answer with the review outcome. The gate refuses a
review whose run log shows a dispatched agent left out.

That agent failed to decide the review of three pull requests. It
took a specialist call's reply that the call runs in the background
for an answer. It answered before the generalists were called, and
the gate refused its outcome. Rewording its skill
(jeduden/parleyinn#99) did not remove the failure, and the polling
that change added spends turns of a fixed budget. Calling every name
in a list and waiting for each is what a program does exactly; a model
does it only most of the time.

The pieces a program needs were already there. `review-bundle` writes
the dispatch. `review-eval` starts one agent as `claude --agent` with
the review's tools, write rule and turn budget, and reads its run log.
`review-gate` reads the outcome, the consulted list, the answers and
the reports.

## Decision

- **Code routes the review.** A tooling crate, `review-run`, reads the
  bundle's dispatch, runs every dispatched specialist, many at once,
  and, once every one has ended, every dispatched generalist, told its
  specialists' report files. Each agent runs through
  `review-eval`'s runner and process host with a 900-second deadline,
  and runs once more when its run did not answer: it would not start,
  ended without a result, missed its deadline or answered nothing. A
  report the agent wrote is judged, never rerun; the usage file counts
  the attempts.
  No agent routes the review, calls another agent or writes the
  review outcome.
- **The verdict is a pure function, and fails closed.** `review-run`
  approves exactly when every dispatched agent answered, every
  specialist's report passes the gate's report check and none ends
  with `outcome: blocking`, every generalist's answer holds a finding,
  a dropped line or a `none —` line, and no generalist kept a blocking
  finding. The findings are the generalists' kept lines; a line whose
  first word is `dropped`, or whose severity is, is left out, by the
  rule the evaluation's grader shares. A silent agent, a refused or
  blocking report and an answer that says nothing readable are each
  named in a blocking finding, so neither a silent generalist nor one
  that does not echo a specialist's blocking report passes the
  review.
- **A change no rule routes gets one generalist.** The review bundle
  dispatches `review-code`, or its deep instance, so something always
  reads the change. A dispatch that is empty all the same requests
  changes.
- **Stakeholder consent is checked in code.** The review bundle raises
  the ENG-21 finding for a change to a path CODEOWNERS assigns: blocking
  until a stakeholder comment consents, a nit once one does.
- **The gate's inputs stay.** `review-run` writes
  `review/outcome.json`, `review/consulted.json` (every agent that
  answered), `review/answers.json` and `review/usage.json`, the files
  the gate already reads. The gate still refuses to post a review that
  left a dispatched agent out, now a review whose run of that agent
  did not answer.
- **The models stay.** Each agent runs on the model, effort and turn
  budget its front matter names; `review-run` passes no model.
- **The workflow.** The review job installs the pinned Claude Code
  command line as the evaluate job does, builds `review-run` before the
  Claude token is in any step's environment, and runs it offline. The
  job keeps its read-only permissions, holds no reviewer-app key and
  hands the agents no GitHub token.
- **What stays.** The layers, the deep instances, the reports as files,
  the post job's gate, the required `review-gate` check and the
  triggers stay as ADR-2610262245 has them.

## Alternatives

- **Keep the orchestrator and reword its skill again.** jeduden/parleyinn#99
  tried it; the failure stayed, and each reword is a guess a fixed
  turn budget pays for.
- **Move the orchestrator to Opus.** A stronger model fails less
  often, but still decides whether to wait, and costs more on every
  review.
- **Keep `claude-code-action` and give it a script.** The action runs
  one agent session; it cannot start an agent per dispatched name
  and wait for each.

## Consequences

- ENG-28 asks for the review to be run by code, not by an orchestrating
  agent, and its scenario holds the review workflow to `review-run`
  and refuses an agent session that would route the review.
- The orchestrator's skill, its reviewer component under ENG-32, its
  cases and its prompt and schema in the runner are retired (the
  skill's file goes in a follow-up once main runs `review-run`), together
  with its mentions in the SRS, the docs and the generalists' input
  section. The accepted records that describe it, ADR-2609301941,
  ADR-2610262245 and ADR-2610290002, keep their text as the record of
  what was decided then.
- A review runs in waves of agents, each up to 900 seconds and each
  agent at most twice. The repository has 56 specialists and the job
  starts 32 at once, so they take two waves and the generalists a
  third: 3 x 2 x 15 = 90 minutes at worst, and the job's cap is 120.
  Agents end in minutes in practice.
- No new third-party dependency.

## Approval

The stakeholder, `jeduden`, gave it in a comment on the pull request
(<https://github.com/jeduden/parleyinn/pull/98>, comment 6534291701). The
comment consents to the specification, scenarios, decision records,
`.github/`, `docs/agents`, `.claude/` and evaluation-case changes. It
also asks for a ruling on this record. The stakeholder confirmed in the
session that the comment is consent. This record changes the review's
workflow under `.github/` and ENG-28, which need that consent (ENG-21).
