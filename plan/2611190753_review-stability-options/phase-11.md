---
n: 11
title: "A specialist reads the requirements its files touch in its context"
status: "✅"
result: false
---
# Phase 11: the requirements a specialist's files touch, in its context

Requirements. ENG-28 gains a clause: the review context the review
bundle writes for a specialist quotes whole each touched requirement
its files change or name. No scenario comes off `@pending`.

What happened. The local review of pull request 154's merged head
requested changes because `lang-gherkin` ended on its turn budget
before it stated a line or its outcome. It had spent nineteen searches
on the ENG-28 row and on the bindings of its steps. A requirement is
one line of its table, padded to the table's widest row: ENG-28's line
was 13,856 characters, and a search shows a matching line only up to
about 500. The specialist was told the line was left out, and searched
again in pieces. Most agents of that review met such lines; one ran
out of turns.

Why it happens. A specialist's context named the touched ids but
quoted none of their rows, so a specialist that weighs a step or a
sentence against its row searched the specification for it. Two of
`lang-gherkin`'s checks are ones CI proves before any review approves:
the scenario gate test holds each scenario's tags to its row, and the
scenario run fails a live step that no binding, or more than one,
matches.

The rule. The bundle names, for each changed file, the touched ids it
holds or names. Each context quotes those requirements whole, one
sentence or clause per line, as the pull request has them, or as main
has one the pull request removes. `lang-gherkin` reads its rows there,
weighs a changed step against main's step before its row, leaves to CI
what CI proves, and searches nothing.

RED. Unit tests of `review-bundle`: the analysis names the ids each
changed file holds or names; a context quotes each requirement its
files touch, one sentence per line, and a row the pull request removes
as main has it. The ENG-28 context check holds the context of a change
that names ENG-28 to its row, quoted whole at its place. A
`lang-gherkin` evaluation case: a step and its binding say "more than
half" where the ENG-28 sentence 8,000 characters into its row says
"more than a third".

GREEN. The bundle's analysis and renderer, and `lang-gherkin`'s
instruction. The ENG-28 row and its scenario step say what a context
quotes.

Gate. Two drift cases fail: a context that quotes no requirement its
files touch, and one that quotes a requirement in part.
`lang-gherkin`'s cases pass on its model, before and after the
instruction change, with its turns counted.

Not in this phase: the shared section of the SRS section specialists,
which still has them search for the rows a diff names but does not
change. Their diffs hold the rows they change, whole.
