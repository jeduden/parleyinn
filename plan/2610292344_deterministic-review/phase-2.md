---
n: 2
title: "The review workflow runs review-run"
status: "✅"
result: false
---
# Phase 2: the review workflow runs review-run

Requirements. ENG-28 is reworded: the review MUST be run by code, not
by an orchestrating agent. A tool runs every dispatched specialist,
then every dispatched generalist, each as its own agent, and writes the
review outcome from what they answered. The gate still refuses a review
that left a dispatched agent out, now one whose run did not answer.
ADR-2610300031 proposes the decision and supersedes ADR-2610262245's
orchestrating agent once accepted.

BDD coverage. The ENG-28 scenario changes with the row. The job that
runs the review's agents holds no write permission and not the reviewer
app's key. It runs every dispatched agent with `review-run`, and no job
runs a harness session that would route the review. review-run's tools
and any `--allowedTools` an agent job passes write only under
`review/reports`. The post job runs after the agents' job. A drift case
proves the routing check fails on a review routed by
`claude-code-action`. No scenario is deleted or put back on `@pending`.

RED. Write tests for each of these, in order:

- The gate names a dispatched agent left out without an orchestrator.
- The workflow checks find the agents' job by `review-run`, refuse an
  harness session, and hold review-run's tool lists and every
  `--allowedTools` line to the reports.

GREEN.

- `review.yml`'s review job installs the pinned Claude Code command
  line as the evaluate job does, builds `review-run` before the Claude
  token is in any step's environment, and runs it offline with 32
  agents at once. Its outputs read `review/outcome.json` and
  `review/consulted.json`; the cost step reads `review/usage.json`. The
  orchestrator's step, its turn budget and its log parsing go.
- The gate's message, the ENG-28 row, scenario, binding and drift
  cases, the generalists' report shape (from PR #98), a new ADR and the
  docs that describe the review's flow.
- The generalists and specialists keep their models.

Gate. The ENG-28 scenario and its drift cases pass against the
workflow; `review-eval check`, mdsmith, rustfmt, clippy and the tests of
the changed crates are clean. A live run of `review.yml` on a pull
request needs the change on main, after the stakeholder's consent.
