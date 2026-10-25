---
id: ADR-2610142220
title: "The random source: the OS's, through getrandom"
status: accepted
scope: dependencies
summary: >-
  Every random bit ParleyInn draws, for keys, commitment keys and room-id
  nonces, comes from the OS's random source through getrandom, behind
  a trait a test replaces. No user-space generator stands between.
---
# ADR-2610142220: The random source

## Context

Keys need 256 random bits each, every event's commitment key 256
(REC-17), and a create room act 128 for its room id (LANE-01). I10
keeps randomness out of anything derived, so randomness enters only
where an event or a key is minted, and the code that draws it must be
replaceable in tests. The core opens no socket and starts no program
(I4).

## Decision

| Module      | Purpose                                                                                                              | License           | Maintenance                                                 |
| ----------- | -------------------------------------------------------------------------------------------------------------------- | ----------------- | ----------------------------------------------------------- |
| `getrandom` | Fills a buffer from the OS's random source: `getrandom(2)` on Linux, `getentropy` on macOS, `ProcessPrng` on Windows | MIT OR Apache-2.0 | Active; rust-random organisation, used across the ecosystem |

The `key` crate wraps it in an `Entropy` trait; a test hands in a
counter. Every draw goes straight to the OS, with no user-space
generator that could be seeded wrong or forked into repeating.

## Alternatives

- **`rand` with `OsRng`.** The same source behind a larger API and
  three more crates.
- **Reading `/dev/urandom`.** Not on Windows, and a file that can be
  missing in a sandbox; `getrandom(2)` cannot be.
- **`ed25519-dalek`'s `rand_core` feature.** It would bring `rand_core`
  for what one call does.

## Consequences

- The core's reach test allows `getrandom`, `libc`, whose declarations
  it calls the system through, and `r-efi`, linked only on UEFI
  targets.
- A failure of the random source is an error the caller returns;
  nothing falls back to a weaker source.
