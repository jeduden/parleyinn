---
n: 1
title: "Every reader takes a requirement written as a block"
status: "✅"
result: false
---
# Phase 1: every reader takes a requirement written as a block

Requirements. None changes, and no scenario comes off `@pending`. The
SRS keeps its tables: this phase lands only the reading, because the
review of a rewrite runs main's tools.

RED. Unit tests of the `srs` crate. A block reads as the row it
replaces, its priority, method, traces and text alike, at its
heading's line. Its text joins its lines and its paragraphs with
single spaces, and a field wrapped onto an indented line reads whole.
A block inside a code fence is none, and a heading that is no
requirement id ends the block before it. A block with an unknown
field, a field twice, no field, no text or a malformed trace is
malformed, at its line. A row and a block that share an id are an id
defined twice. A unit test of the derived cases: a section whose first
requirement is a block gets the defect case that adds a MUST to it,
not the case of excluded terms.

GREEN. A `block` module of the `srs` crate reads the blocks of a
document, and `parse` returns its rows and blocks in the document's
order. `derived.rs` finds a section's first requirement as a row or
as a block, and its defect case adds the MUST on a line of its own
after the block's text. The testing page says how a requirement is
written, and the srs kind's cap in `.mdsmith.yml` rises to 700 lines,
the stakeholder's ruling.

Gate. A drift case creates a section holding a block with ENG-25's
id, which §10's table defines, and the parse gate fails naming it.
Then, on a scratch branch only, the review section is rewritten as
blocks, and each of these is recorded in the handoff:

- `mdsmith check .` passes;
- the scenario gate, the srs gates, the cucumber scenarios `@ENG-28`
  and `@ENG-32`, and the drift cases that inject into §10 pass;
- `review-bundle` against main shows no requirement changed;
- `review-eval run --component srs-review-requirements` passes its
  derived cases on the rewritten tree;
- `scripts/review-local.sh` approves.

The rewrite is a mechanical edit made outside the repository. Main's
bundle proves each one, so no tool of the repository writes blocks.

BDD. No cross-host behaviour: no `@S` scenario.
