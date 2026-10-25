---
id: ADR-2610110553
title: "The SQLite driver: rusqlite with SQLite bundled, extension loading compiled out"
status: accepted
scope: dependencies
summary: >-
  The store reaches SQLite and FTS5 through rusqlite with SQLite
  bundled and default features off, built without extension loading.
  Spike S2, rerun for the Rust core, measured it at 10M events.
  Accepted with the M1 change that links the store.
---
# ADR-2610110553: The SQLite driver

## Context

ADR-07 (§4) compiles SQLite with FTS5 into the core executable, and
[ADR-2610050528](ADR-2610050528-language-rust-typescript.md) proposed
`rusqlite` for it when the core moved to Rust. The driver chosen for
the Go core, `ncruces/go-sqlite3`, left with Go. No measurement
covered `rusqlite` until spike S2 was reopened for it.

The [spike report][report] ran the Go harness's measurements on the
same 10M-event corpus. The store must meet NFR-03, NFR-04, NFR-05 and
NFR-09, cancel a query on a deadline (ENG-03), keep the core inside
its reach (I4, SEC-01) and build as one executable per target, static
on Linux (CON-02).

## Decision

| Module     | Purpose                                                                                                                      | License | Maintenance                                            |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------- | ------- | ------------------------------------------------------ |
| `rusqlite` | The store's SQLite access: the record, the FTS5 search index, WAL with `BEGIN IMMEDIATE`, and query interrupts for deadlines | MIT     | Active; rusqlite organisation, releases track SQLite's |

The crate is taken without its default features, with `bundled`
alone. That compiles SQLite and FTS5 into the binary through
`libsqlite3-sys`. The default `cache` feature stays off. It brings
`foldhash`, whose Zlib license is off ENG-18's allow-list. The core
caches its own statements instead.

The build compiles out extension loading. It sets `LIBSQLITE3_FLAGS`
to `-USQLITE_ENABLE_LOAD_EXTENSION -DSQLITE_OMIT_LOAD_EXTENSION`. No
code that loads native code then links into the core.

Six crates link: `rusqlite` and `libsqlite3-sys`, plus `bitflags`,
`smallvec` and the two `fallible` iterator crates. Each joins the
reach test's allow-list in the change that links the store.

## Alternatives

- **The pure-Go drivers.** Withdrawn with the Go core.
- **`rusqlite` with its default features.** Three more crates for a
  statement cache, one of them under Zlib.
- **`sqlx` or `diesel`.** Async runtimes or an ORM layer the core does
  not need, and each still reaches SQLite through `libsqlite3-sys`.
- **SQLite as a system library.** CON-02 forbids a dynamic library on
  Linux, and a system SQLite's FTS5 and version vary by machine.
- **SQLCipher through `rusqlite`'s `bundled-sqlcipher-vendored-openssl`
  feature.** It works, but its vendored OpenSSL links `socket`, `bind`,
  `accept` and `fork` into the core, which SEC-01 refuses. OQ-04 stays
  open.

## Consequences

- Measured on emulated reference hardware, bounded search (the newest
  2,000 matches ranked) meets NFR-03 at 172 ms p95; full ranking takes
  1.45 s. M1's query compiler ranks the newest matches, and OQ-03 is
  answered for that shape.
- NFR-04 (26.1k events per second) and a hook handler's memory under NFR-09
  (12.5 MiB) hold with room to spare. The store overhead stays at
  1.96× text, the schema's cost, for spike S8 to settle.
- The release workflow builds the musl targets with `rust-lld` alone.
  SQLite is C, so the change that links the store also pins a C
  compiler and both targets' musl headers on the Linux builder, the
  way mingw-w64 is pinned. Clang 18 with musl 1.2.4's headers built
  both musl targets in the spike.
- SQLite's own security fixes arrive through `libsqlite3-sys`
  releases, reviewed as dependency updates.
- Encryption at rest needs a design of its own (OQ-04): SQLCipher on a
  provider without sockets, a VFS of ParleyInn's own with authenticated
  encryption, or encryption outside SQLite.

[report]: ../../plan/2609292004_spike-s2-sqlite-driver/report.md
