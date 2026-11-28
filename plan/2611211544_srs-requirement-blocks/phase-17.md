---
n: 17
title: "The review's agents say requirement"
status: "✅"
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
ask their questions of each touched requirement. outcome-answer.md and
specialist-report.md, the report sections' two copies of the answer
rule, say what is fine gets no line.

The agents' own text. Each SRS specialist reads only the requirements
and text the pull request touches or names. lang-gherkin holds a
scenario to its requirement as the context quotes it, performance reads
the requirements a change touches, and test-engineer-end-to-end proves
the requirement's MUST.

The threat model's specialist. #179 landed `srs-threat-model` ahead of
its section, and #178 put 06a on main. It takes this phase's text with
the other SRS specialists, and the final text phase 13 left to a later
pull request: it links its section as `srs-boundary-register` does,
and the paragraph saying the file is not on main yet goes. Its own
cases still bring a file of their own, which the review bundle hands it
by the file's name, as `srs-review-requirements`' cases do.

RED. None: no case can fail first. A slip fails no run, and no case
plants an id a specialist should be unsure of. Main's cases are the
gate, each run on its component's model, beside the same cases run on
phase 16's text.

GREEN. mdsmith writes every agent file from its shared sections again.
Each component the evaluation plans passes its cases on its model, and
so does each the plan covers through a component on another model.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- `review-eval plan` against phase 16 names the components the change
  reaches, and `review-eval run` passes each one's cases on its model,
  with the slips of each counted;
- `review-eval check`, the workspace's tests and the cucumber scenarios
  `@ENG-28` and `@ENG-32` pass;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
