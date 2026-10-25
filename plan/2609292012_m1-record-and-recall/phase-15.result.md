---
n: 15
title: "Operations: verify, status, audit, the boundary and the threat model"
status: "✅"
result: true
summary: >-
  @ADM-11, @OPS-01 and @SEC-15 off @pending and passing; SEC-01's core
  row passes as gate tests. @ADM-09 passes seven rows of eight and
  stays @pending (landmarks, M2). @SEC-17, @SEC-19 and @ENG-22 are
  bound in part and stay @pending, their findings raised below.
---
# Phase 15 result

## Handoff

Outcome:

- ADM-11, OPS-01, SEC-15: off `@pending`, passing (cucumber). ADM-09:
  seven rows pass with the tag lifted; the landmark row waits for M2.
  SEC-01's core row: gate tests in `tooling/engineering/tests/boundary.rs`.
- SEC-17, SEC-19, ENG-22: bound in part, back to `@pending` as phase
  15's rule asks; their checks, gate tests and drift cases stay.
- Gates: scenario gate, fmt, clippy `-D warnings`, `mdsmith check .`,
  unit tests, `e2e_crash_consistency` (80 kills); the drift suite in CI.

After the reviews:

- ADM-09 (A1): a short source passes only while a running MCP server
  holds it, or for 10 minutes after an ingest left its marker there
  (`marked_at`); verify's clock is a command's, so I10 holds. Verify
  hashes each source's consumed prefix and fails one cut or rewritten.
- Verify derives the search index again on a read into memory, under a
  60 s deadline: no write lock (I9, ENG-03). One out of time leaves it
  unchecked and counts `verify_unfinished`.
- Counter `redacted` is now `redactions`, as OPS-01 names it.
- `short_sources`, `VerifyError::Unshipped`, a served-marker seam, and
  no index slot for an event-less writer (I10).

Findings for the stakeholder:

- SEC-17: the 0.0 review row reads "no one yet" and "awaiting": no
  review is recorded, and no release gate refuses one.
- SEC-19: that a widening principal act turns a tunnel feature on is
  unchecked, and no test shows more reach than a row grants.
- ENG-22: statements are found by 8 phrases in `SECURITY.md` and 4
  doc directories; claims elsewhere, such as key custody, cite no
  test; a cited test is only shown to exist and not be ignored.
- Erasure residue: a purge leaves the source's unkeyed prefix hash and
  the erased lines' offsets (test
  `a_purge_leaves_the_prefix_hash_and_places_of_its_transcript_source`);
  clearing it breaks REC-07's resume.
- Does ADM-09 list a `store` check? The served marker is forgeable by
  a same-user process. CI's `--all-features` is a `.github` change.

Deferred: ENG-22's phrase and file set, reach.rs evidence, the
telemetry deny-list and URL scan, the release gate.
