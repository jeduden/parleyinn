---
n: 13
title: "§6's threat model in its own file"
status: "🔳"
result: false
---
# Phase 13: §6's threat model in its own file

Requirements. None changes, and no scenario comes off `@pending`. This
phase is task 5's eighth pull request. §6's threats and residual risks
held rows of 630 and 571 characters. As lists they would take §6 past
the srs kind's cap of 700 lines, to about 760. Splitting the threat
model out is the option the stakeholder chose on 22 November 2026.

The edit. §6.1, the threat model, moves word for word to
`docs/srs/06a-threat-model.md`, titled 6. Security: threat model. Its
assets and threat sources stay tables, their rows at 130 and 243
characters. Its 28 threats and 8 residual risks become list items: the
id in bold, then a colon and its text one sentence a line, a threat's
vector and controls fields under it, wrapped at 80 columns. §6 keeps
§6.2, and a sentence where §6.1 stood points at 06a and 06b; its
summary says so. No other line of §6 changes.

The specialist. The review runs a specialist as main defines it, and
the evaluation builds a case's world from main's tree. So
`srs-threat-model` lands in a pull request of its own before the
section, as the review section's specialist did (#146, #147). Its text
is `srs-boundary-register`'s but for the file and its title, and it
says its file is not on main yet. Its three cases bring a file of their
own, `docs/srs/06c-threat-model.md`, which the review bundle hands it
too. The section's pull request leaves the specialist as main has it.
Once 06a is on main, a third pull request gives the specialist its
final text, and the cases derived from the section join its three.

The readers. The threat model gate (SEC-17) reads §6.1's threat sources
from 06a, and its drift case's message names 06a. The finding
ledger's three quotes from §6.1 name 06a, where its check finds them.
`SECURITY.md`, the threat model page and an older plan's link point at
06a.

Consent. CODEOWNERS names `.claude/agents` and `tooling/review-eval`.
The specialist's pull request asks for the stakeholder's consent to
both, and the third one to `.claude/agents`. The section's pull
request changes neither.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- a comparison outside the repository finds each item the same as its
  row, and every other line of §6.1 and §6 unchanged but §6's summary
  and the sentence pointing at 06a;
- the srs gates, the scenario gate, the workspace's tests and the ENG
  scenarios pass, and `review-eval check` counts the new specialist;
- the evaluation, run as the review workflow runs it against main's
  checkout, passes the specialist's three cases;
- the drift suite catches the cases that inject the threat model page
  and §6;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
