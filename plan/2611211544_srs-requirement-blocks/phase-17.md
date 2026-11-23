---
n: 17
title: "The review's agents say requirement"
status: "🔳"
result: false
---
# Phase 17: The review's agents say requirement

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 4's last pull request. Where a reviewer component, or a
shared section it includes, calls a requirement a row, it says
requirement. A catalog's `row:` template and a table that stays keep
their name.

The shared sections. srs-section-checks.md holds each touched
requirement to the section, its scenario, the code and the invariants,
and a finding on one ends in its id. review-spec.md and review-code.md
ask their questions of each touched requirement. outcome-answer.md,
which the report sections include, says a fine place gets no line.

The agents' own text. Each SRS specialist reads only the requirements
and text the pull request touches or names. lang-gherkin holds a
scenario to its requirement as the context quotes it, performance reads
the requirements a change touches, and test-engineer-end-to-end proves
the requirement's MUST.

RED. None: the wording asks nothing new of a reviewer, so main's cases
are the gate, each run on its component's model.

GREEN. mdsmith writes every agent file from its shared sections again,
and each component the evaluation plans passes its cases on its model.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- `review-eval plan` against phase 16 names the components the change
  reaches, and `review-eval run` passes each one's cases on its model,
  with the slips of each counted;
- `review-eval check`, the workspace's tests and the cucumber scenarios
  `@ENG-28` and `@ENG-32` pass;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
