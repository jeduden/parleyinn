---
n: 3
title: "Deadlines on every blocking operation (ENG-03)"
status: "🔳"
result: false
---
# Phase 3: deadlines on every blocking operation (ENG-03)

Requirement. ENG-03 (P0, Ver I, T): no mutable global state, I/O
behind injectable traits, every blocking operation takes a deadline,
and code that computes derived artifacts reads no clock or
randomness.

What exists: the clippy bans on clocks and hash collections in each
product crate, and two gate tests that hold every crate under
`crates/` to them. The Go plan's custom analyzers become these lists
and gate tests.

BDD coverage: `@ENG-03` comes off `@pending`. Its global-state and
clock steps bind to the existing checks. A new check fails the build
on a blocking call without a deadline.

RED: the scenario fails on the missing deadline check. GREEN: the
deadline check, the bindings, a drift case.

Gate: `@ENG-03` passes; a deadline-less blocking call is a caught
drift.

Status: open. The deadline check and its drift cases landed. The
stakeholder ruled on 2026-10-13
([ruling]): the ban list's exemptions (local file
I/O, in-process locks, standard output) are accepted. `@ENG-03` stays
`@pending` until M1 adds the injectable I/O traits the row also
requires, which no step checks yet.

[ruling]: https://github.com/jeduden/parleyinn/pull/44#issuecomment-6157262395
