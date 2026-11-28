---
n: 1
title: "Specify the check receipt, its requirements and its decision record"
status: "🔲"
result: false
---
# Phase 1: specify the check receipt, its requirements and its decision record

CLAUDE.md asks that a new concept land in the domain model before
anything uses it, and that a new requirement land with its scenario in
the same change. So this phase writes no code. It proves the design can
be said in ParleyInn's own terms, and it fixes the names every later
phase uses.

Requirements. New ids, numbered when they are written:

- an engineering requirement for the check receipt: what it states,
  which key signs it, and that the signed bytes hold the digest of the
  `parinn` release that made it;
- an engineering requirement for check-receipt verification: the
  checklist of the design's section 5, run with no network, no agent
  and no model credential;
- an engineering requirement for running the default branch's review
  again after every merge;
- ENG-28 gains the review outcome a check receipt carries, and how it
  is weighed;
- ENG-21 gains the stakeholder's ruling on what "a reviewer other than
  its author" means once a check receipt carries the review outcome;
- ENG-16's list of what CI gates on says which checks a verified check
  receipt may answer.

Each lands with its scenario under `features/`, tagged `@pending`.

The model. The domain-model agent read this plan on 28 November. Its
open questions are this phase's first work, each answered in the model
or ruled on by the stakeholder:

- Which key signs a check receipt. The model names a device key and a
  CI key, and a program signs nothing.
- Where a check receipt stands beside `own check` and CI attestation,
  and whether it is a third kind of receipt beside head and purge.
- Whether the GitHub action and the check script are engineering
  tooling or components. The component set is closed, and a new one is
  a model change and a §6.3 row.
- The subcommand names, by the rule `parinn <concept> <verb>`; the
  node-wide `verify` utility already exists.

The list of checks, the verification job and the forge runner stay
engineering tooling words and do not enter the model.

The decision record. A proposed record that says what a pass means
once a check receipt is accepted, the ENG-21 question with the ruling
it needs, and why the forge runner still goes first on the paths a code
owner holds.
It is proposed, not accepted: acceptance waits for the stakeholder.

RED. The SRS gate test fails for each new id until its scenario exists.
GREEN. Each scenario exists, tagged `@pending`, and the gate passes.

Gate. `mise exec -- mdsmith check .` passes, the SRS gate test passes,
and the domain-model agent reports no open finding.

BDD coverage. Only `@pending` scenarios; none is taken off `@pending`
in this phase.
