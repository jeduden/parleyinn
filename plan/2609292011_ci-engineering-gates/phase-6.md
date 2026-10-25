---
n: 6
title: "Main's protection and CODEOWNERS (ENG-21)"
status: "✅"
result: false
---
# Phase 6: main's protection and CODEOWNERS (ENG-21)

Requirement. ENG-21 (P0, Ver I): changes land only through reviewed
pull requests with signed commits. Each needs an approval from a
reviewer other than its author. The stakeholder approves the
requirement text and what enforces it. Security-sensitive crates are
owned by the designated security reviewer, whose approval a change to
them needs as a code owner.

Branch protection is a repository setting, so it becomes a committed
ruleset that a check compares with the live one; the stakeholder
applies the committed ruleset. The security reviewer is still to be
named, so the stakeholder holds that role in CODEOWNERS.

BDD coverage: `@ENG-21` comes off `@pending`. Its steps read the
committed ruleset and CODEOWNERS. Direct pushes are refused and
commits signed. One approval comes from a non-author. The stakeholder
owns every path that enforces the SRS, and no other. The
security-sensitive crates are owned by the security reviewer.

RED: the scenario fails on the missing ruleset. GREEN: the ruleset,
the CODEOWNERS check, the live comparison job, drift cases.

Gate: `@ENG-21` passes.
