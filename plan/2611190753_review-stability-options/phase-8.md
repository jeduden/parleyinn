---
n: 8
title: "A generalist gives its whole weighing in its final answer"
status: "✅"
result: false
---
# Phase 8: the whole weighing in the final answer

Requirements. ENG-28 and ENG-32: neither row changes, and no scenario
comes off `@pending`. ENG-28 still has the review read a generalist's
weighing from its whole run log. This phase changes what a generalist
is asked for, and what the evaluation counts.

What happened. The model's API refused the code generalist of pull
request 152 in two reviews in a row. The run ended on the harness's
error text, with the refusal category `reasoning_extraction`. Each
refused run was charged, 0.30 and 0.38 USD, so ENG-28 ran neither
again, and the gate blocked on the gap.

What the model's documentation says. That category means the request
asks the model to reproduce its reasoning in its answer text. Its
usual cause is a prompt that asks for a running log. It names no
fallback model, and its remedy is to change the prompt. It also says
that what this model writes between tool calls reaches a log as text
only when it is a sentence or two.

What was measured.

- A generalist was told `State each once decided`. In 24 generalist
  runs on record, none stated a weighing line before its final answer.
  The 5 texts written before it were narration.
- The refusal did not occur here: the same review ran three times on
  this machine, once with every specialist answering `none` as in CI,
  and the generalist answered each time. So the fix is not proven
  against the refusal outside CI.

The rule. A generalist's report section asks for every line in the
final answer and none before it. The review reads the whole run log as
before, so a short line that does arrive early still counts.

RED. Unit tests of the grader: a weighing line in a message before the
final answer, which the answer does not hold, is a slip; a line the
answer repeats, a working note, a consultant's line and a run cut off
before its answer are none.

GREEN. One module of the grader lists those lines. The report section
`docs/agents/generalist-report.md` changes, and `mdsmith fix`
regenerates the six generalist definitions.

Evaluation. The three generalists and their deep instances run their
13 cases on their models with the new wording.

Gate. One drift case fails: a grader that no longer counts a line
the final answer leaves out.

Not in this phase: what the review does with a run the API refused.
The review still reads the error's text as an answer and names a turn
budget as the cause. That fix is built on its own branch.
