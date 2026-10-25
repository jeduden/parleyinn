---
n: 4
title: "Redaction before anything is written, input limits and paths"
status: "✅"
result: true
summary: >-
  @ENG-05 off @pending and passing. parinn ingest redacts every line
  before it writes, refuses while a settings layer fails, reads only
  within the allowed roots, and a release build ingests again; hook
  input past 1 MiB or depth 64 is refused. SEC-05 rows 1-2 and SEC-08's
  ingest steps pass narrowed.
---
# Phase 4 result

## Handoff

Outcome:

- ENG-05 holds: `@ENG-05` is off `@pending`, its scenario passing
  against the built core, confined.
- SEC-08's M1 part holds in `parinn ingest`: the rule set, version 1,
  the harness account identifiers and the principal's redaction
  patterns, each match `[REDACTED:<rule>]`, each redaction audited.
  `harness_account.rs` finds no identifier from any recorded
  transcript in the store.
- SEC-05's rows 1 and 2 and SEC-18's transcript roots hold;
  `e2e_redaction.rs` drives all three, confined.
- `@SEC-05` rows 1 and 2 and SEC-08's ingest steps pass with their
  scenarios narrowed locally; none comes off `@pending`.
- I1, I6, I8 and I9 hold: only secrets leave the record, every
  redaction and refusal is counted, nothing outside the roots is read,
  and the hook still fails open.
- The stakeholder ruled that ingest refuses while the principal's
  configuration or the repository configuration fails, audited and
  counted (`ingest_refused_settings_invalid`); ADM-04 now says so, and
  `@ADM-04`'s failing-layer rows show the refusal. The transcript stays
  for an ingest once the layer is fixed (I1).
- Log fields are redacted with the rule set and the redaction patterns
  the principal's configuration and managed policy name, a
  repository's never; while the principal's configuration fails, no
  line is written, each counted as dropped. The logged transcript path
  is redacted with the harness account identifiers too.
- A `.git` file's git directory is judged after resolving it, so no
  link or `..` opens the home as a read-only allowed root.
- The domain model defines allowed root, structured log, redaction
  rule, redaction rule set, harness account identifiers and redaction
  patterns; SEC-08 says redaction, not the rule set, removes the
  harness account identifiers.

Inherited:

- The `redaction` crate: `Redactor::new(&patterns)`,
  `with_identifiers`, `redact_line`, `redact_text`;
  `HarnessAccountIdentifiers::learn`.
- `allowed_root`: `AllowedRoots::with`, `with_worktree`, `resolve`.
  Phase 9's session binding adds the worktree root.
- `structured_log`: `StructuredLog`, installed per run by `main`.
- `parinn::bounded_lines` caps every input line at 1 MiB; `parinn mcp`
  answers a longer request and goes on.
- Counters `redacted`, `redaction_pattern_refused`,
  `hook_input_too_large`, `hook_input_too_deep`,
  `path_outside_allowed_roots`, `log_line_dropped`.
- `testkit::place_transcript` puts a fixture under the isolated home's
  transcript root, which tests now need.

For the stakeholder:

- `@SEC-08` cannot pass in M1: `parinn purge --run` is ADM-07's run
  scope, in M3.
- While a settings layer fails, may hook handlers still record? Their
  redaction patterns are as unknown as ingest's; phase 9 decides.
- A hostile worktree's `.git` file can still name another
  repository's git directory as a read-only root: for security review.

Parked:

- The redactor's fuzz target, to phase 7's first `fuzz/` crate.
- Streaming a long transcript line (SEC-05 row 3), to phase 6.
- Ingest reads a path after resolving it, so a same-user swap can race
  it, and a FIFO in a root blocks the read: phase 8's ingest.
- Ingest failures other than refusals reach stderr, unaudited.
- Redactions are audited just after the write, not with it.
- Only the logged transcript path passes the harness account
  identifiers; a later log field that could carry them must pass
  ingest's redactor too.
