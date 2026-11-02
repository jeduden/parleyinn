---
id: ADR-2611021000
title: "A pull request's tracked findings are kept as an artifact only main's review workflow may write"
status: proposed
summary: >-
  The findings the review tracks across a pull request's reviews are
  kept as the artifact review-findings-pr-NUMBER for 90 days, and the
  next review fetches the newest one that a run of main's review
  workflow uploaded and passes it as the previous review. An artifact of
  that name from any other run is refused and counted. It builds on
  ADR-2610300031.
---
# ADR-2611021000: Findings between reviews

## Context

ADR-2610300031's `review-run` tracks each kept finding across the
reviews of one pull request. A finding is new, still open or resolved.
A blocking finding kept three reviews in a row, its cited file
unchanged, needs a stakeholder ruling. It tracks only when given the
previous review's `review/findings.json` through `--previous`. The
review workflow never gave it one, and a job's files end with the job.
So in CI every finding was new, none was resolved and the stall never
fired.

GitHub keeps artifacts per run and not per pull request. An artifact
name is no proof of origin: any workflow a pull request defines may
upload one of the same name, so a pull request could plant a previous
review that marks its own blocking findings resolved (I2, I8).

## Decision

- **Kept per pull request.** A review that decided uploads
  `review/findings.json` and `review/findings.md`, scrubbed of the
  Claude Code OAuth token first, as `review-findings-pr-<number>`, for 90 days.
- **Fetched by origin.** `scripts/review-previous.sh` lists every page
  of the artifacts of that name and the runs that uploaded them,
  through the job's read-only token. `review-run previous select`
  takes the newest unexpired one whose run is `.github/workflows/review.yml` on
  `workflow_run` or `issue_comment`, the two triggers that run main's
  definition. Each other artifact of the name is refused and counted,
  and so is one whose run is unknown. `review-run previous check`
  refuses a file past 1 MiB or one that does not read; the script cuts
  the unzipped file one byte past that, so none fills the runner's disk.
- **Said, never silent.** The job summary names the artifact carried,
  or says no earlier review was found, or that one did not read, so
  every finding is new (I6). A missing history never fails the review.
- **The step.** It runs after `review-run` is built and before the
  Claude Code OAuth token is staged, holds no such token, and the job
  gains `actions: read` and no write permission. Only a review whose
  scrub exited 0 and whose steps all succeeded uploads its findings.
- **The sweep.** The ENG-28 check treats a step that runs `review-run`
  as an agent start. It exempts the one command `previous`, which
  returns before the command line reads an agent, and nothing longer
  or flag-like.

## Alternatives

- **`actions/cache` keyed by pull request.** Entries are evicted after a
  week unused, a cache written on the default branch is read by other
  runs, and the review's jobs restore no caches.
- **A marker in the posted review.** It needs no storage, but a pull
  request comment would become an input of the review, parsed by code.
- **Trust the artifact's name.** Simplest, and the hole above.
- **An HTTP client in `review-run`.** It would add a dependency
  (ENG-18) to replace `gh`, which the sibling scripts already use.

## Consequences

- ENG-28 asks for it, with its scenario step and six drift cases.
- A pull request open past 90 days loses its history; the next review
  says so and counts every finding new.
- Rulings (a stakeholder settling a tracked finding) build on this and
  follow in plan 2611020828.

## Approval

This record is proposed. The stakeholder accepts it, or not, on the pull
request that carries it.
