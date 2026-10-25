---
n: 4
title: "Redaction before anything is written, input limits and paths"
status: "✅"
result: false
---
# Phase 4: redaction before anything is written, input limits and paths

Requirements. SEC-08's redaction of everything `parinn ingest`
writes, SEC-05's hook input limits, SEC-18's allowed roots and
ENG-05's logs. `@ENG-05` comes off `@pending`; the other three keep
rows a later phase reaches, their M1 part bound and passing. A release
build ingests again: phase 1's refusal is lifted.

Redaction (SEC-08), a new `redaction` crate on `regex` (its ADR lands
here, the stakeholder's choice of 16 October 2026):

- The maintained rule set, version 1: a named pattern per common
  secret format (cloud, forge and API keys, private keys, tokens).
- The harness account identifiers, learnt from the harness's own
  fields: the email address in a `session_context` attachment's
  `userEmail`, the `organizationUuid` of a `credential_org`
  attachment, then redacted throughout that transcript; no generic
  email pattern.
- The principal's redaction patterns, `redaction.extra_patterns` in
  its own configuration or managed policy, under a size limit. One
  that fails to compile is refused, audited and counted; the rest
  still apply.
- Each JSON string in a line is decoded and redacted on its own, keys
  included, so no escape or duplicate key hides a secret; the text
  between strings is redacted as it stands. Each match becomes
  `[REDACTED:<rule>]` (`extra_pattern_<n>` for the principal's),
  carrying nothing derived from the secret. No correlation index is
  kept, so none can confirm a guess.
- `parinn ingest` redacts every line before it draws a commitment key
  or writes. An event's content records its redactions; each is
  counted.

Input limits (SEC-05):

- `parinn hook` stops reading the hook input past 1 MiB. Such input,
  or input nested deeper than 64 levels, is refused before it is
  parsed. Each refusal is audited with the scenario's text and counted
  on its own counter. The hook still exits 0 and injects nothing (I9).
- Streaming a transcript line above the payload threshold needs the
  payload store, so it moves to phase 6.

Allowed roots (SEC-18):

- A path is resolved, symbolic links included, and must lie within an
  allowed root once resolved: `transcript.roots`, the home, or, for
  reading only, the run's worktree and its repository's git directory.
- `parinn ingest --path` reads only within `transcript.roots`, never
  the home's keys or store. A path outside is refused, audited as
  "path outside allowed roots" and counted; nothing is read from it.
- The worktree comes from the session binding (phases 9 and 10);
  until then the read-only roots are built and tested only.

Logs (ENG-05, OPS-05's file):

- `tracing` (its ADR lands here) through an in-house subscriber, a
  `structured_log` module: one JSON object per line in
  `logs/parinn.jsonl`, `0600`, each field redacted by the rule set.

Where the gate's scenarios go live:

- `@ENG-05`: here.
- `@SEC-05`: rows 1 and 2 pass here; row 3 needs phase 6's payload
  store.
- `@SEC-08`: the ingest steps pass here. Its hook input step needs
  phase 9's handlers, and `parinn purge --run` is ADM-07's run scope, in
  M3. Raise this with the stakeholder.
- `@SEC-18`: its step needs phases 9 and 10; unit tests prove its
  rows on the allowed roots.

Tests. Unit tests beside each function. A test over every recorded
transcript finds no account identifier in the store.

For the stakeholder: the domain model defines neither "allowed root"
nor "structured log", which SEC-18 and §8.1 use.

Parked to phase 7: the redactor's fuzz target, which joins the first
`fuzz/` crate with the configuration parser's.

Gate: `@ENG-05` passes. The M1 rows of `@SEC-05` and `@SEC-08` pass
bound as steps, and `e2e_redaction.rs` drives them with SEC-18's
roots. No fixture secret reaches the store or a log.
