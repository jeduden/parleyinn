---
n: 2
title: "Remove the Go tooling; ENG-18 reads Cargo"
status: "✅"
result: true
summary: >-
  The Go gates and godog are gone; the six bound scenarios run in
  cucumber-rs alone. ENG-18 reads go.mod and cargo metadata, ENG-01
  checks rust-toolchain.toml, and the review workflow decides with the
  Rust review gate. 30 drift cases, all caught.
---
# Phase 2 handoff

## Done

- `internal/` and `cmd/review-gate` are deleted, with godog and the
  cucumber messages module; testify stays for the `parinn version`
  stub's tests.
- ENG-18 reads `go.mod` and `cargo metadata`, and admits an SPDX `OR`
  license when one choice is allowed. ADR-2610101442 is accepted and
  supersedes ADR-2609292234.
- ENG-01 checks `rust-toolchain.toml`; ENG-27 and ENG-28 name the
  cargo commands; the review workflow runs `cargo run -p review-gate`.
- Two new drift cases: the Rust toolchain unpinned, and a superseded
  ADR left in force. The suite catches all 30 locally.
- CI gates on cargo-deny and cargo-audit. `deny.toml` allows ENG-18's
  list, plus Unicode-3.0 and BlueOak-1.0.0 for the crates named there:
  a stakeholder decision the pull request asks for.
- CODEOWNERS names `tooling/` and the Rust configuration files.

## Next

- Phase 3 measures coverage per test layer and adds the test-engineer
  agents.
- `.mdsmith.yml`'s comments still name `internal/srs` and
  `internal/adr`; changing that file needs the stakeholder's consent.
