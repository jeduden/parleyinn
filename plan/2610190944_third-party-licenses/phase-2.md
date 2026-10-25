---
n: 2
title: "A generated license file for every release target, kept in step by CI"
status: "✅"
result: false
---
# Phase 2: a generated license file for every release target, kept in step by CI

Requirements. Part of ENG-31: the generator, the committed files and the
CI gate. The `@ENG-31` scenario comes off `@pending` in phase 3, with the
embedding it also checks.

BDD coverage: the scenario reads the repository checkout and runs
`cargo metadata`; its steps are in
[the ENG-31 steps](../../tooling/scenario/tests/bdd/engineering_third_party.rs).

Done, red then green, in `tooling/engineering`:

- `third_party` renders one file per release target from
  `cargo metadata --filter-platform`, the crates' fetched sources and a
  fixed list of what a target compiles in without being a crate: SQLite,
  the Rust standard library and, on the musl targets, the C library. It
  gives each license text once, with the code that uses it and the
  copyright lines it states, in a sorted order with no timestamp, host
  path or randomness. The two musl targets link the same crates and
  carry identical files.
- `license-check` regenerates every target's file and fails when a
  committed file differs, is missing, or has no target. `--write`
  rewrites the committed files and `--out DIR` writes the regeneration
  elsewhere, for the release build.
- The linked set leaves out proc-macro crates and what only they reach.
- `licenses/<target-triple>/THIRD_PARTY_LICENSES` is committed for each
  of the four release targets.
- CI's supply-chain job runs `license-check` on every pull request, so a
  change to `Cargo.lock` regenerates the files or fails.

Drift cases (ENG-27): a committed file out of step with the lockfile, and
a release target with no committed file.

Open for the stakeholder, before a release:

- The fixed list of non-crate code is short. No lawyer has reviewed it.
- The Windows file carries the notices of mingw-w64's runtime, the GNU
  General Public License 3.0 and the GCC Runtime Library Exception 3.1,
  each as published, with its own copyright lines. They come from the
  projects' master branches. Check them against the releases the
  Windows build links: mingw-w64 from Ubuntu's pinned package, and
  libgcc from its pinned compiler.
