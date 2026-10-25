---
n: 15
title: "Operations: verify, status, audit, the boundary and the threat model"
status: "✅"
result: false
---
# Phase 15: operations

Requirements. ADM-09, ADM-11, OPS-01, SEC-01's core row, SEC-15,
SEC-17, SEC-19 and ENG-22, on what phases 1 to 14 record. I4, I6, I10:
the persona-security-officer reviews the boundary and threat-model rows.

- ADM-09: `parinn verify` exits 3 and names the failed check for each
  damage row: transcript source completeness, hash chain, payload
  integrity, and derived artifact consistency (landmarks, trust levels,
  a schema object ParleyInn does not ship, the search index's ranking
  configuration, an index entry whose event lacks the term). A source
  past its ingest position is short, failing only as REC-13 says or
  once its ingest marker is 10 minutes old; one cut below its position
  or no longer holding its consumed prefix fails.
- ADM-11: `parinn status --json` shows the store location and size,
  schema version, deployment mode, non-zero counters, and the settings
  awaiting acceptance; it says per OS whether node identity is checked,
  and the three PEER-10 sentences (second event at one address,
  `equivocated` waits for seals, the peer check waits for the peer
  network). No new counter; `retention_purged` still fails `doctor`
  until acknowledged (OQ-50).
- OPS-01: each degraded operation of the outline appends an audit entry
  of the class named and raises its counter by one.
- SEC-01, SEC-15, SEC-19: the core's reach evidence, a scan for
  telemetry, crash-report and update-check code or dependency, and the
  boundary register's check are bound for the core, the one component
  M1 builds. Rows for later components stay `@pending` with a finding;
  a scenario that covers more than the core is raised, not retagged.
- SEC-17: a threat-model document under `docs/` covers every row of the
  boundary register with the sources acting across it, compares
  ParleyInn with Zed Delta on record signing, trust in other
  principals, central-service dependence and key custody, and records a
  review naming the current minor release. A check reads it.
- ENG-22: every privacy or data-flow statement in the documentation
  cites a test; a check lists the statements and resolves each
  citation to a test that exists, with a drift case (ENG-27).

SRS wording this needs goes to claude/m1-phase-15-docs, not here.

Where scenarios go live: `@ADM-09`, `@ADM-11` and `@OPS-01` through the
built core in worlds the steps build; `@SEC-01`, `@SEC-15`, `@SEC-17`,
`@SEC-19` and `@ENG-22` on the repository checkout. One that cannot pass
as written stays `@pending` and is raised.

Gate: `@ADM-09`, `@ADM-11`, `@OPS-01`, `@SEC-15`, `@SEC-17`, `@SEC-19`
and `@ENG-22` pass; SEC-01's core row passes.
