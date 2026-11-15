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
- One round of every case on the tree with all of that: 5 of 241 runs
  failed. Read: one was the grader's error on an outcome line, fixed;
  one asked the author to remove a sentence in excluded terms where
  the case wants a finding, and the section check now says an excluded
  term breaks ENG-35; one wrote its id inside the sentence, a form
  lapse; two read main's copy of their section. Four more rounds of
  the 31 section specialists' cases followed: 2 of 372 runs failed,
  each on a request that did not hold its form, and none of 21 reads
  of a section took main's copy.
- [review-evaluation.md](../../docs/review-evaluation.md) states the
  rules a stable case, and what a reviewer is handed, hold to.

## Next

- What still fails by chance, each about one run in several hundred:
  a finding or request line that does not hold its form, which the
  review reads as a blocking finding and a case fails; a request to
  the author where a finding is due; a reviewer that opens main's copy
  of a file it was told to read under `pr-head/`; a fair remark on a
  clean change. The cases are there to catch these.
- The word `request:` reads as a request to the author, and three
  runs used it that way. Another word for the form is a change to
  ENG-28's report grammar.
- A reviewer still works out one value: the path it reads a changed
  file at, `pr-head/` before the path its diff names. A work directory
  that holds one tree, the pull request's with main's instruction
  files, would leave nothing to work out. It changes the review's
  workflow, so it is a plan of its own.
- 65 written cases still know a finding by words of its sentence. A
  case moves to place, ids and source when a run shows its words
  miss a right finding.
- ENG-28's row lists what a specialist's context names and does not
  yet list the ENG-29 record.
- The first scorecard is published from the first scheduled run that
  passes on main, with `review-eval publish`.
