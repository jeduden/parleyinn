---
n: 13
title: "§6's threat model in its own file"
status: "✅"
result: true
summary: >-
  §6.1, the threat model, moved word for word to a file of its own,
  06a, on the stakeholder's ruling. Its 28 threats and 8 residual risks
  are list items where they were rows of up to 630 and 571 characters,
  and its assets and threat sources stay tables. §6 falls from 663
  lines to 587, and 06a holds 179. A new specialist checks 06a. No
  requirement changes, and no scenario comes off @pending.
---
# Phase 13 handoff

## Done

- §6.1 moved to `docs/srs/06a-threat-model.md`, titled 6. Security:
  threat model. Its assets and threat sources stay tables. Each threat
  and residual risk is a list item: the id in bold, then a colon and
  its text one sentence a line, a threat's vector and controls fields
  under it, wrapped at 80 columns.
- §6 keeps §6.2. A sentence where §6.1 stood points at 06a and 06b,
  and §6's summary says so. No other line of §6 changed.
- `srs-threat-model` checks 06a as `srs-boundary-register` checks 06b,
  its text the same but for the file and its title. Its evaluation
  cases derive from the section.
- The threat model gate (SEC-17) reads §6.1's threat sources from 06a,
  and its drift case's message names 06a. The finding ledger's three
  quotes from §6.1 name 06a. `SECURITY.md`, the threat model page and
  an older plan's link point at 06a.

## Verified

- A comparison outside the repository found each of the 36 items the
  same as its row, and every other line of §6.1 and §6 unchanged but
  §6's summary and the sentence pointing at 06a.
- The workspace's tests first failed the finding ledger's check: its
  three quotes from §6.1 named §6's file. Named at 06a, they pass.
- `mdsmith check .`, the srs gates, the scenario gate, the workspace's
  4,644 tests and the cucumber scenarios `@ENG-21`, `@ENG-27`,
  `@ENG-28`, `@ENG-32` and `@ENG-34` pass.
- `review-eval check` finds every case applies: 238 cases, 75
  components. The new specialist passed its 3 cases in 6 of 6 runs.
  Three runs slipped: the answer held more than its outcome line.
- The drift suite caught the 9 cases that inject a file the phase
  touches: the disclosure policy's, the threat model gate's seven and
  the privacy statements'.
- `scripts/review-local.sh` against phase 12's head consulted 8 of 8
  agents, with no finding, for 0.25 USD. It asked for changes only for
  the consent the new specialist's file needs. Its one nit: the
  finding ledger reached the merge unread.

## Next

- Every table task 5 names is a list. The review section waits on
  #161, with task 2.
- The size list's file and the finding ledger reach a merge unread: no
  specialist is dispatched for either, so each change to them leaves
  the review a nit.
