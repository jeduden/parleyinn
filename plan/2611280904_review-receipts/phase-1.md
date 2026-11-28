---
n: 1
title: "Specify the result receipt, its requirements and its decision record"
status: "🔲"
result: false
---
# Phase 1: specify the result receipt, its requirements and its decision record

CLAUDE.md asks that a new concept land in the domain model before
anything uses it, and that a new requirement land with its scenario in
the same change. So this phase writes no code. It proves the design can
be said in ParleyInn's own terms, and it fixes the names every later
phase uses.

Requirements. New ids, numbered when they are written:

- an engineering requirement for the result receipt: what it states,
  which key signs it, and that the signed bytes hold the digest of the
  `parinn` release that made it;
- an engineering requirement for the GitHub action as an interface
  other projects rely on;
- an engineering requirement for result-receipt verification: the
  checklist of the design's section 5, run with no network, no agent
  and no model credential;
- an engineering requirement for running the default branch's review
  again after every merge;
- ENG-28 gains the review outcome a result receipt carries, and how it
  is weighed;
- ENG-21 gains the stakeholder's ruling on what "a reviewer other than
  its author" means once a result receipt carries the review outcome;
- ENG-16's list of what CI gates on says which checks a verified result
  receipt may answer.

Each lands with its scenario under `features/`, tagged `@pending`.

The model. The domain-model agent read this plan on 28 November and
left four questions open. The stakeholder took the recommended answer
to each the same day, from the
[options page](../../research/notes/review-attestation/open-questions.html).
This phase writes them into the model and the SRS:

- The key. The editing node's device key signs, or on an ephemeral
  node the token key its access token carries, with its token
  certificate. SEC-10 does not change. A fixed domain tag opens the
  signed bytes, and the check refuses any receipt without it.
- The place. `record.md` gains the result receipt as a third receipt
  kind: for one commit, the own-check results its node recorded, the
  digest of the `parinn` release and the review outcome. Its results
  keep evidence class `own check`, and LANE-05 does not change.
- The action. OQ-32 gains the GitHub action as a third way the CLI
  ships. A new engineering requirement makes it a stable interface:
  its inputs, its outputs, its exit status, and its release signed
  like `parinn`, keyless through Sigstore (ENG-20). The check script
  is a template each project owns, not a component.
- The names. §9.5 gains `parinn result-receipt plan|make|check`,
  beside the head receipt's row.

The list of checks, the verification job and the forge runner stay
engineering tooling words and do not enter the model.

The decision record. A proposed record that carries the four answers
above, says what a pass means
once a result receipt is accepted, the ENG-21 question with the ruling
it needs, and why the forge runner still goes first on the paths a code
owner holds.
It is proposed, not accepted: acceptance waits for the stakeholder.

RED. The SRS gate test fails for each new id until its scenario exists.
GREEN. Each scenario exists, tagged `@pending`, and the gate passes.

Gate. `mise exec -- mdsmith check .` passes, the SRS gate test passes,
and the domain-model agent reports no open finding.

BDD coverage. Only `@pending` scenarios; none is taken off `@pending`
in this phase.
