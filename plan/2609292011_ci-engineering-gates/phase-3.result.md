---
n: 3
title: "Deadlines on every blocking operation (ENG-03)"
status: "🔳"
result: true
summary: >-
  The core's clippy configuration bans 15 blocking calls that take no deadline and a gate test holds every product crate to them; the stakeholder accepted its exemptions, and the ENG-03 scenario stays pending until M1 adds injectable I/O traits.
---
## Handoff

What landed:

- `crates/parinn/clippy.toml` bans the standard library's blocking
  calls that take no deadline, each reason naming the deadline form:
  channel receive and send, condition and barrier waits, thread join
  and park, child waits, `Command::output` and `status`, file locks and
  `io::stdin`.
- Not banned, by decision: local file reads and writes (bounded by the
  disk and the hook budget), `thread::sleep` (it carries its own bound)
  and in-process locks. `docs/development.md` says so.
- The gate test `the_product_crates_ban_every_wait_with_no_deadline`
  holds every product crate to the list; drift cases guard it, and it
  is registered in the drift registry's gate tests beside
  `no_product_crate_keeps_mutable_global_state` and
  `the_product_crates_keep_their_bans`.
- `@ENG-03` is bound in `engineering_analyzers.rs`, its global-state
  and clock steps to the existing checks, but stays `@pending`. Its
  deadline step is a ban list that exempts local file I/O, in-process
  locks and standard output. The stakeholder accepted those exemptions
  on 2026-10-13 ([ruling]), and kept the scenario pending until M1 adds the
  injectable I/O traits the row also requires.

What remains: "all I/O through injectable traits" has no mechanical
check, and a crate's own blocking loop or a dependency's is not seen by
the list.

[ruling]: https://github.com/jeduden/parleyinn/pull/44#issuecomment-6157262395
