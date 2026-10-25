---
id: ADR-2610190950
title: "Third-party license texts embedded from an in-repository tool"
status: proposed
summary: >-
  A small tool under tooling/ reads Cargo.lock and cargo metadata to
  check every crate's license text and to regenerate one committed
  THIRD_PARTY_LICENSES file per release target, which that target's
  executable embeds and prints with parinn licenses. It adds no
  dependency to the core or the repository.
---
# ADR-2610190950: Third-party license texts embedded from an in-repository tool

Proposal. This record proposes the in-repository tool, a file per
release target, and the command name `parinn licenses`. It stays
`proposed` until the stakeholder rules on the pull request.

## Context

ENG-30 asks CI to stop a pull request that brings in a crate with no
license text. ENG-31 asks every shipped executable to carry the license
text and copyright lines of every crate it links, so the texts travel
with the binary wherever it goes. The aim is that this works with no
manual step when a dependency arrives.

The ENG-18 allow-list licenses (MIT, BSD, ISC, Apache-2.0) require the
text and copyright line to travel with a binary. SQLite
(public domain), the Rust standard library (MIT or Apache-2.0) and, on
the musl targets, the C library (MIT) are compiled in without being
listed as crates, so no crate-based tool sees them.

## Decision

Write the generator and the check as one Rust tool under `tooling/`.
It reads only `Cargo.lock`, `cargo metadata` (with `--filter-platform`
per release target, following normal-dependency edges from the binaries
under `crates/`) and the crate sources `cargo` has fetched. A short
fixed list in the tool names the code compiled in without being a
crate.

**One committed file per release target (option b).** The tool writes
`licenses/<target-triple>/THIRD_PARTY_LICENSES` for each release target,
a closed list read from the build workflow. Each file covers exactly the
crates linked for that target, from `cargo metadata --filter-platform`,
plus that target's non-crate code. A target's build embeds its own file
with `include_str!` through a `cfg`-selected path, and `parinn licenses`
prints it. CI regenerates every target's file and fails the pull
request when any committed file differs, so a developer who adds a
dependency regenerates them in the same change. A crate with no license
file fails the check; the repository keeps no substitute text, and the
crate is removed or replaced.

Guards against a wrong selection. A test per target proves the build
embeds that target's own file: it compares the embedded text with the
target's committed file, and the committed files of any two targets whose
linked crates differ differ in bytes, so the comparison catches a swap
between them in both directions, even when one target links a strict subset
of the other's crates. Targets that link the same crates, such as the two
musl targets, carry identical files.
A target in the workflow with no file, or a file for no target, fails
the build. The release build runs each built executable's
`parinn licenses` on its own platform and compares it with that target's
regenerated file. The file is inside the executable, so the existing
signature and attestations (ENG-20) cover it and no separate asset
exists.

Why a file per target, and not one union (option a):

| Question         | (a) One file, union                          | (b) A file per target through `cfg`                           |
| ---------------- | -------------------------------------------- | ------------------------------------------------------------- |
| Exactness        | Windows-only crates appear in the Linux text | Each binary lists exactly its own crates                      |
| Compliance       | Over-discloses, never under                  | Exact                                                         |
| Files and checks | One file, one regeneration check             | A file per target, one regeneration each, a closed list       |
| Wrong selection  | Cannot happen                                | Caught by the per-target embedding test and the release check |
| Size cost        | The union's text in every binary             | Smaller on the targets with fewer crates                      |

This record proposes (b) for exactness. Its one extra failure mode, the
`cfg` picking the wrong file, is mitigated by the guards above.

**Constraints.** The core gains no dependency: the text is a `&str`
compiled in, so ENG-18's list and its ten-dependency target are
unchanged. It gains no network access: `parinn licenses` only prints,
which I4 and SEC-01 allow, and the crate's clippy and reach test still
pass. The tool uses `serde_json`, which ADR-2610142221 accepts, and
adds no module. Each binary grows by the size of its target's text, a
few hundred kilobytes at most for the current tree; no requirement
bounds binary size. `parinn licenses` is a new node-wide utility name.
The domain model's list names it.

## Alternatives

How the three candidates compare, for the tool that writes the file:

| Question                           | In-repository tool                                      | `cargo-about`                                            | `cargo-bundle-licenses`                              |
| ---------------------------------- | ------------------------------------------------------- | -------------------------------------------------------- | ---------------------------------------------------- |
| Maintenance burden                 | Ours: a few hundred lines to keep with `cargo metadata` | Upstream's; we track its config and releases             | Upstream's, a smaller project with fewer releases    |
| License-file discovery             | Our rules; wrong guesses show in the tests we write     | Mature, with per-crate clarifications in a config        | Takes the first license-like file; no clarifications |
| Non-crate code (SQLite, std, musl) | A fixed list in the tool                                | A clarification entry or a template addition, by hand    | None; appended by hand                               |
| Reproducibility, byte stability    | We fix the order and format; tested twice per run       | Template-driven; stable for one pinned version           | Output order and metadata shift between versions     |
| Supply-chain trust                 | Our code, under the ENG-21 review                       | A pinned external binary fetched in CI                   | A pinned external binary fetched in CI               |
| CI time                            | Builds with the workspace, seconds                      | Install or cache the binary, then run                    | Install or cache the binary, then run                |
| Review surface                     | The tool's source, in stakeholder paths                 | Its config and template, plus a trust decision           | Its output only; less control                        |
| Fit with ENG-18 and ENG-27         | No new module; drift cases written against our own code | A tool outside the manifests; its own ADR and drift case | Same as `cargo-about`                                |

**Recommendation.** The in-repository tool: it is the
only candidate that knows the non-crate code, reproduces byte for
byte by construction and adds no external binary to the release
chain. `cargo-about` would need its own accepted ADR, a drift case, a
hand-kept entry for each non-crate library, and a change to ENG-30 and
ENG-31, which name a tool in the repository.

- **`cargo-deny`'s license check alone.** It already runs in CI and
  checks the declared SPDX expression, but it never ships a text and
  does not compare a license file with that expression, so it cannot satisfy
  ENG-30 or ENG-31. The in-repository tool keeps a short list of the
  canonical texts of the licenses on the allow-list and of those a
  `deny.toml` exception grants, with the holder, year and name a template
  fills in left free, and does its own comparison; it does not reach
  `cargo-deny`'s matcher.
- **The SBOM (ENG-20).** SPDX lists license identifiers, not texts or
  copyright lines.
- **A separate release asset.** Rejected here: a file
  beside the binary can be lost when the binary is copied.

## Consequences

- A crate with no license file blocks its pull request, and the crate
  is removed or replaced; the repository keeps no text for it.
- A pull request that changes `Cargo.lock` regenerates every target's
  `THIRD_PARTY_LICENSES` in the same change, or CI fails it.
- Adding a release target adds its file; the build fails until it has one.
- The non-crate entries are a short list in the tool, reviewed on the
  stakeholder's paths; a new compiled-in library needs a line there.
- Every executable is a little larger and answers `parinn licenses`.
- The requirements name a tool in the repository (ENG-30, ENG-31), so
  choosing `cargo-about` instead would change those rows.
