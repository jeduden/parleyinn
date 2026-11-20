---
n: 1
title: "A finding line off its form is read as the finding it states"
status: "✅"
result: true
summary: >-
  ENG-28 now has the review read a finding line that names its place
  and ids off its form as the finding it states: a remark after the
  ids, a sentence that ends with them, a request with no motivation set
  apart. The report holds it in its form, and the slip is listed in
  review/ignored.json and counted apart. A line that names no id still
  blocks. The ENG-28 scenario step, a new kind of log in its record
  check and three drift cases land with it.
---
# Phase 1 handoff

## Done

- ENG-28's row states the rule, and its scenario step on the reading of
  every log carries it. `@ENG-28` ran before and runs now; no scenario
  came off `@pending`.
- The reader takes the three near forms and nothing wider. A remark
  after the ids joins the consequence. A sentence that ends with its
  ids is the consequence, whole. A request with no dash before its
  motivation is cut at `;`, `:`, a full stop, `since` or `because`, as
  it was since #132, and one with no motivation reads with
  `no motivation given`.
- The assembly writes each such finding in its form and a note beside
  it, `note: read off its form: <line>`. The review lists the note in
  `review/ignored.json` with its own reason, and its summary and
  progress count it apart from the lines set aside as no finding (I6).
- An id no requirement has still blocks, read off its form or in it.
  The problem "writes more after its ids" is gone: nothing gives it.
- The grader counts the slip `read off its form, as the finding it
  states` and grades the finding, so ENG-32's rule that a report is
  graded as the review reads it holds unchanged.

## Verified

- Unit tests of the reader, the review's reading and summary, and the
  grader: 1,924 tests of the six crates touched pass.
- The cucumber scenarios `@ENG-28` and `@ENG-32` pass.
- The ENG-28 record check runs `review-run` and the gate on a
  specialist whose one finding is off its form: the report holds it in
  its form, the note is listed and counted, the gate posts an approval.
- The drift suite, on the 47 cases that inject a file this phase
  touches, catches each. Three are new: the near reading removed, the
  note unwritten, the count left out.

## Next

- The reviewers' instructions still say text after the ids refuses a
  report. That keeps them writing the form. If the count of findings
  read off their form stays near what it was, about one line in two
  hundred runs, leave it; if it grows, the instructions want a look.
- A line with no id, 92 on record, stays a blocking finding. Reading
  one would be a guess at what it breaks.
- Phase 2: a generalist's blocking finding stands until a `dropped`
  line withdraws it.
