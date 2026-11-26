---
n: 14
title: "A derived defect plants one problem"
status: "✅"
result: true
summary: >-
  The case each SRS specialist gets for a MUST added with no change to
  its scenario adds a MUST that names its requirement and serves only
  what it traces. On the old sentence about two answers in five asked
  for the requirement to be fixed; on the new one none did. No
  requirement changes, and no scenario comes off @pending.
---
# Phase 14 handoff

## Done

- A requirement with a Traces field gains "Each failure to meet SEC-01
  MUST also be recorded in the audit log.", with its own id, and I6
  among its traces when it lacks it, in their order.
- An engineering requirement, which carries no Traces field, gains
  "Each failure to meet ENG-01 MUST also fail CI, naming ENG-01."

## Verified

- RED: the defect's unit test failed on the fixed sentence, and the
  trace's and the CI duty's tests on the functions they call. The test
  of an empty Traces field fails without the branch that reads it.
- `review-eval run` of the 13 SRS specialists that hold the case, on
  their models, against phase 16's tree of plan 2611211544:
  - the old sentence, on phases 16 and 17 of that plan: 21 of 53 runs
    asked for a change, most saying "It" and "the change" name nothing;
  - the MUST naming its id alone, run twice: 13 of 55 asked, most for
    I6 among the traces, and one only asked, failing; 396 of 401 runs,
    1.34 USD;
  - the MUST with its trace, every case run three times and all three
    held to pass: none of 38 asked; 295 of 296 runs, 1.00 USD.
- The run first taken for the old sentence planted the MUST naming its
  id alone: worktrees sharing a cargo target dir share its
  fingerprints, so the older tree ran the newer one's build. Each run
  is now built from its own tree's crates.
- The one failed run of the last read main's `features/` for SEC-28's
  scenario. srs-boundary-register and srs-peer-requirements, the two
  whose runs did so, then passed 6 of 6 under the gate's rule.
- The workspace's 4,651 tests, clippy denying warnings, rustfmt,
  `review-eval check` (236 cases, 74 components), `mdsmith check .`
  and the cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28` and
  `@ENG-32` pass: 4 scenarios, 63 steps.
- `scripts/review-local.sh` against phase 16 of plan 2611211544: the
  agents approve, 5 of 5 consulted, 0.35 USD. It blocks only on the
  consent `tooling/review-eval/` needs. Its evaluation plans no
  component: main's derived cases judge a pull request, so the runs
  above are this one's proof, and the weekly run takes the new MUST
  once it merges.

## Next

- A specialist that checks the scenario of a touched requirement
  searches for it, and 2 of 66 runs of the new MUST searched main's
  `features/`. Its context could name the scenario's place in the
  pull request's tree, as it quotes the requirement (ENG-28).
- 06b's first requirement, SEC-28, is one ENG-29 holds to an ADR, so
  its case drew that finding too in one run of two on either sentence:
  a true second finding, which cites SEC-28 and fails nothing.
