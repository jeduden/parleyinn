---
n: 1
title: "The isolation guard and the agent instruction check (ENG-14)"
status: "✅"
result: false
---
# Phase 1: the isolation guard and the agent instruction check (ENG-14)

Requirement. ENG-14 (P0, Ver T, I): tests run with isolated `HOME`
and `PARINN_HOME`; a guard aborts any test that touches the real
`HOME`, a real ParleyInn home or real Claude Code configuration; no agent
instruction file in the repository directs a destructive command at
non-isolated state.

What exists: the testkit's `isolate` and the scenario runner's
`is_isolated` point the processes a test starts at a temporary home.
Nothing aborts a test that resolves the real home in its own process,
and nothing reads the agent instruction files.

BDD coverage: `@ENG-14` comes off `@pending`. Its steps resolve a path
under the real `HOME` inside a guarded test and see it abort, then
scan every agent instruction file (`CLAUDE.md`, `AGENTS.md`,
`.claude/**`) for a destructive command aimed at the real home.

RED: the scenario fails on the missing guard and the missing scan.
GREEN: the guard every test path resolution goes through, armed for
every test run; the instruction-file scan; drift cases for both.

Gate: `@ENG-14` passes; the drift suite catches a guard that lets the
real home through and a destructive instruction.
