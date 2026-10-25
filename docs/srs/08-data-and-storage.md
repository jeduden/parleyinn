---
title: "8. Data and storage"
summary: >-
  Normative home layout, logical schema, canonical encoding (RFC 8785
  + SHA-256), the conservative model-token estimator, and the lifecycle
  grid naming what each M1–M5 lifecycle event does to each derived
  artifact.
---
# 8. Data and storage

Semantics in this section are normative. Table and column names are
illustrative; the implementation MAY differ as long as every constraint marked
**(N)** is met.

## 8.1 Home layout

```text
$PARINN_HOME/                          0700, belonging to one OS user       (N)
├── config.toml                       the principal's configuration        0600
├── home.json                         home id binding (home.id, SEC-03)    0600
├── audit/audit-NNNNNN.jsonl          hash-chained audit log               0600
├── logs/                             structured logs (optional)
├── store.db  (+ -wal, -shm)          the store: record and derived artifacts, partitioned by writer  0600
├── payloads/<name>                   the store's payload store, named per REC-09  0600
└── ingest/                           ingest markers for deferred ingestion
```

A home contains one store, `store.db` with its payload store, partitioned by
writer, never by repository or room (N). Names in the home MUST NOT reveal a
repository's path or identity (N). ParleyInn and its storage engine MUST keep
every
temporary file in memory or in a directory in the home, never elsewhere, and,
while encryption at rest is on, in memory or encrypted under the at-rest key
(N).

## 8.2 Logical schema

| Table                | Kind       | Key contents                                                                                                                                                                                                                                                                                                                | Constraints                                                                                                                                                                                                    |
| -------------------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `meta`               | state      | schema version, home id                                                                                                                                                                                                                                                                                                     | single row                                                                                                                                                                                                     |
| `transcript_sources` | state      | path, generation, the harness's transcript id, the runs it contains (a harness session's transcripts contain its main agent's run and one per subagent, a subagent's possibly in its own file, ASM-03), each with its parent run and the harness's agent id, ingest position (byte offset and consumed-prefix hash), status | UNIQUE(path, generation) **(N)**                                                                                                                                                                               |
| `events`             | **record** | writer, `seq`, run, transcript source, generation, transcript line, kind, provenance, recorder, tool name, inline text or preview, payload reference and size, commitment (REC-17), model-token estimate, flags, transcript timestamp, `prev_hash`, `hash`, redaction count                                                 | (writer, `seq`) PRIMARY KEY, `seq` strictly increasing and gap-free per writer **(N)**; UNIQUE(transcript source, generation, transcript line) **(N)**; rows immutable except content removal by purge **(N)** |
| `events_fts`         | derived    | FTS5 over event text                                                                                                                                                                                                                                                                                                        | rebuildable **(N)**                                                                                                                                                                                            |
| `spans`              | derived    | run, address range, span boundary kind                                                                                                                                                                                                                                                                                      | rebuildable **(N)**                                                                                                                                                                                            |
| `landmarks`          | derived    | tier, address range, rendered text, taint                                                                                                                                                                                                                                                                                   | rebuildable **(N)**                                                                                                                                                                                            |
| `pins`               | derived    | creating address, pin version, type, priority, room, text, creating event's commitment, each version's author, stamps, active                                                                                                                                                                                               | rebuildable from pin events (`operator` for device-seat pins, `assistant` for run-seat pins), stamps and unstamps **(N)**                                                                                      |
| `quarantine`         | derived    | creating address, selector, active                                                                                                                                                                                                                                                                                          | rebuildable from quarantine events **(N)**                                                                                                                                                                     |
| `tombstones`         | derived    | purge event address, purged range, counts, reason or retention policy, commitments of removed events, structural fields, provenance and recorder of each removed act, tombstone and event the key set holds                                                                                                                 | rebuildable **(N)**                                                                                                                                                                                            |

Purge replaces event content with nothing and keeps the row's address, `hash`,
and `prev_hash`, so the chain stays verifiable and each writer's `seq` stays
gap-free **(N)**.

## 8.3 Canonical encoding

The hash chain (REC-10) and the audit chain (OPS-02) MUST use the JSON
Canonicalization Scheme (RFC 8785) over a documented field set, with SHA-256
**(N)**. The field set and encoding version are recorded in `meta` so that
future changes remain verifiable.

The [canonical digest](../domain-model/record.md) of a derived artifact
(ADM-08) MUST be SHA-256 over the canonical encoding of its content: every
value any read of it can return, row by row in key order, together with its
schema and every stored setting that decides what a read returns, such as the
search index's tokenizer and ranking configuration. Its field set and version
are recorded in `meta` **(N)**.

## 8.4 Model-token estimation

The pin budget (PIN-08) and the model-token caps of INJ-07, RCL-03 and
CMP-06 depend on model-token counts.
ParleyInn MUST use a conservative estimator, calibrated in M0 against
Claude's own model-token counts, such that the estimate is greater
than or equal to the true count for at least 99% of a representative
sample **(N)**.

Spike S8 calibrated one: each character adds the weight of one of
eleven character classes, split by script and kind, and each change
between lowercase, uppercase and digit from one letter or digit to the
next adds another; the sum, rounded up, plus 6 is the estimate. Its
weights and the committed sample are in the spike's report until M1
moves them into the core and `testdata/`, with a test of this bound. The pin
budget, the restore block limit and the output caps of RCL-03 and CMP-06
count the estimate, not the true count; spend counts the true count.

## 8.5 Lifecycle grid

The grid names, for each lifecycle event that ships in M1 to M5, the
requirement that decides what it does to each derived artifact I10
lists, to provenance and to room membership. Origins follow PRV-13 in every
row: an event's origin changes only when a node identity change, a backup
restore or an arrival changes what PRV-13 reads. An event a lifecycle event
appends is indexed, counted and shown like any other; a cell names only
what goes beyond that, and a dash means no effect. Run status (VIEW-04),
the Needs you queue (VIEW-05), evidence classes (LANE-05) and proof
classes (LANE-06) ship in M7, so their columns hold only what M1 to M5
already derive. The final review and M1's scenarios check against the
grid, and each later milestone's review adds the events it ships
(OQ-47).

| Event                                       | Search index   | Landmarks      | Active pins             | Quarantine set | Room state             | Trust levels   | Run status | Integrity status | Queues | Evidence classes | Proof classes | Stats           | Provenance     | Room membership         |
| ------------------------------------------- | -------------- | -------------- | ----------------------- | -------------- | ---------------------- | -------------- | ---------- | ---------------- | ------ | ---------------- | ------------- | --------------- | -------------- | ----------------------- |
| Node identity change                        | —              | LMK-01         | ADM-06, PIN-10          | ADM-06         | REC-24, ADM-06         | ADM-06, PRV-02 | —          | —                | —      | —                | —             | REC-24          | ADM-06, REC-19 | REC-24, LANE-01         |
| Node clone                                  | —              | LMK-01         | ADM-06, PIN-10          | ADM-06         | REC-24, ADM-06         | ADM-06, PRV-02 | —          | —                | —      | —                | —             | REC-24          | ADM-06, REC-19 | REC-24, LANE-01         |
| Backup restore                              | ADM-06, ADM-08 | ADM-06         | ADM-06, PIN-10          | ADM-06         | ADM-06                 | ADM-06, PRV-02 | —          | ADM-06, REC-21   | —      | —                | —             | ADM-06          | ADM-06, PRV-09 | ADM-06                  |
| Harness resume or clear                     | REC-07         | LMK-01         | REC-02, PIN-10          | —              | REC-02, SEC-27         | SEC-13         | —          | —                | —      | —                | —             | RCL-01          | REC-19, REC-22 | REC-02, LANE-01         |
| Subagent start                              | —              | LMK-01         | REC-02, PIN-10          | —              | LANE-16, LANE-23       | SEC-13, PRV-08 | —          | —                | —      | —                | —             | RCL-01          | PRV-08, REC-19 | LANE-23                 |
| Run end                                     | —              | LMK-01, LMK-02 | —                       | —              | —                      | SEC-13         | —          | REC-19           | —      | —                | —             | REC-19          | REC-13, REC-19 | —                       |
| Run-seat key loss                           | —              | LMK-01         | PIN-10                  | SEC-12         | SEC-27, REC-19         | —              | —          | REC-19           | —      | —                | —             | SEC-27, REC-19  | REC-19, SEC-10 | SEC-27, LANE-23         |
| Ingest                                      | REC-03, SEC-31 | LMK-01         | REC-22                  | SEC-12         | REC-19                 | REC-22         | —          | ADM-09, REC-08   | —      | —                | —             | REC-19, REC-05  | REC-22         | REC-19, REC-22          |
| Compaction                                  | —              | LMK-01, REC-13 | INJ-01, INJ-05          | —              | —                      | PRV-03         | —          | —                | —      | —                | —             | PIN-08          | PRV-08, INJ-09 | —                       |
| Quarantine                                  | RCL-06, SEC-12 | LMK-04         | PIN-10, SEC-12          | SEC-12         | SEC-12                 | —              | —          | —                | —      | —                | —             | —               | SEC-12         | LANE-01                 |
| Release of a quarantine                     | RCL-06, SEC-12 | LMK-04         | PIN-10                  | SEC-12         | SEC-12                 | —              | —          | —                | —      | —                | —             | —               | SEC-12         | LANE-01                 |
| Purge                                       | ADM-07, SEC-31 | ADM-07         | PIN-10                  | ADM-07         | ADM-07                 | ADM-07         | —          | ADM-07, REC-17   | —      | —                | —             | ADM-07, SEC-31  | PRV-01, ADM-07 | LANE-01                 |
| Retention purge                             | REC-15, ADM-07 | ADM-07         | PIN-10, ADM-04          | ADM-07         | ADM-07                 | ADM-07         | —          | ADM-07           | —      | —                | —             | REC-15, SEC-22  | PRV-01, SEC-22 | LANE-01                 |
| Configuration acceptance                    | —              | PRV-07         | ADM-04, PIN-03, PIN-08  | —              | ADM-04, PIN-10         | PRV-02, PRV-04 | —          | PRV-02           | ADM-11 | —                | —             | ADM-04, OWN-12  | OWN-02         | —                       |
| Room creation                               | RCL-05         | LMK-01         | PIN-10                  | —              | LANE-01, LANE-16       | PRV-01         | —          | —                | —      | —                | —             | LANE-01         | PRV-01, SEC-10 | LANE-01                 |
| Join of one's own room                      | RCL-05         | LMK-01         | PIN-10, REC-02          | —              | LANE-16, LANE-23       | PRV-01, PRV-02 | —          | PRV-02, PRV-11   | —      | —                | —             | LANE-23, OWN-12 | OWN-02, REC-19 | LANE-23                 |
| Pin-candidate confirmation                  | —              | —              | PIN-05, PIN-10          | SEC-12         | PIN-05, LANE-20        | PRV-02         | —          | PRV-02           | PIN-05 | —                | —             | OWN-12          | OWN-02         | LANE-23                 |
| Branch link                                 | —              | LMK-01         | —                       | —              | LANE-01, LANE-31       | —              | —          | —                | —      | —                | —             | OPS-01          | PRV-01         | LANE-23                 |
| Risk acceptance recorded or withdrawn       | —              | —              | PRV-02, OWN-22, PIN-10  | PRV-02         | PRV-02                 | PRV-02         | —          | PRV-02           | —      | —                | —             | OWN-12, OPS-01  | OWN-02, OWN-22 | LANE-01                 |
| Pin act, or intent set or revised           | —              | —              | PIN-04, PIN-10, LANE-20 | SEC-12, PIN-10 | PIN-10, LANE-20        | PRV-02, PIN-10 | —          | PRV-02           | —      | —                | —             | OWN-12, OPS-01  | OWN-02         | LANE-01, LANE-23        |
| Pin candidate proposed or detected          | —              | —              | PIN-02, PIN-05          | SEC-12         | PIN-05, LANE-20        | PIN-02, PRV-06 | —          | —                | PIN-05 | —                | —             | OPS-01          | PIN-02, PIN-05 | —                       |
| Device or seat key lost, identity unchanged | —              | LMK-01         | REC-24                  | SEC-12         | SEC-10, SEC-27, REC-19 | REC-24         | —          | REC-19           | —      | —                | —             | SEC-10, OPS-01  | REC-24         | SEC-10, REC-19, LANE-01 |
| MCP server start or restart                 | —              | LMK-01         | PIN-10                  | SEC-12         | LANE-16, SEC-27        | PRV-02, REC-19 | —          | REC-19           | —      | —                | —             | REC-19          | REC-19, PRV-11 | LANE-01, LANE-23        |
| Harness session a tool call starts          | —              | LMK-01         | PIN-10, REC-02          | —              | LANE-16                | PRV-08, SEC-13 | —          | —                | —      | —                | —             | —               | PRV-08         | LANE-01, LANE-23        |
| Transcript rewritten, shrunk or deleted     | REC-07         | LMK-01         | —                       | —              | —                      | —              | —          | REC-08, ADM-09   | —      | —                | —             | REC-08          | —              | —                       |
| User turn                                   | —              | LMK-01         | PIN-05, INJ-04          | —              | —                      | PRV-02, PRV-04 | —          | —                | PIN-05 | —                | —             | INJ-04, PIN-08  | PRV-08         | —                       |
| Hook handler cut short by its budget        | —              | REC-13         | —                       | SEC-12         | REC-19                 | REC-19, REC-22 | —          | ADM-09           | —      | —                | —             | REC-13          | REC-19, REC-22 | REC-19                  |
