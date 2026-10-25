---
id: ADR-2610221545
title: "The product is named ParleyInn and every spelling follows"
status: accepted
summary: >-
  The invariants I1 to I10, PRV-10 to PRV-13 and the rest of the SRS name
  the product ParleyInn instead of Cairn. The executable, crate, home,
  environment variables, addresses, trailers, file names, on-disk labels
  and Keychain service follow as parinn or ParleyInn. No invariant or
  requirement changes meaning. There are no installations to migrate.
  The stakeholder, jeduden, approved it as the named human security
  reviewer on 22 October 2026, as ENG-29 asks.
---
# ADR-2610221545: The product is named ParleyInn

## Context

[Pull request 66](https://github.com/jeduden/parleyinn/pull/66) renames the
product from Cairn to ParleyInn and its executable and core crate from
`cairn` to `parinn`. The rename edits the invariants in
`docs/srs/invariants.md` (§1.3), which CLAUDE.md and AGENTS.md include. It
also edits the rows of PRV-10, PRV-11, PRV-12 and PRV-13. ENG-29 marks those
four for an I2 security review.

ENG-29 lets an invariant or one of those requirements change only with an
accepted decision record. That record names a human security reviewer's
approval. The wording of an invariant is part of the contract. So a pure
rename counts as a change to it.

## Decision

The product's name in prose is ParleyInn. The executable, crate, commands,
release assets and MCP server name are `parinn`.

Each of I1 to I10 keeps its meaning. Only the product name in its text
changes, and with it the line wrapping of the include copies. PRV-10 to
PRV-13 change only the product name. The reach and confinement gates
(SEC-01, ENG-16) and the I10 determinism gate follow the renamed crate and
binary and are not loosened.

Every other spelling follows the name, because two names for one product
confuse people and agents alike:

- The home is `~/.parinn`, or the directory `PARINN_HOME` names. The
  other environment variables move from the `CAIRN_` prefix to `PARINN_`.
- Addresses read `parinn:room/…`. The commit trailers are `ParleyInn-Room:`,
  `ParleyInn-Link:` and `ParleyInn-Session:`.
- The repository configuration is `.parinn.toml`, and the structured log is
  `logs/parinn.jsonl`.
- The domain-separation labels that name payloads and derive identities
  start with `parinn`, and the macOS Keychain service is `parinn`.
  [ADR-2610171300](ADR-2610171300-key-stores.md) says so too.
- Plans, research notes and earlier decision records use the new name, so
  the history does not read as a second product. Recorded harness sessions
  under `testdata/` stay as captured.

The stakeholder decided on 22 October 2026 that nothing keeps the old
spelling. No installation exists, so no home, key or peer needs a
migration, and the format versions stay at 1.

## Alternatives

- **Keep the name Cairn.** No edit to an invariant. But the stakeholder
  chose the new name.
- **Rename only the product and the executable.** That leaves storage
  and protocol spellings under the old name. It would need a later record
  with a format version, once installations exist.

## Consequences

- Scripts and hooks that call `cairn` must call `parinn`. The Unreleased
  section of CHANGELOG.md says so (ENG-23).
- A home made under the old names is not read. No release has shipped, so
  none is known.
- The stakeholder, jeduden, is the named human security reviewer. They
  accepted this record on 22 October 2026, which is ENG-29's approval for
  the invariant and PRV-10 to PRV-13 edits. Their consenting comments on
  [pull request 66][c1], [its second consent][c2] and
  [the ruling on the on-disk labels][c3] are the evidence.

[c1]: https://github.com/jeduden/parleyinn/pull/66#issuecomment-6344684337
[c2]: https://github.com/jeduden/parleyinn/pull/66#issuecomment-6344690911
[c3]: https://github.com/jeduden/parleyinn/pull/66#issuecomment-6344871015
