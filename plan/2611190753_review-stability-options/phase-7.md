---
n: 7
title: "The evaluation's world holds the files a pull request adds"
status: "✅"
result: false
---
# Phase 7: the world holds the files a pull request adds

Requirements. ENG-32: its row has a pull request's changed components
run their cases before the review gate decides. The row does not
change. No scenario comes off `@pending`. ENG-28 is not touched: the
half of this phase that would have changed the bundle was measured and
not built.

The option had two halves. Each was measured before it was built.

The first half: the bundle marks added lines that stand word for word
on main. Its evidence was a specialist that spent 19 to 21 of its 21
turns on a row a pull request had moved. That cost no longer shows.

- A derived case moved a row inside one specialist's files. In 56 runs
  a specialist used 5.4 turns on average, and none reached its 20.
- Pull request 147's own change was replayed on main as it stood
  before it, with today's tools: 12 runs, 5 to 14 of 20 turns, all
  passed, none read outside `pr-head/` and `review/`.

Earlier phases of plan 2610282358 closed that cost: the numbered diff,
the review context, and a work directory that names no tree. A mark
would add a rule to ENG-28 for no measured gain, so it is not built.

The second half: the world holds the files a pull request adds. A
world is main's tree with the component under test copied in. Pull
request 147 added a section with its specialist. The specialist was
told of a section file its world lacked, and one of its cases failed 2
of 2 runs, each for a search outside `pr-head/`.

The rule. Each file the evaluated tree tracks and main lacks goes into
both trees of the world, as both hold it once the pull request is on
main. An instruction file among them goes into `pr-head/` alone, where
it moves out as text with the others. So nothing the pull request
wrote loads beside the agent but the component under test. A file main
holds stays as main has it. A link, or a path that leaves the tree, is
left out and named in the run's notes.

RED. Unit tests of the world: a tree that adds a section and a skill
builds a world with the section in both trees and the skill in neither
load path. An end-to-end test on a real clone: a reviewer that looks
for the added section finds it.

GREEN. One module lists the evaluated tree's files and copies those
main lacks, before the component is copied in.

Evaluation. No reviewer's behaviour changes, so no evaluation case
lands. The derived moved-row case was written for the first half and
taken out again: 52 of its 56 failures were fair remarks on a fixture
that was not clean, so the case demanded something wrong.

Gate. Four drift cases fail: a world built without the added files, an
instruction file among them put beside the agent, a link among them
followed, and a path that leaves the tree written where it points.
