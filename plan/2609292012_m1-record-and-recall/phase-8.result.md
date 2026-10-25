---
n: 8
title: "parinn ingest: runs, ingest positions, generations and repeated lines"
status: "✅"
result: true
summary: >-
  @REC-01, @REC-03 and @REC-07 off @pending and passing. Ingest resumes
  each transcript source past its ingest position, splits it by run,
  starts a new generation on a shrink or rewrite, and records a repeated
  line once. REC-02, REC-04, REC-08 and PRV-05's bound steps pass
  narrowed; both ENG-08 properties hold; NFR-04 benchmarked, not met.
---
# Phase 8 result

## Handoff

Outcome:

- REC-01, REC-03 and REC-07 hold: their scenarios are off `@pending`,
  passing against the built core, confined. The position moves in the
  transaction that appends its lines, so a kill neither drops nor
  repeats one.
- REC-02's M1 part: a transcript's own run comes from its path, a line
  naming the harness's agent id goes to that agent's run, each run an
  ingested run with its own ingest seat; a subagent's run reports its
  agent id and parent link. After a node identity change a run goes on
  in a new ingest seat.
- REC-04's M1 part: a line whose `uuid` names another source's event in
  the directory is a repeat reference, carrying that event's class, or a
  counted conflict. With no fork observation the first recorder owns it,
  noted by a repeat owner event in the device seat. A fork's line read
  first is held, up to 30 s, on the clock ingest is given.
- REC-08's M1 part: `parinn verify` and `parinn status` name each source
  gone before it was fully ingested; `parinn ingest --all` audits it once.
- PRV-05: each layer's trust policy key is audited as "<layer>: rejected
  trust override for <class>"; every ingested event stays untrusted.
- ENG-08: `ingest_property.rs` proves ingest twice equals once, and
  both orders of ingesting a fork alike; 150 seeded cases each passed.
- The parked reading race: ingest opens only the regular file the path
  resolved to, never blocking on a FIFO.
- NFR-09's overhead is now 1.41x, within its frozen 1.5x.
- I1, I6 and I10 hold: no line is lost or duplicated, each generation,
  conflict, owner choice, hold and lost source is counted and audited,
  and a transcript splits alike however it is read.

Inherited:

- `store::source`: `Source`, `Position`, `Place`, `Placed`,
  `IngestedRun`; `Store::append_lines`, `start_generation`, `advance`,
  `hold`, `lose`, `placed`, `line_owner`, `ingested_runs`.
- `ingest::Inputs` (payload settings, fork observations, clock),
  `RunReport`, `Ingested::runs`; modules `read`, `run`, `repeat`.
- Event kinds `repeat_reference`, `repeat_owner`; six new counters.
- `parinn::sources`; steps in `record_ingest.rs`, `provenance_trust.rs`.
- The benchmark `ingest_transcript` and `benchmark::fixture`.

For the stakeholder:

- NFR-04 is not met: 2,600 to 3,400 events/s in a release build on this
  loaded container. Redaction, four JSON parses a line, software
  SHA-256 and payload fsyncs dominate.
- REC-22 conflicts with phase 4's SEC-18 refusal of a `--path` outside
  the roots, so it is left unbound: rule which holds.
- A repeat compares content without `sessionId` and `promptId`, which a
  fork rewrites; REC-04 says "canonical content". Confirm.
- Stakeholder paths: `tooling/` (steps, REC-09's re-ingest step, the
  benchmark), and `features/record.feature`. The domain-model agent was
  not consulted: fork observation, repeat owner and held are new terms.

Parked:

- Fork observations, and the CLI's re-read of a held source: phase 9.
- Live ingest and a witnessed line never a repeat reference: phase 10.
- A purged line refused, keyed hashes for a new generation: phase 14.
- The `rooms` scope and restore for a later run: M2, M7.
- `parinn event expand` for `@REC-08`: phase 13.
