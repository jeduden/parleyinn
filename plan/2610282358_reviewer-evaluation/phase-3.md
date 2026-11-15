---
n: 3
title: "Stable cases, so the scheduled run passes"
status: "🔳"
result: false
---
# Phase 3: stable cases, so the scheduled run passes

Requirements. The scheduled half of ENG-32: a scheduled run runs every
component's cases on its model and fails naming each component whose
cases fail. It failed on each of its first three weeks, and the same
cases blocked pull requests that did not change them. ENG-35 is new
here: the domain model's terms rule as a requirement a finding can
cite.

What the first full measurement showed. Four runs of each of 239 cases
on main, 956 runs: 83 failed, and 50 cases failed at least one run.
Every failed run was read. None was noise: each came from a choice the
instructions or the case left to the reviewer.

- 31 runs: a true concern with no requirement to cite, most of them
  the derived case for a sentence in excluded terms. A blocking finding
  ends with the id it breaks (ENG-28), and no requirement stated the
  terms rule, so the case had no right answer.
- 18 runs: the right finding, its path written as the file's name
  alone.
- 12 runs: a persona's clean case was a plan, which is never clean to
  a persona.
- 5 runs: a fixture called clean that held a true defect.
- 4 runs: a request written without its dash, which the review has
  read since.
- 8 runs: the right finding in other words than the case's.
- 5 runs: the reviewer's own slips, which the cases are there to catch.

BDD coverage: `@ENG-35` lands `@pending`; `@ENG-32` is unchanged.

RED: the derived case for excluded terms expects a finding citing
ENG-35. The grader's unit tests take a finding at a tail of the case's
path.

GREEN sites:

- the ENG-35 row;
- the specialists' report rule under `docs/agents`, where one question
  decides a finding's form;
- the grader, which knows a finding by its place, ids and source;
- the fixtures;
- [review-evaluation.md](../../docs/review-evaluation.md), which states
  the rules a stable case holds to.

Gate: `review-eval check` passes, and every failed run of four runs of
every case on the branch is read and traced to its cause, recorded in
the handoff. Publishing the first scorecard stays with the first
scheduled run that passes on main.
