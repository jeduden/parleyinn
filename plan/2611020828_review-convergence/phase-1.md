---
n: 1
title: "Keep each pull request's tracked findings between reviews"
status: "✅"
result: false
---
# Phase 1: keep each pull request's tracked findings between reviews

Why first. `review-run` tracks findings across reviews only when given
`--previous`, which the workflow never did, so in CI every finding was
new and the stall never fired. Later phases build on it.

Requirements. This phase changes ENG-28. The review MUST keep the
tracked findings of each review of a pull request and pass the latest
earlier ones to the next review of it. It MUST read them only from an
artifact a run of main's review workflow uploaded, never from one
another workflow or a pull request's definition uploaded. A review
that finds no earlier findings MUST say so in its summary, never imply
a clean history (I6). ENG-28's row, scenario and drift cases change in
this phase. Those paths are code-owned, so the change waits for the
stakeholder's approval (ENG-21).

BDD coverage. No new scenario id. The ENG-28 scenario gains steps: the
review workflow passes `--previous` from the trusted lookup, and keeps
`review/findings.json` as a per-pull-request artifact. A drift case in
`tooling/drift/src/cases.rs` removes the lookup's origin check and
proves the scenario fails (ENG-27).

Storage. GitHub keeps artifacts per run, not per pull request. Each
review uploads `review/findings.json` and `review/findings.md` as
`review-findings-pr-<number>`, with `retention-days: 90`. The next
review lists artifacts by that name, newest first. It takes the first
one whose run is `.github/workflows/review.yml`, started by
`workflow_run` or `issue_comment`, the two triggers that run main's
definition. It downloads it with `actions:read` alone.
Anyone can upload an artifact of that name from a workflow the pull
request defines. An artifact from any other run is skipped and counted
(I6), never read. `actions/cache` is not used: entries are evicted
after a week unused, and the review's jobs restore no caches.

RED. Write tests for each of these, in order:

- The lookup picks the newest artifact of the pull request's name whose
  run is main's review workflow, started by `workflow_run` or
  `issue_comment`.
- It skips an artifact of that name from another workflow, from a run
  of the pull request's own definition, or from another event, and
  counts each one skipped.
- It returns none when no artifact qualifies, an expired one among
  them, and the review's summary then says no earlier review was found.
- A malformed or oversized `findings.json` is refused with a named
  error, never read as an empty history.
- The workflow passes `--previous` with the file the lookup returned,
  and uploads the new `findings.json` under the pull request's name.

GREEN. The lookup is `review-run previous select` and `previous check`,
subcommands of the crate that already reads the tracked findings. The
transport stays a thin script, `scripts/review-previous.sh`, which only
fetches through `gh` as its siblings do: it lists the artifacts and the
runs that uploaded them, and the tool decides which artifact to trust.
The step holds the job's read-only token and not the Claude token, and
runs before that token is staged. The tool takes files in place of an
HTTP client, so its tests need no network and no new third-party
dependency is added. The engineering sweep counted `review-run previous`
as an agent start, since it inlines scripts so none can hide one; it now
exempts that one command, which returns before the command line reads an
agent, and nothing longer or flag-like (`--previous`).

Gate. The lookup's unit tests at the crate's coverage floor. The ENG-28
scenario passes with its new steps, and the drift case fails it. The
live check, a second review on main reporting findings as still open
or resolved, is the first task of phase 2.
