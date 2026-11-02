---
n: 1
title: "Keep each pull request's tracked findings between reviews"
status: "🔲"
result: false
---
# Phase 1: keep each pull request's tracked findings between reviews

Why first. `review-run` tracks findings across reviews of a pull
request only when given `--previous`, and the review workflow never
gives it. So in CI every finding is new, none is resolved, and ENG-28's
stall after three reviews in a row never fires. Rulings and contests
(phases 2 and 3) build on that tracking, so it must work in CI first.

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
one whose run is `.github/workflows/review.yml` on the default branch,
started by `workflow_run`. It downloads it with `actions:read` alone.
Anyone can upload an artifact of that name from a workflow the pull
request defines. An artifact from any other run is skipped and counted
(I6), never read. `actions/cache` is not used: entries are evicted
after a week unused, and the review's jobs restore no caches.

RED. Write tests for each of these, in order:

- The lookup picks the newest artifact of the pull request's name whose
  run is main's review workflow, started by `workflow_run`.
- It skips an artifact of that name from another workflow, from a run
  of the pull request's own definition, or from another event, and
  counts each one skipped.
- It returns none when no artifact qualifies, an expired one among
  them, and the review's summary then says no earlier review was found.
- A malformed or oversized `findings.json` is refused with a named
  error, never read as an empty history.
- The workflow passes `--previous` with the file the lookup returned,
  and uploads the new `findings.json` under the pull request's name.

GREEN. The lookup lives in tooling, a subcommand of `review-run` or a
small `mdsmith-release`-style tool, called from one workflow step. No
inline shell logic: the step calls the tool. It reads the GitHub API
through a token with `actions:read` alone. It holds no Claude token and
runs before that token is staged. The tool takes the HTTP transport as
an injected trait, so its tests need no network. No new third-party
dependency. If a dependency proves unavoidable, it lands with an ADR
and its line in DEPENDENCIES.md (ENG-18, ENG-26).

Gate. The lookup's unit tests at the crate's coverage floor. The ENG-28
scenario passes with its new steps, and the drift case fails it. Once
on main, a second review of a real pull request reports its findings
as still open or resolved, not all new.
