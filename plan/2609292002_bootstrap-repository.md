---
id: 2609292002
title: "Bootstrap the repository: SRS, requirement matrix, CI and release"
status: "✅"
summary: >-
  The SRS, one pending scenario per requirement, CI, nightly fuzzing
  and the reproducible signed release pipeline — no product
  features.
model: opus
depends-on: []
---
# Bootstrap the repository: SRS, requirement matrix, CI and release

## Goal

Stand the repository up before any product code. Every later plan then
lands against a live specification and an executable requirement
matrix. The pipeline already gates builds, tests, lint and
vulnerabilities. It also gates Markdown, workflows and releases.

## Context

The setup follows frit and mdsmith. From frit come the plan format, the
skills and the godog scenario gate. From mdsmith come the lint rules
and the docs index. Every product scenario stays `@pending`.

## Tasks

1. Import the SRS as one file per section under
   [docs/srs](../docs/srs/index.md), with a summary each
2. Parse the requirement tables in `internal/srs`; check Appendix B
   against the traces
3. Write one `@pending` scenario per requirement and assumption under
   `features/`
4. Keep SRS and scenarios in bijection with `internal/scenario`,
   priority and invariant tags included
5. Run the scenarios from `cmd/parinn` with godog in strict mode, with an
   isolated `HOME` and `PARINN_HOME`
6. Add `parinn version` as the one command, stamped by the release build
7. Write ci.yml, nightly.yml and release.yml, plus dependabot,
   CODEOWNERS and codecov
8. Scaffold frit's plan machinery and skills; write CLAUDE.md,
   SECURITY.md, DEPENDENCIES.md

## Acceptance Criteria

- [x] All 149 requirement and assumption ids have exactly one scenario,
  and the gate test passes
- [x] ENG-01, ENG-18 and ENG-24 scenarios pass; every other scenario is
  reported pending
- [x] The shipped binary's import closure holds no `net` or `os/exec`
  package
- [x] All tests pass: `go test -race ./...`
- [x] `go tool -modfile=tools/go.mod golangci-lint run` is clean
- [x] `mdsmith check .` passes and zizmor reports no findings
