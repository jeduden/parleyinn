---
n: 10
title: "The command-line interface written as a list"
status: "✅"
result: false
---
# Phase 10: the command-line interface written as a list

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 5's fifth pull request. The command table of §9.5 held
83 rows, each padded to 1,545 characters, so the section held 134,889
bytes in 140 lines. No gate reads it.

The edit. Each row becomes a list item, as the open questions did: its
commands, then a colon and its purpose one sentence a line, wrapped at
80 columns. A code span stays whole where it fits on a line of its own
and breaks at its own spaces where it does not, which Markdown renders
the same. The sentence after the table that names the acts install's
and uninstall's rows name says their entries, and the domain model's
hub points at the `licenses` entry where it pointed at its row. No
other line changes.

The line length. The synopsis of `parinn purge` and its nine scopes,
`--room|…|--principal`, is one word of 79 characters, so its line holds
81 columns in a list item. The srs kind's line length turns stern, as
the stakeholder chose on 22 November 2026: a line may pass 80 columns
only with no space past them. A drift case proves that a sentence run
on past 80 columns at a space is still caught.

RED. With the section rewritten and the line length not stern,
`mdsmith check .` fails on the synopsis's line: 81 > 80.

GREEN. The srs kind's line length is stern, `mdsmith check .` passes,
and the drift suite catches the run-on sentence.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- a comparison outside the repository finds each item's commands and
  purpose the same as its row's, and every other line of the section
  unchanged but the one reworded;
- the srs gates, the scenario gate and the workspace's tests pass, and
  the drift suite catches the new case;
- `scripts/review-local.sh` approves.

Consent. CODEOWNERS names `.mdsmith.yml`, so the pull request asks
once for the stakeholder's consent.

BDD. No cross-host behaviour: no `@S` scenario.
