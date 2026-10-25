---
n: 7
title: "The core confined in the end-to-end suite (ENG-12)"
status: "✅"
result: true
summary: >-
  The core's end-to-end tests run it with all network denied, and a
  socket probe fails the suite naming the core and the caller; the CI
  wiring has passed on the hosted runner.
---
## Handoff

What landed:

- The testkit's `boundary` module confines a start of the core: a
  user and network namespace of its own (`unshare --map-current-user
  --net`), under `strace -f`, which fails every `socket`,
  `socketpair` and `io_uring_setup` with `EACCES` and records every
  network call. Any recorded call fails the start, naming the
  component and the test that started it (thread and source place).
- `crates/parinn/tests/e2e_parinn.rs` starts the core only through it,
  and names no `Command`, so the core crate's clippy.toml refuses an
  unconfined start there. The gate test
  `the_core_s_tests_start_it_confined` (engineering crate,
  `confinement` target) refuses a product crate's test that starts its
  executable any other way.
- The probe: the testkit's unit tests start their own binary,
  filtered to a probe test that opens a socket, under the
  confinement, and require the failure to name the core and the
  calling test. The scenario runner hosts the same probe behind
  `--parinn-boundary-probe` for the ENG-12 steps.
- A machine that cannot confine fails every start, loudly. Outside
  CI, `PARINN_E2E_UNCONFINED=1` runs the tests unconfined; CI refuses
  it.
- CI: the test, coverage and drift jobs lift the runner's AppArmor
  limit on user namespaces and install strace when it is missing.
- Drift cases: the core started outside the confinement, and the
  core opening a socket (the `e2e_parinn` test then fails naming the
  core and the test).
- The ENG-12 steps are bound in `engineering_boundary.rs`. With the
  outline narrowed locally to its core row, all three pass.

`@ENG-12` stays `@pending`: its room-view component, launcher, peer,
publish and bridge rows need those components (M7 to M9) and their
rows in the testkit's confinement.

CI wiring: the `test`, `coverage` and `drift` jobs pass on `ubuntu-latest`
at main (CI run 668), so the confinement holds on the hosted runner.
