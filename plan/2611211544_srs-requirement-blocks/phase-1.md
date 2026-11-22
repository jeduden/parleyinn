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
not the case of excluded terms. A drift case writes a block of 51
lines before §10.4's heading, which `mdsmith check .` passes. A line
under a field continues it up to a blank line, and a line of the text
that starts with a pipe stays text unless a delimiter row is under it.
Six drift cases write a block of a broken shape before §10.4's
heading, each of which `mdsmith check .` passes.

GREEN. A `block` module of the `srs` crate reads the blocks of a
document, and `parse` returns its rows and blocks in the document's
order. `derived.rs` finds a section's first requirement as a row or
as a block, and its defect case adds the MUST on a line of its own
after the block's text. The testing page says how a requirement is
written, and the srs kind's cap in `.mdsmith.yml` rises to 700 lines,
the stakeholder's ruling. The srs kind caps a requirement at 50 lines
through mdsmith's `max-section-length`, the six longer ones listed at
their length, the stakeholder's ruling of 22 November 2026. Its
schema holds a block to the shape the parser reads, as the stakeholder
asked the same day: a list of fields marked with dashes, then one
paragraph and nothing else, at its level. The parser and the derived
cases read a field's lines and a table's start as Markdown does.

Gate. A drift case creates a section holding a block with ENG-25's
id, which §10's table defines, and the parse gate fails naming it.
The drift case of a block of 51 lines fails `mdsmith check .`, naming
it, and so does each of the six of a broken shape. Then, on a scratch
branch only, the review section is rewritten as blocks, and each of
these is recorded in the handoff:

- `mdsmith check .` passes;
- the scenario gate, the srs gates, the cucumber scenarios `@ENG-28`
  and `@ENG-32`, and the drift cases that inject into §10 pass;
- `review-bundle` against main shows no requirement changed;
- `review-eval run --component srs-review-requirements` passes its
  derived cases on the rewritten tree;
- `scripts/review-local.sh` approves.

With every section rewritten on a scratch tree, `mdsmith check .`
passes under the cap, and fails with any listed length lowered by
one.

The rewrite is a mechanical edit made outside the repository. Main's
bundle proves each one, so no tool of the repository writes blocks.

BDD. No cross-host behaviour: no `@S` scenario.
