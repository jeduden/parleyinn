---
n: 1
title: "A finding line off its form is read as the finding it states"
status: "🔳"
result: false
---
# Phase 1: a finding line off its form is read as the finding it states

Requirements. ENG-28: its row says what the review makes of each line
of a specialist's run log. No scenario comes off `@pending`: `@ENG-28`
runs already, and its step on the reading of every log gains the new
clause with the row. ENG-32's row is unchanged: a specialist's report
is graded as the review reads it.

The rule. A line at a place that starts with `if not addressed:` or
`request:` is a finding its specialist meant to state. Three near
forms name everything the form asks for:

- a linked finding whose last part starts with its ids and goes on,
  `… — ENG-11 (provisional)` or `… — ENG-04 and I9`;
- a linked finding with no part of ids, whose sentence ends with them,
  `… the test sleeps, breaking ENG-03.`;
- a request with no dash before its motivation, or with none.

The review reads each as the finding it states. It writes the line in
its form in the report, so the generalist, the gate and the stability
report see one shape. It keeps a note beside it,
`note: read off its form: <line>`, lists the note in
`review/ignored.json` and counts it apart in its summary (I6). An id
no requirement has still blocks, as it does in the form.

What stays refused: a line that names no id, one that says nothing
beside its ids, and one missing the dash after its place. None of them
names what a finding needs, or the reading would be a guess.

RED. Unit tests in `tooling/review-gate/src/report/near/tests.rs` for
each near form, for what stays malformed, and for the note the assembly
writes. Tests in `review-run` for the listed note and the summary's
count, in `review-eval` for the grader's slip, and a new kind of log,
`off-form`, in the ENG-28 record check.

GREEN. `report.rs` reads with the near forms and says when a line read
only through one (`off_form`); `near.rs` holds them; `stated.rs` sets
the note aside with its own reason. `review-run` tallies the two kinds
of listed line. The grader names the slip.

Gate. The engineering crate's record check of the review's reading of
every log, in `review_logs`, runs `review-run` and the gate on a
specialist whose one finding is off
its form: the report holds it in its form, the note is listed and
counted, and the gate posts an approval. Three drift cases break the
reading, the note and the count, and each fails that check.

Not changed: the reviewers' instructions. They still ask for the form
and say text after the ids refuses a report. The form is still what a
reviewer should write; the reader forgives a slip and counts it.
