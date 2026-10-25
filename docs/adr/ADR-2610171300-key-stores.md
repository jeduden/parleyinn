---
id: ADR-2610171300
title: "Key stores: the macOS Keychain, DPAPI files on Windows, 0600 files on Linux"
status: accepted
scope: dependencies
summary: >-
  The core keeps each key in an entry named per home: on macOS in the
  Keychain through security-framework, on Windows in a file DPAPI
  protects through windows-sys in the platform crate (once the dpapi shim),
  the one crate with unsafe code, which Miri checks; on Linux in a 0600 file, and parinn
  status says so. The stakeholder's ruling 2 of 14 October 2026.
---
# ADR-2610171300: Key stores

## Context

SEC-10 keeps the device key, the recovery key and every seat key the
core holds in a platform key store the core reaches from inside the
core, with no socket and no program start, where one exists; else in a
file only the principal's OS user can read, and `parinn status` says
when a key is such a file. Each key sits in an entry named per home, so
two homes of one OS user never share a key. PRV-12 tells a missing key
apart from a key store that is locked or cannot be read: only the first
may be minted anew.

The stakeholder ruled on the key stores on 14 October 2026, ruling 2 of
plan [2609292012](../../plan/2609292012_m1-record-and-recall/plan.md),
and consented to it on [pull request #46][pr46]. Phase 5 records it
here.

[pr46]: https://github.com/jeduden/parleyinn/pull/46

## Decision

| Module               | Purpose                                                                                                  | License           | Maintenance                                                                 |
| -------------------- | -------------------------------------------------------------------------------------------------------- | ----------------- | --------------------------------------------------------------------------- |
| `security-framework` | Keeps each key as a generic password of the service `parinn` in the user's macOS Keychain                | MIT OR Apache-2.0 | Active; kornelski, the bindings most macOS Rust crates use for the Keychain |
| `windows-sys`        | Declares DPAPI's `CryptProtectData` and `CryptUnprotectData`, `LocalFree` and `GetLastError` for Windows | MIT OR Apache-2.0 | Active; Microsoft, generated from Windows' own metadata                     |

- **macOS.** Each key is a generic password of the service `parinn`,
  its account named `home-<digest>/<entry>`, the digest taken over the
  home's path, so two homes never share an entry. `security-framework`
  goes in without default features: the TLS ones are off. Its `unsafe`
  stays inside that crate.
- **Windows.** Each key is a file in the home's keys directory holding
  the key's bytes as DPAPI protected them for the running OS user, with
  no prompt. `windows-sys` is reached only from the `platform` crate, the
  one crate in the workspace that does not forbid unsafe code. It
  denies it instead, allows it in one module, and documents every
  unsafe block. That module calls DPAPI through a trait a test fakes,
  so Miri runs its pointer and buffer handling on Linux (ENG-09). The
  static-analysis gate lists the crate as its one exception. The crate
  was first called `dpapi`; it is `platform` now, and
  [ADR-2610171301](ADR-2610171301-node-identity.md) adds its second
  module.
- **Linux and other Unix.** Each key is a `0600` file in the home's
  keys directory. The Secret Service needs D-Bus, a socket the core may
  not open (I4), and the kernel keyring does not survive a reboot. A
  key file an unsandboxed agent of the same OS user could read is what
  `parinn status` names.

Both crates are target-specific dependencies: `security-framework`
links only on macOS, `windows-sys` only on Windows.

## Alternatives

- **The `keyring` crate.** One API over every platform, but on Linux it
  reaches the Secret Service over D-Bus, a socket, and its Windows
  backend keeps keys in the Credential Manager through more `unsafe`
  than DPAPI's two calls.
- **The Windows Credential Manager.** A platform store, but its entries
  roam with the profile and its API is larger than DPAPI's.
- **Files everywhere.** No dependency, but SEC-10 asks for the platform
  key store where one exists.

## Consequences

- The core's reach test allows `security-framework`,
  `security-framework-sys`, `core-foundation`, `core-foundation-sys`,
  `windows-sys`, `windows-link` and `platform`. None opens a socket or
  starts a program.
- The direct set linked into the core is ten, ENG-18's target.
- The Keychain and DPAPI stores are checked with `cargo check` for
  their targets on Linux; only a macOS or Windows runner exercises the
  OS's own store.
