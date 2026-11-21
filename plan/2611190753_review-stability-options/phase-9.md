---
n: 9
title: "A run that ended on an error says so"
status: "✅"
result: false
---
# Phase 9: a run's error named as its cause

Requirements. ENG-28: its row does not change, and no scenario comes
off `@pending`. ENG-28 already has a step that fails without deciding
name each agent that did not run with its reason. This phase keeps a
run's error apart from its answer and names it where the review
reports that run.

What happened. In both reviews of pull request 152 that the model's
API refused, the code generalist's run ended on a result that is an
error, the harness's text standing where the answer goes. The review
read that text as the generalist's answer: no finding, no `none —`
line, and the error's last line as its closing sentence. The blocking
finding said only that the generalist answered no finding, and the
summary named a turn budget, deadline or cost cap as the cause. Only
the run log, read by hand, showed the refusal.

The rule. A result that is an error holds no answer. The harness's
text is kept apart, the finding on a generalist so ended quotes it on
one line, and the summary names an error among the ways a run ends
without a successful result.

RED. Unit tests: a log whose result is an error holds the error and no
answer, and one that answers holds no error; the finding on a silent
generalist quotes its run's error, one whose run ended on no error
keeps its finding, and a finding on another agent or path is left
alone.

GREEN. The run log in `review-eval` keeps the error apart. One module
of the review outcome in `review-run` adds it to the finding on the
generalist's silence, and the summary's wording changes.
[ci.md](../../docs/ci.md) says what the review makes of such a run.

Gate. Two drift cases fail: the error read as the agent's answer, and
a generalist so ended reported silent with no word of why.

Not in this phase: an end-to-end record check of a run that ended on
an error, and the error's text for a specialist so ended, whose log is
read as its report as before.
