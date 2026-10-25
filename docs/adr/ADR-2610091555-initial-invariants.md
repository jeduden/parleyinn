---
id: ADR-2610091555
title: "The initial invariants"
status: superseded
superseded-by: ADR-2610112209
summary: >-
  The security reviewer's record of ParleyInn's ten invariants as they stand
  in docs/srs/invariants.md: the initial contract, approved by the
  stakeholder by a comment on the pull request that adds it, on 9 October
  2026. It replaces the change records of the draft, and every later
  change to an invariant lands under ENG-29.
---
# ADR-2610091555: The initial invariants

## Context

ParleyInn is pre-implementation. While the SRS and the domain model were
drafted, the invariants (§1.3, [invariants.md](../srs/invariants.md))
were reworded many times, and each rewording got its own security-review
record: 20 decision records from SRS 2.0 to 2.7. None of those wordings
ever shipped, so the records describe a draft rather than a contract.

[ENG-29](../srs/10-engineering-quality.md#104-process) lets a change to an
invariant land only with an accepted record of a named human security
reviewer's approval. Until a security reviewer is named, the stakeholder,
@jeduden, holds that role (CLAUDE.md, Review).

## Decision

The ten invariants I1 to I10 are ParleyInn's initial contract. Their text is
the one [docs/srs/invariants.md](../srs/invariants.md) holds in the commit
that adds this record. The file's SHA-256 is:

```text
3d669831fe07f49646b8a7cff9a15ae133ec70592f7ab5e56de042d63a8240ec
```

The stakeholder, @jeduden, approved this record as security reviewer on
9 October 2026 by a [comment on jeduden/parleyinn#18][approval] naming the
record's id and the hash above. That comment is the approval
ENG-29 asks for, and it can be checked apart from this record.

This record replaces the draft's change records, which are deleted; git
history keeps them. The invariants' single source stays invariants.md,
which CLAUDE.md, AGENTS.md and README include.

## Alternatives

- **Keep the 20 change records.** They document how a draft evolved, not
  what ParleyInn promises, and every reader had to replay them to learn the
  current wording.
- **Delete them with no replacement.** ENG-29 would then have no recorded
  approval for the wording every later change is measured against.

## Consequences

- Any change to an invariant from here on is a design change (§1.3): it
  needs a new accepted record with a named security reviewer's approval
  (ENG-29) that supersedes this one (ENG-26), and a new major version.
- The domain-model convergence loop never edits invariant wording; a
  finding that would need it goes to the stakeholder.
- The file's hash above lets a reviewer check that invariants.md still
  matches what was approved.

[approval]: https://github.com/jeduden/parleyinn/pull/18#issuecomment-6087607800
