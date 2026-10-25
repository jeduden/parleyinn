---
id: 2609292156
title: "The repository's knowledge follows ParleyInn's layers"
status: "🔳"
summary: >-
  Hold the repository's own specs, scenarios, decisions and plans to
  ParleyInn's invariants, so many agents with limited context keep them
  in sync: one file per decision, verbatim pinned context, stable ids
  with a recall bundle, generated derived state, and loud drift gates.
model: sonnet
depends-on: []
---
# The repository's knowledge follows ParleyInn's layers

## Goal

Make this repository workable for many agents with small contexts.
Hold it to the same rules ParleyInn holds long sessions to. Nothing is
lost, and constraints stay verbatim. Recall goes by stable id. Derived
state is rebuilt, and drift fails loudly.

## Context

ParleyInn is built by many agents at once. Each reads a small slice of
the repository, and agents approve each other's pull requests. The
stakeholder approves only requirement and gate changes (ENG-21). No
human reads most diffs. So every kind of drift between Markdown, the
SRS, the scenarios and the code has to fail a gate. Otherwise a
reviewer agent has to catch it from a small, exact bundle.

The rule this plan applies is ParleyInn's own contract, turned on the
repository:

| ParleyInn    | In the repository                                                    |
| ------------ | -------------------------------------------------------------------- |
| Record, I1   | Decisions are appended and superseded, never rewritten; stable ids   |
| Pinned, I3   | The invariants reach every session verbatim; the pinned set is small |
| Working view | Small indexes, then one id's material pulled on demand               |
| I10          | Every table, index and status line is generated and re-checked       |
| I6           | Drift fails CI, naming the id and the file and line                  |
| I2           | Author summaries are untrusted; reviewers read the sources           |
| I5           | Retired ids keep their files, marked by status, and are never reused |

What exists and is reused:

- `internal/scenario`'s gate keeps SRS ids and scenario tags in
  bijection, with priority and invariant tags. Every phase extends it
  and replaces none of it.
- `internal/srs` parses the SRS tables. The same `Tables` reader parses
  DEPENDENCIES.md today, and the ENG-18 steps read it through that reader.
- mdsmith's `<?catalog?>`, `<?include?>`, `token-budget` and
  `unique-frontmatter`, and its merge driver, already generate and
  merge PLAN.md, AGENTS.md and the CLAUDE.md doc index. Phases 1, 2
  and 4 reuse them instead of writing generators.
- `cmd/parinn/imports_test.go` walks the shipped binary's import
  closure. Phase 3 reuses the walk to check "test only" claims.
- `frit phase` bundles one phase for a session. Phase 5's
  `trace <id>` bundle is the same idea for one requirement id.

Not reused: Sphinx-Needs, OpenFastTrace and StrictDoc would each add
a toolchain and a second id syntax next to the one the gate already
owns. Their best idea, versioned ids that flag stale coverage, is
phase 7's requirement hash.

The stakeholder consented to the `.mdsmith.yml` changes this plan
needs: an `adr` kind and a token budget on CLAUDE.md. Changes under
`docs/srs/`, `.github/` and the gate packages wait for their approval
(CODEOWNERS). Keep them apart from code pull requests.

## Tasks

1. Proving slice: an `adr` kind, one ADR file for the test stack,
   DEPENDENCIES.md rendered as a catalog over dependency ADRs, and
   ENG-18 checking the ADRs both ways (ENG-18, new ENG-26)
2. A drift-injection suite: every check that keeps the specification,
   scenarios and records in step has a registered drift case, and a
   CI job injects each drift into a copy of the repository and fails
   when the check lets it through (new ENG-27)
3. Move ADR-01 to ADR-10 out of SRS §4.5 into files, and render §4.5
   as a catalog. ADR-07 absorbs the SQLite row spike S2 would otherwise
   write twice.
4. Verify each dependency ADR's License column against the license
   detected from the module's source. Check its "test only" claim
   against the shipped import closure.
5. Pinned context: CLAUDE.md includes the §1.3 invariant table
   verbatim, and mdsmith gives CLAUDE.md a token budget
6. Stable ids: every id mentioned in Markdown resolves, and
   `trace <id>` prints one id's requirement, scenario, bound steps,
   ADRs and plans
7. No hand-written derived state: pending and passing status and
   coverage are generated, and status sentences leave the prose
8. Wording and pending gates: a requirement hash tag on each scenario, and a
   merge-base check that no existing id returns to `@pending` and
   that no ✅ plan names a pending id
9. A reviewer-agent protocol skill: fresh context, reads `trace`
   bundles and gate output, never the author's summary. Plan
   2609301942 ships the skill and the reviewer app; this phase
   teaches it to read the `trace` bundles once phase 6 has them
10. Retire, never delete: retired requirements and superseded ADRs
   keep their files, an accepted ADR's decision is never rewritten
   (checked against the merge-base), and the gate rejects a reused id

## Execution

| Phase | Model  | Gate                                                                  |
| ----- | ------ | --------------------------------------------------------------------- |
| 1     | opus   | `@ENG-18` and `@ENG-26` pass; a stray go.mod or ADR change fails them |
| 2     | opus   | The drift job fails when any registered drift goes uncaught           |
| 3     | sonnet | `mdsmith check .`; the gate still parses every ADR id §4 cites        |
| 4     | sonnet | `@ENG-18` fails on a fixture ADR with a mislabelled license           |
| 5     | haiku  | `mdsmith check .` fails when CLAUDE.md's invariants differ from §1.3  |
| 6     | sonnet | A dangling id fails the gate; `trace REC-03` matches a golden file    |
| 7     | sonnet | `mdsmith check .` fails on a stale generated status table             |
| 8     | opus   | Editing a requirement without its scenario hash fails the gate        |
| 9     | opus   | A dry review of a seeded drifting pull request flags the drift        |
| 10    | sonnet | The gate fails on a reused id and on a deleted retired scenario       |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                                                                                                                             |
| --- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | ✅     | [Proving slice: dependency decisions as ADR files](phase-1.md)                                                                                                    |
|     | ↳      | ENG-26 added and @ENG-26 off @pending; @ENG-18 now reads ADRs both ways. The test stack is ADR-2609292234, and DEPENDENCIES.md is a catalog over dependency ADRs. |
| 2   | ✅     | [Drift injection: prove every check catches its drift](phase-2.md)                                                                                                |
|     | ↳      | ENG-27 added and off @pending. 14 registered drifts, 13 caught and one known gap; the drift CI job fails when a check lets one through.                           |
<?/catalog?>

## Acceptance Criteria

- [ ] Every decision lives in one ADR file; DEPENDENCIES.md and SRS
  §4.5 are catalogs over them
- [ ] Scenarios @ENG-18 and @ENG-26 pass, and each later requirement
  this plan adds has a passing scenario
- [ ] CLAUDE.md carries the invariants verbatim within its token budget
- [ ] No Markdown file names an id that does not resolve
- [ ] No status line in the prose is written by hand
- [ ] All tests pass: `go test -race ./...`
- [ ] `mdsmith check .` is clean
- [ ] `go tool -modfile=tools/go.mod golangci-lint run` is clean
