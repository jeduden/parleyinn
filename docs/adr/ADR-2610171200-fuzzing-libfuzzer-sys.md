---
id: ADR-2610171200
title: "Fuzzing: cargo-fuzz targets on libfuzzer-sys"
status: accepted
scope: dependencies
summary: >-
  The fuzz targets of ENG-07 run under cargo-fuzz on libfuzzer-sys,
  LLVM's libFuzzer compiled in, in a fuzz/ crate that is its own cargo
  workspace and builds on a pinned nightly. Nothing it links reaches a
  shipped executable. Its license adds NCSA, which the allow-list does
  not name; the stakeholder's ruling of 19 October 2026 admits it for
  this one crate only.
---
# ADR-2610171200: Fuzzing on libfuzzer-sys

## Context

ENG-07 requires coverage-guided fuzzing with `cargo-fuzz`, run nightly
from a committed corpus. It names seven parsers, the transcript parser
and the redactor among them. REC-13 requires the transcript parser to
be fuzzed before the MCP server ingests live. That server parses
untrusted lines while it holds run-seat keys. Phase 7 of plan
[2609292012](../../plan/2609292012_m1-record-and-recall/plan.md) adds
the first targets: the transcript parser, the redactor (parked from
phase 4) and the configuration parser (parked from phase 3).

The stakeholder ruled on 14 October 2026 that ENG-18's target counts
only what links into a shipped executable. A fuzz crate still needs an
accepted ADR and an allowed license.

## Decision

| Module          | Purpose                                                                                           | License                      | Maintenance                                                        |
| --------------- | ------------------------------------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------ |
| `libfuzzer-sys` | Links LLVM's libFuzzer into each fuzz target and gives the `fuzz_target!` macro cargo-fuzz drives | (MIT OR Apache-2.0) AND NCSA | Active; the rust-fuzz organisation, which maintains cargo-fuzz too |

The targets live in `fuzz/`, its own cargo workspace with its own
`Cargo.lock`, so libFuzzer, its C++ build and the nightly toolchain
stay out of the product's build, its clippy and its reach test.
`fuzz/rust-toolchain.toml` pins the exact dated nightly (ENG-01).
Each target's seed corpus is committed under `fuzz/corpus/<target>`;
the transcript parser's is one recorded line per shape from
`testdata/claude-code`. The nightly workflow builds every target and
fuzzes each in turn; a crash fails the job and keeps its input.

## Alternatives

- **`afl` (AFL++).** Also coverage-guided, but ENG-07 names
  cargo-fuzz, and AFL++ needs its own instrumenting compiler on the
  runner.
- **`bolero` or `proptest` alone.** Property tests run under
  `cargo test`, but without libFuzzer's coverage feedback a parser's
  deep branches stay unexplored; ENG-08's properties already use the
  test stack.
- **Fuzz targets inside the workspace.** One build, but every
  `cargo test --workspace` would compile libFuzzer's C++, and the
  `fuzz_target!` macro's exported symbol meets the workspace's
  `unsafe_code = "forbid"`.

## Consequences

- The license expression adds NCSA, the University of Illinois/NCSA
  Open Source License libFuzzer ships under: BSD-style, but not a name
  ENG-18's allow-list holds. The stakeholder's ruling of 19 October
  2026 accepts `libfuzzer-sys` with a per-crate license exception for
  NCSA, not a general allow-list entry. [deny.toml](../../deny.toml)
  names the exception for `libfuzzer-sys` alone. No other crate in
  `fuzz/Cargo.lock` carries NCSA: `arbitrary`, `cc`, `shlex` and the
  rest are MIT or Apache-2.0. A crate that does later gets its own
  reviewed line there. ENG-18's allow-list text is unchanged.
- The ruling was given in the session chat. It must be confirmed as a
  consent comment on jeduden/parleyinn#60 before the pull request lands.
- ENG-18's check reads `fuzz/Cargo.toml` beside the product's
  workspace: each direct dependency there needs an accepted ADR and a
  line in `DEPENDENCIES.md`. None of them counts toward the shipped
  executable's limit, since the fuzz crates sit outside `crates/`.
  CI runs cargo-deny and cargo-audit over `fuzz/Cargo.lock` too, and a
  drift case proves the check fails when a fuzz dependency has no ADR.
- No crate linked into a shipped executable changes; the core's
  direct set stays at eight.
