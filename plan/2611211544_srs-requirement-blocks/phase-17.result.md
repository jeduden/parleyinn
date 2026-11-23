---
n: 17
title: "The review's agents say requirement"
status: "✅"
result: true
summary: >-
  The SRS specialists, the generalists for specification and code,
  lang-gherkin, performance, test-engineer-end-to-end and the report
  sections say requirement where they said row. Each reviewer component
  the change reaches passes its cases on its model: the 41 the
  evaluation plans and the security persona it left to another. No
  requirement changes, and no scenario comes off @pending.
---
# Phase 17 handoff

## Done

- srs-section-checks.md holds each touched requirement to the section,
  its scenario, the code and the invariants, and a finding on one ends
  in its id. An id a specialist is unsure of is checked in the SRS. Each
  of the 32 SRS specialists reads only the requirements and text the
  pull request touches or names.
- review-spec.md and review-code.md, and so both generalists and their
  deep instances, ask their questions of each touched requirement;
  review-spec also asks whether a live step exercises the requirement's
  text. lang-gherkin holds a scenario to its requirement as the context
  quotes it, performance reads the requirements a change touches, and
  test-engineer-end-to-end proves the requirement's MUST.
- outcome-answer.md and specialist-report.md, the report sections' two
  copies of the answer rule, say what is fine gets no line, where they
  said a fine row.
- mdsmith writes each agent's copy of its shared sections again: 59
  agent files and 7 shared sections, test-engineer-end-to-end kept at
  its cap of 90 lines.

## Verified

- RED: none, since a slip fails no run and no case plants an id a
  specialist should be unsure of.
- `review-eval plan` against phase 16 names 41 components and 135
  cases: each whose own text changed, and one for each set of shared
  sections the change reaches.
- `review-eval run` passes every case of the 41 on its model: 270 of
  270 runs, 2.61 USD, 175 of them slipped. On phase 16's text they
  passed in 270 of 271 runs, 191 slipping, 2.55 USD; srs-security's
  derived case once asked to change SEC-01 instead.
- persona-security-officer, a haiku specialist the plan covered through
  persona-agent on the default model, passes its 3 cases in 6 of 6
  runs, 4 slipping where phase 16's text had 5. review-spec passes in 8
  of 8 once its question names the requirement's text.
- A slip is an answer holding more than its outcome line. On clean
  cases slips fell from 61 to 45 of 82 specialist runs, all in the SRS
  specialists and lang-gherkin: 49 of phase 16's said no requirement row
  was touched, and none of phase 17's names a row. Defect and injection
  cases slipped 130 times on either text, most repeating a finding line.
  One paired run fits the drop to the change but does not prove it.
- `mdsmith check .`, the srs gates, the scenario gate, `review-eval
  check` (236 cases, 74 components), the workspace's 4,648 tests and the
  cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32` and
  `@ENG-34` pass: 5 scenarios, 77 steps.
- The drift suite catches the 10 cases that inject a file the change
  touches, among them a specialist given a writing tool, raised to
  medium effort or run without its turn cap.
- The reviewer-engineer agent found no blocking finding; its two gaps,
  review-spec's "its text" with three readings and the security
  persona's own model, are closed above.
- `scripts/review-local.sh` against phase 16: lang-markdown and
  review-spec-deep, the two agents 69 Markdown files dispatch, approve,
  0.19 USD. review-spec-deep dropped lang-markdown's one finding, the
  summary's 42 components against the plan's 41; the summary now names
  both. It blocks only on the consent `.claude/agents/` and
  `docs/agents/` need.

## Next

- On defect and injection cases four specialist answers in five slip:
  "Last message: `outcome: …`" reads as ending with the outcome line,
  not as that line alone. A fix to the report section shows in the slip
  count, since a slip fails no case.
- The evaluation plan lets one component stand for every other with its
  shared sections, whatever its model, as persona-agent stood for
  persona-security-officer. It could plan one for each model.
- The derived case of a MUST added with no change to its scenario plants
  "It MUST also record the change in the audit log.", whose "the
  change" names nothing, so a specialist may ask for the requirement to
  change instead. Naming what it records would plant one problem, not
  two.
- Task 6: split each requirement the srs kind lists past 50 lines.
