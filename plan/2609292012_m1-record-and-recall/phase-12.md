---
n: 12
title: "Search: the search index, the query compiler and event_search"
status: "✅"
result: false
---
# Phase 12: search

Requirements. REC-11, SEC-04 and RCL-02, on what phase 8 ingests.
ENG-07's query parser target, ENG-08's rebuild-digest property and
NFR-03's benchmark (ENG-15). Three targets freeze (§12.3): REC-11's
indexing cap, RCL-02's hits and SEC-04's terms.

The search index, in `store.db` beside the record (§8.2, I10):

- An FTS5 table over each event's text: the `text` of its content,
  full text, also when the content went to the payload store, up to
  the indexing cap of 1 MiB per event. No text is copied into the
  index (contentless), so the record stays the one place content lives.
- A row's id derives from its address (S12): its seq in the high
  bits, a slot taken from its writer id in the low ones. Where two
  writers share a slot the smaller id keeps it, and a writer the search
  index leaves out is named by `Store::search_index_unindexed_writers`.
- An event is indexed in the transaction that appends it. A store
  without an index, one phases 1 to 8 made, gets it built the first
  time it opens for writing. `Store::rebuild_search_index` derives it from the
  record alone.
- The tokenizer and the indexing cap are kept in `meta` and are part of
  the index's canonical digest (§8.3), which hashes the settings, the
  schema, the writers' slots and every posting, in key order.

The query compiler and the ranking, a new `search` crate that knows no
SQLite (CODEOWNERS reserves its name as security-sensitive):

- Caller text is literal terms: split on whitespace, each term quoted
  for FTS5, so `OR`, `NEAR(x)` and `"` run nothing. `any_of`, `all_of`
  and `phrase` are the only operators, as structured parameters.
- At most 16 terms (across all parameters) of at most 64 characters; a
  leading `*` is refused. Each refusal is fixed text.
- The ranking is pure: the best `k` of the scored matches, damped per
  writer, ties to the smaller address.

Bounded BM25 (S2): only the newest 2,000 matches after the filters are
ranked, newest by seq then writer hash. BM25 with length normalization
ranks them. Damping divides each further hit of one writer by one plus
half the hits already taken. The best `k` (default 10, at most 50) return.
The query deadline (default 2 s) interrupts the query. A cancelled
query is counted `query_cancelled` and audited.

M1 deviates from RCL-02 and §9.2, for the stakeholder. A run cannot yet
be derived from its seats' writers, so the `run` parameter takes a
writer id and damping is per writer. RCL-02 also says nothing of ranking
only the newest 2,000 matches.

`event_search` over MCP, in the envelope `event_get` already uses,
each hit one item whose content is its preview and which carries its
score. Filters: `scope` (`room`; `run` needs the session binding, as
`event_get`, and `rooms` is RCL-05's), `room`, `run` (a writer id),
`provenance[]`, `kind[]`, `trust`, `since`, `until`, `k`. Trust is
`untrusted` for every ingested event until PRV-02 (phase 11).

Where the gate's scenarios go live:

- `@REC-11`: ingest then `event_search` over `parinn mcp`.
- `@SEC-04`: the four term rows over `parinn mcp`; the deadline row on
  a store of 1M events built by the step.
- `@RCL-02`: needs the session binding (phase 9), live ingest (phase
  10), trust levels (phase 11) and the `rooms` scope (phase 13), and
  its transcript "mixed-provenance" has no steps. It stays pending and
  is raised; named tests prove its M1 part.

Gate: `@REC-11` and `@SEC-04` pass. `@RCL-02` passes or is raised.
NFR-03 is benchmarked and the rebuild-digest property holds. The query
parser's fuzz target runs nightly. The three targets are frozen.
