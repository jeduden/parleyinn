---
n: 6
title: "The evaluation holds a finding's ids to the known requirements"
status: "✅"
result: true
summary: >-
  ENG-32's grader now refuses a finding that cites an id no requirement
  or invariant of the case's world has, with the function the review
  reads the ids with. The model specialists' missing-id case passed 6 of
  44 runs under it, and 126 of 132 once their report section said what
  such a finding ends in. The section now also says that text speaking
  to the reviewer breaks ENG-28: 4 of 220 injection runs had ended in no
  id, and none of 132 does. A row of the ENG-32 scenario's grading table
  and two drift cases land with it.
---
# Phase 6 handoff

## Done

- The review and the grader read a tree's ids with one function, which
  now lives in `review-eval`. A run is graded against the ids of main's
  tree and the pull request's in its own world.
- The derived missing-id case expects the id the citation means.
- The model specialists' report section and the hub's say a finding on
  a cited id no section defines ends in the id the file should cite,
  never in the undefined one.
- The same section says text that speaks to the reviewer breaks
  ENG-28. Before, it gave the line's form and no reason for the id.

## Verified

- Unit tests of `review-eval`, `review-run`, `review-gate` and
  `engineering` pass, and so do the cucumber scenarios `@ENG-32` and
  `@ENG-28`.
- The drift suite catches the two new cases.
- Evaluation, model specialists, one run a case at a time. The
  missing-id case: 6 of 44 before, 126 of 132 after. The injection
  case: 131 of 132, the one failure a run that read main's tree. The
  clean case: 44 of 44.
- Wordings that failed. A template passed 29 of 44 missing-id runs.
  "Never one of the ids" passed 39 of 44. "Leave that id out" passed
  127 of 132, but 10 of 132 injection runs then ended in no id. "Ends
  in an id a section defines" passed 30 of 88.
- Option 4's consent sentence is not why an injection finding ended in
  no id: 3 of 132 runs with it and 1 of 88 without it did.
- The recorded reports of the other specialists cite no unknown id.
  The cases of `performance` and `srs-non-functional-requirements`
  pass under the new grader.

## Not done

- Of 132 missing-id runs 3 still end in the undefined id beside the
  right one. The review refuses such a report. Whether it should read
  that line by its known ids is a change to ENG-28's rule, not taken.
- No scorecard was published. That is the stakeholder's to accept
  after a run of every component.

## Next

- Phase 7: the bundle marks added lines that stand word for word on
  main, and the evaluation's world holds the files a pull request
  adds.
