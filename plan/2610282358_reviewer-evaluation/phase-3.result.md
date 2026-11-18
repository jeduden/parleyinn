---
n: 3
title: "Stable cases, so the scheduled run passes"
status: "✅"
result: true
summary: >-
  The cases no longer fail by chance where the instructions, a case or
  the bundle left the reviewer a choice or a value to work out: 83 of
  956 runs failed on main, 15 of 964 on the branch, and the causes
  those 15 showed are closed. ENG-35 states the terms rule; the bundle
  writes each line's number into the diffs a reviewer reads. ENG-32's
  two-of-three rule is unchanged. The first scorecard waits for the
  first scheduled run that passes on main.
---
# Phase 3 handoff

## Done

- ENG-35 (P0) is in the SRS with its scenario `@pending`: the domain
  model's terms rule as a requirement a finding can cite. `@ENG-32` is
  unchanged, and so is its rule that a case failing more than one of
  three repetitions blocks.
- The measurement, every run local on the reviewers' own models. Main:
  83 of 956 runs failed, 50 cases at least once;
  [phase-3.md](phase-3.md) lists the causes. The branch before the
  line numbers: 15 of 964, one case twice. Of the 15, 4 took the
  diff's line number, 1 lost a blocking finding to a nit stated later
  at its place, 3 drew a fair remark on a clean fixture, 1 could not
  see the ADR its change lands with, 3 gave the right finding in other
  words or at another fair place, 2 broke a line's form and 1 wrote a
  request where a finding is due.
- One cause was in no instruction and no case. A reviewer's tool
  numbers the lines of the diff file it reads, and 83 of 826 places at
  a created file took that number, not the file's. The bundle now
  writes each line's number in the pull request's file into every diff
  it hands out, and the prompt says a place takes it. After that, 0 of
  195 such places were wrong.
- A specialist's context names the ENG-29 record a change lands with,
  which its own diff does not hold. A generalist states one line a
  place. The grader takes a tail of a file under a wanted module and
  no longer takes an outcome line for a finding.
- One round of every case after that: 5 of 241 runs failed, each
  closed where a cause showed. Four more rounds of the 31 section
  specialists' cases: 2 of 372 failed, each on a request's form.
- Two specialists counted a long diff's read offsets from nought and
  left a line unread. The instruction now gives each call's `offset`
  and `limit`.
- #141, #142 and #143 landed the harness first, with one cause more:
  a run's work directory named `main` made agents look for `pr-head/`
  one directory up, refused in 36 of 3,308 runs, 0 of 217 since.
- CI's first whole evaluation failed 7 of 233 runs. Four named a path
  the tool refused, which the grader counted as a read of main's tree
  until #149. Two were `test-engineer-end-to-end` lines off their
  form; its report section now holds its siblings' sentence on a
  line's end, and 216 of 216 runs passed since.
- ENG-28's row and scenario name the ENG-29 record.
- Three false-alarm cases know the dropped line by its place and
  source, where a word of its reason failed a right drop.
- [review-evaluation.md](../../docs/review-evaluation.md) states the
  rules a stable case, and what a reviewer is handed, hold to.

## Next

- What still fails by chance, each about one run in several hundred:
  a finding or request line off its form, which the review reads as
  a blocking finding; a request to the author where a finding is due;
  a reviewer that opens main's copy of a file it was told to read
  under `pr-head/`; a fair remark on a clean change.
- The word `request:` reads as a request to the author, and three
  runs used it that way. Another word for the form is a change to
  ENG-28's report grammar.
- A reviewer still works out one value: the path it reads a changed
  file at, `pr-head/` before the path its diff names. A work directory
  that holds one tree, the pull request's with main's instruction
  files, would leave nothing to work out. It changes the review's
  workflow, so it is a plan of its own.
- 63 written cases still know a finding by words of its sentence. A
  case moves to place, ids and source when a run shows its words
  miss a right finding.
- The first scorecard is published from the first scheduled run that
  passes on main, with `review-eval publish`.
