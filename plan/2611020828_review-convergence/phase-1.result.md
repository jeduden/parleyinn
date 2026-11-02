---
n: 1
title: "Keep each pull request's tracked findings between reviews"
status: "✅"
result: true
summary: >-
  ENG-28 now has the review carry the previous review's findings: each
  review uploads them as review-findings-pr-NUMBER, the next one fetches
  the newest that a run of main's review workflow uploaded and passes it
  as --previous, and an artifact of that name from any other run is
  refused and counted. The ENG-28 scenario step, five drift cases and
  ADR-2611021000 land with it; the live check waits for main.
---
# Phase 1 handoff

## Done

- `review-run previous select` and `previous check`
  (`tooling/review-run/src/previous.rs`): the newest unexpired artifact
  of the pull request's name whose run is `review.yml` on `workflow_run`
  or `issue_comment` wins; every other artifact of the name, and one
  whose run is unknown, is counted as skipped; a file past 1 MiB or
  one that does not read is named. Red commit first, then green.
- `scripts/review-previous.sh` only fetches, through `gh`; the review
  job gained `actions: read`, fetches before the Claude token is staged,
  passes `--previous`, scrubs `review/findings.*` and uploads them under
  the pull request's name for 90 days.
- The ENG-28 sweep treated `review-run previous` as an agent start,
  since it inlines scripts. It now exempts that one command, never a
  longer word or a flag (`--previous`); a test pins that.
- ENG-28's row and scenario carry the new step, bound to
  `engineering::review_previous`; five drift cases guard it. The three
  older drift cases that inject into the Review step's text follow its
  new shape. ADR-2611021000 records the decision; the existing ADR
  stays under its 300-line limit.

## Verified

- Cucumber scenarios `@ENG-18 @ENG-26 @ENG-27 @ENG-28 @ENG-32` pass;
  `cargo test` for `review-run`, `engineering`, `drift` and `srs`,
  and clippy on the workspace, are clean; `mdsmith check .` passes.
- Each new drift case was applied by hand and caught with its message.
  The drift suite itself did not run: it needs `taplo`, absent here.

## Not verified

- The live check: a second review of a real pull request on main
  reporting findings as still open or resolved. The job never ran in
  CI, so the `actions: read` grant, `unzip` of the artifact and the
  `gh api .../zip` call are untested outside this tree. Phase 2 should
  not start until one real review has shown it.
- `frit doctor`, `scripts/review-local.sh` and the per-crate coverage
  floors did not run here.

## Inherits

- Phase 2 (a stakeholder ruling settles a finding) reads comments with
  the ENG-21 consent parser in `tooling/review-bundle/src/owners.rs`
  and extends `findings.rs`; the tracked finding's identity is in
  `findings/identity.rs`. Both paths are code-owned for ENG-28's row.
