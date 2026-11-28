---
n: 16
title: "The tools, the scenarios and the docs say requirement"
status: "✅"
result: true
summary: >-
  Where the tools, the scenarios and the docs called a requirement a row,
  or a section's requirements a table, they say requirement or block. The
  evaluation's derived case of a MUST added with no change to its
  scenario reads the requirement as a block alone. No requirement
  changes, and no scenario comes off @pending.
---
# Phase 16 handoff

## Done

- The srs crate's manifest states its responsibility as reading the
  requirement blocks. Phase 15 changed only the crate's doc, though its
  handoff said both, as #181's review found; that handoff now says what
  phase 15 changed.
- The engineering checks of ENG-11, ENG-21 and ENG-34 name the
  requirement in their messages, not its row.
- The review bundle's context speaks of a requirement the pull request
  removes or neither tree defines, and its finding of a change to ENG-29
  itself.
- The evaluation's derived case of a MUST added with no change to its
  scenario is `defect-requirement-without-scenario`, read from the
  section's first block. The row path, which matched no line of the SRS,
  is gone, with its 3 tests.
- ENG-34's step reads "the review bundle's line limits are those ENG-34
  states", with its binding and the drift case that quotes the check's
  message. The ASM, NFR and ENG feature files describe their
  requirements without a table.
- testing.md says a requirement is a block, which the srs parser reads,
  and that it refuses a requirement table. development.md,
  review-evaluation.md and use-cases.md say requirement where they said
  row, each within its length.
- The soak binding's comment quotes ENG-09, and the ENG-11 drift cases'
  doc names the floor ENG-11 states. The drift case of a requirement
  ENG-29 marks for I2 review runs the bundle test by its new name.

## Verified

- RED: with the tests expecting the new words and the tools as phase 15
  left them, 11 tests failed, and ENG-34's scenario failed on a step no
  binding matched.
- `mdsmith check .` passes.
- The srs gates, the scenario gate, the workspace's 4,648 tests, clippy
  denying warnings, rustfmt and the cucumber scenarios `@ENG-21`,
  `@ENG-27`, `@ENG-28`, `@ENG-32` and `@ENG-34` pass: 5 scenarios, 77
  steps.
- `review-eval check` finds every case applies: 236 cases, 74
  components, as before.
- `review-bundle` against phase 15 shows each touched requirement with
  its text alone, so none changed. Its one finding is the consent the
  paths CODEOWNERS assigns need.
- The drift suite catches the 26 cases that inject a file the change
  touches, the srs crate's responsibility among them, and the 14 that
  inject ENG-34's, ENG-29's or I2's text, among them the 4 whose
  message or test moved: ENG-34's limit, the two of ENG-29's list and
  the requirement ENG-29 marks for I2 review. That last case first went
  uncaught: it filtered for the bundle test's old name, which matches
  no test and so passes.
- `scripts/review-local.sh` against phase 15: the agents approve, 17 of
  17 consulted, 1.38 USD, with one nit, this handoff's quote of ENG-34's
  step, now whole. It blocks only on the consent the paths CODEOWNERS
  assigns need, which the pull request asks the stakeholder for.

## Next

- Phase 17: the agents' shared sections and the SRS specialists' own
  text say requirement where they say row, each reviewer component run
  on its model against its cases (ENG-32).
- The srs kind's comment in `.mdsmith.yml` still calls the requirements
  tables, every row a MUST. The linter configuration changes only with
  the stakeholder's consent.
- A drift case whose filter names no test passes on the unedited copy,
  so a renamed test reads as an uncaught drift. The suite could refuse
  a check that runs no test.
- Task 6: split each requirement the srs kind lists past 50 lines.
