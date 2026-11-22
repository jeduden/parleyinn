---
title: "7. Non-functional requirements"
summary: >-
  Normative non-functional requirements NFR-01..15: latency,
  throughput, scale, availability, durability, concurrency, footprint,
  portability, compatibility, usability, maintainability,
  documentation.
---
# 7. Non-functional requirements

Reference hardware for performance targets: 2 vCPU, 4 GiB RAM, local SSD.

## NFR-01

- Category: Latency
- Ver: A

Hook handler wall-clock time at p95 on a store with 1M events:
`UserPromptSubmit` ≤ 50 ms; `SessionStart` ≤ 150 ms.
A `PermissionRequest` whose held request is kept waiting: ParleyInn's own
processing ≤ 50 ms† beyond the wait, and the wait itself ≤ the principal's hold
window, capped 10 s† below the harness timeout.
These latency targets MUST be met with the room view open and ten harnesses
writing.

## NFR-02

- Category: Latency
- Ver: T

Every hook handler MUST honour its hook budget, below the harness timeout
(ASM-10), and leave a start marker that it clears when it finishes; the next
hook handler, the MCP server as it starts, or `parinn status` MUST count and
audit each start marker left behind, once, whichever finds it first (I6).

## NFR-03

- Category: Latency
- Ver: A

MCP `event_search` ≤ 200 ms p95 and `event_expand` ≤ 100 ms p95 (excluding
transfer of large payloads) at 10M events.

## NFR-04

- Category: Throughput
- Ver: A

Ingestion ≥ 5,000 events/s on one core.

## NFR-05

- Category: Scale
- Ver: A

Per store: ≥ 10M events, ≥ 100 GiB payloads, single runs above 10M model tokens,
≥ 50 concurrent runs, subagents included, writing.

## NFR-06

- Category: Availability
- Ver: T

On any internal error, a hook handler MUST exit so that the harness proceeds
normally, with no injection, and MUST audit the error (I9).
Exceptions are limited to violations of I2, I4, or I8, which MUST fail closed.

## NFR-07

- Category: Durability
- Ver: T

The store MUST remain consistent if any ParleyInn process is killed at any
instruction (crash-consistency test, ENG-06), losing at most the in-flight
transaction, which is re-ingested on the next ingestion.

## NFR-08

- Category: Concurrency
- Ver: T

Concurrent appends MUST never corrupt the store or lose events, and no two
events MAY share an address (writer, `seq`): no `seq` is duplicated within a
writer's log.

## NFR-09

- Category: Footprint
- Ver: A

The core MUST leave no resident process while no run is active; peak RSS per
hook handler invocation ≤ 50 MiB; record overhead (the record file and payload
store, not the derived search index, which is measured apart) ≤ 1.5× stored
text.
The room-view component, the launcher, and the peer, publish and bridge
components run only while the person, a service manager or an ephemeral node's
entrypoint runs them and the act turning each on stands, each with peak RSS ≤
256 MiB† and steady-state CPU ≤ 5% of one core† while idle, measured on the
reference hardware; for the launcher these footprint limits MUST be met in total
for ten concurrent instances, and it MUST add at most 10 ms† p95 to
keystroke-to-echo latency.

## NFR-10

- Category: Portability
- Ver: T

linux/amd64, linux/arm64, darwin/arm64, windows/amd64; one core executable per
target linking no dynamic library beyond the operating system's, and none on
Linux (CON-02).

## NFR-11

- Category: Compatibility
- Ver: T

ParleyInn MUST support the latest Claude Code release and the previous two minor
versions, tolerate unknown fields on untrusted lines and unknown line types
(REC-05, REC-26), and detect format changes loudly (REC-26).

## NFR-12

- Category: Usability
- Ver: D

A new person MUST reach a working setup with defaults in ≤ 5 minutes via plugin
install, with zero required configuration.

## NFR-13

- Category: Maintainability
- Ver: I

Adding support for a new transcript format version MUST require changes only in
the harness adapter's transcript and hook part.

## NFR-14

- Category: Documentation
- Ver: I

Administration guide, threat model, settings reference, MCP tool reference, and
upgrade notes MUST ship with every release.

## NFR-15

- Category: Latency
- Ver: A

On the reference hardware, for a node holding 10M events across 50 rooms, at
p95: a line the harness appends to a transcript appears in the room view within
1 s† (REC-13); Catch up paints within 1 s†; search shows first hits within 300
ms†; opening a hit in context takes ≤ 150 ms†; replay steps between events
within 50 ms† and rebuilds a worktree within 500 ms†.
A surface that misses its target MUST say so on screen.
