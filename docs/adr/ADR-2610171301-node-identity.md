---
id: ADR-2610171301
title: "The node identity is the machine id, kept in the home only as a digest"
status: accepted
summary: >-
  REC-24 names no source for the node identity. On Linux ParleyInn reads
  systemd's machine id, /etc/machine-id or D-Bus's copy, and the home
  keeps only its digest. An unavailable value has a digest of its own,
  so a node that never reads one changes once. On macOS it is the
  hardware UUID gethostuuid(2) returns, on Windows the MachineGuid
  registry value, both read in the platform crate, the one crate with
  unsafe code. parinn status says per OS whether the identity is
  checked.
---
# ADR-2610171301: The node identity

## Context

REC-24 checks a node identity before the first append of each start:
a value outside the home that a cloned runner image, a copied volume
or a restored snapshot cannot carry over. On a change ParleyInn mints a new
device key and new seat keys and records a node identity change event.
An unavailable value counts as a change, though ParleyInn may read it again
within the hook budgets. The SRS names no source for the value. Phase 5
of plan [2609292012](../../plan/2609292012_m1-record-and-recall/plan.md)
builds the check.

The core opens no socket and starts no program but its kernel worker
(I4, SEC-01), and only the `platform` shim may hold unsafe code
([ADR-2610171300](ADR-2610171300-key-stores.md)). So the source must be
a file the core can read, or a call through that shim.

## Decision

- **Linux.** The node identity is systemd's machine id: the first of
  `/etc/machine-id` and `/var/lib/dbus/machine-id` that holds a value.
  A provisioning tool regenerates it for each machine an image starts
  on; a copied volume or a restored snapshot does not carry it.
- **The home keeps a digest.** It keeps SHA-256 over a label and the
  value, never the value, in `node-identity`. A home that recorded none
  yet records the one it reads and counts as unchanged.
- **Unavailable.** A value still unavailable after a second read counts
  as a change, as REC-24 says. Its digest is a fixed one of its own, so
  a node that never reads a value changes once, not at every start.
- **macOS** reads the hardware UUID `gethostuuid(2)` returns, the value
  `ioreg` shows as `IOPlatformUUID`. It is a system call, not a program
  start (I4, SEC-01), made with a 100 ms wait (ENG-03); a timeout or a
  failure is an unavailable value.
- **Windows** reads the `MachineGuid` value under
  `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography`. The read is
  read-only, through the 64-bit registry view, and needs no administrator.
  A missing value is unavailable. So is a value of the wrong type, an odd
  or oversized length, text that is not UTF-16, or an empty string.
- **The `platform` crate.** The `dpapi` shim is renamed `platform` and
  holds a second module, `identity`, beside `dpapi`; each module allows
  unsafe code, justifies each unsafe block and sits behind a trait a
  test fakes, so Miri checks the pointer logic on any OS (ENG-09). The
  macOS call is declared in the crate with `extern "C"`, so no
  dependency is added; the Windows reads use `windows-sys`, with the
  `Win32_System_Registry` feature added.
- **Status.** `parinn status` says whether the node identity is checked
  on this OS and against what, and that finding a fork across nodes
  arrives with the peer network (PEER-10) while `parinn verify` scans
  the writer logs held here.
- **A hint, not the guard.** The identity only tells a home that it
  moved. Two nodes with one identity, or a machine cloned for backup,
  are fine. What must never happen is one seat key active in two
  homes, a second event at one (writer, seq); PEER-10 refuses that after
  the fact.

- **Tests.** A development build reads the value from the file
  `PARINN_TEST_NODE_IDENTITY` names, as it reads managed policy from
  `PARINN_TEST_MANAGED_POLICY`; a release build never does.

## Alternatives

- **The hostname.** Containers and cloned images share or reuse one,
  and a person can rename a machine without moving the home.
- **A random id kept in the home.** A copied home carries it, which is
  the case REC-24 exists to catch.
- **The boot id.** It changes at every boot, so every restart would
  mint new keys.
- **A process start on macOS (`ioreg`).** I4 refuses it: the core
  starts no program but its kernel worker.
- **A crate (`machine-uid` 0.6.0, MIT).** It holds unsafe code of its
  own and would be a new direct dependency in a core already at
  ENG-18's ten.
- **Extending the `dpapi` shim crate into a platform shim** (chosen).
  It keeps the unsafe code in one crate, `platform`, in two modules,
  but it widens the scope ADR-2610171300 gave that shim. The
  stakeholder consented to the widening and the rename on 21 October
  2026, and accepted this record the same day.

## Consequences

- A container started without a machine id changes once, then keeps
  its keys while the value stays unavailable. A copy of that home on
  another machine without one is not found changed; the deployment
  documentation should require a machine id per runner.
- REC-24's check holds on Linux, macOS and Windows. The macOS and
  Windows reads are built for their targets in CI but not run there,
  and Miri runs their pointer logic over fakes.
- A fork the check misses, or a cloned home it cannot tell apart, is
  caught by PEER-10: the first head seen wins and the later is kept for
  forensics.
