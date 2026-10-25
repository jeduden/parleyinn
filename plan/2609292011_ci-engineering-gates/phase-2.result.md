---
n: 2
title: "Declared dependency directions between crates (ENG-02)"
status: "✅"
result: true
summary: >-
  Each crate's manifest declares its one responsibility and the crates it may depend on; a gate test fails naming both crates on a forbidden direction, and the ENG-02 scenario is off pending.
---
## Handoff

What landed:

- Every workspace manifest carries `[package.metadata.parinn]` with
  `responsibility`, `may-depend-on` and `may-dev-depend-on`, beside
  what it governs. The tooling crates are covered too.
- A product crate under `crates/` never depends on a `tooling/` crate
  as a normal or build dependency, declared or not; a declared dev
  dependency on tooling is allowed.
- `tooling/engineering/src/architecture.rs` reads `cargo metadata
  --no-deps` and names both crates on an undeclared dependency, and the
  crate on a missing, blank, repeated or shared responsibility. The gate
  test is `the_workspace_keeps_its_declared_directions`; four drift
  cases guard it.
- `@ENG-02` is off `@pending`, bound in `engineering_architecture.rs`.

What remains: the browser room view client's TypeScript package joins
the check when it lands in M7.
