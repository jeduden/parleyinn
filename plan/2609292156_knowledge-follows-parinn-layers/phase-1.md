---
n: 1
title: "Proving slice: dependency decisions as ADR files"
status: "✅"
result: false
---
# Phase 1: dependency decisions as ADR files

Prove the pattern every later phase copies. One source per fact, a
generated view, and a gate that reads the source both ways. The slice
is the test stack: godog, cucumber messages and testify. They are
today's only direct dependencies.

Requirements. This phase amends ENG-18 and adds ENG-26. Both edits sit
under `docs/srs/`, so they wait for the stakeholder's approval. Land
them in their own pull request, ahead of the code.

- ENG-18: every direct dependency MUST be justified by an accepted ADR,
  listed in `DEPENDENCIES.md`. The license allow-list and the target
  stay as they are.
- ENG-26 (new, P0, Ver T): every design decision MUST live in one ADR
  file under `docs/adr/`, named for its unique id, with a title, a
  status and a summary. A changed decision MUST be a new ADR, and the
  one it replaces MUST be marked superseded and name its successor.
  "Never rewritten once accepted" needs git history to check, so it
  moved to phase 9.

BDD coverage: `@ENG-18` stays off `@pending`, and its steps change.
`@ENG-26` lands `@pending` with the SRS edit, then comes off
`@pending` in this phase. Both bind in `cmd/parinn/bdd_engineering_test.go`.

RED. Rewrite the ENG-18 scenario first:

- every direct dependency in `go.mod` is named by exactly one accepted
  ADR;
- every module an accepted ADR's Decision table names is a direct
  dependency in `go.mod`, so a stale decision fails;
- each named ADR gives a purpose, alternatives and a license on the
  allow-list;
- there are at most 10 direct dependencies, read from the ENG-18 row
  rather than restated in the scenario.

Write the ENG-26 scenario too. Every ADR under `docs/adr/` has an id,
a status and a summary. Ids are unique. A superseded ADR names its
successor. Both scenarios fail against today's tree.

GREEN, in this order.

First, add an `adr` kind to `.mdsmith.yml` (the stakeholder
consented):

- path pattern `docs/adr/ADR-*.md`;
- required front matter `id`, `title`, `status` and `summary`;
- required sections Context, Decision, Alternatives and Consequences;
- a `unique-frontmatter` rule on `id`;
- `/docs/adr/` in `.github/CODEOWNERS`.

Second, write the test-stack ADR under `docs/adr/`. Its Decision
table lists all three modules, each with its purpose, license and
maintenance status. Front matter stays flat scalars, with
`scope: dependencies` marking a dependency decision. New ids use the
minute-precision UTC time, like plan ids, so parallel agents never
collide. ADR-01 to ADR-10 keep their numbers in phase 2.

Third, replace DEPENDENCIES.md's hand-written table with a
`<?catalog?>` over `docs/adr/*.md`:

- keep only decisions with `scope: dependencies`;
- render one row per ADR with its id, status and summary, linked.

Fourth, point the ENG-18 steps at the ADRs instead of the table. The
front-matter reader and the Decision-table matching get their own
unit tests, and `cmd/parinn` and `internal/adr` keep 100% coverage.

Gate: `go test ./cmd/parinn -run 'TestFeatures/^ENG-(18|26):'` passes.
Then check that the gate actually fails:

- add a fake requirement to `go.mod` and confirm `@ENG-18` fails;
- delete the ADR and confirm `@ENG-18` fails;
- duplicate the ADR id and confirm `@ENG-26` fails.

Revert each change. `mdsmith check .`, `go test -race ./...` and the
coverage floor are clean:

```sh
scripts/check-coverage.sh 100 ./cmd/parinn ./internal/srs ./internal/scenario ./internal/adr
```
