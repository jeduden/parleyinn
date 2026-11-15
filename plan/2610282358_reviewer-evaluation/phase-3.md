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

What the measurement on the branch showed. Four runs of each of 241
cases: 15 of 964 runs failed, 14 cases once and one twice. Every failed
run was read:

- 4 runs: a place carried the diff file's line number, not the file's.
  A reviewer's tool numbers the lines of the diff it reads, and 83 of
  826 places took that number.
- 1 run: a nit stated after a blocking finding at the same place
  replaced it, as the review reads a later line there.
- 3 runs: a clean fixture that drew a fair remark.
- 1 run: the invariants' specialist could not see the ADR the change
  lands with, which its own diff does not hold.
- 3 runs: the right finding in other words or at another fair place
  than the case's.
- 2 runs: a finding line that did not hold its form.
- 1 run: a request where the case wants a finding.

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
- the review bundle, which writes each line's number in the file into
  the diffs an agent reads, and names the ENG-29 record in a
  specialist's context;
- the prompt, which says a place takes the number its diff line starts
  with;
- the generalists' report rule, where a place takes one line;
- [review-evaluation.md](../../docs/review-evaluation.md), which states
  the rules a stable case holds to.

Gate: `review-eval check` passes, and every failed run of four runs of
every case on the branch is read and traced to its cause, recorded in
the handoff. Publishing the first scorecard stays with the first
scheduled run that passes on main.
