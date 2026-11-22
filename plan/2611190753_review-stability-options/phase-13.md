---
n: 13
title: "A specialist's turn budget sits above the turns its runs take"
status: "✅"
result: false
---
# Phase 13: a turn budget above the turns a run takes

Requirements. None changes, and no scenario comes off `@pending`.
ENG-28 has the review name a specialist partial when its run is cut
off before it stated any line or an outcome, never a clean pass. This
phase keeps a specialist's turn budget from being what decides that.

What happened. The review of #165, which writes §2, §5, §5a and §7 as
blocks, requested changes on one finding. One of the four parts of
`domain-model-hub` ran out of its 20 turns before it stated anything,
so the review of it was partial. The same part had answered in 12 and
20 turns in two local reviews of the same diff.

Why it happens. A specialist's turns vary from run to run with the
searches its model chooses. Across the runs the review kept, the hub
reached 20 turns in 5 of 48, its 90th percentile, and took 25 at most.
In its longest runs, 3 to 10 of its calls read requirement rows other
sections still hold, each wider than a search shows. `lang-markdown`
reached 20 in 5 of 73 runs, `test-engineer-unit` in 3 of 33, and
`lang-gherkin` and `test-engineer-integration` once each. No other
specialist's run reached its budget.

The rule. Each of the five gets 30 turns, above the most any recorded
run of them took, 26. The others keep 20.

RED. None: the instructions alone change.

GREEN. The front matter of the five agent files.

Gate. `review-eval run` passes each case of the five on its model.
The review of the pull request runs them again before its gate
decides. `mdsmith check .` passes.

BDD. No cross-host behaviour: no `@S` scenario.
