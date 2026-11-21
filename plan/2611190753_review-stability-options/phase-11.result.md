---
n: 11
title: "A specialist reads the requirements its files touch in its context"
status: "✅"
result: true
summary: >-
  ENG-28 gains a clause: each specialist's review context quotes whole
  each touched requirement its files change or name, one sentence or
  clause per line. A requirement is one table line a search shows only
  in part, and `lang-gherkin` had spent its twenty turns searching
  ENG-28's 13,856 characters. It now weighs each step against its row
  as its context quotes it, and leaves to CI the tags and bindings CI
  proves. The ENG-28 context check, two drift cases and a new
  `lang-gherkin` case land with it.
---
# Phase 11 handoff

## Done

- The review bundle names the touched ids each changed file holds or
  names. Each context quotes those requirements whole, one sentence or
  clause per line, as the pull request has them, or as main has one it
  removes.
- ENG-28's row and its scenario step say so. The ENG-28 context check
  holds the context of a change that names ENG-28 to its row, quoted
  whole at its place.
- `lang-gherkin` reads its diff and its context and searches nothing.
  It leaves to CI the tags and bindings that the scenario gate test and
  the scenario run prove.

## Verified

- Unit tests of `review-bundle` and `engineering` pass, with clippy
  denying warnings.
- The cucumber scenario `@ENG-28` and the scenario gate test pass.
- The drift suite catches the two new cases and the thirteen others
  that inject the files this phase changes.
- `review-eval run --component lang-gherkin` on haiku, with the new
  case. Before: 5 of 5 cases, 10 of 11 runs, 5.1 turns a run, the new
  case in 8 and 5 turns. After: 5 of 5 cases, 10 of 11 runs, 3.3 turns
  a run, the new case in 3 and 3.

## Not done

- The shared section of the SRS section specialists still has them
  search for a row a diff names but does not change.
- A clean case of `lang-gherkin` fails one run in three, before and
  after the change: it reads a reworded step as weaker than its row.

## Next

- Land after pull request 154, and see a review of a change to a long
  row run without a specialist's searches.
