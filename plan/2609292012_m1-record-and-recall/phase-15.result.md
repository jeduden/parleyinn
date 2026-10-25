---
n: 15
title: "Operations: verify, status, audit, the boundary and the threat model"
status: "✅"
result: true
summary: >-
  @ADM-11, @OPS-01, @SEC-15, @SEC-17, @SEC-19 and @ENG-22 off @pending
  and passing. SEC-01's core row passes as gate tests. @ADM-09 passes
  seven rows of eight and stays @pending: no landmark exists before M2.
  The security and domain-model reviews are fixed or raised below.
---
# Phase 15 result

## Handoff

Outcome:

- ADM-11, OPS-01, SEC-15, SEC-17, SEC-19, ENG-22: off `@pending`,
  passing (cucumber). ADM-09: seven rows pass with the tag lifted; the
  landmark row waits for M2, so it stays `@pending`. SEC-01's core row:
  gate tests in `tooling/engineering/tests/boundary.rs`.
- Gates run: scenario gate, fmt, clippy `-D warnings`, `mdsmith check .`,
  unit tests, `e2e_crash_consistency` (80 kills); the drift suite in CI.

After the reviews:

- ADM-09's bound (A1): a source with a whole line past its position is
  short only while a running MCP server holds it, or while an ingest
  left its ingest marker there under 10 minutes ago; else exit 3. Each
  marker keeps when ingest left it (`marked_at`, ingest's clock), and
  verify reads its own clock: a command, so I10 holds. Ten minutes
  covers a kill and a rerun (ENG-06) and bounds a lost record.
- Verify hashes each recorded source's consumed prefix once and fails
  one cut, replaced or rewritten (A3, I1). Paths are shown escaped (T16).
- `short_sources`, `VerifyError::Unshipped`; the search index is current
  only when no event-less writer holds a slot (I10).
- Threat model: a status per row, test-checked; S5 controls; the
  threats M1's record adds; review rows name their author.
- The boundary check reads executables from `cargo metadata`; the
  telemetry check reads `fuzz/Cargo.lock`; the core denies a bare
  `allow`. Drift cases for each.

For the stakeholder:

- Erasure residue: a purge leaves the source's prefix hash, an unkeyed
  SHA-256 over bytes erased lines are among, and their offsets (test
  `a_purge_leaves_the_prefix_hash_and_places_of_its_transcript_source`).
  Clearing it breaks REC-07 resume: a ruling is needed.
- Does ADM-09 list a `store` check, or does that failure name none?
- CI clippy `--all-features` is a `.github` change (no crate has
  features). The release gate refusing an awaiting review needs it too.
- The served marker is forgeable by a same-user process (threat model).

Deferred: ENG-22's phrase and file set, reach.rs textual evidence,
telemetry deny-list and URL scan, the release gate.
