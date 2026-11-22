---
n: 12
title: "A read the file system resolves to the diff covers it"
status: "✅"
result: true
summary: >-
  ENG-28's review now counts every read of a specialist's diff, by
  whatever path the file system resolves to it. The local review of
  §2, §5, §5a and §7 written as blocks had named two specialists' parts
  partial that read their whole diffs: one by a path that climbed out
  of the pull request's tree, two kept on a resume and read under the
  first run's work directory. A climb through a link the pull request
  plants still covers nothing. No requirement or instruction changes.
---
# Phase 12 handoff

## Done

- A read covers a specialist's diff when the file system resolves its
  path to that very file, a relative path taken under the work
  directory. A read of `pr-head/../review/diffs/<name>.diff` covers it.
- A read under the directory the run's log names it started in is
  taken under the work directory, so a part a resumed review keeps is
  held to its diff as it was when it ran.
- A climb through a link in the pull request's tree names the file the
  link reaches, as the file system resolves it, never the diff.

## Verified

- Unit tests of `review-run`: a climb out of the pull request's tree,
  relative and absolute, covers the diff, and a kept part read under
  its run's directory is whole. Both failed before the change. A climb
  through a link to a planted file of the same name covers nothing,
  before and after.
- The tests of `review-run`, `drift` and `engineering` pass, with
  clippy denying warnings and rustfmt. The cucumber scenarios
  `@ENG-27` and `@ENG-28` pass, and `mdsmith check .` passes.
- The drift case that has the review no longer decide from a
  specialist's reads what it read moved to the changed call. The drift
  suite catches it and the 12 other cases that inject a file this
  phase touches.
- `scripts/review-local.sh` against main: 6 of 6 agents consulted, no
  finding, 0.25 USD. Its one blocking finding is the consent this
  change needs, as it touches `tooling/review-run/`.
- With this change, the local review of plan 2611211544's phase 2,
  which had blocked on the false partial read, approved: 7 of 7 agents
  consulted, no finding, 0.22 USD.

## Next

- None in this phase. The open phases of this plan are 4 to 7.
