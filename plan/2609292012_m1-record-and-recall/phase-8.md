---
n: 8
title: "parinn ingest: runs, ingest positions, generations and repeated lines"
status: "✅"
result: false
---
# Phase 8: `parinn ingest`

Requirements. REC-01 to REC-04, REC-07, REC-08 and REC-22 on what
`parinn ingest` reads. Also PRV-05's transcript rows and ENG-08's two
ingest properties. NFR-04 gets its benchmark (ENG-15). `@REC-01`,
`@REC-03` and `@REC-07` come off `@pending`.

Runs (REC-02):

- A transcript splits by the harness's agent id. Each run is an
  ingested run with its own ingest seat, kept per transcript source
  and agent id. A later ingest appends to the same writer.
- A subagent's run keeps its agent id and a parent link to its harness
  session's main run. Each run is its own agent.

Ingest positions (REC-03, REC-07, REC-08), state beside the record:

- Per source and generation: byte offset, SHA-256 of the consumed
  prefix, and the bytes last seen. Each event keeps its source,
  generation, line and offset. The position moves inside the append
  transaction.
- A shrunk source, or one whose prefix no longer hashes the same,
  starts a new generation, audited and counted. It is ingested past the
  longest prefix equal by content to the earlier one.
- An unterminated last line waits unless it parses.
- `parinn verify` names each source gone before it was fully ingested.

Repeated lines (REC-04), within one transcript directory:

- A line whose `uuid` names another source's event is recorded once.
  The owner is the run no fork observation names, else the first
  recorder, noted by a structural event and the audit log. The other
  run records a repeat reference when the content matches but for
  `sessionId` and `promptId`. Else it is its own event, counted as
  `repeat_conflicts`.
- A fork's line read before its owner's is held at an ingest marker,
  up to 30 s on an injected clock. It is released when the owner's
  transcript is gone. Holds are counted and shown by `parinn status`.
- Fork observations are phase 9's. Until then the CLI passes none.

Reading: the resolved path opens without following a link or blocking
on a FIFO. It must be the regular file resolution found.

PRV-05: each layer's trust policy key is audited as "rejected trust
override for <class>".

Properties (ENG-08), seeded: ingest twice equals ingest once. Every
order of ingesting a harness fork, with its observation, gives the same
writers and repeat references.

NFR-04: a benchmark of `parinn ingest --all` over a synthetic
transcript, a fresh home per sample.

Where the gate's scenarios go live:

- `@REC-02`: run steps bound; the rooms a later run reaches are M2's.
- `@REC-04`: fork observations need phase 9, live ingest phase 10,
  purge phase 14.
- `@REC-08`: `parinn event expand` is phase 13's.
- `@REC-22`: unbound, raised: its transcript from outside the roots
  meets SEC-18's refusal. Its marks and hook markers come later.
- `@PRV-05`: `post` and `summary` rows come later.

Gate: `@REC-01`, `@REC-03` and `@REC-07` pass. The bound steps of
`@REC-02`, `@REC-04`, `@REC-08` and PRV-05 pass. Both
properties hold; NFR-04 is benchmarked.
