---
id: 2611020828
title: "Review convergence: rulings settle, contests are checkable, a fresh reader paraphrases"
status: "🔲"
summary: >-
  Three ideas from google/elephant-goldfish-model, fitted to a review
  run by code (ENG-28). A stakeholder ruling on a tracked finding
  settles it, so later reviews of the pull request keep it as a nit and
  never stall on it again. An author may contest a finding only by a
  citation code can check, which the generalist weighs on re-review.
  A comprehension specialist paraphrases a changed SRS section, ADR or
  domain-model file and reports what a fresh reader could not follow.
  Opus designs and implements.
model: opus
depends-on: [2610292344, 2610282358]
---
# Review convergence: rulings settle, contests are checkable

## Goal

A re-review of a pull request converges. It never raises again a
finding the stakeholder ruled on. It weighs an author's checkable
contest instead of repeating the finding. A fresh reader restates each
spec change once.

## Context

The source is google/elephant-goldfish-model (EGM). Its
`egm-review-loop` keeps a settled list across rounds. Its
`egm-resolve` accepts a contest only with evidence the reviewer could
read. Its comprehension reviewer paraphrases a design before any
critic reads it. EGM runs these through agents, with a human at each
gate. Here code runs them, and every input from a pull request is
untrusted (I2, ENG-28).

What was searched, and what is reused:

- `review-run`'s [findings module][findings]
  already tracks each kept finding by identity across reviews of one
  pull request, marks it new, still open or resolved, and after three
  reviews in a row, its file unchanged, raises `stalled: needs a
  stakeholder ruling`. It has no input for that ruling: the stall says
  "which no further run gives". Phase 1 adds the input; the tracking is
  reused unchanged.
- The finding identity
  ([identity.rs](../../tooling/review-run/src/findings/identity.rs))
  is path, sorted ids and the hash of the normalised consequence. It is
  reused as the key a ruling names, through a short handle derived
  from it.
- The ENG-21 consent parser in [owners.rs][owners]
  already reads a stakeholder's comment by code, never by an agent. A
  ruling is read the same way, from the same identity.
- The domain model's finding ledger (`tooling/finding-ledger`) closes
  recurring themes as fixed, deferred or accepted, citing the text that
  closes them. It governs the model's prose, not a pull request's
  review, so it is not reused; its three closing kinds inform the
  ruling's kinds.
- No reviewer paraphrases today: the `review-spec` specialists and the
  domain-model agents check text against rules, not whether a fresh
  reader can restate it. Phase 3 adds that component under ENG-32.

Not taken from EGM: the synthesizer agent, since ADR-2610300031
rejected an orchestrating agent. Nor a severity per reviewer: only
generalists weigh. Nor a human gate between stages.

[findings]: ../../tooling/review-run/src/findings.rs
[owners]: ../../tooling/review-bundle/src/owners.rs

## Tasks

1. A stakeholder ruling settles a finding (phase 1): a comment by the
   stakeholder, `ruling: <handle> accepted|rejected — <reason>`, read
   by code; `findings.md` shows each finding's handle; a later review
   that keeps a ruled finding lists it as settled, a nit, and raises no
   stall on it. A fixed finding that returns is reopened, as today.
2. An author's checkable contest (phase 2): a comment by the author,
   `contest: <handle> — <path>#<symbol>|<ADR id>|<requirement id>`,
   read by code, which checks the citation exists on the head and
   passes the generalist only the handle and the citation, never the
   author's prose. A contest code cannot check is escalated to the
   stakeholder as a nit asking for a ruling.
3. A comprehension specialist (phase 3): dispatched for a change to
   `docs/srs/`, `docs/adr/` or `docs/domain-model/`; it restates the
   changed text in its run log and states as requests what a fresh
   reader could not follow. It lands with its ENG-32 cases.
4. ENG-28 is reworded for tasks 1 to 3 with its scenario and gate
   table in the same change; an ADR records the decision.

## Execution

| Phase | Model | Gate                                                                                                                                                       |
| ----- | ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus  | `review-run` unit and integration tests settle a ruled finding and skip its stall; the ENG-28 scenario's new row passes                                    |
| 2     | opus  | A contest with a missing citation is escalated and one with a present citation reaches the generalist as handle and citation alone; ENG-28 scenario passes |
| 3     | opus  | The comprehension specialist's ENG-32 cases pass on its model; the bundle dispatches it for an SRS, ADR or model change                                    |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                        |
| --- | ------ | ------------------------------------------------------------ |
| 1   | 🔲     | [A stakeholder ruling settles a tracked finding](phase-1.md) |
<?/catalog?>

## Acceptance Criteria

- [ ] A finding the stakeholder ruled on is listed as settled in every
      later review of that pull request and raises no stall
- [ ] A finding fixed and then raised again at the same identity is
      reopened
- [ ] A ruling or contest from anyone but its stated author is ignored
      and counted (I6)
- [ ] No author prose from a contest reaches an agent's prompt
- [ ] A comprehension specialist runs on every SRS, ADR or domain-model
      change, with ENG-32 cases that pass on its model
- [ ] ENG-28's row, scenario and gate table describe rulings and
      contests, and an ADR records them
