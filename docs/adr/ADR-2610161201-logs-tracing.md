---
id: ADR-2610161201
title: "Structured logs: the tracing crate, with a subscriber of ParleyInn's own"
status: accepted
scope: dependencies
summary: >-
  ParleyInn logs through the tracing crate's events and macros, default
  features off, std on. A subscriber of ParleyInn's own writes each event as
  one JSON line to the home's log file, every field redacted first; no
  tracing-subscriber, no log bridge, no proc-macro attributes.
---
# ADR-2610161201: Structured logs

## Context

ENG-05 requires logging through `tracing` with structured fields, each
field passed through redaction. OPS-05 asks for JSON logs to a file or
stderr, a hook handler logging only to the home's log file, and no log
shipped anywhere. Phase 4 of plan
[2609292012](../../plan/2609292012_m1-record-and-recall/plan.md)
builds the logs, and the stakeholder's ruling 1 of 14 October 2026
counts `tracing` among the core's dependencies.

## Decision

| Module    | Purpose                                                                                          | License | Maintenance                                                  |
| --------- | ------------------------------------------------------------------------------------------------ | ------- | ------------------------------------------------------------ |
| `tracing` | The events, fields and macros the core logs through; the `Subscriber` trait ParleyInn implements | MIT     | Active; the tokio-rs organisation, the de-facto Rust log API |

The core takes it without default features, with `std`. The
`attributes` feature, a proc macro for `#[instrument]`, stays off, as
does `log`, a bridge to the `log` crate. The `structured_log` crate
implements `Subscriber` itself: it encodes each event's fields as
JSON through `serde_json`, redacts each text by the maintained
redaction rule set, and appends the line to `logs/parinn.jsonl`, owner
only. It installs as the scoped default for one run, so no global
state is set.

## Alternatives

- **`tracing-subscriber` with its JSON formatter.** A second direct
  dependency, and its formatter writes a field before any hook could
  redact it.
- **The `log` crate.** ENG-05 names `tracing`, and `log` carries no
  typed fields.
- **A logger of ParleyInn's own.** ENG-05 names `tracing`.

## Consequences

- The core's reach test allows `tracing`, `tracing-core`,
  `pin-project-lite` and `once_cell`: events and their fields in
  memory, and the cell holding the registry of call sites. None opens a
  socket or starts a program; the subscriber writes only the file the
  core opens.
- The direct set linked into the core is eight, against ENG-18's
  target of ten.
