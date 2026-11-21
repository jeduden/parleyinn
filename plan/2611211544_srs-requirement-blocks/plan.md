---
id: 2611211544
title: "Each SRS requirement under its own heading, in short lines"
status: "🔳"
summary: >-
  A requirement of the SRS is one row of a table, padded to the
  table's widest row: ENG-28's line holds 13,239 characters, and a
  search shows a line only up to about 500. This plan writes each
  requirement under a heading that is its id, with its fields as a list
  and its text one sentence per line, so every tool shows it, a diff
  shows the sentences a change edits, and every gate reads it as it
  read its row.
model: opus
depends-on: []
---
# Each SRS requirement under its own heading, in short lines

## Goal

Every requirement of the SRS is written under a heading that is its
id. Its fields are a list, and its text has one sentence per line, no
line over 80 characters. Every tool an agent searches or reads with
then shows a requirement's lines. A diff shows only the sentences a
change edits, and every gate reads each requirement as it read its
row.

## Context

The local review of pull request 154 requested changes because
`lang-gherkin` ran out of turns before it stated a line. It spent
nineteen searches on ENG-28's row: a search shows a matching line only
up to about 500 characters, and ENG-28 is one line of 13,239. Phase 11
of plan [2611190753](../2611190753_review-stability-options/plan.md)
has the review context quote each touched requirement whole (#161).
Every other way an agent or a person reads the SRS still meets such
lines. 24 of the 33 files under `docs/srs` hold lines over 500
characters, 15 of them in their requirement tables. On 21 November
2026 the stakeholder chose to rewrite the requirements, not to rely on
the quotes alone.

Why the rows are long. A requirement's text is a paragraph or more,
and a table cell holds it on one line. mdsmith's `table-format` pads
each column to its widest cell, so every row is as wide as the widest,
and a change that widens one row repads them all: its diff shows the
whole table changed.

What was searched, and what is reused:

- [invariants.md](../../docs/srs/invariants.md) already writes each
  invariant under its own heading, which agents include one at a time.
  A requirement block takes that shape.
- The `srs` crate's parser ([lib.rs](../../tooling/srs/src/lib.rs)) is
  the one reader the gates share: the scenario gate, the Appendix B
  and C gates, the review bundle, the review run's known ids and the
  engineering checks all call `srs::load` or `srs::parse`. Reading
  blocks there reaches them all.
- The review bundle compares a requirement by its fields and its text,
  not by where it sits. A rewrite that keeps both changes no
  requirement in its eyes, so main's own bundle proves each rewrite.
- Two readers parse rows themselves: the evaluation's derived cases
  ([derived.rs](../../tooling/review-eval/src/derived.rs)) and the
  drift cases that inject into a row. Both move with the rewrite.

The order. The review of a pull request runs main's review bundle and
main's evaluation. A pull request that rewrote a section before main
read blocks would show main's bundle every requirement of the section
removed, and the evaluation would derive its specialist another case.
So reading blocks lands first, alone. Each rewrite then lands through
main's own tools.

The size. Rewritten, the 15 files grow from 797 lines to about 6,020
and shrink from 965 KB to 318 KB. Eleven exceed the 300 lines mdsmith
allows an SRS section; the review section, 10a, takes 247. A section
specialist reads the requirements a change touches, not its whole
section. On 21 November 2026 the stakeholder ruled to raise the srs
kind's cap in `.mdsmith.yml` to 700 lines, so no section splits;
phase 1 lands it.

## The requirement block

```markdown
## ENG-28

- Pri: P0
- Ver: T

An agent's approval (ENG-21) MUST be posted by a GitHub App identity that
authors no change, from a workflow that runs as the default branch defines it,
never as the pull request under review defines it.
The review's agents MUST have no credential that can approve or write.
```

- The heading is one level below its section's, and its text is the id
  alone.
- The fields are one list item for each column of the table other than
  the id and the text, in the table's order: `Label: value`, the value
  as its cell holds it. A long value wraps onto lines indented by two
  spaces.
- The text runs from the list to the next heading: one sentence per
  line, a sentence longer than 80 characters wrapped at a space. No
  line opens with what Markdown reads as a list, a quote, a table or a
  heading.
- The parser reads Pri, Ver and Traces as it read those columns, and
  the text as its lines joined by single spaces. A row and its block
  read the same.

## Tasks

1. Every reader of the SRS takes a requirement written as a block,
   beside the tables. A rewrite of the review section on a scratch
   branch proves it (phase 1).
2. The review section is rewritten through main's tools, once main's
   `lang-gherkin` reads a requirement as its review context quotes it
   (#161), not by a search for a row (phase 1's handoff).
3. The other fourteen sections are rewritten, a few for each pull
   request, with each drift injection that quotes a rewritten row
   moved. Each keeps its one file, under the srs kind's cap of 700
   lines.
4. The parser refuses a requirement table, with a drift case. ENG-28,
   the agents' shared sections, the docs and the scenarios speak of a
   requirement where they say row.
5. The other wide tables follow, each with the reader its gate uses:
   the threats, the boundary register, the command-line interface, the
   delivery plan, the open questions, the change log and Appendices A
   and B.

## Execution

| Phase | Model | Gate                                                                                                                       |
| ----- | ----- | -------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus  | A block that redefines a row's id fails the parse gate's drift case; the review section rewritten locally passes its gates |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| --- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | ✅     | [Every reader takes a requirement written as a block](phase-1.md)                                                                                                                                                                                                                                                                                                                                                                                                               |
|     | ↳      | Every reader of the SRS now takes a requirement written as a block beside the tables: the srs crate's parser, which every gate shares, and the evaluation's derived cases. No requirement changes, and no scenario comes off @pending. Rewritten as blocks on a scratch branch, the review section passed its gates, its derived cases and the local review, whose bundle showed no requirement changed. A drift case lands with it, and the srs kind's cap rises to 700 lines. |
<?/catalog?>

## Acceptance Criteria

- [ ] Every requirement under `docs/srs` is a block, and the parser
      refuses a requirement table
- [ ] No line of a requirement is longer than 80 characters, and
      `mdsmith check .` passes
- [ ] Each rewrite's review bundle shows no requirement changed, and
      its review approves within the pull request's budget
- [ ] The scenario gate, the srs gates, the derived cases and the drift
      suite pass after every rewrite
- [ ] Unit tests, clippy denying warnings and rustfmt pass
