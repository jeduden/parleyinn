---
n: 5
title: "Keys, seats and the personal room"
status: "✅"
result: true
summary: >-
  @PRV-11 off @pending and passing. Keys live in this OS's key store per
  ruling 2, the dpapi shim under Miri; a changed node identity moves the
  node to a new device key and device seat in the same personal room,
  audited. REC-24, SEC-10 and PRV-12's genesis steps pass narrowed;
  @PRV-09 and @REC-24 cannot pass in M1 and are raised.
---
# Phase 5 result

## Handoff

Outcome:

- PRV-11 holds: `@PRV-11` is off `@pending`. A run seat is certified as
  a run seat and no ingest seat; `parinn verify --json` reports each
  seat's kind and ingest flag from its seat certificate alone. Its I2
  review is ADR-2610112004's.
- SEC-10's M1 part holds: each key in its own entry of the key store,
  named per home: the macOS Keychain, a DPAPI file on Windows, a `0600`
  file on Linux. `parinn status` names it and says an unsandboxed agent
  could read a key file. A missing key is told apart from an unreadable
  key store; neither starts a second genesis. `e2e_keys.rs` finds no
  seed in the store, a log, the audit log or any output.
- REC-24 and SEC-27's M1 part hold on Linux: before a start's first
  append the node identity is checked; a change mints a new device key,
  recovery key and device seat in the same personal room, recording a
  node identity change event, the new `minted` and its certificate
  record, audited and counted (`node_identity_changed`). Old writers
  get nothing more, and old principal acts no longer count.
- `parinn verify` follows the key event logs through node identity
  changes; a forged change is named.
- LANE-16's M1 roles and LANE-23's seat id are proven by unit tests.
- ADR-2610171300 (key stores, accepted on ruling 2); the `dpapi` shim
  is the one crate with unsafe code, with drift cases and CI's `miri`
  job. ADR-2610171301 (node identity) is proposed.

Inherited:

- `key::key_store`: `KeyStore`, `Entry`, `KeyStoreKind`, `platform`;
  `DeviceKey::keep`/`fetch` replace `save`/`load`.
- `Node::certify_run_seat` for phase 10's MCP server, which keeps the
  key in memory. `Node::identity_change`, `node_identity`, `role`.
- `parinn::open_node` opens the node for ingest and principal acts.
- Counters `node_identity_changed`, `device_key_missing`,
  `key_store_unavailable`. Seam `PARINN_TEST_NODE_IDENTITY`.

For the stakeholder:

- `@PRV-09` cannot pass in M1: it receives segments from a peer.
- `@REC-24` waits on phase 9's appending `SessionStart`, quarantine,
  origin and confirms. Its event also carries PRV-12's chain head,
  which REC-24's "only the fingerprints" omits: reconcile the rows.
- `@SEC-10` waits on `parinn backup create` (M3) to start the node.
- macOS and Windows read no node identity (ADR-2610171301).
- Accept ADR-2610171300's crates and ADR-2610171301; and is a node
  identity change rightly audited as `dropped`?
- A device seat key lost cannot start a new device seat: PRV-12 puts
  certificate records in the seat that lost its key.

Parked:

- Rotation, `parinn device-key rotate`, `minted anew`, `recovered`,
  confirms, chooses, rejects, duplicities, the cross-process lock.
- `parinn recovery-key write` needs a key derivation crate past ENG-18.
- SEC-27's seat-key rotation and revocation; LANE-16 and LANE-23's
  scenarios need rooms (M7).
