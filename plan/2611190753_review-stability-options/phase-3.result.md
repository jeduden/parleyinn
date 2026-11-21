---
n: 3
title: "A stakeholder's comment changes nothing an agent reads"
status: "✅"
result: true
summary: >-
  ENG-28 now has the review bundle write an agent the same bundle and
  review context whatever the stakeholder's comments say: the findings
  that turn on consent go to the review gate alone, and each agent
  reads that consent is the gate's check. So the outcome a comment's
  review reuses holds no line that restates a missing consent. Two
  clean cases on stakeholder paths with no consenting comment failed 5
  of 8 runs before the change and none for consent after it. The
  ENG-28 scenario step, a record check and three drift cases land
  with it.
---
# Phase 3 handoff

## Done

- ENG-28's row states the rule, and its scenario step on a comment's
  review carries it. No scenario was pending.
- The review bundle sets apart the findings that turn on consent, in
  either state: ENG-21's on a stakeholder path, ENG-29's on a guarded
  change and ENG-34's on a lengthened size list. The gate weighs them
  as before. The bundle and each review context leave them out and
  carry one fixed sentence in their place.
- ENG-29's finding is now one finding a cause: the record a change
  must bring, and the consent a comment gives. The record is named to
  an agent whether or not a comment consents.
- Two written cases drop the consenting comment they carried so the
  reviewer would not repeat the bundle's finding.
  [review-evaluation.md](../../docs/review-evaluation.md) and
  [ci.md](../../docs/ci.md) say what an agent is shown.

## Verified

- Unit tests of the bundle and of the record check pass, with the
  tests of `review-eval`, `review-run`, `review-gate` and
  `engineering`.
- The cucumber scenario `@ENG-28` passes. Its step runs the checkout's
  bundle on one change that waits on all three consents, with no
  comment and with one that consents: every file an agent reads is the
  same, and the comment lifts three blocking findings. A findings file
  of either run that does not parse is its own problem, never read as
  one with no finding, which would count every finding lifted.
- The evaluation, local, on the reviewers' models, four runs a case.
  Without the change the two cases with no consenting comment failed
  5 of 8 runs, each for a finding that restated the missing consent.
  With it they passed 7 of 8. The eighth reported a copy of the shared
  section as a second source, the case's own subject, seen once in 26
  earlier runs. The six defect and injection cases beside them passed
  every run both ways, the `reviewer-engineer` case on a real defect
  in how consent is applied among them.
- The drift suite catches the three new cases and every other case
  that injects a file this phase touches.

## Next

- A case's `consent` now changes nothing its reviewer reads. The field
  can go once no case on main names it; `lang-gherkin`'s
  `clean-step-reworded-beside-a-gap` still does.
- The fixed sentence is in every bundle and context, so every agent's
  answer key changes once and no stored answer is reused for the first
  review after this lands.
- An agent can still infer from the class that a change is on a
  stakeholder path. None restated consent in 30 runs with the sentence;
  the weekly run will show the rate over every case.
- Phase 4: a review that runs out of its budget names the budget, what
  was spent and how it is reset.
