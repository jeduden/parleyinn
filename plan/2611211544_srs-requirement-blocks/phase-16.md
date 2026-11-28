---
n: 16
title: "The tools, the scenarios and the docs say requirement"
status: "✅"
result: false
---
# Phase 16: The tools, the scenarios and the docs say requirement

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 4's second pull request. Since phase 15 no requirement
can be a table's row, so where a tool, a scenario or a doc calls a
requirement a row, or a section's requirements a table, it says
requirement or block. A row of a table that stays, such as the
boundary register's, or of a scenario outline's examples, keeps its
name, and so does a decision record's text.

The tools. The srs crate's manifest states its responsibility as
reading the requirement blocks: phase 15 changed the crate's doc, not
the `responsibility` its manifest declares. The review bundle's
context speaks of a requirement the pull request removes or neither
tree defines, and its finding of a change to ENG-29 itself. The
engineering checks that read ENG-21 and ENG-34 name the requirement,
not its row. The evaluation's derived cases lose their row path,
which matches no line of the SRS, and the case of a MUST added with
no change to its scenario is `defect-requirement-without-scenario`.
The doc comments and the names of the crates that read the SRS follow.

The scenarios. The ASM, NFR and ENG feature files describe their
requirements without a table, and ENG-34's step reads "the review
bundle's line limits are those ENG-34 states", with its binding and
the drift case that quotes the check's message.

The docs. testing.md says a requirement is a block, which the parser
reads, and that it refuses a table; development.md,
review-evaluation.md and use-cases.md say requirement where they say
row. Each keeps its length, under its size-list entry.

RED. With the tests expecting the new words and the tools as main has
them, the tests of the review bundle's context, the engineering
checks' messages and the derived case's name fail, and ENG-34's
scenario fails on a step no binding matches.

GREEN. The tools, the scenarios and the docs say requirement, and
every test passes.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- the srs gates, the scenario gate, the workspace's tests, clippy and
  the cucumber scenarios `@ENG-21`, `@ENG-27`, `@ENG-28`, `@ENG-32` and
  `@ENG-34` pass;
- `review-eval check` finds every case applies, as many as before;
- the drift suite catches each case whose message moves;
- `scripts/review-local.sh` approves.

What waits. The agents' shared sections and the SRS specialists' own
text, which say row, are the next pull request: a change to the
review's agents, run on their model against their cases (ENG-32).

BDD. No cross-host behaviour: no `@S` scenario.
