---
n: 2
title: "Remove the Go tooling; ENG-18 reads Cargo"
status: "✅"
result: false
---
# Phase 2: remove the Go tooling; ENG-18 reads Cargo

Requirements. None closes. The bound scenarios run in Rust alone.

What lands:

- `internal/` and `cmd/review-gate` go, with godog and the cucumber
  messages module; `cmd/parinn` keeps `parinn version` and the
  import-closure test until phase 4.
- ENG-18 reads the direct dependencies from `go.mod` and from `cargo
  metadata`, and reads an SPDX `OR` expression as allowed when one
  choice is. The Rust stack's ADR is accepted and supersedes the test
  stack's.
- ENG-01 also checks `rust-toolchain.toml`; ENG-27 and ENG-28 name the
  cargo commands; the drift cases follow.
- The review workflow decides with `cargo run -p review-gate`.
- CI gates on `cargo-deny` and `cargo-audit` (ENG-16); dependabot
  watches Cargo.
- CODEOWNERS names `tooling/` and the Rust configuration files.

Gate: CI green with no Go gate left, and the drift suite catching
every case through the Rust checks.
