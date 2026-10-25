---
id: ADR-2610251100
title: "Git reading stays own code, no git library"
status: proposed
summary: >-
  The core keeps reading git data with its own dependency-free code in
  crates/repository and extends it for the M7 rows (tree diff, patch-id,
  index, ignore, worktree diff) instead of depending on gix, Grit or any
  other library. The ADR also sets the measured-budget gate, the caching
  rules and the real-git oracle that code must meet, and records why the
  original C code is not a source.
---
# ADR-2610251100: Git reading stays own code, no git library

Proposal. It stays `proposed` until the stakeholder rules on the pull
request. The SRS cites a "git-reader ADR" (S11 in the delivery plan);
this is that record.

## Context

The core needs read-only git data for three rows. LANE-02 is repository
identity, done in M1. LANE-06 and the landed-commit check need a tree
diff and patch-id. REC-20 needs a worktree checkpoint: the checked-out
commit, the branch, and a diff of tracked and non-ignored untracked
files.

The core writes no git object or ref and fetches nothing. Fresh
checkouts belong to the launcher (ADM-13), outside the core.

Several rules bound the choice. I4 and ENG-16 forbid a socket or a
program start in the core. SEC-18 bounds what the core may read. ENG-03
puts a deadline on every blocking call. ENG-18 and ENG-26 require an
accepted ADR for every direct dependency.

## Decision

Extend `crates/repository` with its own code for the remaining rows. Do
not depend on `gix`, `gix-*`, Grit, `git2`, `weavatrix-git` or any other
git crate. Operations the reader does not support (alternates, multi-pack
index, reftable, SHA-256, sparse index, submodule contents) return
`Error::Unsupported`, which the caller counts and audits (I6). This ADR
adds no fallback of its own. What happens next is whatever each row
already allows, and the Open questions below name where the SRS says
nothing.

The code meets four rules.

1. **A measured budget gate.** A benchmark job runs the REC-20 checkpoint
   path on a fixture of 100,000 tracked files and fails the build when it
   exceeds a committed time and peak-RSS ceiling. NFR-01 already sets the
   hook latency targets (`UserPromptSubmit` ≤ 50 ms, `SessionStart` ≤
   150 ms at p95) and NFR-09 caps peak RSS at 50 MiB per hook handler
   invocation. The ceilings are the share of those limits the checkpoint
   path may use. Only that share is open, and the first spike run sets
   it. Scenarios and unit tests prove
   behaviour only; the gate proves cost. It lands with a drift case
   (ENG-27).
2. **Cost rules in the design.** Parse the index once per command. Cache
   ignore rules per directory. Prune tree diffs at equal object ids. Hash
   and diff blobs in a stream, never whole in memory. Take no per-file
   path to a file read that a per-directory read serves. A code path that
   re-reads state per file needs a test that names why.
3. **A real-git oracle.** Every new reader is checked against the output
   of the real `git` binary on generated repositories, in test code only
   (the binary never ships and the core never starts it), and fuzzed. A
   scenario step counts only if it would fail were the behaviour absent.
4. **Clean provenance.** Implement from git's published format
   documentation and from black-box output of the real binary. Do not
   copy or translate git's C source (see Alternatives).

## Alternatives

- **gix (gitoxide), whole crate.** 0.x with frequent breaking releases;
  34 non-optional direct dependencies before features; default features
  pull in process-spawning (`gix-command`, `gix-filter`) and network
  code. I4 could be met by feature selection, but only as "this
  configuration does not call it", with the crates still linked. Past
  advisories in config, path, transport and checkout code would fail the
  advisory check on code ParleyInn never calls.
- **gix sub-crates only** (`gix-index`, `gix-ignore`, `gix-diff`),
  default features off. The fallback if the spike misses the budget or
  the shipped line count passes about 4,000. Adopt it for those three
  modules only, and only when the reach-test delta stays under about 15
  crates and no `std::process` or `std::net` symbol is linked.
- **Grit.** Agent-written, self-described as untested and slow (open
  issues show quadratic `add`, whole-history walks and multi-GB memory on
  large files), and large. It passes 99.3% of git's tests, which proves
  conformance, not cost. Its MIT release of a GPL reimplementation rests
  on its authors' own non-derivative judgement.
- **weavatrix-git.** Dependency-free and read-only like ours, but
  pre-1.0, one maintainer, no ignore matching. Useful as a reference
  to compare our output against, not as a dependency.
- **git2 (libgit2 bindings).** The crate is MIT OR Apache-2.0. It binds
  the libgit2 C library, which is GPL-2.0 with a linking exception, and
  CON-01 names Rust for the product.
- **Own code from git's C source.** Refused. git is GPL-2.0-only. A
  translation of its code, or code written while following it, is
  likely a derivative work, which would put the repository's MIT licence
  and the ENG-18 allow-list (MIT, Apache-2.0, BSD, ISC) in conflict with
  the GPL and oblige source offers with every binary. The C code also
  assumes process-wide state and exits through `die()`, which ENG-03 and
  ENG-04 forbid. This is a project reading, not legal advice; the
  stakeholder rules on it.

## Consequences

- No new direct dependency; `DEPENDENCIES.md` and the reach test are
  unchanged.
- Every line is ours to cover at 99% and to fuzz. The spike's estimate is
  3,000 to 4,000 more shipped lines for tree diff, patch-id, index v2 to
  v4, wildmatch ignore rules and a blob diff.
- The budget gate may show a repository size beyond which the reader is
  too slow. The ceiling is then documented, and REC-20 leaves the rest
  to an ingest marker, as the row says, instead of widening the budget.
- If the spike misses its budget, a later ADR supersedes this one for the
  three modules named above.

## Open questions

- **Unsupported formats under LANE-02.** The row gives a provisional
  identity only to a repository with no commit, or a shallow clone with
  neither. A clone that holds the bound commit in a format the reader
  does not support (SHA-256, reftable, alternates) has no fallback, and
  the row says to use the bound identity. An SRS change should say what
  the node does then.
- **Unsupported reads under LANE-06.** A `not proven` link takes one of
  six closed reasons. None names a read the reader does not support. An
  SRS change should add a reason or say which one applies.
- **SHA-1 collision detection.** The reader verifies object ids with
  plain SHA-1. Decide before this ADR is accepted: add collision
  detection, or state why a planted colliding object is out of scope.
