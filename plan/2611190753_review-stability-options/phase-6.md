---
n: 6
title: "The evaluation holds a finding's ids to the known requirements"
status: "✅"
result: false
---
# Phase 6: the evaluation holds a finding's ids to the known requirements

Requirements. ENG-32: its row has a specialist's report graded as the
review reads it (ENG-28). The row does not change. No scenario comes
off `@pending`: `@ENG-32` runs already, and its grading table gains a
row.

What the record shows. The review holds a specialist's linked finding
to the ids its two trees define, and refuses a report whose line cites
another. The grader held the ids to their shape alone. Each model
specialist has a derived case whose file cites an id no section
defines. Of 45 recorded findings in that case, 38 ended in the missing
id. The review would have refused each of those reports, and the
grader passed them.

RED. A unit test of the grader: a finding that cites an id no
requirement of its world has fails its run. Under that grader the
missing-id case passed 6 of 44 runs with the instructions as they
stood.

GREEN. One function reads a tree's ids for the review and for the
grader. A run is graded against the ids of its own world. The derived
case expects the id the citation means. The model specialists' report
section says what a finding on such a citation ends in.

Evaluation. Every wording is measured on the model specialists' cases,
one run a case at a time, so a pass rate is runs passed of runs made.
The injection case is watched beside the missing-id case: a rule about
ids can move both.

Gate. The record check of `@ENG-32` grades a report whose finding
cites a known and an unknown id, and expects the run to fail on the
report. Two drift cases fail: a grader that holds ids to their shape,
and a run graded without the ids of its world.
