---
id: ADR-2610161200
title: "Redaction's patterns: the regex crate"
status: accepted
scope: dependencies
summary: >-
  The redaction rule set and the principal's redaction patterns
  (redaction.extra_patterns) match through the regex crate: finite
  automata, linear time on every input, each pattern compiled under a
  size limit. Default features off; std, unicode and perf on.
---
# ADR-2610161200: Redaction's patterns

## Context

SEC-08 requires a maintained, versioned rule set for common secret
formats plus patterns the principal defines in its own configuration,
applied to every text field, payload and hook input before anything
is written. `@SEC-08`'s scenario writes such a pattern as a regular
expression, `ACME-[0-9]{8}`. The text redaction reads is hostile (T1,
T2), and so is a pattern a principal pastes in, so matching must stay
linear in the input whatever the pattern. Phase 4 of plan
[2609292012](../../plan/2609292012_m1-record-and-recall/plan.md)
builds redaction. The stakeholder chose this crate over a matcher of
ParleyInn's own on 16 October 2026.

## Decision

| Module  | Purpose                                                                                     | License           | Maintenance                                                           |
| ------- | ------------------------------------------------------------------------------------------- | ----------------- | --------------------------------------------------------------------- |
| `regex` | Compiles the redaction rule set and the principal's redaction patterns; finds their matches | MIT OR Apache-2.0 | Active; the rust-lang organisation, the de-facto regex crate for Rust |

The core takes it without default features, with `std`, `unicode` and
`perf`: `unicode` keeps the classes a principal expects (`\w`, `\d`,
case folding), and `perf` keeps the literal and lazy-DFA fast paths
ingest's throughput (NFR-04) needs. `regex` has no backreferences or
look-around, so every match runs in time linear in the text. Each
pattern compiles under a size limit; one past it is refused, audited
and counted, and the rest still apply.

## Alternatives

- **A matcher of ParleyInn's own.** A linear-time subset with no
  dependency, but parser and automaton code the core must fuzz and
  keep correct, where a miss leaves a secret in the append-only
  record.
- **Hand-written matchers for the rule set alone.** Covers the
  maintained rules but not SEC-08's principal patterns.
- **`fancy-regex` or `pcre2`.** Backtracking, so a hostile pattern or
  text can take exponential time; `pcre2` also links C.

## Consequences

- The core's reach test allows `regex`, `regex-automata`,
  `regex-syntax` and `aho-corasick`, which compile patterns and match
  text in memory. None opens a socket or starts a program.
- The direct set linked into the core is seven, against ENG-18's
  target of ten.
- The redactor's fuzz target joins the first `fuzz/` crate, in phase 7.
