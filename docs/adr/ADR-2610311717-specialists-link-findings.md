---
id: ADR-2610311717
title: "Specialists link findings to requirements; their run log is their report"
status: proposed
summary: >-
  A review specialist no longer marks a finding blocking or nit. Each
  finding says what happens if it is not addressed and links the
  requirement ids or invariants that outcome breaks, or requests that a
  requirement be added, changed or removed. A specialist's outcome no
  longer blocks the review by itself; the generalists alone weigh. A
  specialist writes no file: it states each finding in its text as it
  finds it, answers its outcome line alone, and review-run assembles
  its report from its run log, keeping what a cut-off run stated while
  the run still fails the review closed.
---
# ADR-2610311717: Specialists link findings; the run log is the report

It amends two decisions of ADR-2610300031. That record stays accepted:
ENG-26 supersedes a whole record, and every other decision in it stands.
This record replaces one clause of its "The verdict is a pure function,
and fails closed" decision, that a specialist's report ending `outcome:
blocking` blocks the review by itself, and its "reports as files" clause
of "What stays", under which each specialist wrote its own report file
and answered two lines, the file's outcome line and its path.
ADR-2610300031's Consequences name this record.

## Context

Under ADR-2610300031 each specialist marked its findings `blocking` or
`nit` and ended its report `outcome: sound`, `nits` or `blocking`, and a
blocking report failed the review whatever its generalist decided. The
specialists run on Haiku at low effort and see one language, section or
file. Their severity was a guess made without the whole change in view,
and a guess that blocked a pull request on its own.

The stakeholder ruled that a specialist must not categorize its
findings. A finding instead says what happens if it is not addressed,
linked to the requirement that outcome breaks. Where no requirement
exists, the specialist may ask for one to be added, changed or removed,
with an informal motivation, and the generalist weighs that request
like any other finding.

The report file was a failure class of its own. A specialist had to
write `review/reports/<name>.md` with its one writing tool and rewrite
it as it checked. It then answered two lines: the file's outcome line,
copied character for character, and the file's path. `review-run` read
the file and the run log and compared the two. Runs answered an outcome
the file did not end with. They wrapped the outcome line. They wrote the
file late and ran out of turns before answering, leaving no file at all.
Their instructions were spent on when to write: "by your fifth turn,
answer by your fifteenth". The stakeholder asked why the review did not
read the session log instead of asking the agent to log. Then the
stakeholder asked that a specialist state each finding as it goes, so a
run cut off by its turn budget still leaves its findings in the log, and
that the answer not repeat them, which costs tokens.

## Decision

- **A specialist's report has two kinds of finding.** A linked finding,
  `path:line — if not addressed: <consequence> — <ID>[, <ID>...]`, each
  id a requirement id of the SRS or an invariant, `I1` to `I10`. A
  requirement request,
  `path:line — request: add|change|remove <requirement> — <motivation>`.
  A specialist writes no severity word.
- **The run log is the report.** A specialist has no writing tool; its
  tools are Read, Grep and Glob, as a generalist's are. It states each
  finding as one line of its text the moment it finds it, between tool
  calls, never held back for the end. Its final answer is the outcome
  line alone, `outcome: findings — <sentence>` or
  `outcome: none — <sentence>`; a blank or stray line after it is
  tolerated, and the findings are not repeated.
- **A specialist changes its mind in two forms.**
  `revised: <finding line>` replaces every earlier finding at that
  line's `path:line`; `revised: <old path:line> → <finding line>` does
  so for a finding whose place moved. `withdrawn: path:line — <reason>`
  removes every earlier finding at that place. One rule: the place names
  the finding.
- **`review-run` assembles the report.** From the specialist's run log
  it replays the finding, request, revision and withdrawal lines of the
  main thread's text, in order, a line identical to an earlier one
  dropped, each finding in its canonical shape, then appends the outcome
  line the answer gives, and writes the result to
  `review/reports/<name>.md`, the file its generalist reads. A malformed
  line, and a revision or withdrawal that names a place no earlier line
  found at, stay in the report as written, so the check names them.
- **The form fails closed.** The gate and `review-run` refuse a report
  with a line in no form, a revision or withdrawal of a finding never
  stated, an outcome of `none` beside a finding or `findings` beside
  none, an answer with no outcome line or another outcome than the
  report ends with. `review-run` also refuses a cited id that neither
  main's SRS nor the pull request's defines, so a made-up requirement
  cannot carry a finding. Free prose is left out of the report, but the
  instructions tell a specialist to write an unlinked concern as a
  request, never as prose, and to start no other line with
  `path:line —`.
- **An incomplete run is salvaged and still fails closed.** A run that
  did not end with a successful result, at its turn budget, its
  deadline, an error or an empty answer, stays a blocking finding and
  is not consulted, so the gate posts no approval. Its report is
  assembled all the same, headed `incomplete: <why>` and ending with no
  outcome line, so the findings it stated reach its generalist and the
  author instead of being lost with the run.
- **The generalists alone weigh.** A specialist's outcome no longer
  becomes a blocking finding. A generalist keeps a linked finding as
  `blocking` only when its consequence breaks the requirement or
  invariant it names on this change, or masks a bug; otherwise as a
  `nit`, or drops it. A request is never blocking: the generalist keeps
  it as a nit that asks the stakeholder the requirement question, or
  drops it with a reason, and `review-run` keeps a blocking finding
  kept from a request as a nit.
- **Personas use the same forms.** A persona agent dispatched by the
  review states its findings and answers as a specialist does; one a
  skill consults answers with its finding lines and then its outcome
  line, so every specialist the review reads speaks one grammar.
- **The evaluation grades the assembled report.** The grader reads the
  run log alone and assembles the report as `review-run` does; a case
  may want a finding stated before the final answer, or withdrawn.
- **What stays.** Every other clause of ADR-2610300031: a dispatched
  agent that did not answer, a refused report, an answer that says
  nothing readable and a kept blocking finding each request changes;
  the generalists read report files; the retry of a run that did not
  answer; the usage file counts the attempts.

## Alternatives

- **Keep severity and calibrate it.** Better instructions and more
  cases could make a specialist's `blocking` rarer, but the one with
  the narrowest view would still decide alone that a pull request
  cannot land.
- **Let a specialist name a severity the generalist may override.** The
  generalist already weighs every finding; a second severity is a
  label the generalist must argue against, and the stakeholder ruled
  out categorizing.
- **Accept unlinked findings as prose.** A concern no requirement
  covers would then reach the generalist with no requirement question
  attached, and requirement work would lose it.
- **Keep the report file.** The file kept a second copy of what the log
  already held, under a writing tool the agents otherwise never need,
  and every mismatch between file, answer and log was a review that
  failed for its shape, not its findings. A run cut off before writing
  left nothing. The log is written by the harness, not by the model,
  so it is always there and always whole.
- **Have the final answer repeat every finding.** The log would hold
  each finding twice, the answer would cost as many output tokens as
  the review had findings, and a run cut off before its answer would
  lose them all the same. The outcome line is all the answer must add.
- **Discard what a cut-off run stated.** Simpler, and the run fails the
  review either way, but a half-finished check still found what it
  found; the author and the generalist see more with it kept, headed as
  partial, than with it gone.

## Consequences

- ENG-28 describes the finding grammar, the revision and withdrawal
  forms, the assembly from the run log, its refusals, the incomplete
  report, that the agents write nothing and that the generalists alone
  weigh; its scenario's gate table takes a report with a request, which
  approves, and one with a malformed finding, an incomplete one, one
  whose answer gives no or another outcome, and one revising a finding
  never stated, each an error.
- The shared report sections under `docs/agents/`, the specialists'
  and the personas' own report text, the generalists' input section and
  the persona-review skill follow the new forms; every specialist's
  tools line loses Write. The evaluation cases (ENG-32) expect
  `findings` or `none` and a `linked` or `request` line, may expect a
  finding stated before the answer or withdrawn, and the generalists'
  seeded reports use the new lines; the grader reads the run log alone.
- `review-run` keeps the report files for the generalists and the
  review's artifact, now writing them itself; the review workflow's
  agents have no writing tool to scope.
- A requirement request lands with the stakeholder as a nit; the SRS
  changes only through the stakeholder's own change.
- No new third-party dependency.

## Approval

This record is proposed. The stakeholder accepts it, or not, on the
pull request that carries it.
