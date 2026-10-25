---
n: 6
title: "The store: writer logs, commitments, payloads, segments and format versions"
status: "✅"
result: false
---
# Phase 6: the store

Requirements. REC-09's payload store, REC-12's transcript order and
transcript timestamps, REC-17's commitments over payloads, ADM-05's
versions (ruling 3), REC-19's segment closing, ENG-08's gap-free `seq`
property and ENG-06's crash consistency over ingest. NFR-09's store
overhead and REC-17's secret freeze with evidence (§12.3). `@REC-09`
comes off `@pending`.

The payload store (REC-09, REC-17), in the `store` crate:

- Content whose canonical encoding is longer than
  `payload.threshold_bytes` goes to `payloads/<name>` beside
  `store.db`. The name is HMAC-SHA-256 under the event's commitment key
  over a fixed label: nothing confirms a guess at the content without
  that key, and two equal contents get two names.
- The event row keeps a preview of at most 512 bytes, the name, the
  size and the commitment, which covers the whole content; reading the
  event reads its content back from the payload.
- Each payload is written to a temporary file in `payloads/`, synced,
  then renamed, through a trait a test fakes.

Segments and format versions (REC-19, ADM-05, ruling 3):

- A segment is a range of one writer with its format version, kept in
  the store beside the log; no address, commitment or chain head
  changes with it (REC-25). Each append extends the writer's open
  segment or opens one at its next `seq`.
- A core append closes the segment it ends, in the same transaction.
  `parinn ingest` closes one at each turn end and subagent end its
  transcripts record and at the end of each transcript; a writer may
  also append and leave its segment open, for phase 10's MCP server.
- `parinn verify` checks the segments tile each writer's log, and
  `parinn verify` and `parinn status` report every writer chain ended
  without a closed segment.
- The store's version is checked before any write. A store or segment
  of a newer version is refused and named and left unchanged: `parinn
  status` and `parinn verify` exit 3, other commands refuse.

Transcript order (REC-12): events are appended in transcript order.
A line's transcript timestamp, when well formed, is kept as metadata.
Ingestion time never is.

Properties and crash consistency:

- ENG-08: random appends, rollbacks, payloads, closes and concurrent
  writers keep each writer's `seq` gap-free.
- ENG-06: a test kills `parinn ingest` at random points, then checks
  `parinn verify` exits 0. CI runs a few iterations; a nightly job runs
  10,000. A first start cut short verifies against its staged key.

Freezes (§12.3): NFR-09's store overhead, measured over every recorded
transcript, and REC-17's 256-bit secret, each with a test as evidence.
Removing their † and rows is an SRS change, for the stakeholder.

Where the gate's scenarios go live:

- `@REC-09`: here.
- `@REC-12`: its last step runs `parinn event expand`, the principal's
  recall (phase 13). Raised; its other steps bound and passing.
- `@REC-19`: its segment steps bound and passing narrowed; the rest
  needs the live MCP server (phase 10) and seals (M7).
- `@ADM-05`: `parinn migrate` and its backup are M3's; the refusal of a
  newer store by `parinn status` bound and passing narrowed.
- `@ENG-06` and `@ENG-08`: purge, migration and rebuild, and the other
  properties, come later; the ingest half and the first property here.

Tests. Unit tests beside each function, 99% per crate.
`@REC-09` and `e2e_crash_consistency.rs` drive the built core.

For the stakeholder: SEC-05's third row streams a long line to the
payload store without holding it whole. Yet SEC-08 redacts each JSON
string whole, and PRV-08 classes a line from its parsed structure.

Gate: `@REC-09` passes. `@REC-12` is bound but for its raised step.
REC-19's segment steps pass. A newer segment or store is refused. The
crash-consistency job runs nightly.
