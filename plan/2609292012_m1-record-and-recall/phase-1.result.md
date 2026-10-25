---
n: 1
title: "Proving slice: one transcript recorded, recalled and verified"
status: "✅"
result: true
summary: >-
  @REC-10 off @pending and passing. A recorded transcript is ingested,
  recalled through event_get in the envelope and verified, end to end
  and confined; a flipped header byte exits 3 naming the event.
---
# Phase 1 result

## Handoff

Outcome:

- REC-10 closes: `@REC-10` passes, two writers taking turns, one clock
  an hour behind, its order read from `parinn verify --json`.
- `crates/parinn/tests/e2e_slice.rs` drives the confined core: it
  ingests the 57-line `-w-my-app` transcript, recalls line 3 inside the
  envelope, warning first, and verifies. A flipped hex digit of event
  10's commitment then exits 3, naming its full address. Run locally;
  CI runs the pushed head.
- First paths through REC-06, REC-17, PRV-11, PRV-12 and RCL-04. I2, I4
  and I10 hold.
- ADR-2610142218 to 2610142221 (`sha2`, `ed25519-dalek`, `getrandom`,
  `serde_json` in the core) are accepted, as are ADR-2610110553 and
  ADR-2610102211: seven direct dependencies, five in the core.

Inherited:

- Crates per ENG-02: `record` (no I/O), `key`, `store`,
  `administration`, `ingest`, `envelope`, `mcp`, and `parinn`, whose
  `run` injects the home and stdin.
- `store.db` is format 1: `meta`, `seats`, `events`, `event_heads`. An
  append's first event cites the heads that advanced since its writer
  last cited them.
- A first start stages its keys and nonce in `keys/genesis`, records
  `minted`, the create room act and the device seat's certificate
  record in one transaction, then places the keys, device key last. A
  start that finds the staging finishes it. The personal room is the
  device seat beginning with that key's `minted`.
- Each ingest starts an ingest seat atomically and records each line as
  `unparsed_line`, provenance `unparsed`. A partial failure names what
  landed.
- `verify(store, node_key)` reads one snapshot and checks the genesis,
  the room id, certificate records against held seats, and trusted
  provenance in device seats only. It returns the causal order and
  names each failure's class.
- `event_get` checks commitment, hash and `prev_hash` first. A seat
  not both certified by the node's key and recorded in its genesis
  reads `peer`. Scope defaults to the personal room; refusals are
  fixed text; the result is the envelope's text alone, in §9.3's
  order.
- Removed content with no tombstone fails verify until phase 14.
- Steps start the core through `testkit::core_executable()` and
  `Confinement`; `an isolated ParleyInn home`, `the person runs "parinn …"`
  and `the command exits N` are bound.
- SQLite is built without extension loading. `build.yml` pins
  `clang-18` and musl 1.2.4's headers.

Deviations, now in phase-1.md: the device seat order (`minted` first);
`parinn ingest --path P`; `event_get` needs `scope: "room"` until phase
9; integration tests in their own crates; keys as `0600` files until
phase 5.

For the stakeholder: whether PRV-12's signature kinds cover room acts
and seals; PRV-11's I2 security review.

Parked:

- `cargo test -p scenario` alone can start a stale `parinn`.
- The testkit's fed start has no deadline (ENG-03).
- `store.db` holds content and commitment keys in clear, accepted
  until M5's SEC-09.
- A release build refuses `parinn ingest` until phase 4's redaction
  lands, which lifts it.
- Two first starts on one fresh home are not serialized: one can fail
  on a constraint or the staging rename, with the record intact.
  PRV-12's lock is phase 5's.
- `parinn verify` reads the device key's file for its public half; a
  public-key file arrives with phase 5's key stores.
- From the review panel, to their phases: every failure listed, unheld
  heads told from contradicted ones, re-ingest dedupe (REC-03), size
  limits and cursors (SEC-08), typed failure counters (I6), the home's
  ownership (SEC-02), certificates shipped to peers, redaction.
