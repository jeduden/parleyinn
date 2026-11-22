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
row. No requirement holds more than 50 lines, so none grows as
ENG-28's row did.

## Context

The local review of pull request 154 requested changes because
`lang-gherkin` ran out of turns before it stated a line. It spent
nineteen searches on ENG-28's row: a search shows a matching line only
up to about 500 characters, and ENG-28 is one line of 13,239. Pull
request #161 has the review context quote each touched requirement
whole, a phase it adds to plan
[2611190753](../2611190753_review-stability-options/plan.md).
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

The length. Nothing capped a requirement: a table is exempt from the
80-column limit, and the srs kind turns the readability rules off.
Rewritten as blocks, the 310 requirements and assumptions run to a
median of 14 lines, from the heading to the next. Six run past 50:
ENG-28 to 192, PRV-12 to 135, PRV-08 to 63, LANE-34 to 60, SEC-31 to
53 and LANE-01 to 51. On 22 November 2026 the stakeholder capped a
requirement at 50 lines, through mdsmith's `max-section-length` on
the srs kind, with those six listed at their length until each is
split; phase 1 lands it. ENG-28's entry is 193, its length with the
clause #161 adds, since its section is rewritten only after #161
lands.

The shape. The srs crate reads a block line by line, and mdsmith
reads it as Markdown. On 22 November 2026 the stakeholder asked that
mdsmith lint the shape the parser depends on. Phase 1 gives the srs
kind a schema for it, and has the parser read a field's lines and a
table's start as Markdown does, so the two read a block alike.

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
  line, a sentence longer than 80 characters wrapped at a space, all
  one paragraph.
- The parser reads Pri, Ver and Traces as it read those columns, and
  the text as its lines joined by single spaces. A row and its block
  read the same. A field's lines run to the blank line that ends the
  list, as Markdown reads a list item's, and a block ends at the next
  heading or where a table opens.
- mdsmith's schema for the srs kind holds a block to this shape: a
  list marked with dashes, then one paragraph and nothing else, at its
  level. What it passes, the parser reads as Markdown does.
- A block holds at most 50 lines, from its heading to the next, or
  the length the srs kind lists for it until it is split.

## Tasks

1. Every reader of the SRS takes a requirement written as a block,
   beside the tables, a block holds at most 50 lines, and mdsmith
   checks its shape. A rewrite of the review section on a scratch
   branch proves it (phase 1).
2. The review section is rewritten through main's tools, once main's
   `lang-gherkin` reads a requirement as its review context quotes it
   (#161), not by a search for a row (phase 1's handoff).
3. The other fourteen sections are rewritten, a few for each pull
   request, with each drift injection that quotes a rewritten row
   moved. Each keeps its one file, under the srs kind's cap of 700
   lines.
4. The parser refuses a requirement table, with a drift case. ENG-28,
   the agents' shared sections, the docs, the scenarios and the srs
   crate's stated responsibility speak of a requirement where they say
   row or table.
5. The other wide tables follow, each with the reader its gate uses:
   every table with a row over 500 characters, the most a search shows
   of a line. They are the change log, the delivery plan's milestones,
   the open questions, the command-line interface, the reference
   architecture's components and decisions, the threats and the
   residual risks, the store's tables, the MCP tools and Appendices A
   and B.
6. Each requirement the srs kind lists past 50 lines is split into
   requirements that fit, each with its scenario, and leaves the
   list. One only shortened has its entry lowered to its length.

## Execution

| Phase | Model  | Gate                                                                                                                                                                 |
| ----- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus   | A block redefining a row's id fails the parse gate's drift case, one too long or out of shape fails mdsmith's; the review section rewritten locally passes its gates |
| 2     | sonnet | Main's review bundle shows no requirement of the four sections changed, and mdsmith, the gates and the local review pass                                             |
| 3     | sonnet | Main's review bundle shows no requirement of the four sections changed, and mdsmith, the gates and the local review pass                                             |
| 4     | sonnet | Main's review bundle shows no requirement of the three sections changed, and mdsmith, the gates and the local review pass                                            |
| 5     | sonnet | Main's bundle shows no requirement changed; its drift cases are caught                                                                                               |
| 6     | sonnet | The change log is a list, word for word; the gates pass                                                                                                              |
| 7     | sonnet | The milestones are headings, word for word; the gates pass                                                                                                           |

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

| #   | Status | Phase                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| --- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | ✅     | [Every reader takes a requirement written as a block](phase-1.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|     | ↳      | Every reader of the SRS now takes a requirement written as a block beside the tables: the srs crate's parser, which every gate shares, and the evaluation's derived cases. No requirement changes, and no scenario comes off @pending. Rewritten as blocks on a scratch branch, the review section passed its gates and the local review, and its bundle showed no requirement changed. The srs kind's cap rises to 700 lines, a requirement may hold 50, six longer ones listed until each is split, and mdsmith checks a block's shape. |
| 2   | ✅     | [§2, §5, §5a and §7 written as blocks](phase-2.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|     | ↳      | The 22 assumptions of §2.3 and the 66 requirements of §5, §5a and §7 are each written as a block, and none changes: main's review bundle shows each with main's text alone. The four files fall from 184,608 bytes to 62,879, and no line of a block passes 80 columns, where a row reached 2,772 characters. No scenario comes off @pending.                                                                                                                                                                                             |
| 3   | ✅     | [The room sections and §5d written as blocks](phase-3.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
|     | ↳      | The 67 requirements of the room family, LANE, the room collaboration and the room view, and of the peer network are each written as a block, and none changes: main's review bundle shows each with main's text alone. The four files fall from 204,921 bytes to 70,291, and no line of a block passes 80 columns, where a row reached 3,757 characters. No scenario comes off @pending.                                                                                                                                                  |
| 4   | ✅     | [§5c, §5e and §5f written as blocks](phase-4.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
|     | ↳      | The 89 requirements of the principal acts, provenance and trust, and pins, recall and restore are each written as a block, and none changes: main's review bundle shows each with main's text alone. The three files fall from 298,088 bytes to 70,944, and no line of a block passes 80 columns, where a row reached 9,037 characters. No scenario comes off @pending.                                                                                                                                                                   |
| 5   | ✅     | [§6, §6b and §10 written as blocks](phase-5.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
|     | ↳      | The 64 requirements of security, the boundary register's own and engineering quality are each written as a block, and none changes: main's review bundle shows each with main's text alone. The three drift cases that quoted §10's rows inject into its blocks and are caught. Every requirement of task 3's fourteen sections is now a block. No scenario comes off @pending.                                                                                                                                                           |
| 6   | ✅     | [The change log written as a list](phase-6.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
|     | ↳      | The SRS's change log is a list, its 48 entries each a version, a date and a summary one sentence a line, word for word as the table held them. The index falls from 611,025 bytes in 136 lines to 41,442 in 521, and an entry no longer repads the others. No requirement changes, and no scenario comes off @pending.                                                                                                                                                                                                                    |
| 7   | ✅     | [The delivery plan's milestones written as headings](phase-7.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
<?/catalog?>

## Acceptance Criteria

- [ ] Every requirement under `docs/srs` is a block, and the parser
      refuses a requirement table
- [ ] No line of a requirement is longer than 80 characters, and
      `mdsmith check .` passes
- [ ] No requirement holds more than 50 lines, and the srs kind lists
      none
- [ ] Each rewrite's review bundle shows no requirement changed, and
      its review approves within the pull request's budget
- [ ] The scenario gate, the srs gates, the derived cases and the drift
      suite pass after every rewrite
- [ ] Unit tests, clippy denying warnings and rustfmt pass
