---
summary: >-
  Build commands, static analysis and supply-chain checks, and how CI,
  the nightly fuzz job and the signed release pipeline work. Testing
  has its own page, docs/testing.md.
---
# Development

Build and release reference, plus the mechanics behind the rules in
[CLAUDE.md](../CLAUDE.md). How ParleyInn is tested, and how the tests are
measured, is in [testing.md](testing.md).

## Build & test commands

See [tooling.md](tooling.md) for setup, tool pins, check commands and
cache behavior. Its configuration examples are imported from the source
files; `mdsmith check .` rejects them when they drift.

## Static analysis

The workspace's lints in [Cargo.toml](../Cargo.toml) forbid `unsafe`,
and refuse `unwrap`, `expect` and `panic!` outside tests
([clippy.toml](../clippy.toml) lets a test use them). CI runs clippy
with every warning denied, and rustfmt. The workspace lints deny the
complexity lints too, under the ceilings each clippy.toml sets, and
`every_crate_holds_the_complexity_limits` refuses a ceiling raised.
[deny.toml](../deny.toml) holds every crate in the tree to ENG-18's
license allow-list, each exception named, and to crates.io as the
only source. cargo-audit refuses a crate with a known vulnerability.
[CodeQL](../.github/workflows/codeql.yml) scans all but the tests.

Crates may only depend on the crates they declare (ENG-02). Each
crate's `Cargo.toml` holds a `parinn` metadata table with three keys:

- `responsibility`: the one thing the library crate does.
- `may-depend-on`: the workspace crates it may depend on, an empty
  list included.
- `may-dev-depend-on`: the workspace crates its tests may depend on.

The architecture check (`engineering::architecture`) reads every
workspace manifest through `cargo metadata`. It fails naming both
crates when a dependency is not declared, and when a declaration names
a crate that is no workspace member. It also fails when a library
crate declares no responsibility or several, or when two crates
declare the same one. The tooling crates fall under the same table.

One rule needs no declaration. A product crate under `crates/` never
depends on a tooling crate under `tooling/` as a normal or build
dependency; only its tests may, as declared. The manifests are the
stakeholder's paths in CODEOWNERS, so changing a direction waits for
the stakeholder. The browser room view client, a TypeScript package,
does not exist yet (M7); its dependency boundary joins this check
when it lands.

The core executable's reach is held two ways (SEC-01, ENG-16). Its
own [clippy.toml](../crates/parinn/clippy.toml) refuses the
`std::net` socket types and `std::process::Command` in its code, and
[its reach test](../crates/parinn/tests/reach.rs) refuses any crate
linked into it, so neither can arrive through a dependency. Both are
the build-time half of SEC-01. With unsafe code forbidden, the core's
own code cannot add code that runs as the process starts either. The
`platform` crate alone only denies it, its tests under Miri (ENG-09).

The gate tests in the `engineering` crate's `static_analysis` target
hold this in place. CI gates on clippy, rustfmt, cargo-deny,
cargo-audit and `cargo test --workspace`. Each runs in a job the `CI`
job needs, and none may fail. Every workspace crate inherits the
workspace's lints (`[lints] workspace = true`), or forbids unsafe code
in its own crate roots. The core's bans and reach test are in place.
`cargo run --locked -p engineering --bin license-check` holds every crate
a release links to a license text that fits its declared license
(ENG-30) and each target's `THIRD_PARTY_LICENSES` to equal its
regeneration (ENG-31); the supply-chain job runs it on every pull request.
It reads only the sources cargo has fetched, so run `cargo fetch --locked`
first; `--write` rewrites the committed files after a dependency changes.
Every product crate's clippy.toml refuses the clocks and the random
order of the hash collections. These checks are ENG-16's core row,
each with its drift case. The ENG-16 scenario stays pending until the
browser room view client (M7), the other components (M7 to M9) and
TrustedText (plan 2609292013) land.

The run-time half is ENG-12's core row. Every end-to-end test of the
core starts it through the testkit's `boundary` confinement: in a
user and network namespace of its own (`unshare --map-current-user
--net`), under `strace -f`, which fails every `socket`, `socketpair`
and `io_uring_setup` with `EACCES` and records every network call.
Any recorded call fails the test, naming the core and the test that
started it. The gate test `the_core_s_tests_start_it_confined` refuses
a test that starts the core any other way; a step needing a terminal gets one
from `script`, outside the trace. It needs Linux, `unshare` and `script` from
util-linux with unprivileged user namespaces, and `strace`; a machine without
them fails the end-to-end tests with what is missing.
Outside CI, `PARINN_E2E_UNCONFINED=1` runs them unconfined instead, on
a machine that cannot confine; CI refuses it.

The same clippy.toml holds the rest of ENG-03 in the product crates.
It bans the clocks and the hash collections, so derived artifacts
read no wall clock and no random order (I10). It also bans each
blocking call that waits with no deadline, and names the form that
takes one: `recv` for `recv_timeout`, `Child::wait` for polling
`try_wait`, `File::lock` for `try_lock`, and so on. A thread join,
a read of standard input and a wait on a condition variable or a
barrier are on the list too. A call site that has to block says so
with an `expect` and a reason, which review reads.

Local file reads and writes are not on that list. The disk bounds
them, and the hook's time budget (I9) covers them. Nor are
`std::thread::sleep`, which carries its own bound, and the in-process
`Mutex` and `RwLock`. The engineering gate test
`the_product_crates_keep_their_bans` requires every product crate's
clippy.toml to carry every ban. The ENG-03 scenario reads the same
bans, and stays `@pending` until the stakeholder rules on what the
list exempts.

## CI, nightly and release

Every workflow lives in `.github/workflows`. Every action is fixed
to a commit SHA with its version in a trailing comment. Dependabot
proposes bumps under the policy in
[dependabot.yml](../.github/dependabot.yml).

[ci.yml](../.github/workflows/ci.yml) runs on every push and pull request to
`main`. It runs every test and scenario, with `HOME` pointed at an empty
decoy that must stay empty (ENG-14), the core's end-to-end tests confined
(ENG-12), clippy and rustfmt, cargo-deny and cargo-audit, the drift suite,
the benchmark gate, and the release build of every target (NFR-10, CON-02).
It also runs `mdsmith check .`, [taplo](../.taplo.toml), and zizmor over the
workflows and local actions. A final job named `CI` passes only when
every other job passed.
It is the one check the branch and release-tag rulesets require, because
GitHub names a required check after a job, never after the workflow.

The `benchmark` job (ENG-15) is described in
[testing.md](testing.md#benchmarks-and-the-regression-gate).

[nightly.yml](../.github/workflows/nightly.yml) is where
cargo-fuzz runs each target under `fuzz/` (ENG-07) for 15 minutes,
from its corpus under `fuzz/corpus`, on the nightly
`fuzz/rust-toolchain.toml` pins. The crash-consistency test (ENG-06),
the soak (ENG-09) and the live contract test (ENG-17) join it later.

[build.yml](../.github/workflows/build.yml) is the release build.
CI runs it on every change, and the release workflow runs it too.
The workflow defines the targets, builder platforms and exact system
dependency pins. Linux binaries link statically against musl with the
rust-lld Rust ships (CON-02).
`--remap-path-prefix` keeps the builder's paths out. `PARINN_VERSION`
and `PARINN_COMMIT` stamp the version and the commit (ENG-01). Each
binary then runs once on its own platform and must report what it
was stamped with. Each target is built once: the release build is not
checked for reproducibility.

[review.yml](../.github/workflows/review.yml) reviews each pull
request from a branch of this repository once its CI passes (ENG-21,
ENG-28). It skips drafts, so a pull request is reviewed when it is
marked ready, which reruns CI. The agent runs on the stakeholder's
Claude subscription, not an API key billed per model token. It triggers on
`workflow_run`, so GitHub runs it as `main` defines it, never as the
pull request does. It also triggers on `issue_comment`, which GitHub
runs as `main` defines it too: a code owner's comment on a pull
request whose CI passed reviews it again. Consent to a stakeholder
path is such a comment, so the review that reads it follows at once.
The review job first runs main's `review-bundle`
over main's tree and the pull request's: it writes the review bundle
the agent reads, sets the review's class and makes the deterministic
findings
(see Review below). The agent then runs with read-only tools on the
pull request's tree, following
[the review skill](../.claude/skills/review/SKILL.md). The agent only
writes a review outcome. The post job alone has the reviewer app's
key. It runs the `review-gate` crate's executable, which approves only an
approving review outcome with no blocking finding, its own or a
deterministic one, on the reviewed head, with `CI` green there;
otherwise it requests changes.
[ADR-2609301941](adr/ADR-2609301941-agent-review.md) records why.

Main's protection is committed as a ruleset,
[main.json](../.github/rulesets/main.json), in GitHub's export format
(ENG-21). It allows no direct push and no bypass, and requires signed
commits, `CI`, one approval from a reviewer other than the author, and
a code owner's approval where [CODEOWNERS](../.github/CODEOWNERS) names
one. The ENG-21 gate tests read it, and CODEOWNERS: the stakeholder
owns the requirement text and every path that enforces it, and nothing
else is owned but the decision records, the supply-chain policy and the
security-sensitive crates. Two tag rulesets sit beside it.
`releases-semver` keeps each `v*.*.*` tag fixed on a signed commit that
passed `CI`. `no-other-tags-allowed` refuses every other tag. The
stakeholder applies the rulesets on GitHub. The nightly `protection`
job runs `protection-check` on each file under `.github/rulesets`. It
fails when the live ruleset of that name is missing or weaker. The
workflow's token cannot see a ruleset's bypass actors. The
security-sensitive crates are owned by the designated security
reviewer, so a change to one needs that reviewer's approval as a code
owner. The stakeholder holds the role. One approval count holds for the
whole branch, so a second approval on those crates alone would need a
second reviewer account and a check that counts reviews; ENG-21 does
not ask for it until one is named.

[release.yml](../.github/workflows/release.yml) runs from the Actions
"Run workflow" button on `main`, with a version like `v0.1.0`. A pushed
tag is deliberately not the trigger: a failed build would leave a
public tag pointing at nothing. The workflow takes these steps:

1. Validate the version, refuse any ref but `main`, run the suite.
2. Run the release build above, with the version stamped, and stop
   when any binary fails to report it; then attest its provenance.
3. Generate an SPDX SBOM, attest it to every binary, and sign
   `checksums.txt` keylessly with cosign (ENG-20).
4. Create the tag and publish the release, only now that everything
   above succeeded.
5. Verify the release as published: a last `verify` job downloads the
   release's own assets and checks the checksum bundle with
   `cosign verify-blob`, then each binary with `gh attestation verify`,
   for its provenance and its SBOM attestation (ENG-20).

Only the release calls
[release-build.yml](../.github/workflows/release-build.yml). It calls
`build.yml` and grants it nothing. Its `attest` job alone holds
`id-token` and `attestations` write; it runs no repository code and
attests the binaries. CI's call of `build.yml` holds neither. A
certificate names its workflow at the dispatched ref, and any branch
can be that ref. So `verify` pins each signer's exact `--cert-identity`
at `refs/heads/main`, with `--source-ref` and
`--deny-self-hosted-runners`. The tag cannot be that ref: the release
creates it last. The ENG-20 gate tests read these workflows. Its
scenario stays `@pending` until the stakeholder rules, as a checkout
holds no release. [SECURITY.md](../SECURITY.md) has the commands.

## Review

The SRS is enforced at the cheapest layer that can decide each rule.
About nine in ten requirements name a test as their verification
method, so their scenario, once off `@pending`, enforces them on every
pull request without an agent.

What CI decides, on every pull request:

- the requirement–scenario matrix, Appendix B and C and the persona
  agents (the gate tests), and every drift case (ENG-27);
- dependencies and their ADRs (ENG-18) and the decision records
  (ENG-26);
- rustfmt and clippy, with each product crate's bans on sockets and
  program starts (I4), on clocks and hash order (I10) and on waits with
  no deadline (ENG-03), the complexity limits, and the reach test;
- the crates' declared dependency directions and responsibilities
  (ENG-02);
- no mutable global state in a product crate (ENG-03);
- coverage per test layer (ENG-11), and every live scenario;
- the benchmark gate (ENG-15).

What `review-bundle` decides, comparing the pull request with main:

- a scenario live on main deleted or put back on `@pending`: blocking;
- an invariant, or a row ENG-29 marks for I2 review, changed without
  both an accepted ADR in the same change that names ENG-29 and each
  guarded id, and a comment of the stakeholder on the pull request:
  blocking;
- a clippy ban lifted at a call site, or a diff header it cannot read:
  a nit, and an unread header makes the review deep.

The review class sets the agent's model, turn budget and tools. A change
holding only Markdown outside the stakeholder's paths, the domain model,
plans and scenarios is light. Any path CODEOWNERS assigns makes it
deep, and only a deep review may consult the read-only reviewer
subagents. Anything else, an empty diff included, is standard.
The review bundle quotes only the touched rows, their scenarios and
the invariants, a few thousand model tokens where the SRS holds
hundreds of thousands. A re-review starts from the diff since the reviewer app's
last review, found by the app's own login. The pull request's agent
instructions move out of the agent's directory, so none loads; the
agent reads them as data. The posted review names the gate as its
author, states the agent's verdict, and keeps the agent's findings,
the deterministic ones and failed checks under their own headings.

What is left to the agent is judgement: whether the code and the step
bindings do what each touched row says, new paths to the model (I2),
new reach (I4), content of another principal (I8), errors and seams
no lint sees, and the domain model's terms. Persona reviews belong to
plans and SRS changes (the `persona-review` skill), not to each pull
request. The nightly job takes what is too slow for a pull request:
fuzzing, crash consistency, soak and slow benchmarks, as plans land.

## The plan catalog and its merge driver

Work is tracked as plans under `plan/`, indexed by
[PLAN.md](../PLAN.md), in frit's format: see
[plan/proto.md](../plan/proto.md) and the `plan-*` skills under
`.claude/skills`. Run `mdsmith merge-driver install` once per clone.
`PLAN.md` is a generated catalog, so two branches adding a plan
conflict on it by construction; the driver regenerates it during a
merge instead of leaving conflict markers.
