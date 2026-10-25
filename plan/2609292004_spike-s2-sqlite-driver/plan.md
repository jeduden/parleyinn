---
id: 2609292004
title: "Spike S2: the SQLite driver for the Rust core"
status: "✅"
summary: >-
  Measure rusqlite with SQLite and FTS5 bundled for FTS5, cancellation,
  WAL concurrency, encryption, reach and 10M-event performance. Reopened
  after the move to Rust; the Go comparison stays as history.
model: opus
depends-on: []
---
# Spike S2: the SQLite driver for the Rust core

## Goal

Choose the SQLite driver that ADR-07 leaves open, on measured evidence
at the scale NFR-03 and NFR-05 demand.

## Context

Resolves ADR-07, OQ-03 and OQ-04. The choice must keep CON-02 (one
executable per target, static on Linux) and SEC-01 (no socket, no
program start in the core).

The spike first ran for the Go core and chose `ncruces/go-sqlite3`
over `modernc.org/sqlite`. The move to Rust
([ADR-2610050528](../../docs/adr/ADR-2610050528-language-rust-typescript.md))
withdrew that choice: ADR-07 now proposes `rusqlite` with SQLite and
FTS5 bundled, which no measurement covers. The plan is reopened for
it. The Go harness and report stay as history in
[bench-go](bench-go/main.go) and [report-go.md](report-go.md).

## Tasks

1. [x] Port the harness to Rust, generating the Go harness's corpus
   byte for byte, so the two reports measure the same store
2. [x] Build the 10M-event store with `rusqlite` and measure FTS5 BM25
   search p95, expand, ingestion throughput, hook handler memory and store
   overhead against NFR-03, NFR-04 and NFR-09, on the large host and
   on emulated reference hardware
3. [x] Check FTS5's features, query cancellation through SQLite's
   interrupt, and WAL correctness under 50 concurrent writing processes
4. [x] Probe SQLCipher, bundled with its crypto, for OQ-04
5. [x] Check the reach and licenses of what `rusqlite` links, and build
   it statically for every release target the way the release
   workflow does (CON-02)
6. [x] Write the report and record the driver in a dependency ADR,
   proposed until the M1 change that links the store accepts it
   (ENG-18, ENG-26)

## Outcome

The [benchmark report](report.md) holds the numbers.
[ADR-2610110553](../../docs/adr/ADR-2610110553-sqlite-rusqlite.md)
proposes `rusqlite` with SQLite bundled. Its default features stay
off, and extension loading is compiled out. It passed every check the
Go drivers passed, on the same corpus, and was faster on each measure.
On emulated reference hardware:

- Bounded search (the newest 2,000 matches ranked) meets NFR-03 at
  172 ms p95; full ranking takes 1.45 s. OQ-03 is answered for the
  bounded shape.
- Ingest over 1M events runs at 26.1k events per second (NFR-04), and
  a hook handler peaks at 12.5 MiB (NFR-09).
- The store overhead stays at 1.96× text, the schema's cost.

Deviations from the tasks:

- The reference runs ran on the merged index; on the host, merging
  changed full-ranking p95 by under 1%.
- The aarch64 musl binary was built but not run: the host has no
  emulator.
- The ADR is `proposed`, not `accepted`: ENG-18 fails on an accepted
  ADR whose crate no manifest requires. The M1 change that links the
  store flips it.

Handed on:

- M1's query compiler ranks the newest matches and watches the p99,
  548 ms on reference hardware.
- The change that links the store pins a C compiler and both musl
  targets' headers on the Linux builder. The release workflow builds
  musl with `rust-lld` alone today, and bundled SQLite does not build
  that way.
- OQ-04 stays open: SQLCipher with vendored OpenSSL links socket code
  into the core. The options go to the security review.
- Spike S8 freezes NFR-09's 1.5× store overhead.

No requirement closes and no scenario leaves `@pending`. The spike
gathers evidence for NFR-03, NFR-04, NFR-05 and NFR-09.

## Go outcome (history)

The [benchmark report](report-go.md) holds the numbers. Both drivers
passed the FTS5, cancellation and 50-writer WAL checks. On a large
host ncruces takes 11–21% longer at full-ranked search and ingests
21–31% fewer events per second in batches. On emulated reference
hardware the gaps are about 10% and 21%, and no target's outcome
turns on the driver. It wins:
modernc links `os/exec` and `net` into the binary, and only ncruces
offers an encrypting VFS.
ADR-2609302341
records the choice.

Deviations from the tasks:

- Task 5 used a minimal binary per driver rather than the parinn
  import test, which bans the whole `net/` prefix. ncruces needs
  `net/url` and `net/netip` allow-listed there when M1 first links
  the store.
- The ADR is `proposed`, not `accepted`. ENG-18 fails on an accepted
  ADR whose module `go.mod` does not require. The M1 change that adds
  the module flips the status.

Handed on:

- M1 must bound BM25 ranking for NFR-03. Full ranking of a common
  term takes seconds with either driver. Ranking only the newest
  2,000 matches still reaches 268–299 ms p95 on emulated reference
  hardware.
- M1's hooks should commit their events together. Both drivers meet
  NFR-04 over 1M events at 100 per commit (10–13k events per second),
  while one event per commit ranges from 0.7k to 4.2k.
- Spike S8 freezes NFR-09's 1.5× store overhead. This schema measures
  1.96× before payload files count as stored text.
- The OQ-04 default and key management go to the security review.

No requirement closes and no scenario leaves `@pending`. The spike
gathers evidence for NFR-03, NFR-04, NFR-05 and NFR-09.

## Acceptance Criteria

- [x] A benchmark report for the Rust driver is committed beside this
  plan, with the Go report kept as history
- [x] ADR-07's driver is measured and recorded; OQ-03 is answered for
  the bounded query shape, and OQ-04 is re-planned to the security
  review
- [x] The driver is justified by a dependency ADR, proposed until the
  M1 change that links the store accepts it, and the ENG-18 scenario
  still passes
