---
n: 5
title: "Keys, seats and the personal room"
status: "✅"
result: false
---
# Phase 5: keys, seats and the personal room

Requirements. SEC-10's key stores, PRV-11's seat certificates,
REC-24's node identity check, SEC-27's new seat and writer, and the M1
parts of PRV-12, LANE-16 and LANE-23. `@PRV-11` comes off `@pending`.

Key stores (SEC-10, ruling 2), in the `key` crate, with an ADR:

- Each key in its own entry, named per home: the macOS Keychain
  through `security-framework`; on Windows a file DPAPI protects,
  through `windows-sys`; on Linux a `0600` file.
- A missing key and a locked or unreadable key store are told apart;
  only the first counts as missing (PRV-12). The second is refused,
  audited and counted.
- `parinn status` names the key store, and says when a key is a file an
  unsandboxed agent of the same OS user could read.

The Windows shim, a `dpapi` crate: the one crate holding `unsafe`,
behind a trait a test fakes, so Miri runs its pointer handling on
Linux (ENG-09). The static-analysis gate lists it as the one exception
to forbidding unsafe code, with a drift case (ENG-27).

Seats (PRV-11, LANE-16, LANE-23):

- `Node::certify_run_seat` certifies a run seat whose key its caller
  keeps in memory, for phase 10's MCP server.
- `parinn verify --json` reports each seat's writer, id, room, seat
  kind and ingest flag, read only from its seat certificate.
- M1's roles: the owner's device seat has every capability but writing
  a room summary; a run's personal-room seat is a contributor; the
  seat ingest starts takes its run's role. No model decides.
- A seat id derives from the room id and the seat key alone.

Node identity (REC-24, SEC-27), with a proposed ADR:

- Read from outside the home before each start's first append:
  `/etc/machine-id` on Linux. The home keeps only its digest. A value
  still unavailable on a second read counts as a change. macOS and
  Windows read none yet: for the stakeholder.
- On a change ParleyInn mints a new device key, recovery key and device
  seat key and keeps the personal room's id. The new device seat
  records a node identity change event, carrying only both keys'
  fingerprints and the chain head of the key event log it leaves, then
  the new key's `minted`, then its certificate record. The change is
  audited and counted; nothing is appended under an old key.
- `Node::find` and `parinn verify` follow the key event logs: a seat
  counts when a key with an inception in the record certified it, and
  that key's device seat holds its certificate record.

Where the gate's scenarios go live:

- `@PRV-11`: here.
- `@PRV-09`: its steps receive segments from a peer, which needs
  segments (phase 6) and an import or peer path (M3). Raised; unit
  tests prove a writer derives from its seat key alone.
- `@REC-24`: its steps need phase 9's appending `SessionStart`,
  phase 14's quarantine, phase 11's origin and confirms. Raised; its
  node identity steps pass narrowed.
- PRV-12, SEC-10, SEC-27, LANE-16 and LANE-23: the M1 steps reachable
  now are bound and pass narrowed.

Tests. Unit tests beside each function, 99% per crate.
`e2e_keys.rs` drives the key store, a run seat's certificate and a
node identity change through the built core, confined. The other key
stores pass `cargo check --target`.

Parked. Rotation and `parinn device-key rotate`. Also `minted anew`,
`recovered`, confirms, chooses, rejects and duplicities. `parinn
recovery-key write` needs a key derivation crate past ENG-18's budget.

Gate: `@PRV-11` passes; the M1 steps above pass narrowed. `@PRV-09`
and `@REC-24` cannot pass here and are raised.
