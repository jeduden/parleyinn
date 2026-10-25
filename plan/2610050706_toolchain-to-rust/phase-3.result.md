---
n: 3
title: "Coverage per test layer"
status: "✅"
result: true
summary: >-
  The coverage executable measures each test layer, holds 99% unit
  and 100% overall per crate, and fails an inverted pyramid. Locally
  every crate is at 100% from its unit tests; the shape is unit 199,
  integration 18, end-to-end 14. Four test-engineer agents and two
  skills review and shape the pyramid.
---
# Phase 3 handoff

## Done

- `cargo run -p coverage` runs cargo-llvm-cov once per test layer,
  writes `target/coverage/<layer>.lcov` and `all.lcov`, prints the
  table and the shape, and appends both to the CI job summary.
- Floors: `unit = 99`, `all = 100`, per crate; `src/main.rs` is an
  entry point, held only by the overall floor.
- Shape: unit 199, integration 18, end-to-end 14 (6 bound scenarios).
  Integration tests for `engineering` and `review-gate` were added to
  right it, and a unit test for `testkit::repo_root`.
- A nested run exits 2 with "refusing to recurse"; the executable
  marks every cargo command it starts.
- Two drift cases for the gates that were not yet guarded:
  `specification_parses` and `decision_records_parse`. The suite holds
  32 cases.
- Agents `test-engineer`, `test-engineer-unit`,
  `test-engineer-integration` and `test-engineer-end-to-end`; skills
  `test-review` and `test-shape`.
- `docs/testing.md` split from `docs/development.md`.

## Next

- Phase 4: the product's Rust executable, the release pipeline on
  cargo, and the Go `parinn version` stub removed.
