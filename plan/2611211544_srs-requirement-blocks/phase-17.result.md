---
n: 17
title: "The review's agents say requirement"
status: "✅"
result: true
summary: >-
  The SRS specialists, the generalists for specification and code,
  lang-gherkin, performance, test-engineer-end-to-end and the report
  sections say requirement where they said row, and the threat model's
  specialist takes its final text. Each reviewer component the change
  reaches passes its cases on its model: the 42 the evaluation plans
  and the security persona it left to another. No requirement changes,
  and no scenario comes off @pending.
---
# Phase 17 handoff

## Done

- srs-section-checks.md holds each touched requirement to the section,
  its scenario, the code and the invariants, and a finding on one ends
  in its id; an id a specialist is unsure of is checked in the SRS. Each
  of the 33 SRS specialists reads only the requirements and text the
  pull request touches or names.
- srs-threat-model, which #179 landed ahead of its section, links 06a
  as srs-boundary-register links 06b, now that #178 put it on main.
- review-spec.md and review-code.md, and so both generalists and their
  deep instances, ask their questions of each touched requirement;
  review-spec also asks whether a live step exercises its text.
  lang-gherkin, performance and test-engineer-end-to-end name the
  requirement where they named its row.
- outcome-answer.md and specialist-report.md say what is fine gets no
  line, where they said a fine row.
- mdsmith writes 60 agent files from 7 shared sections again,
  test-engineer-end-to-end kept at its cap of 90 lines.

## Verified

- RED: none, since a slip fails no run and no case plants an id a
  specialist should be unsure of.
- `review-eval plan` against phase 16 merged with main names 42
  components and 141 cases.
- `review-eval run` passed every case of the 41 planned before main's
  merge on its model: 270 of 270 runs, 2.61 USD, 175 slipping; on phase
  16's text, 270 of 271, 191 slipping. Merged with main,
  srs-threat-model passes its 6 cases in 12 of 12 runs and
  srs-security, whose §6 lost the threat model, its 3 in 6 of 6.
- persona-security-officer, which the plan covers through persona-agent,
  passes its 3 cases in 6 of 6 runs on haiku, and review-spec in 8 of 8.
- A slip is an answer holding more than its outcome line. Clean cases'
  slips fell from 61 to 45 of 82 specialist runs: 49 of phase 16's said
  no requirement row was touched, and none of phase 17's names a row.
  Defect and injection cases slipped 130 times on either text.
- Merged with main, `mdsmith check .`, the srs and scenario gates,
  `review-eval check` (242 cases, 75 components), the workspace's 4,648
  tests and the cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`,
  `@ENG-32` and `@ENG-34` pass, and the drift suite catches the 10 cases
  that inject a file the change touches.
- The reviewer-engineer agent found no blocking finding, and its two
  gaps are closed above.
- `scripts/review-local.sh` against phase 16 merged with main approves:
  2 of 2 agents consulted, no finding, 0.17 USD. It blocks only on
  the consent `.claude/agents/` and `docs/agents/` need.

## Next

- On defect and injection cases four specialist answers in five slip:
  "Last message: `outcome: …`" reads as ending with the outcome line,
  not as that line alone.
- Phases 14 and 15 of plan 2611190753 plant a derived MUST that names
  what it records, and plan a shared section's change once per model.
- Task 6: split each requirement the srs kind lists past 50 lines.
