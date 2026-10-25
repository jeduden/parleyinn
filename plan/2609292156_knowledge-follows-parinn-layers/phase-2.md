---
n: 2
title: "Drift injection: prove every check catches its drift"
status: "✅"
result: false
---
# Phase 2: drift injection

Phase 1 showed its checks fail by breaking the tree by hand and
reverting. That proof is gone the moment it is made. With agents
approving agents, a check can also be weakened and every scenario
goes greener. This phase makes the proof repeatable: each check gets
a registered drift case, and CI injects every drift into a copy of the
repository and fails when the check lets it through.

Requirements. This phase adds ENG-27 (P0, Ver T): every check that
keeps the specification, its scenarios and the repository's records
in step MUST have a drift case. CI MUST inject each drift into a copy
of the repository and fail when the check does not catch it. The SRS
edit waits for the stakeholder's approval.

BDD coverage: `@ENG-27` lands `@pending` with the SRS edit and comes
off `@pending` here. Its steps check the registry, so they run in
`go test ./...` without mdsmith:

- every drift case's edit still applies to the checkout, so a case
  cannot rot silently;
- every non-pending scenario that inspects the repository has a drift
  case guarding its id, and so do the requirement–scenario gate and
  the Appendix B check;
- the CI workflow runs the drift suite.

RED. Write the `@ENG-27` scenario and the registry's unit tests first.

GREEN:

- `internal/drift` holds the registry as data: each case names what it
  guards, one declarative edit (replace, remove or copy a file), the
  check to run (a `go test` run, or `mdsmith check .`) and the message
  that check must print. A case the checks miss today is marked a
  known gap. The suite fails when a known gap starts being caught,
  so the mark cannot outlive its fix.
- A `drift`-tagged test copies the repository per case, applies the
  edit, runs the check and asserts that it fails with the message. It
  first asserts the unedited copy passes every check.
- A `drift` CI job installs mdsmith, runs the suite, and joins the
  `CI` gate job.

Gate: `go test -tags drift ./internal/drift -v` passes locally with
every case caught or a known gap. Stubbing one check out makes it
fail, then the stub is reverted. `go test -race ./...`, the coverage
floor, golangci-lint and `mdsmith check .` stay clean.
