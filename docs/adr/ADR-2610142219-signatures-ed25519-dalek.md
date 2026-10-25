---
id: ADR-2610142219
title: "The signatures: Ed25519 through ed25519-dalek"
status: accepted
scope: dependencies
summary: >-
  Device and seat keys, and the recovery keys ParleyInn mints, are Ed25519
  keys; every signature ParleyInn makes is Ed25519 through ed25519-dalek,
  verified strictly. Each
  signature signs a label, its kind and the canonical body, so one kind
  never verifies as another (PRV-12).
---
# ADR-2610142219: The signatures

## Context

A node's device key certifies each seat key by a seat certificate the
seat key countersigns (PRV-11). An inception key event is signed by
its key, and a rotation by the recovery key the key event before it
committed to (PRV-12). Seat keys will sign room acts and seal writers
(REC-18). PRV-12 asks that every signature name its kind, so a
signature of one kind never verifies as another. The core opens no
socket and starts no program (I4). Phase 1 of plan
[2609292012](../../plan/2609292012_m1-record-and-recall/plan.md)
signs the first certificates and key events.

## Decision

| Module          | Purpose                                                                            | License      | Maintenance                                                |
| --------------- | ---------------------------------------------------------------------------------- | ------------ | ---------------------------------------------------------- |
| `ed25519-dalek` | Ed25519 keys and signatures: minting from a seed, signing, and strict verification | BSD-3-Clause | Active; dalek-cryptography organisation, audited, 3.x line |

Keys are minted from 32 bytes of the OS's random source as Ed25519
seeds. A signature signs `parinn signature v1`, the kind's name and
the body's canonical encoding (RFC 8785), and verifies only through
`verify_strict`, which refuses small-order keys and malleable
signatures. The default features stay on: precomputed tables, and
`zeroize`, which wipes a secret key when it is dropped.

## Alternatives

- **ECDSA over P-256 (`p256`).** Hardware authenticators and some key
  stores prefer it, but its signatures need a random nonce done right
  and are malleable; Ed25519's are deterministic.
- **`ring`.** C and assembly, and no seed import without PKCS#8.
- **Minisign or SSH signatures.** Formats for files, not for the
  structured bodies ParleyInn signs.

## Consequences

- The core's reach test allows `ed25519-dalek` and what it brings:
  `ed25519`, `signature`, `curve25519-dalek` with its build-time derive
  macro, `fiat-crypto`, `subtle` and `zeroize`, all computation.
- Phase 5 keeps keys in the platform key stores the plan's ruling 2
  names; their records add the key-store crates, not another
  signature scheme.
- An authenticator that offers only P-256 (OWN-11) signs presence
  proofs in its own scheme; that choice is its requirement's to
  record.
