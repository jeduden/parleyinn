---
n: 10
title: "A place spelled in the pull request's tree is the same place"
status: "✅"
result: false
---
# Phase 10: a place spelled in the pull request's tree

Requirements. ENG-28: its row does not change, and no scenario comes
off `@pending`. ENG-28 has the review decide a weighing partial when a
specialist's finding is at a place no line of the weighing is at. This
phase has the review read one place one way.

What happened. The review of pull request 154 blocked on one finding:
`review-code-deep` weighed no finding at four places its specialists
reported. It had weighed all four. Each of its lines named the place
as `pr-head/<path>:<line>`, where the work directory holds the pull
request's tree, and the review compared the places as text. The
evaluation's grader already takes that spelling, so a weighing the
evaluation passes was one the review blocked.

Why it happens. Every agent is told to read the pull request's files
as `pr-head/<its path>`. A generalist that reads a file there may name
it there too, and nothing tells it otherwise. The place is the same: a
line number is the pull request's, whichever spelling.

The rule. The review reads a place spelled under `pr-head/` as the
file at the rest of its path. A weighing line there weighs the place,
and the finding it keeps is posted, tracked and matched to a
requirement request at the repository's path. No instruction changes,
so no evaluation run is needed.

RED. Unit tests of `review-run`: a finding or dropped line at
`pr-head/src/a.rs:3`, or a line that names that place, weighs a
finding at `src/a.rs:3`; a blocking finding kept there from a
requirement request at `src/a.rs:3` is a nit at `src/a.rs`. The ENG-28
weighing check gains a fifth run, a generalist that spells its place
in the pull request's tree, which must approve with its nit kept at
the place.

GREEN. The outcome module of `review-run`: the check of a weighing
takes the spelling, and a generalist's kept findings are read at their
path in the repository. The ENG-28 scenario step names the new run.

Gate. Two drift cases fail: a check of a weighing that takes the
spelling for another file, and a kept finding left at its spelling.

Not in this phase: a specialist that spells its place there. A
generalist's line at the repository's path still weighs it, as a
shorter spelling of a reported path does, and none was seen.
