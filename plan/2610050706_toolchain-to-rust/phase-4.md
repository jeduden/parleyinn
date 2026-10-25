---
n: 4
title: "The core executable in Rust; Go removed"
status: "✅"
result: false
---
# Phase 4: the core executable in Rust; Go removed

Requirements. None closes. ENG-01's scenario now checks the Rust
release flags; SEC-01 stays pending, with the build-time evidence
below in place. ENG-19 was withdrawn (SRS 2.35-draft): the release
build is no longer checked for reproducibility.

What lands:

- [x] `parinn version` as the Rust core executable, the `parinn` crate
  under `crates/`: the same command line as the Go stub, and
  `--json` adds the commit the build stamped.
- [x] The release build in one reusable workflow, `build.yml`, which
  CI runs on every change and the release workflow runs before it
  publishes. It needs the pinned Rust and nothing else. One x86_64
  Linux runner builds `linux/amd64` and `linux/arm64` (musl, static,
  linked by rust-lld) and `windows/amd64` (mingw-w64); one arm64 macOS
  runner builds `darwin/arm64`. Each binary runs once on its own
  platform. Each target was first built by two builders and compared
  bit for bit (ENG-19); that check went with ENG-19. A first version
  linked every target with zig through cargo-zigbuild on the two Linux
  runners; it was dropped to keep the build to the Rust toolchain
  alone.
- [x] ENG-01's steps check the Rust release flags:
  `--remap-path-prefix`, the stamped commit, and the musl targets.
- [x] Reach evidence for the core crate replaces the import-closure
  test (SEC-01): its own clippy.toml refuses the `std::net` socket
  types and `std::process::Command`, and a reach test refuses any
  crate linked into it. Each has a drift case.
- [x] `go.mod`, `go.sum`, `.golangci.yml`, `tools/`, `scripts/`'s Go
  coverage gate and `cmd/parinn` go; CON-01, ENG-01 and ENG-16 drop
  their Go clauses (SRS 2.32-draft). testify leaves with them;
  ADR-2610101913 supersedes ADR-2610101442.
- [x] The nightly job waits for cargo-fuzz targets instead of Go's.

Spike S2's Go benchmark stays in its plan folder as archived
evidence, as the plan's context says. No job builds it any more.

Gate: no Go source remains outside that archived spike, and CI runs
the release build the release workflow publishes.
