# Spike S2 report: rusqlite for the Rust core

Spike S2 settles ADR-07 for the Rust core: the store runs on SQLite
with FTS5 compiled in, through `rusqlite`. It bears on OQ-03 (is
SQLite fast enough at 10M events) and OQ-04 (encryption at rest). The
decision record is ADR-2610110553.

The spike first ran for the Go core and compared two pure-Go drivers;
that [report](report-go.md) stays as history. This run ports its
harness to Rust and generates the same corpus byte for byte, so every
figure here sits beside the Go figure for the same store.

## Decision

`rusqlite` 0.40.2 with SQLite bundled, default features off. It passes
every correctness check the Go drivers passed, and it is faster on
every measure:

- Bounded search, ranking only the newest 2,000 matches, meets NFR-03
  on emulated reference hardware: 172 ms p95, against 268 and 299 ms
  for the Go drivers.
- Ingest over 1M events runs at 26.1k events per second on reference
  hardware, twice the Go drivers' 13.1k and 10.4k.
- A hook handler process peaks at 12.5 MiB, against 20–22 MiB.

It links six crates, all MIT or Apache-2.0, and no socket or
program-start code. Two things need work before the core links it:

- The bundled build enables SQLite's extension loading, which links
  `dlopen`. `LIBSQLITE3_FLAGS=-DSQLITE_OMIT_LOAD_EXTENSION` compiles it
  out.
- The release workflow builds the musl targets with `rust-lld` alone.
  SQLite is C, so those builds need a pinned C compiler and both
  targets' musl headers on the Linux builder.

The store overhead still misses NFR-09's 1.5×: the schema causes it,
not the driver.

## Setup

- Candidate: `rusqlite` 0.40.2 (MIT) on `libsqlite3-sys` 0.38.2 (MIT),
  feature `bundled`, which compiles SQLite 3.53.2 with FTS5 and
  `dbstat` into the binary. The Go run used SQLite 3.53.4.
- Harness: the standalone crate in [bench](bench/src/main.rs), outside
  the parinn workspace. It ports [bench-go](bench-go/main.go) command
  for command: the same schema, connection settings (WAL,
  `synchronous=NORMAL`, a busy timeout, `BEGIN IMMEDIATE`), statements
  and query shapes.
- Corpus: the generator ports Go's `math/rand/v2` (PCG-DXSM, its
  bounded draw and its Zipf) draw for draw. `s2bench corpus -n
  10000000` and the Go generator print the same SHA-256 over every
  field of all 10M events, so the store holds the same 4,605,812,271
  bytes of text in 10M events and 2,000 harness sessions.
- Large host: 4 vCPU, 15 GiB RAM, Linux, Rust 1.99.0, release profile
  (one codegen unit, thin LTO).
- Emulated reference hardware (SRS §7: 2 vCPU, 4 GiB RAM): the same
  host, with a cgroup pinning two CPUs and capping memory, page cache
  included, at 4 GiB. The file cache was dropped before each search
  pass. The cgroup peaked at 0.78 GB during search.

## Correctness

| Check                                                         | rusqlite                               | Go drivers (history)   |
| ------------------------------------------------------------- | -------------------------------------- | ---------------------- |
| FTS5 available                                                | compiled in                            | compiled in, or module |
| BM25, snippet, highlight, phrase, prefix, NEAR, NOT           | pass                                   | pass                   |
| unicode61 with diacritic folding, trigram, porter tokenizers  | pass                                   | pass                   |
| FTS5 `rebuild` from the record (I10)                          | pass                                   | pass                   |
| Deadline stops a CTE, a ranked FTS query and a table scan     | within 1.2 ms                          | within 1–2 ms          |
| Error returned on deadline                                    | `OperationInterrupted` ("interrupted") | per driver             |
| Cancelled write transaction rolls back                        | yes; SQLite ends the transaction       | yes                    |
| Connection usable after an interrupt                          | yes                                    | yes                    |
| 50 writing processes, 2,000 transactions, 4 searching readers | gap-free, chain intact                 | gap-free, chain intact |
| `integrity_check` and FTS5 `integrity-check` afterwards       | ok                                     | ok                     |
| `SQLITE_BUSY` retries / failed writers                        | 0 / 0                                  | 0 / 0                  |
| Worst commit under contention                                 | 0.73 s                                 | 0.56–1.45 s            |
| `dbstat` virtual table                                        | yes                                    | one driver             |

The deadline is carried the way the core would carry ENG-03's: a timer
fires SQLite's interrupt through `rusqlite`'s `InterruptHandle`. Each
writing process opened its own connection per transaction, as a hook
handler process does, and appended one to three events inside `BEGIN
IMMEDIATE`. The verification recomputed the whole hash chain.

## Reach, licenses and builds (SEC-01, I4, ENG-18, CON-02)

The smallest binary that opens a store with FTS5 links six crates:
`rusqlite`, `libsqlite3-sys`, `bitflags`, `fallible-iterator`,
`fallible-streaming-iterator` and `smallvec`. All are MIT or offered
as MIT or Apache-2.0. `rusqlite`'s default `cache` feature adds
`hashlink`, `hashbrown` and `foldhash`, and `foldhash` is Zlib, off
ENG-18's allow-list. The core can cache its own statements instead.
The build adds `cc`, `pkg-config`, `vcpkg`, `find-msvc-tools` and
`shlex`, none of which reach the binary.

The stripped binary is 2.3 MB, against 8.8 MB for the Go driver
chosen before. It imports no `socket`, `connect`, `bind`, `accept`,
`fork`, `execve`, `posix_spawn`, `system` or `popen` from libc. It
does import `dlopen`, `dlsym` and `dlclose`: `libsqlite3-sys` compiles
SQLite with `SQLITE_ENABLE_LOAD_EXTENSION`. Loading stays off at run
time unless enabled, but the code to load foreign native code is
there. Building with `LIBSQLITE3_FLAGS="-USQLITE_ENABLE_LOAD_EXTENSION
-DSQLITE_OMIT_LOAD_EXTENSION"` removes all three imports.

| Release target               | With the release workflow's flags | What builds it                                                    |
| ---------------------------- | --------------------------------- | ----------------------------------------------------------------- |
| `x86_64-unknown-linux-musl`  | fails: no `x86_64-linux-musl-gcc` | `musl-gcc`, or clang 18 with the musl headers; static, runs       |
| `aarch64-unknown-linux-musl` | fails the same way                | clang 18 `--target=aarch64-linux-musl` with musl 1.2.4's headers  |
| `x86_64-pc-windows-gnu`      | builds with the pinned mingw-w64  | links only `KERNEL32`, `ntdll`, `msvcrt` and an `api-ms-win` stub |
| `aarch64-apple-darwin`       | not run here                      | the runner's Apple clang                                          |

The aarch64 binary was built, not run: the host has no emulator. Two
builds of the static amd64 binary from different directories, with
the workflow's path remapping, gave the same bytes. ENG-19 is
withdrawn, so the release needs no more than that: one x86_64 Linux
builder builds both musl targets and the Windows target, and it needs
a pinned C compiler and both targets' musl headers.

## Performance at 10M events

Figures are p95 by nearest rank after 30 warm-up queries: 300 queries
(full ranking) or 600 (bounded) on the host, 150 or 300 on reference
hardware, a third per term band. The reference runs ran after the
index was merged. On the host, merging changed full-ranking p95 by
under 1%.

| Target                               | Measure                            | Host rusqlite | Host Go      | Ref. rusqlite | Ref. Go       | Met on reference? |
| ------------------------------------ | ---------------------------------- | ------------- | ------------ | ------------- | ------------- | ----------------- |
| NFR-03 search ≤ 200 ms               | BM25 over all matches, query mix   | 1.32 s        | 3.57–4.32 s  | 1.45 s        | 2.95–3.24 s   | no                |
|                                      | the same, rare anchor term         | 92 ms         | 167–310 ms   | 201 ms        | 274–340 ms    | no, by 1 ms       |
|                                      | BM25 over the newest 2,000 matches | 130 ms        | 313–390 ms   | 172 ms        | 268–299 ms    | yes               |
| NFR-03 expand ≤ 100 ms               | 50-event window by `seq`           | 0.25 ms       | 0.25–0.26 ms | 0.87 ms       | 0.26–1.24 ms  | yes               |
| NFR-04 ingest ≥ 5,000 ev/s, one core | 1M events, 100 per transaction     | —             | —            | 26,143        | 10,358–13,134 | yes               |
|                                      | 50k events, 100 per transaction    | 28,698–32,440 | 5,657–8,237  | —             | 3,970–15,246  | —                 |
|                                      | 1 event per transaction            | 6,247         | 909–953      | 4,153         | 694–4,214     | not NFR-04's load |
| NFR-09 hook peak RSS ≤ 50 MiB        | open, append 100, search, close    | 12.4–12.5     | 20.1–21.8    | 12.4–12.5     | 20.2–21.8     | yes               |
| NFR-09 store overhead ≤ 1.5× text    | file size / text size              | 1.96          | 1.96         | 1.96          | 1.96          | no                |

The bulk load ran alone at 28.9k events per second, with 10,000
events per transaction; the Go loads ran at once, at 15.0k and 13.5k.
The ingest statements are prepared once per connection from
`rusqlite`'s cache, where the Go harness prepared them again in every
transaction.

### Search

Native SQLite runs full ranking about three times faster than either
Go translation, but a common term still matches millions of events:
full ranking of the query mix takes 1.3–1.5 s. Bounding by recency
brings the mix inside the budget on both machines. Its median is
27 ms on the host and 37 ms on reference hardware. Its tail still comes
from multi-term queries that pair a rare term with common ones: the
reference p99 is 548 ms.

| Query shape, host          | p95 before merge | p95 merged | median merged |
| -------------------------- | ---------------- | ---------- | ------------- |
| BM25 over all matches, mix | 1.32 s           | 1.32 s     | 79 ms         |
| BM25 over the newest 2,000 | 130 ms           | 108 ms     | 20 ms         |

The bulk load left 17 FTS5 segments. `optimize` merged them into one
in 43 s, and the store grew by 0.6%.

So OQ-03 has a positive answer at the p95 NFR-03 names, with the
bounded query shape, and none with full ranking. M1's query compiler
should rank the newest matches, as the Go report proposed, and treat
the p99 as the margin to watch.

### Store overhead

The store is 9.02 GB for 4.61 GB of text, the Go drivers' figure to
within 0.1%, since SQLite writes the same file format:

| Part                             | Bytes   | × text |
| -------------------------------- | ------- | ------ |
| `events` (rows and inline text)  | 6.89 GB | 1.49   |
| FTS5 index and docsize           | 1.95 GB | 0.42   |
| UNIQUE(source, generation, line) | 0.17 GB | 0.04   |

As before, the `events` table alone uses the 1.5× budget. Spike S8
freezes NFR-09's target, and whether payload text in the payload
store counts as stored text decides this store.

## Encryption at rest (OQ-04)

`rusqlite`'s `bundled-sqlcipher-vendored-openssl` feature builds
SQLCipher (SQLite 3.51.3, BSD-3-Clause) on OpenSSL 3.6.3
(Apache-2.0). The probe loaded 200k events into a keyed store with
FTS5 and WAL:

- A known harness session id appears 5,001 times in the plain file and never
  in the encrypted one. Opening it without the key fails with "file is
  not a database".
- The load ran 25% slower (30.4k against 40.7k events per second), and
  bounded search took about 1.5 times as long (3.6 ms against 2.4 ms
  mean on the small store).
- 50 writing processes left it gap-free with the chain intact, at a
  worst commit of 1.67 s.

But the vendored OpenSSL links `socket`, `bind`, `accept`, `recv`,
`setsockopt` and `fork` into the binary, which SEC-01 refuses in the
core, and the binary grows to 8.2 MB. SQLCipher as built here is
therefore out. What stays open for OQ-04:

- SQLCipher on a crypto provider without sockets: OpenSSL built
  `no-sock`, which the `openssl-src` crate does not offer, or another
  provider SQLCipher supports.
- A VFS of ParleyInn's own with authenticated encryption from the RustCrypto
  crates, which would also close the gap the Go report found in
  ncruces's unauthenticated ciphers.
- Application-level AEAD for payload files, or encrypted volumes only.

Key management on ephemeral runners, the threat model and the default
stay with the security review OQ-04 names.

## Reproducing

```sh
cd plan/2609292004_spike-s2-sqlite-driver/bench
cargo build --release
B=target/release/s2bench
$B corpus -n 10000000      # compare with the Go generator's hash
$B load -db /tmp/s2.db -n 10000000
$B search -db /tmp/s2.db -n 300
$B search -db /tmp/s2.db -n 600 -recent 2000
$B expand -db /tmp/s2.db -n 300
$B cancel -db /tmp/s2.db
$B optimize -db /tmp/s2.db
$B ingest -db /tmp/s2.db -n 1000000 -batch 100
$B hook -db /tmp/s2.db
$B wal -db /tmp/s2-wal.db -workers 50 -n 40
$B fts5 -db /tmp/s2-fts.db
cargo build --release --no-default-features --features cipher --target-dir target-cipher
target-cipher/release/s2bench load -db /tmp/s2-enc.db -n 200000 -key
```

The reference runs ran each command inside a cgroup v1 pair,
`cpuset.cpus=0-1` and `memory.limit_in_bytes=4G`, after `echo 3 >
/proc/sys/vm/drop_caches`.

The reach and build checks used a separate minimal crate: `rusqlite`
with `default-features = false, features = ["bundled"]`, a `main`
that creates an FTS5 table and runs one query, and `nm -D
--undefined-only` over the result. The musl builds set
`CC_<target>` to `musl-gcc`, or to clang with `--target`, `-nostdinc`
and the musl headers as `-isystem`.
