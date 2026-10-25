---
n: 1
title: "The isolation guard and the agent instruction check (ENG-14)"
status: "✅"
result: true
summary: >-
  `@ENG-14` passes: a guard aborts any test that resolves a path outside
  the temporary directory, only the guard may read the home, CI runs the
  suite against a decoy `HOME`, and no agent instruction file directs a
  destructive command at the real home. Seven drift cases hold it.
---
## Handoff

### Done

- The isolation guard is `testkit::guard`. A test resolves the home, a
  ParleyInn home, Claude Code's configuration directory or its state file
  through it, and it aborts the process unless the answer lies strictly
  under the system's temporary directory (`..` and symlinks counted).
  The testkit is test-only, so linking it arms the guard: no test opts
  in or out.
- `@ENG-14` is off `@pending`. Its steps start the scenario runner as a
  probe test that resolves each place, once with a stand-in real home,
  which must abort before it prints a path, and once with an isolated
  one, which must resolve. The scenario then scans the sources, and the
  agent instruction files.
- Only the guard reads the home. The gate test
  `only_the_isolation_guard_reads_the_home` fails when a Rust source
  under `crates/` or `tooling/` reads `HOME`, `PARINN_HOME`,
  `CLAUDE_CONFIG_DIR`, `USERPROFILE` or the standard library's home
  lookup, but the guard and the scenario runner's own start. That is
  what makes the guard cover every test.
- CI's `test` job runs the suite with `HOME` pointed at an empty decoy
  and fails when anything appears there: it finds a test that went
  around the guard, such as a program started without `isolate`. The
  gate test `the_ci_test_job_runs_with_a_decoy_home` keeps the step.
- The scan of `CLAUDE.md`, `AGENTS.md` and everything under `.claude/`
  flags a statement that names a way to delete or overwrite and the
  real home, unless prose forbids it or `HOME` points at a temporary
  directory first. This repository had no violation: its only
  statement about the real home, in `CLAUDE.md`, forbids the act.
- Seven drift cases: the guard letting the real home through, the guard
  refusing every home, a helper reading the home around the guard, CI
  dropping the decoy, `CLAUDE.md` losing its "Never", a skill with
  `rm -rf ~/.parinn`, and a settings hook clearing `~/.claude`.
- Shape: unit 462, integration 41, end-to-end 38 (22 scenarios).

### Next

- A product crate that resolves a home in shipped code (M1) cannot call
  the testkit. It joins `RESOLVERS` in
  `tooling/engineering/src/isolation_guard.rs` with its own guard, which
  refuses a home outside the temporary directory when a test marks the
  run. The stakeholder approves that change (`tooling/` is a code-owner
  path).
- The decoy step is unrun. This lane's sandbox refuses a command that
  sets `HOME`, so only the reasoning was checked: a full `cargo test
  --workspace` under the real `HOME` wrote nothing there outside
  `~/.cargo` and `~/.rustup`, which the step keeps through `CARGO_HOME`
  and `RUSTUP_HOME`. The first CI run proves the step.
