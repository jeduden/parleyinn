---
n: 7
title: "The evaluation's world holds the files a pull request adds"
status: "✅"
result: true
summary: >-
  ENG-32's evaluation now builds a run's world with the files the
  evaluated tree adds: each in both trees, an instruction file in the
  pull request's tree alone, so only the component under test loads
  beside the agent. A specialist added with its section had failed a
  case 2 of 2 runs for the section its world lacked. The mark on added
  lines that stand on main was measured and not built: a moved row cost
  5.4 of 20 turns in 56 runs. Four drift cases land with it.
---
# Phase 7 handoff

## Done

- A run's world holds each file the evaluated tree tracks and main
  lacks, in main's tree and in `pr-head/`.
- An instruction file among them is in `pr-head/` alone and moves out
  as text, as the pull request's other instruction files do.
- A link and a path that leaves the tree are left out, each named in
  the run's notes.
- [docs/review-evaluation.md](../../docs/review-evaluation.md) says so
  where it describes the world.

## Verified

- Unit tests of `review-eval` pass, among them the world built from a
  tree that adds a section and a skill.
- The end-to-end test builds a world from a real clone that adds a
  section, and its reviewer finds the section.
- The drift suite catches the four new cases.
- The cucumber scenarios `@ENG-32` and `@ENG-28` pass, and
  `review-eval check` applies all 235 cases for 74 components.

## Not done

- The bundle does not mark added lines that stand word for word on
  main. A moved row cost a specialist 5.4 of 20 turns on average in 56
  runs. A replay of pull request 147's change took 5 to 14 of 20 turns
  in 12 runs, and all passed. The cost the option named no longer
  shows, so ENG-28 keeps its rule.
- A file the pull request changes, and does not add, stays as main has
  it in a world. A case edits main's text, so it would not apply to the
  changed one.
- No scorecard was published. That is the stakeholder's to accept
  after a run of every component.

## Next

- Nothing in this plan. The options not taken are in its Context.
