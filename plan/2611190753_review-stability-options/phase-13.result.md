---
n: 13
title: "A specialist's turn budget sits above the turns its runs take"
status: "✅"
result: true
summary: >-
  The five specialists whose recorded runs reached their budget of 20
  turns get 30, above the 26 the longest of them took: domain-model-hub,
  lang-markdown, lang-gherkin, test-engineer-unit and
  test-engineer-integration. The review of §2, §5, §5a and §7 written
  as blocks had named a part of the hub partial, cut off at 20 turns
  before it stated anything. No requirement or case changes.
---
# Phase 13 handoff

## Done

- `domain-model-hub`, `lang-markdown`, `lang-gherkin`,
  `test-engineer-unit` and `test-engineer-integration` each get 30
  turns where they had 20. Every other specialist keeps 20.

## Verified

- `review-eval run` of the five on haiku, with 30 turns: 23 of their
  24 cases passed, for 0.23 USD. One passing run of `lang-markdown`
  took 22 turns, which a budget of 20 would have cut off.
- `lang-markdown`'s `defect-large-scope` passed 1 of 3 runs. Its two
  failed runs answered at 14 and 12 turns: one stated another defect
  instead, the other stated the right one without the requirement id
  its line needs. Run 4 more times on main and 4 on this change, it
  passed all 8, at 12 to 17 turns, so the budget does not cause its
  misses.
- `mdsmith check .` passes.

## Next

- None in this phase. The open phases of this plan are 4 to 7.
