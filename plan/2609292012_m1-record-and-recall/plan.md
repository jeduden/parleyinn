---
id: 2609292012
title: "M1: record and recall"
status: "✅"
summary: >-
  The append-only record of writer logs under seat keys a node's
  device key certifies, provenance, trust and origin, pull-only recall
  over MCP, and the security and operations floor under them.
milestone-order: 1
model: opus
depends-on: [2609292003, 2609292004, 2609292005, 2609292009, 2609292010, 2610012322]
---
# M1: record and recall

## Goal

Record every agent run in writer logs under seat keys its node's
device key certifies. Claude recalls exact history through MCP recall
tools, inside the envelope. Nothing is lost, and every failure is
counted.

## Context

M1 is [§12.2](../../docs/srs/12-delivery-plan.md)'s record and recall
milestone. It replaces a pre-SRS 2.0 plan keyed by local sequence. SRS
2.0 to 3.0 put the record on the
[domain model](../../docs/domain-model/index.md) under [ADR-2610112209],
[ADR-2610112004] and [ADR-2610122234]:

- A node's device key starts its genesis log, a key event log (PRV-12),
  and certifies every seat key (PRV-11).
- Each seat has a writer. Events use (writer, `seq`) addresses on a
  commitment-bearing hash chain (REC-06, REC-10, REC-17).
- The personal room exists from first start. MCP records witnessed runs;
  an ingest seat records `parinn ingest`, origin `ingested` (REC-19, REC-22).
- Origin and trust derive from the record and key set (PRV-02, PRV-13).
  Its search index rebuilds identically in any arrival order
  (I10, ADR-2610112004).

[ADR-2610050528] and [ADR-2610042341] settle language and shell; OQ-32 is
how the components ship.

### Scope

§12.2 lists M1's scope, and this plan takes all of it. These rows land in full:

- REC P0, PRV P0 but PRV-09 and the in-part rows below, and RCL P0,
  with RCL-08
- MEM-01, SEC-02 to SEC-07, SEC-15, SEC-16, SEC-18 and SEC-22
- OPS-01 to OPS-03, ADM-04 and ADM-11
- OWN-01

These land in part, as §12.2 bounds them or as their scenarios reach past M1:

| Row                             | M1's part                                                               | The rest                                                                |
| ------------------------------- | ----------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| REC-19                          | ingest seats, holding a run's events, closing segments, verify's report | sealing, M7                                                             |
| REC-02                          | runs, parent links, resumes, forks, clears and agent ids                | restore blocks reach an agent's rooms, joins, M2 and M7                 |
| LANE-16                         | the owner's device seat, a run seat's contributor role                  | other roles, M7                                                         |
| LANE-23                         | how a seat id derives                                                   | joins, M2 and M7                                                        |
| LANE-31                         | the rule for two branch links naming one branch                         | room state, M7                                                          |
| SEC-01                          | the core's row: no socket, no program start, one component per process  | the B1 components, M7                                                   |
| SEC-27                          | the new seat and writer a key minted in place starts                    | rotation, M7                                                            |
| OWN-11, OWN-12                  | the act classes; widening acts only through the CLI at a terminal       | other surfaces, M7                                                      |
| OWN-22                          | the precondition: every run counts as unsandboxed                       | sandbox state, M7                                                       |
| PRV-12                          | the key event log, rotation and recovery                                | M3, M7, M8, PRV-10                                                      |
| PRV-13                          | origin from the record and the key set                                  | arrival, M3, M8, M9                                                     |
| REC-22, REC-24                  | REC-22: all but its freshness step; REC-24: chain head, origin `peer`   | freshness, VIEW-04, M7; the confirm a managed-policy key signs, M7      |
| ENG-09                          | the soak at 10,000 events in every PR, the Miri job                     | the 1,000,000-event run is nightly's evidence                           |
| ENG-08                          | every property but the restore delimiters'                              | restore delimiters, M2                                                  |
| ENG-10                          | the envelope's golden files                                             | the restore block's, M2                                                 |
| ENG-17                          | the contract replay, drift and residual-identifier checks               | the nightly run through restore, M2                                     |
| ADM-05                          | versions, and the refusal of a newer one                                | migration and its backup, M3                                            |
| ADM-07                          | `parinn purge --range`                                                  | other scopes, SEC-31, seals, M3                                         |
| SEC-10                          | keys kept per ruling 2; no key in the store, a log or output            | backup M3, at-rest key M5, launch M7                                    |
| LANE-01                         | routing to personal-room and device seats; branch links                 | joins, M2 and M7                                                        |
| ADM-13                          | the core writes to no git repository                                    | install M2; commit hook, launcher M7                                    |
| SEC-17, SEC-19                  | the threat model and the register's check, as tripwires                 | review and release gate, M5; B1 to B3 reach, M7                         |
| ENG-22                          | statements found by phrase, each citing a test that exists              | every statement, a passing test, M5                                     |
| SEC-12                          | quarantine by selector: seven of its eight rows                         | the flag row, PRV-07, M5                                                |
| SEC-08                          | redaction in `parinn ingest` and the hook handlers                      | `parinn purge --run`, M3                                                |
| ADM-09                          | verify's rows but the landmark row                                      | the landmark row, M3                                                    |
| ENG-06                          | kills of ingest, purge and rebuild                                      | kills of migration, M3                                                  |
| ENG-15                          | the suite and gate for NFR-02 to NFR-04 and NFR-05's floor              | NFR-01, M7; NFR-05's target, M5                                         |
| ENG-23, ENG-29                  | semantic versions, a changelog, the ADR gate                            | migration tests, M5; B1 to B3 past a prototype, M7                      |
| PRV-01, -02, -04, -05, -06, -08 | rows a transcript, hook input or the key set prove                      | a signed sandbox state, rooms, the launcher, M7; another node's key, M8 |

LANE-02 is partial. M1 records repository identity and local worktree binds.
M7 completes its branch-link, room-state and rebinding steps.

A scenario of a row that lands in part stays `@pending`, its M1 part
proven by bound steps or named tests, as ENG-12 in plan 2609292011. One
that cannot pass as written is raised, never re-pended.

The reach test and release build prove CON-02 to CON-05; M1's last phase
closes plan 2609292011's ENG-03 and ENG-15.

S2 ([ADR-2610110553]): `rusqlite`, BM25 over 2,000 matches in 172 ms p95.
S7: an in-house MCP server ([ADR-2610102211]). S1, S3 and S13 use Claude
Code 2.1.296 fixtures; S8 is the estimator; S12 covers purge storage.

### The stakeholder's rulings of 14 October 2026

The stakeholder ruled on these when M1 was re-planned and consents on
[pull request #46][pr46]; rulings 3 and 4 changed M1's row in §12.2.

1. **ENG-18:** only dependencies linked into a shipped executable count.
2. **SEC-10:** Keychain on macOS, a Miri-checked DPAPI file on Windows,
   `0600` files on Linux; in part.
3. **ADM-05:** M1 ships format version 1, refusing a newer; M3 migrates.
4. **ADM-07:** M1 ships `parinn purge --range` as a widening act; the
   other scopes, SEC-31's copies and seal rewrites are M3's.

### Where M1 stands

Rulings of 2 November 2026, now in the SRS and scenarios; passing means
cucumber:

- **ADM-09:** 16 of 17 rows pass, the landmark row is M3's.
- **Off `@pending` and passing:** ENG-09 (10,000 events per PR, 1M
  nightly), SEC-18, REC-04, REC-08, REC-12, REC-26, RCL-02 (10 of 10), REC-17
  extended.
- **SEC-05 and windowed ingest are M5's (user's final ruling):** ingest and
  live capture read a transcript whole; only SEC-05's hook rows pass.
  Blockers: I1 and an oversize line, quadratic cost, poll latency.
- **Purge residue (ruled):** only each purged line's position, length and
  the harness's own `uuid` remain (beside the tombstone, REC-04); SEC-31
  and ADM-07 stay `@pending` (M3). NFR-03..05 land in part (benchmarks,
  ENG-15 gate); NFR-04 was benchmarked but its target, NFR-05's target and
  MEM-02 wait for M5.
  `parinn ingest --path` keeps its OWN-22 gate; a path inside
  `PARINN_HOME` stays refused (safe default, not changed).
- **REC-22 in part:** all steps pass but the first Then, a freshness mark
  with source and ingest position on every surface: none shows one before
  VIEW-04 (M7). **REC-24 in part:** its confirm step is M7's.
- **LANE-02** stays `@pending` for its M7 remainder. Verify's 60 s bound:
  postponed. PRV-09,
  PRV-02's peer rows: §11.2 option C.
- Stakeholder consent still needed on `docs/srs/**`,
  `docs/domain-model/record.md` and `features/**`, as the PR lists.

## Tasks

1. Proving slice: one recorded transcript ingested into an ingest
   seat's writer and recalled through `event_get` in the envelope, the
   chain checked by `parinn verify`
2. The home, the audit log and counters: SEC-02, SEC-03, OPS-02,
   OPS-03; typed errors per counted failure class (ENG-04)
3. Principal acts at the terminal, configuration and managed policy:
   OWN-11, OWN-12 and OWN-22 in part, ADM-04, PRV-04, PRV-05, SEC-22;
   the configuration parser's fuzz target
4. Redaction before anything is written, input limits and paths:
   SEC-05, SEC-08, SEC-18; logs through redaction (ENG-05); the
   redactor's fuzz target. It lifts phase 1's refusal of `parinn ingest`
   in a release build
5. Keys, seats and the personal room: PRV-09, PRV-11, PRV-12 in part,
   SEC-10, SEC-27, LANE-16 and LANE-23 in part, REC-24; the
   key-store ADR and the three key stores, with Miri over the Windows
   shim (ENG-09)
6. The store: writer logs, commitments, payloads, segments and format
   versions: REC-06, REC-09, REC-12, REC-17's commitments, ADM-05 in part,
   REC-19's segment closing; the gap-free `seq` property (ENG-08);
   crash consistency over ingest (ENG-06); freeze
   NFR-09's overhead and REC-17's secret
7. The transcript parser and provenance: PRV-01, PRV-03, PRV-08, REC-05,
   REC-26; the parser's fuzz target and its nightly run; the transcript
   contract replay and the residual-identifier check (ENG-17)
8. `parinn ingest`: REC-01 to REC-04, REC-07, REC-08, REC-22; the
   properties of a harness fork's ingest order and of ingesting twice
   (ENG-08); the NFR-04 benchmark
9. The hook handlers: SEC-16, the session binding, prompt commitments
   and the attended mark (PRV-04, PRV-08), holding a run's hook
   observations at its ingest marker until its MCP server holds its
   run-seat key (REC-19); the hook input decoder's fuzz target and the
   hook input contract replay (ENG-17); the NFR-01 and NFR-02 benchmarks
10. The MCP server and live ingest: REC-13, PRV-14, run-seat keys in
    memory (SEC-10), concurrent appends (REC-06), with the gap-free
    `seq` property rerun under concurrency
11. Trust, origin, branch links and repository identity: PRV-02,
    PRV-06, PRV-13, LANE-01 and LANE-31 in part, LANE-02, OWN-01,
    MEM-01, ADM-13 in part; the field sanitizer's fuzz target
12. Search: REC-11, SEC-04, RCL-02 with bounded BM25; the query
    parser's fuzz target; the rebuild-digest property (ENG-08); the
    NFR-03 benchmark; freeze REC-11's cap, RCL-02's hits and SEC-04's
    terms
13. Recall and the envelope: RCL-01, RCL-03 to RCL-05, RCL-07, RCL-08,
    SEC-06, SEC-07; the estimator ported from S8; the envelope's golden
    files (ENG-10), its round-trip property and its encoder's fuzz
    target; freeze RCL-03's cap
14. Quarantine and `parinn purge --range`: SEC-12 in part, RCL-06, REC-17,
    ADM-07 in part; crash consistency over purge (ENG-06)
15. Operations: ADM-09 in part, ADM-11, OPS-01, SEC-01's core row, SEC-15,
    SEC-17 and SEC-19 in part; every privacy statement tied to its test
    (ENG-22 in part)
16. Exit: the NFR-05 floor benchmark; ENG-11's coverage floors; the
    `parinn-eval` module of §11.1 and its recall evaluation; the
    two-writer rebuild test (S12)

## Execution

Phase 1 fixed the test approach; ENG-18's rewording landed before 4, 5
and 7. Tracks 2-3; 4; 5-6; 7 ran in parallel; 8 to 16 run in order.

| Phase | Model  | Gate                                                                                                                                   |
| ----- | ------ | -------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus   | The slice's end-to-end test in an isolated home; `@REC-10` passes                                                                      |
| 2     | sonnet | `@SEC-02`, `@SEC-03`, `@OPS-02`, `@OPS-03` and `@ENG-04` pass                                                                          |
| 3     | opus   | Its end-to-end tests pass; so do ADM-04's rows 1 to 4 and the M1 steps of OWN-11, -12 and -22                                          |
| 4     | sonnet | SEC-05 rows 1-2, `@SEC-08`, `@SEC-18` and `@ENG-05` pass; no fixture secret reaches the store or a log                                 |
| 5     | opus   | `@PRV-11` passes; REC-24's M1 part; PRV-12, SEC-10, SEC-27, LANE-16 and LANE-23's M1 steps pass                                        |
| 6     | opus   | `@REC-09` and `@REC-12` pass; REC-19's segment steps pass; a newer segment or store is refused; the crash-consistency job runs nightly |
| 7     | opus   | `@PRV-03`, `@REC-05` and `@REC-26` pass; PRV-01 and PRV-08's M1 rows pass; the parser's fuzz target runs                               |
| 8     | opus   | `@REC-01` to `@REC-04`, `@REC-07`, `@REC-08`, `@REC-22` and PRV-05's transcript rows pass; both properties hold; NFR-04 benchmarked    |
| 9     | opus   | `@SEC-16`, `@NFR-02` and `@PRV-04`'s rows 1-3 pass; REC-19's holding steps bound; NFR-01 and NFR-02 benchmarked                        |
| 10    | opus   | `@REC-06`, `@REC-13` and `@PRV-14` pass against a live MCP server                                                                      |
| 11    | opus   | `@OWN-01` and `@MEM-01` pass; LANE-02 stays `@pending`; PRV-02, PRV-06, PRV-13, LANE-01, LANE-31 and ADM-13's M1 steps pass            |
| 12    | sonnet | `@REC-11`, `@SEC-04` and `@RCL-02` pass; NFR-03 benchmarked                                                                            |
| 13    | opus   | `@RCL-01`, `@RCL-03` to `@RCL-05`, `@RCL-07`, `@RCL-08`, `@SEC-06` and `@SEC-07` pass; the envelope's golden files match               |
| 14    | opus   | `@RCL-06`, `@REC-17` and `@SEC-22` pass; SEC-12's seven rows pass; a purged event leaves nothing confirming a guess                    |
| 15    | sonnet | `@ADM-11`, `@OPS-01` and `@SEC-15` pass; ADM-09, SEC-17, SEC-19 and ENG-22's M1 parts pass; SEC-01's core row passes                   |
| 16    | opus   | `@ENG-03`, `@ENG-07` and `@ENG-11` pass; ENG-06 and ENG-15's M1 parts hold; 95% recall in `parinn-eval`; the S12 rebuild test passes   |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                                                                                                                                                                                                                                                                                                                                                                                  |
| --- | ------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | ✅     | [Proving slice: one transcript recorded, recalled and verified](phase-1.md)                                                                                                                                                                                                                                                                                                                                            |
|     | ↳      | @REC-10 off @pending and passing. A recorded transcript is ingested, recalled through event_get in the envelope and verified, end to end and confined; a flipped header byte exits 3 naming the event.                                                                                                                                                                                                                 |
| 2   | ✅     | [The home, the audit log and counters](phase-2.md)                                                                                                                                                                                                                                                                                                                                                                     |
|     | ↳      | @SEC-02, @SEC-03, @OPS-02, @OPS-03 and @ENG-04 off @pending and passing, confined. Every command checks the home first; the audit log chains across rotation; parinn hook fails open and audits.                                                                                                                                                                                                                       |
| 3   | ✅     | [Settings layers and principal acts at the terminal](phase-3.md)                                                                                                                                                                                                                                                                                                                                                       |
|     | ↳      | Settings layers validated and layered; widening acts gated by OWN-22 and managed policy; counter ack, configuration accept and risk acceptance recorded as principal acts. @ADM-04 off @pending with eight rows, its flags row moved to @PRV-07 by ruling; the M1 steps of OWN-11, -12 and -22 pass narrowed.                                                                                                          |
| 4   | ✅     | [Redaction before anything is written, input limits and paths](phase-4.md)                                                                                                                                                                                                                                                                                                                                             |
|     | ↳      | @ENG-05 off @pending and passing. parinn ingest redacts every line before it writes, refuses while a settings layer fails, reads only within the allowed roots, and a release build ingests again; hook input past 1 MiB or depth 64 is refused. SEC-05 rows 1-2 and SEC-08's ingest steps pass narrowed.                                                                                                              |
| 5   | ✅     | [Keys, seats and the personal room](phase-5.md)                                                                                                                                                                                                                                                                                                                                                                        |
|     | ↳      | @PRV-11 off @pending and passing. Keys live in this OS's key store per ruling 2, the dpapi shim under Miri; a changed node identity moves the node to a new device key and device seat in the same personal room, audited. REC-24, SEC-10 and PRV-12's genesis steps pass narrowed; @PRV-09 and @REC-24 cannot pass in M1 and are raised.                                                                              |
| 6   | ✅     | [The store: writer logs, commitments, payloads, segments and format versions](phase-6.md)                                                                                                                                                                                                                                                                                                                              |
|     | ↳      | @REC-09 off @pending and passing. Content above the payload threshold goes to the payload store under a keyed name; segments close at turn ends; a newer store or segment is refused, parinn status exits 3. REC-12, REC-19's segment and ADM-05's status steps pass narrowed; 10,000 kills of ingest run nightly; NFR-09 and REC-17's secret frozen.                                                                  |
| 7   | ✅     | [The transcript parser and provenance](phase-7.md)                                                                                                                                                                                                                                                                                                                                                                     |
|     | ↳      | @REC-05 and @PRV-03 off @pending and passing. Every transcript line is checked against its version's recorded shape and classed from its structure; unparsed and drifting lines are kept, counted and audited. PRV-08, REC-26, PRV-01 and ENG-17's M1 steps pass narrowed; three fuzz targets run nightly.                                                                                                             |
| 8   | ✅     | [parinn ingest: runs, ingest positions, generations and repeated lines](phase-8.md)                                                                                                                                                                                                                                                                                                                                    |
|     | ↳      | @REC-01, @REC-03 and @REC-07 off @pending and passing. Ingest resumes each transcript source past its ingest position, splits it by run, starts a new generation on a shrink or rewrite, and records a repeated line once. REC-02, REC-04, REC-08 and PRV-05's bound steps pass narrowed; both ENG-08 properties hold; NFR-04 benchmarked, not met.                                                                    |
| 9   | ✅     | [The hook handlers: hook observations held at the ingest marker](phase-9.md)                                                                                                                                                                                                                                                                                                                                           |
|     | ↳      | @SEC-16 and @NFR-02 off @pending and passing. Hook handlers decode each hook input against its schema and recorded shapes, and hold their observations at the ingest marker within their hook budgets; ingest uses them for user lines and prompt matches. PRV-04 rows 1-3, PRV-08, REC-19 and REC-26's hook steps pass narrowed; NFR-01 and NFR-02 benchmarked.                                                       |
| 10  | ✅     | [The MCP server and live ingest](phase-10.md)                                                                                                                                                                                                                                                                                                                                                                          |
|     | ↳      | @REC-13 off @pending and passing against a live MCP server. It takes its own harness process's session binding, ingests its transcripts into run seats whose keys it holds in memory, and leaves an ingest marker when it stops short. @REC-06 and @PRV-14 pass narrowed: purge (phase 14) and event_search (phase 12) keep them pending.                                                                              |
| 11  | ✅     | [Trust, origin, branch links and repository identity](phase-11.md)                                                                                                                                                                                                                                                                                                                                                     |
|     | ↳      | @MEM-01 and @OWN-01 off @pending and passing. Each event's trust level and origin derive from the record and the node's key set alone, nothing stores them, and parinn rebuild and parinn verify keep the kept origins honest. A repository's identity is read inside the core; @LANE-02 stays @pending for a ruling. @PRV-02 passes narrowed; @PRV-06 and @PRV-13 are proven by named tests.                          |
| 12  | ✅     | [Search: the search index, the query compiler and event_search](phase-12.md)                                                                                                                                                                                                                                                                                                                                           |
|     | ↳      | @REC-11 and @SEC-04 off @pending and passing. Event and payload text is indexed in the append transaction up to 1 MiB, and event_search finds it by literal terms, ranked by BM25 over the newest 2,000 matches, damped per run. @RCL-02 cannot pass in M1 and is raised; NFR-03 is benchmarked; the index is a derived file of its own, so the record holds 1.42x its text, within NFR-09's 1.5x; the index is 0.75x. |
| 13  | ✅     | [Recall and the envelope: the six recall tools, scopes and recall events](phase-13.md)                                                                                                                                                                                                                                                                                                                                 |
|     | ↳      | @RCL-01, @RCL-03, @RCL-04, @RCL-05, @RCL-07, @SEC-06 and @SEC-07 off @pending and passing. The six recall tools answer in the envelope under the 8,000-token cap and a cursor, in the calling run's scope unless the call widens it, and each leaves a recall event; the envelope's golden files match. @RCL-08 stays @pending on purge and quarantine (phase 14).                                                     |
| 14  | ✅     | [Quarantine and parinn purge --range](phase-14.md)                                                                                                                                                                                                                                                                                                                                                                     |
|     | ↳      | @RCL-06, @REC-17 and @SEC-22 off @pending and passing; so are @RCL-08 and @REC-06, whose purged-range steps now run. A purge erases a range and records a tombstone in one transaction, a quarantine holds events out of recall by selector, and an ingest purges what a retention policy has aged out. @SEC-12 passes seven rows of eight and stays @pending: its flag row needs PRV-07 (M5).                         |
| 15  | ✅     | [Operations: verify, status, audit, the boundary and the threat model](phase-15.md)                                                                                                                                                                                                                                                                                                                                    |
|     | ↳      | @ADM-11, @OPS-01 and @SEC-15 off @pending and passing; SEC-01's core row passes as gate tests. @ADM-09 passes seven rows of eight and stays @pending (landmarks, M3). @SEC-17, @SEC-19 and @ENG-22 are bound in part and stay @pending, their findings raised below.                                                                                                                                                   |
| 16  | ✅     | [Exit: the engineering gates, the recall evaluation and the S12 rebuild test](phase-16.md)                                                                                                                                                                                                                                                                                                                             |
|     | ↳      | @ENG-03, @ENG-07, @ENG-09 and @ENG-11 off @pending and passing. parinn-eval recalls 216 of 216 planted values after one to three compactions within ten hits; the S12 rebuild test passes. @ENG-06, @ENG-15, @ENG-23 and @ENG-29 stay @pending, each raised.                                                                                                                                                           |
<?/catalog?>

## Acceptance Criteria

- [x] Every scenario of a row M1 lands in full passes, off `@pending`
- [x] Every M1 part is proven. Remainders: REC-02 → M2; ADM-07, ADM-09
  and SEC-31 → M3; SEC-12, NFR-04/05, SEC-05's transcript rows and
  PRV-14 → M5; REC-22/24 and LANE-02 → M7; PRV-09's peer rows → M8
- [x] CI `eval` recalls 216/216 at one to three compactions (§11.1)
- [x] `parinn verify` proves completeness (16/17 ADM-09 rows), and
  quarantine works (RCL-06; seven SEC-12 rows); remainders are assigned
- [x] S12's `s12_rebuild` makes one identical index from two writer logs
- [x] REC-17 leaves no value that confirms a purged event: only position,
  length and the harness's own `uuid` remain (stakeholder ruling)
- [x] M1's dependencies have accepted ADRs and meet ENG-18/ENG-26
- [x] ENG-11's coverage floors hold for every security-sensitive crate
- [x] The fuzz targets of ENG-07 run nightly from committed corpora
- [x] The six §12.3 targets of M1 are frozen, each with evidence
- [x] ENG-03's mechanical clauses pass. [PR #91][consent] proves injectable
  I/O; cucumber does not claim it. ENG-15 closes
  with NFR-01 in M7 and NFR-05 in M5
- [x] All tests pass: `cargo test --workspace`
- [x] `cargo clippy ... -D warnings` and `cargo fmt --all --check` are clean
- [x] `mdsmith check .` is clean

[ADR-2610112209]: ../../docs/adr/ADR-2610112209-invariants-after-s1-s3.md
[ADR-2610122234]: ../../docs/adr/ADR-2610122234-key-event-log.md
[ADR-2610112004]: ../../docs/adr/ADR-2610112004-derived-artifacts-by-content.md
[ADR-2610050528]: ../../docs/adr/ADR-2610050528-language-rust-typescript.md
[ADR-2610042341]: ../../docs/adr/ADR-2610042341-app-shell-tauri.md
[ADR-2610110553]: ../../docs/adr/ADR-2610110553-sqlite-rusqlite.md
[ADR-2610102211]: ../../docs/adr/ADR-2610102211-mcp-server-in-house.md
[pr46]: https://github.com/jeduden/parleyinn/pull/46
[consent]: https://github.com/jeduden/parleyinn/pull/91#issuecomment-6402652493
