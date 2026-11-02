---
n: 1
title: "A stakeholder ruling settles a tracked finding"
status: "🔲"
result: false
---
# Phase 1: a stakeholder ruling settles a tracked finding

Requirements. This phase changes ENG-28: the findings tracked across
reviews of one pull request gain a settled state, set only by a
stakeholder ruling that code reads. ENG-28's row, its scenario and its
gate table change in this phase, so the scenario stays green; the row
and the scenario are paths CODEOWNERS assigns, so the change waits for
the stakeholder's approval (ENG-21).

BDD coverage. No new scenario id: ENG-28 keeps its one scenario. Its
gate table gains two rows, a ruled finding kept again (approves when it
is the only finding) and a ruling from someone other than the
stakeholder (ignored, counted, the stall still fires). The binding in
`tooling/scenario/tests/bdd/engineering_review.rs` drives
`review-run` with a previous `findings.json` and a comments file.

RED. Write tests for each of these, in order:

- A handle for a finding identity: short, stable for the same identity,
  different for a different one, shown beside each finding in
  `review/findings.md`.
- Reading a ruling from a comment: `ruling: <handle> accepted —
  <reason>` and `ruling: <handle> rejected — <reason>`; a line in no
  form, an unknown handle or an empty reason is no ruling and is
  counted as ignored (I6).
- Only a comment by the stakeholder rules, checked by the same identity
  the ENG-21 consent check uses; any other author's ruling is ignored
  and counted.
- A ruled finding keeps its ruling in `findings.json` across reviews.
- A later review that keeps a finding ruled `accepted` lists it as
  `settled (accepted)`, a nit, never blocking, whatever its generalist
  weighed.
- A finding ruled `rejected` (the finding is wrong) is left out of the
  outcome and listed as `settled (rejected)` in `findings.md`.
- A settled finding raises no stall, however many reviews keep it.
- A finding marked resolved and then kept again is reopened as today,
  ruled or not only by its own later ruling.

GREEN. In `tooling/review-run`: a handle in the identity module, a
rulings reader beside the findings module, the settled state in its
tracked record, and the outcome step that applies it before the stall
check. The workflow passes the pull request's comments to `review-run`
as a file, read by code; no agent sees a ruling. No new third-party
dependency.

Gate. `cargo test -p review-run` at the crate's coverage floor, and the
ENG-28 scenario with its two new gate table rows. The live check
follows on main: a stalled finding on a real pull request, ruled once,
is settled in the next review.
