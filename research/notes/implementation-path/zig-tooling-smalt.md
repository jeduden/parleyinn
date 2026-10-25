# Zig tooling the factory already runs: smalt

Scope: how the stakeholder's Zig project smalt (Zig 0.16.0, about 790
files and 266k lines, built by agents on the same frit and mdsmith
tooling as ParleyInn) gets line coverage, lint and its gates, read at
commit c5903ed on 4 October 2026, and what of it carries over to
ParleyInn's engineering requirements. smalt is private; the links below
need access to it. It feeds the Zig option in the
[options](options.md) and the [comparison](comparison.md).

## Line coverage: an own ptrace tracer

`zig build coverage` runs the tests and enforces the gate in one pass
([docs/coverage.md][cov], code in `tools/coverage/`, about 5.3k lines
of Zig, wiring in `build/coverage.zig`).

- **Why not kcov.** kcov reads debug information through elfutils'
  libdw, and libdw rejects the line table of every Zig 0.16 compile
  unit: Zig writes DWARF 5 file entries with a vendor content type
  (`0x2001`) and without `DW_LNCT_directory_index`. With kcov 42 and 43
  on libdw 0.190 and 0.192, kcov reported zero Zig files, while readelf
  and llvm-dwarfdump read the same tables ([docs/coverage.md][cov]).
- **How it works.**
  1. Build the GPU-free test binaries at Debug, non-PIE.
  2. Read each line table with `std.debug.Dwarf`, the code that wrote
     it, so libdw never enters.
  3. Run each binary under ptrace on Linux x86_64 with a breakpoint at
     every line address; record the hits and remove each breakpoint on
     its first hit.
  4. A line is covered if any traced binary executed it.
- **What is not counted.** Closing-brace lines: a `}` carries a row at
  the function-end address, which an always-inlined function never
  runs, and which is not an instruction start, so a breakpoint there
  corrupts the run. Test code: `*_spec.zig`, embedded `test {}` blocks
  and the test framework.
- **Waivers.** `// coverage:ignore`, `ignore-next`, `ignore-start` and
  `ignore-end`, and `ignore-file`. Waived lines leave the denominator,
  so the gate reads "100% minus documented waivers".
- **Exit codes.** 0 when the threshold is met, 1 when short, 2 when a
  test failed, since coverage of a failing run is not trusted.
- **No vacuous green.** A traced binary that measures zero lines fails
  the run; before that check, a root outside the measured scope printed
  100%.
- **Speed.** About 99% of the time is the Debug tests themselves and
  under 1% the tracer, so each binary's tests are sharded across cores
  (`-Dcoverage-shards`, default 8). A ReleaseSafe build runs 3.8 times
  faster but silently drops 44% of the coverable lines while reporting
  about 98%, so it was rejected ([docs/coverage.md][cov],
  [docs/coverage-speed.md][speed]).
- **Testing the tracer.** A process has one tracer, so the tool cannot
  trace itself: its pure logic runs as unit tests, and an end-to-end
  step runs the installed binary on a throwaway repository and checks
  the exit codes and the report.
- **Policy.** Non-test Zig is held at 100% minus waivers in CI and the
  pre-push hook; a layering rule keeps GPU-facing code out of the
  traced binaries so the logic stays measurable.

## Lint: the zlint fork

- **Tool.** zlint from the stakeholder's fork, pinned at 0.13.0 in
  `mise.toml`; the fork added `no-anonymous-tests` (v0.12.0) and
  lints named files or directories since v0.12.0.
- **Errors.** `cognitive-complexity` at 15 per function and 100 per
  file, set at the current worst and only tightened;
  `no-anonymous-tests`; `homeless-try`; `no-unresolved`
  ([zlint.json][zlint]).
- **Warnings.** `avoid-as`, `unused-decls`, `unsafe-undefined`,
  `no-print`, `no-catch-return`, `suppressed-errors`,
  `must-return-ref`, `empty-file`.
- **Formatting.** `zig fmt --check` over every source tree in CI.

## One gate table

`zig build check` is a gate runner written in Zig. One table in
`tools/check/gates.zig` drives pre-commit, pre-push and CI: zlint,
`zig fmt --check`, the tests, mdsmith and the visual regression tests.
A passing gate prints one line; a passing zlint run alone prints about
3.7k lines (238 KB). The factory also carries `tools/layering`, an
import walk with per-directory floors against vacuous greens, and
agent aids pinned to 0.16: a `zig-reviewer` agent that knows the 0.16
idioms and a session-start hook that pins the Zig version.

## What carries over to ParleyInn

| ParleyInn row                                   | smalt has                                   | Carries over                                                                                                                       | Still to build for ParleyInn                                                                                                                                                                     |
| ----------------------------------------------- | ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| ENG-11 coverage (≥ 90% sensitive, ≥ 80% all)    | line coverage gated at 100% minus waivers   | yes; per-package floors are a scope setting                                                                                        | runs on Linux x86_64 only; the floor is measured there                                                                                                                                           |
| ENG-16 vet, analysers, errcheck                 | zlint errors and warnings, `zig fmt`        | yes; the compiler already refuses an ignored error, which covers `errcheck`                                                        | no security linter (`gosec`); no reachability-aware vulnerability scan (`govulncheck`), so ported code needs an advisory watch                                                                   |
| ENG-16 custom checks (I10 clock, `TrustedText`) | the fork adds rules (`no-anonymous-tests`)  | an owned linter makes ParleyInn's rules cheap: no `std.time` or randomness in projection modules, one constructor for trusted text | the rules themselves                                                                                                                                                                             |
| SEC-01 and ENG-16 reach per component           | `tools/layering` import walk with floors    | the shape                                                                                                                          | `std.net` and `std.process` come through `@import("std")`, so an import walk proves nothing; needs a lint rule on those references per component root, plus a symbol check of the built artifact |
| ENG-27 drift cases                              | zero-line failure, per-directory floors     | the same philosophy: a check must fail when it measures nothing                                                                    | a registered drift case per check                                                                                                                                                                |
| ENG-07 fuzzing                                  | not in the factory                          | —                                                                                                                                  | `zig build --fuzz` or an external fuzzer, nightly, corpus committed                                                                                                                              |
| ENG-09 race detector                            | not in the factory                          | —                                                                                                                                  | thread sanitizer support to verify, or single-writer designs that need none                                                                                                                      |
| ENG-13 mutation testing                         | not in the factory                          | —                                                                                                                                  | a mutation tool for Zig, own or none                                                                                                                                                             |
| ENG-15 benchmark gate                           | `tools/coverage_bench`, `tools/test_timing` | partly                                                                                                                             | a statistical regression gate                                                                                                                                                                    |

## What this means for the options

- The two Zig weaknesses rated as factory-level, the verification
  toolchain and agent fluency on a moving language, are paid for in
  smalt: coverage, lint, a gate table, version pinning and a Zig
  reviewer agent.
- What smalt never needed is what ParleyInn needs most: per-component
  network evidence, fuzzing of hostile input, a race story and a
  vulnerability watch. Each is buildable in the same style, and the
  owned zlint fork is where the first two rules would live.
- The bake-off can start from smalt's gate table and coverage tool
  instead of building a Zig factory from nothing.

[cov]: https://github.com/jeduden/smalt/blob/c5903ed/docs/coverage.md
[speed]: https://github.com/jeduden/smalt/blob/c5903ed/docs/coverage-speed.md
[zlint]: https://github.com/jeduden/smalt/blob/c5903ed/zlint.json
