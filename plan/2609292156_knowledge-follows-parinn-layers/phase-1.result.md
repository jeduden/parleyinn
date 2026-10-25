---
n: 1
title: "Proving slice: dependency decisions as ADR files"
status: "✅"
result: true
summary: >-
  ENG-26 added and @ENG-26 off @pending; @ENG-18 now reads ADRs both
  ways. The test stack is ADR-2609292234, and DEPENDENCIES.md is a
  catalog over dependency ADRs.
---
# Phase 1 result

## Handoff

Outcome, in the SRS's terms:

- ENG-26 (P0) is new, and its scenario passes off `@pending`. Every
  ADR has an id, title, status and summary. Its file is named for its
  id, ids are unique, and a superseded ADR names an existing
  successor.
- ENG-18 is amended, and its scenario passes. Each direct dependency
  is justified by exactly one accepted ADR. An accepted ADR naming a
  module `go.mod` no longer requires fails. Licenses and the target of
  ten are read from the ENG-18 row itself, not restated.
- Both were verified by godog (`TestFeatures/^ENG-(18|26):`).
  Deliberate breaks each failed with a precise message, then were
  reverted: a fake `go.mod` requirement, the ADR removed, a duplicate
  id and a GPL license.

What the next phase inherits:

- `internal/adr` reads ADRs: flat scalar front matter, level-2
  sections and a Module table. Lists in front matter are refused, so
  no YAML dependency was added. It is at 100% coverage and in the CI
  floor. Phase 2 migrates ADR-01 to ADR-10 into it unchanged.
- The `adr` mdsmith kind checks the id shape, status, optional
  `scope` and `superseded-by`, and the four sections. `docs/adr/*.md`
  joined `unique-frontmatter`. The catalog's `where:` filters on
  `scope: "dependencies"`, so no file-name fallback was needed.
- `DEPENDENCIES.md` lists id, status and summary per ADR. Per-module
  licenses live in each ADR's Decision table.
- `/docs/adr/` is stakeholder-owned in CODEOWNERS.

Deviations from the spec:

- Modules sit in a Decision table, not in `modules:` front matter. A
  list needs a YAML parser, which is a new dependency.
- ENG-26 drops "never rewritten once accepted", which needs git
  history to check. It moved to phase 9 in the plan.
- `internal/srs`'s requirement count moved to 26 ENG rows, and
  Appendix B's engineering P0 count to 24.
- The SRS, ADR and code changes share PR #4. The session could push
  only to its own branch, so they could not land ahead as a separate
  pull request.

Parked: `frit phase` refuses to run outside the plan's own lane, so
this phase ran from the spec files directly.
