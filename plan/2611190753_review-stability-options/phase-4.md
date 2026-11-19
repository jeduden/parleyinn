---
n: 4
title: "A review a cost cap stopped says what to do"
status: "✅"
result: false
---
# Phase 4: a review a cost cap stopped says what to do

Requirements. ENG-28: its row has the step that posts fail without
posting a review when a dispatched agent did not run. No scenario
comes off `@pending`: `@ENG-28` runs already, and it gains one step
with the row's new sentence.

What the record shows. On pull request 136 the reviews had spent 4.10
of the 5 USD a pull request may cost. The three generalists did not
start, and the gate did not decide. The check it failed said only that
the gate failed before deciding and that a job log says why. The
review outcome did name each agent with its reason, the spend and the
cap, but the gate posts no review of such an outcome. Neither place
said how the spend is reset. The author had to read a job log, then
`docs/ci.md`.

The rule. The gate's rule stays: it decides no review that left an
agent out. What changes is what it says. A step that fails without
deciding posts the check failed with its own error in the summary.
That error names each dispatched agent that did not run with the
reason the review outcome gives. Where a cost cap kept one from
starting, the reason holds what the reviews cost and the cap, and the
error adds how what they spent is reset.

RED. Unit tests of the gate: an outcome that names why an agent did
not run, and an error that repeats it; the reset named once a cost cap
kept an agent out, and not otherwise.

GREEN. The outcome's finding and the gate's reading of it share one
form, which the gate's crate writes and reads. The decide script keeps
the gate's errors, prints them to the log as before, and puts the last
line in the check's summary, in a code span with its backticks dropped
so nothing in it is read as Markdown.

Evaluation. No reviewer's behaviour changes, so no evaluation case
runs: the proof is unit tests, the scenario step, a record check and
drift cases.

Gate. The record check in `review_capped` runs `review-run` under a
cap its first run reaches, then the gate on what it wrote, and reads
the job that holds the reviewer app's key. Four drift cases fail it:
an agent named without its reason, the outcome naming it in a form the
gate does not read, the reset left out, and the check left without the
gate's error.
