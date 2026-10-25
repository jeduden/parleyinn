---
n: 4
title: "The core executable in Rust; Go removed"
status: "✅"
result: true
summary: >-
  `parinn version` is the Rust core executable, built by one workflow
  CI and the release share: one Linux and one macOS builder, four
  targets, each run on its own platform. Its clippy config and reach test hold
  it to no socket, no program start and no linked crate. Go is gone
  outside the archived spike S2 benchmark; SRS 2.32-draft drops the Go
  clauses.
---
# Phase 4 handoff

## Done

- The `parinn` crate under `crates/` is the core executable. Its unit
  tests hold it at 100%, an end-to-end test runs the built binary, and
  `parinn version --json` prints `{"version": ..., "commit": ...}`.
- `.github/workflows/build.yml` builds `linux/amd64`, `linux/arm64`
  and `darwin/arm64` on an x86_64 and an arm64 Linux runner with Rust
  1.99.0, zig 0.15.2 and cargo-zigbuild 0.23.4, compares the two
  builders' SHA-256 sums, and runs each binary on its own platform.
  Locally, two checkouts at different paths gave identical bytes for
  all three targets; `-Wl,-S` keeps object timestamps out of the
  Mach-O UUID. Later, zig left, `windows/amd64` joined, and with
  ENG-19 withdrawn each target is built once, so the second builders
  and the comparison are gone.
- ENG-01's scenario checks `--remap-path-prefix`, `PARINN_COMMIT` and
  both musl targets in the release workflow and the workflow it calls,
  comments left out.
- Reach evidence for the core crate: `crates/parinn/clippy.toml` and
  `crates/parinn/tests/reach.rs`, each with a drift case. The suite
  holds 35 cases.
- Go removed: `cmd/`, `go.mod`, `go.sum`, `tools/`, `.golangci.yml`,
  the Go coverage scripts, the Go CI jobs, dependabot's gomod entries
  and the `go` Codecov flag. ADR-2610101913 drops testify. SRS
  2.32-draft drops the Go clauses of CON-01, ENG-01 and ENG-16.
- Shape: unit 215, integration 21, end-to-end 17 (6 scenarios).

## Next

- The first dispatched release proves the publish job on the new
  build; nothing in it changed but the artifact names.
- ENG-07 pins a dated nightly and runs cargo-fuzz when its first
  target lands.
