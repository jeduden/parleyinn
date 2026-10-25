---
n: 16
title: "Exit: the engineering gates, the recall evaluation and the S12 rebuild test"
status: "✅"
result: true
summary: >-
  @ENG-03, @ENG-07 and @ENG-11 off @pending and passing. parinn-eval
  recalls 216 of 216 planted values after one to three compactions
  within ten hits; the S12 rebuild test passes. @ENG-06, @ENG-09,
  @ENG-15, @ENG-23 and @ENG-29 stay @pending, each raised.
---
# Phase 16 result

## Handoff

Outcome:

- ENG-03, ENG-07, ENG-11: passing (cucumber); ENG-07 and ENG-11 have two
  drift cases each.
- parinn-eval: seeded runs in the 2.1.296 line shapes; a scripted agent
  asks event_search then event_get. Within ten hits, 72/72 at each depth
  (Wilson 95% from 94.9%). Within five, depth 3 falls to 87.5%. Without
  recall, the context after the last compaction holds 0/216. CI's
  `eval` job holds `tooling/eval/recall-v1.txt` to a fresh run.
- S12: `crates/store/tests/s12_rebuild.rs` covers either order, row ids
  from addresses, secure-delete and truncated logs. The trust policy's
  unit tests show trust needs the mode.
- ENG-06: kills of purge, at a terminal, and of rebuild. They found
  verify exiting 3 after a kill between a purge's transaction and its
  finishing. Verify stays read-only: it names the unfinished purge
  (`pending_purges`), counts it, leaves the index unchecked and exits
  0; the next purge, ingest or rebuild finishes it. A built case and
  the seeded kills both check it.
- ENG-09: a soak of 50 writers; at 1M events (release, 13 min), 4,293 of
  10,000 appends passed the 5 s busy deadline and were retried, as live
  ingest retries. It runs nightly.
- NFR-05: a floor benchmark; each run costs ingest about 12 ms.
- Gates run: scenario gate, fmt, clippy `-D warnings`, `mdsmith check .`,
  `cargo test --workspace` (156 scenarios), `cargo run -p coverage`
  (100% in all layers, unit 99% or more), after merging main.

Raised (scenarios stay `@pending`):

- ENG-06: no migration before M3 (ruling 3).
- ENG-09: 1M events cannot run in a pull request's scenario budget; a
  ruling on splitting nightly from PR steps is wanted.
- ENG-15: NFR-01 needs the room view (M7); NFR-05's target needs the
  reference hardware.
- ENG-23: no release and no migration yet.
- ENG-29: no mechanical check; ADR-2610221545 pins no invariants hash.

For the stakeholder:

- Names (no domain-model agent here): `parinn-eval`, `ingest_fifty_runs`,
  `run_killed_at_terminal`.
- Counters, domain-model agent verdict: `verify_unfinished` did not fit
  (the cause is a deadline); renamed here to `verify_timed_out`, and the
  `parinn verify` report key `unfinished_purges` to `pending_purges`
  (purges awaiting the next rebuild). `purge_unfinished` does not fit
  either: the model says `purge_failed` or `purge_timed_out`, and the
  report key's analogue likewise; both kept for now on main, stakeholder
  to rule. `recall_refused` fits; `recall_rejected` is the strict-literal
  alternative.
- Verify's derived-index recheck is bounded at 60 s
  (`INDEX_CHECK_SECS`). On a store whose derivation always takes longer,
  ADM-09's derived artifact check never runs (NFR-05). A ruling on the
  bound is wanted.
- ENG-03's hold on injectable I/O: the seams are listed in
  development.md.
- Verify no longer writes (ruled); wording on PR #90.
- §11.1 measurement wording on claude/m1-phase-16-docs (PR #90).
- Stakeholder paths: `.github/workflows/ci.yml` and `nightly.yml`,
  `tooling/`, `Cargo.toml`, `tooling/*/Cargo.toml`.
- Inherited items untouched: S2 `integrity`, phase 14/15 counters and
  renames, `tombstones`.

Plan close (stays 🔳): met are recall, S12, dependencies, ENG-11,
fuzzing, frozen targets, tests and checks. Open: the raised full-row
scenarios (SEC-12, SEC-17, SEC-19, ENG-22, ADM-09, RCL-02, PRV-09,
REC-24, LANE-02), the in-part rows' audit, verify's landmark row, the
erasure residue (phase 15), and ENG-15.
