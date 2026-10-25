---
n: 1
title: "Every gate in Rust, beside its Go original"
status: "✅"
result: false
---
# Phase 1: every gate in Rust, beside its Go original

Requirements. None closes. ENG-01, ENG-18, ENG-24, ENG-26, ENG-27 and
ENG-28 keep their scenarios bound; this phase binds them in Rust as
well. No scenario leaves `@pending`.

BDD coverage: the phase moves the runner itself, so its gate is the
runner running the existing scenarios.

What lands, a Cargo workspace under `tooling/`:

- `rust-toolchain.toml` naming an exact Rust release (ENG-01).
- `srs` and `scenario`: the SRS tables, Appendix B and C, the personas
  and the requirement–scenario gate, ported from `internal/srs` and
  `internal/scenario`.
- `adr`, `finding-ledger`, `review-gate` (with its executable) and
  `drift`, ported from their Go packages.
- `engineering`: the checks behind §10's bound scenarios, with unit
  tests, so the step bindings stay thin.
- A `bdd` target with `harness = false` in `scenario`, running
  `features/` through cucumber-rs. It re-runs itself with `HOME` and
  `PARINN_HOME` in a fresh temporary directory (ENG-14). `@pending`
  scenarios are skipped and counted; a step with no definition fails.
- A proposed ADR for the three crates; ENG-18 still reads `go.mod`
  here, so the ADR is accepted with phase 2.
- CI runs `cargo test`, clippy, rustfmt and the Rust drift suite beside
  every Go job.

RED: the Rust gates' tests, run against the repository, fail until the
parsers exist; each drift case fails its Rust check only once the check
exists.

Gate: `cargo test --workspace` and `go test ./...` both pass, and CI's
`drift` and `rust-drift` jobs both pass on the same commit: every
registered drift is caught by both languages.
