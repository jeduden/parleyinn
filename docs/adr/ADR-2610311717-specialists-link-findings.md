---
id: ADR-2610311717
title: "Specialists link findings to requirements; generalists alone weigh"
status: proposed
summary: >-
  A review specialist no longer marks a finding blocking or nit. Each
  finding says what happens if it is not addressed and links the
  requirement ids or invariants that outcome breaks, or requests that a
  requirement be added, changed or removed. A specialist's outcome no
  longer blocks the review by itself; the generalists alone weigh.
---
# ADR-2610311717: Specialists link findings; generalists alone weigh

It amends the verdict rule of ADR-2610300031. That record stays
accepted: ENG-26 supersedes a whole record, and every other decision in
it stands. This record replaces one clause of its "The verdict is a pure
function, and fails closed" decision: that a specialist's report ending
`outcome: blocking` blocks the review by itself. ADR-2610300031's
Consequences name this record.

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

## Decision

- **A specialist's report has two kinds of finding.** A linked finding,
  `path:line — if not addressed: <consequence> — <ID>[, <ID>...]`, each
  id a requirement id of the SRS or an invariant, `I1` to `I10`. A
  requirement request,
  `path:line — request: add|change|remove <requirement> — <motivation>`.
  A specialist writes no severity word.
- **The report ends with its outcome.** `outcome: findings — <sentence>`
  when it holds a finding or a request, `outcome: none — <sentence>`
  otherwise. `sound`, `nits` and `blocking` are no longer outcomes.
- **The form fails closed.** The gate refuses a report with a line that
  starts with a place, `path:line` and a dash, in neither form, an
  outcome of `none` beside a finding, or `findings` beside none.
  `review-run` also refuses a cited id that neither main's SRS nor the
  pull request's defines, so a made-up requirement cannot carry a
  finding. Free prose is passed by, but the instructions tell a
  specialist to write an unlinked concern as a request, never as prose.
- **The generalists alone weigh.** A specialist's outcome no longer
  becomes a blocking finding. A generalist keeps a linked finding as
  `blocking` only when its consequence breaks the requirement or
  invariant it names on this change, or masks a bug; otherwise as a
  `nit`, or drops it. A request is never blocking: the generalist keeps
  it as a nit that asks the stakeholder the requirement question, or
  drops it with a reason, and `review-run` keeps a blocking finding
  kept from a request as a nit.
- **Personas use the same forms.** A persona agent, dispatched or
  consulted by a skill, writes its findings in the two forms and its
  outcome the same way, so every specialist the review reads speaks one
  grammar.
- **What stays.** Every other clause of ADR-2610300031's verdict rule:
  a dispatched agent that did not answer, a refused report, an answer
  that says nothing readable and a kept blocking finding each request
  changes, and the two-line answer contract stays.

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

## Consequences

- ENG-28 describes the finding grammar, its refusals and that the
  generalists alone weigh; its scenario's gate table takes a report
  with a request, which approves, and one with a malformed finding
  line, which is an error.
- The shared report sections under `docs/agents/`, the specialists'
  and the personas' own report text, the generalists' weighing section
  and the persona-review skill follow the new forms. The evaluation
  cases (ENG-32) expect `findings` or `none` and a `linked` or
  `request` line; the generalists' seeded reports use the new lines.
- A requirement request lands with the stakeholder as a nit; the SRS
  changes only through the stakeholder's own change.
- No new third-party dependency.

## Approval

This record is proposed. The stakeholder accepts it, or not, on the
pull request that carries it.
