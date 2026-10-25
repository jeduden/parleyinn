# Spike S2 benchmark report: the pure-Go SQLite driver

Spike S2 settles ADR-07: which pure-Go SQLite driver backs the store.
It also bears on OQ-03 (is pure Go fast enough at 10M events) and
answers OQ-04 (encryption at rest). The decision record is
ADR-2609302341.

## Verdict

`github.com/ncruces/go-sqlite3`. Both drivers are correct. The import
closure decides:

- `modernc.org/sqlite` links `os/exec` and `net` into the binary.
- `ncruces/go-sqlite3` links neither, and it ships an encrypting VFS.

On speed the two are close. On the large host ncruces took 11–21%
longer at full-ranked search and ingested 21–31% fewer events per
second in batches. There a rare anchor term kept modernc inside
NFR-03 (167–191 ms) and put ncruces outside (286–310 ms). On emulated
reference hardware both missed that query (340 and 274 ms), and the
full-ranking gap shrank to about 10%. Over 1M events ncruces still
ingested 21% fewer events per second. No target's outcome on the
reference hardware turns on the driver.

Both drivers meet NFR-04 on the reference hardware: 13.1k (modernc)
and 10.4k (ncruces) events per second over 1M events. Both meet
expand latency and hook memory. Both miss two targets: NFR-03 search
and NFR-09 store overhead. FTS5's cost model and the schema cause
those, so a CGO build would miss them too. OQ-03 stays open until
M1's bounded ranking meets NFR-03.

## Setup

- Candidates: `modernc.org/sqlite` v1.60.1 and
  `github.com/ncruces/go-sqlite3` v0.35.6, both on SQLite 3.53.4.
- Harness: the separate module in [bench-go](bench-go/main.go), so neither
  driver enters parinn's module graph. It builds one seeded synthetic
  store per driver. The store holds §8.2's `events` record and its
  external-content FTS5 projection, chained by SHA-256.
- Corpus: 10M events in 2,000 sessions, 4.61 GB of text. 60% are
  short messages of 40–200 bytes, 30% are 200–1,000 bytes, and 10%
  are 2 KiB payload previews. Words follow a Zipf distribution over a
  60,000-term vocabulary of pseudo-words, identifiers and
  numbers.
- Connection settings for both drivers: WAL, `synchronous=NORMAL`, a
  busy timeout, and `BEGIN IMMEDIATE` (ADR-01).
- Large host: 4 vCPU, 15 GiB RAM, Linux, Go 1.26.8,
  `CGO_ENABLED=0`. Both drivers write the same file format, so the
  latency runs read one warmed file with each driver in turn. That
  isolates the driver from page-cache effects.
- Emulated reference hardware (SRS §7: 2 vCPU, 4 GiB RAM): the same
  host, with a cgroup pinning two CPUs and capping memory, page cache
  included, at 4 GiB. The store's file cache was dropped before the
  search runs. Pages cached before the cgroup started could still
  serve reads: the cgroup peaked at 2.2 GiB during search, so those
  runs may understate cold-disk latency. The 1M-event ingest runs hit
  the 4 GiB cap. The disk is the host's.

## Correctness

| Check                                                        | modernc                  | ncruces                                    |
| ------------------------------------------------------------ | ------------------------ | ------------------------------------------ |
| FTS5 available                                               | compiled in              | separate module, registered per connection |
| BM25, snippet, highlight, phrase, prefix, NEAR, NOT          | pass                     | pass                                       |
| unicode61 with diacritic folding, trigram, porter tokenizers | pass                     | pass                                       |
| FTS5 `rebuild` from the record (I10)                         | pass                     | pass                                       |
| Deadline stops a CTE, a ranked FTS query and a table scan    | within 1 ms              | within 2 ms                                |
| Error returned on deadline                                   | wraps `DeadlineExceeded` | `sqlite3: interrupted`                     |
| Cancelled write transaction rolls back                       | yes                      | yes                                        |
| Connection usable after an interrupt                         | yes                      | yes                                        |
| 50 writer processes, 2,000 transactions, 4 searching readers | gap-free, chain intact   | gap-free, chain intact                     |
| `integrity_check` and FTS5 `integrity-check` afterwards      | ok                       | ok                                         |
| `SQLITE_BUSY` retries / failed writers                       | 0 / 0                    | 0 / 0                                      |
| Worst commit under contention                                | 1.45 s                   | 0.56 s                                     |
| `dbstat` virtual table                                       | yes                      | no                                         |

Every WAL writer opened its own connection per transaction, as a hook
process does. It read the tail and appended one to three events
inside `BEGIN IMMEDIATE`. The verification recomputed the whole hash
chain.

## Import closure and static build (SEC-01, I4, CON-02)

`go list -deps` on the smallest binary that opens each driver with
FTS5:

| Driver  | Network or process packages linked       | Linked by                                                                                             |
| ------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| modernc | `net`, `net/netip`, `net/url`, `os/exec` | `modernc.org/libc` (`system()`, `popen()` run `sh -c`); `google/uuid` (`net`); the driver (`net/url`) |
| ncruces | `net/netip`, `net/url`                   | the driver and its VFS parse `file:` URIs; `net/url` imports `net/netip`                              |

Neither of ncruces's two packages opens a socket. modernc links real
process-spawning code and the socket package for code paths SQLite
never calls. The SEC-01 import test cannot tell the two apart, so
admitting modernc would mean allow-listing `net` and `os/exec`.

Both build statically with `CGO_ENABLED=0` for linux/amd64,
linux/arm64 and darwin/arm64. The stripped minimal binary is 6.4 MB
with modernc and 8.8 MB with ncruces. Besides the driver itself,
modernc links seven more modules into the binary, and ncruces three.

## Performance at 10M events

The large-host search figures are the p95 of 300 to 600 queries after
30 warm-up queries, taken one rank above nearest-rank, so they are
slight upper bounds. The reference figures use nearest rank over 150
queries (full ranking) or 300 (bounded), 50 or 100 per term band.
The ingest runs prepare both INSERT statements again in every
transaction. The ingest and hook rows were rerun after the harness
switched appended events to the store's vocabulary.

About 14% of the rare band's anchors are identifiers such as
`ka_lo12`. FTS5's default tokenizer splits them on `_`, so they run
as phrases led by a common syllable, at about three times the median
latency of a single-token rare anchor. That inflates the rare band's
tail for both drivers alike.

| Target                               | Measure                            | Host modernc | Host ncruces | Ref. modernc | Ref. ncruces | Met on reference? |
| ------------------------------------ | ---------------------------------- | ------------ | ------------ | ------------ | ------------ | ----------------- |
| NFR-03 search ≤ 200 ms               | BM25 over all matches, query mix   | 3.57–3.81 s  | 4.22–4.32 s  | 2.95 s       | 3.24 s       | no, both          |
|                                      | the same, rare anchor term         | 167–191 ms   | 286–310 ms   | 340 ms       | 274 ms       | no, both          |
|                                      | BM25 over the newest 2,000 matches | 313 ms       | 390 ms       | 268 ms       | 299 ms       | no, both          |
| NFR-03 expand ≤ 100 ms               | 50-event window by `seq`           | 0.26 ms      | 0.25 ms      | 1.24 ms      | 0.26 ms      | yes               |
| NFR-04 ingest ≥ 5,000 ev/s, one core | 1M events, 100 per transaction     | —            | —            | 13,134       | 10,358       | yes               |
|                                      | 50k events, 100 per transaction    | 7,295–8,237  | 5,657–5,725  | 4,777–15,246 | 3,970–13,971 | merge timing      |
|                                      | 1 event per transaction            | 953          | 909          | 694–4,167    | 885–4,214    | not NFR-04's load |
| NFR-09 hook peak RSS ≤ 50 MiB        | open, append 100, search, close    | 21.7–21.8    | 20.1–20.5    | 21.6–21.8    | 20.2–20.8    | yes               |
| NFR-09 store overhead ≤ 1.5× text    | file size / text size              | 1.96         | 1.96         | 1.96         | 1.96         | no, both          |

The bulk load ran at 15.0k (modernc) and 13.5k (ncruces) events per
second, with 10,000 events per transaction and both loads running at
once.

### Ingest

Short ingest runs measure FTS5's automerge more than the driver. An
insert that tips a level rewrites a large index segment, so a
50k-event run either lands on a merge or misses it. The same run gave
4.8k and then 15.2k events per second back to back. NFR-04's scenario
ingests a 1M-event transcript, long enough to average the merges, and
both drivers clear 5,000 there with room to spare. One event per
commit ranges from 0.7k to 4.2k with either driver. That is not
NFR-04's workload, but it argues for committing a hook's events
together.

### Search

Full BM25 ranking scores every document that matches. A common term
matches millions of the 10M events, so a one-word query on it takes
2–8 s with either driver. Bounding by recency, ranking only the most
recent 2,000 matches, cuts the common-term p95 to about 0.3 s. It does
not cut multi-term queries below the target, because FTS5 still walks
the common terms' doclists to intersect them.

Merging the index barely helps. The bulk load left 22 FTS5 segments.
`optimize` merged them into one in 52 s, and the store grew by 0.6%.

| Query shape, merged index        | modernc p95 | ncruces p95 | Before merge   |
| -------------------------------- | ----------- | ----------- | -------------- |
| BM25 over all matches, query mix | 3.46 s      | 4.12 s      | 3.57 s, 4.22 s |
| BM25 over the newest 2,000, mix  | 299 ms      | 363 ms      | 313 ms, 390 ms |
| the same, median                 | 41 ms       | 67 ms       | 48 ms, 74 ms   |

The bounded shape's median sits well inside the budget. Its tail
comes from multi-term queries that pair a rare term with common ones.

This is FTS5's cost model, not the driver's. A CGO build typically
runs SQLite 1.5–2× faster than either translation. That would not
bring 3–4 s near 200 ms. So OQ-03 stays open, with no CGO build:
M1's query design decides NFR-03. Candidates are bounded ranking, a
merged index, and a stop-list for terms that match too much to rank.

### Store overhead

Both drivers write the same 9.01 GB for 4.61 GB of text. `dbstat`
splits it as follows:

| Part                             | Bytes   | × text |
| -------------------------------- | ------- | ------ |
| `events` (rows and inline text)  | 6.89 GB | 1.49   |
| FTS5 index and docsize           | 1.95 GB | 0.42   |
| UNIQUE(source, generation, line) | 0.17 GB | 0.04   |

The `events` table alone uses the whole 1.5× budget. Each row carries
about 228 bytes beyond its text: two 32-byte hashes, the metadata
columns, the record header, and slack around 2 KiB previews that
overflow 4 KiB pages. Most rows hold short text, so that fixed cost
weighs heavily. NFR-09's 1.5× is a
target for spike S8 to freeze. Whether payload text in the
content-addressed files counts as stored text decides whether this
store passes. If it does, the ratio falls well under 1.5×.

## Encryption at rest (OQ-04)

| Option                        | modernc                    | ncruces                             |
| ----------------------------- | -------------------------- | ----------------------------------- |
| Encrypting VFS in the module  | none                       | `vfs/xts` (AES-XTS), `vfs/adiantum` |
| Writable VFS API to build one | no (read-only `fs.FS` VFS) | yes                                 |
| SQLCipher or SEE              | CGO or licensed C only     | CGO or licensed C only              |

The probe loaded 200k events into an encrypted store with ncruces,
with FTS5 and WAL. A `grep` for a known session id found no plaintext
in the file, and opening it without the key fails with "file is not a
database". Ingest ran 35% slower with xts and 27% slower with
adiantum, and search latency did not change measurably. Both
ciphers are deterministic and unauthenticated. An attacker holding
two snapshots learns which pages changed, and a page can be rolled
back undetected. The hash chain (REC-10) catches tampering with the
record, though not with the projections.

That makes encryption at rest feasible as an opt-in VFS. The key
management on ephemeral runners, the threat model and the default
stay with the security review that OQ-04 names.

## Reproducing

```sh
cd plan/2609292004_spike-s2-sqlite-driver/bench-go
go build -o /tmp/bench .
/tmp/bench load -driver modernc -db /tmp/s2.db -n 10000000
/tmp/bench search -driver ncruces -db /tmp/s2.db -n 300
/tmp/bench search -driver ncruces -db /tmp/s2.db -n 600 -recent 2000
/tmp/bench expand -driver ncruces -db /tmp/s2.db -n 300
/tmp/bench ingest -driver ncruces -db /tmp/s2.db -n 1000000 -batch 100
/tmp/bench hook -driver ncruces -db /tmp/s2.db
/tmp/bench cancel -driver ncruces -db /tmp/s2.db
/tmp/bench optimize -driver ncruces -db /tmp/s2.db
/tmp/bench wal -driver ncruces -db /tmp/s2-wal.db -workers 50 -n 40
/tmp/bench fts5 -driver ncruces -db /tmp/s2-fts.db
/tmp/bench load -driver ncruces -vfs xts -db /tmp/s2-enc.db -n 200000
```

Running the two `search` lines again after `optimize` measures the
merged index. `ingest` and `hook` append about 1M events first, so
run `optimize` and the repeated searches on a copy of the store saved
before them, or the merged and unmerged runs differ in size too.

The closure check:

```sh
CGO_ENABLED=0 go list -deps ./closure/ncruces | grep -E '^(net|net/.*|os/exec)$'
```
