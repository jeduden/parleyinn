---
summary: >-
  How ParleyInn is tested: the test pyramid of unit, integration and
  end-to-end tests, line coverage and test counts per test layer with
  their floors, the executable requirement matrix (one Gherkin scenario
  per SRS id), drift injection, and the test-engineer agents that
  monitor and shape the pyramid.
---
# Testing

Every behaviour is proven at the lowest test layer that can prove it.
[CLAUDE.md](../CLAUDE.md) states the rules; this page is how they are
kept. Build and CI commands are in [development.md](development.md).

## The test pyramid

- **Unit tests** prove every function and module directly. They live
  in the `tests.rs` beside the module they test, use fakes for I/O,
  and run in milliseconds. They are most of the tests.
- **Integration tests** guard behaviour at a crate's public API: the
  gates against the real repository, and each real process or file
  boundary. They live in a crate's `tests/`.
- **End-to-end tests** guard requirements through the entry points:
  the `bdd` scenario runner, and every `tests/e2e*` target, which runs
  a built executable as a process. Each start of the core runs it
  confined to its boundary, all network denied, and a socket it opens
  fails the test, naming the core and the test (ENG-12's core row; see
  [development.md](development.md#static-analysis)). ENG-17's live
  test joins this test layer when it lands.

## Coverage and shape

`cargo run -p coverage` measures the pyramid in one run. It runs
[cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) once per
test layer, over one build, and prints line coverage per crate for
each test layer and for all of them. It counts the tests in each test
layer too, scenarios included, and writes each layer's lcov to
`target/coverage/`. Install the tool first, with `cargo install
--locked cargo-llvm-cov@0.9.1`.

The policy lives in [Cargo.toml](../Cargo.toml) under
`[workspace.metadata.coverage]`, and the run fails when a crate falls
below it:

- **Unit: 99% of each crate's lines**, from unit tests alone. Unit
  tests prove every function and module directly.
- **Entry points** (`src/main.rs`) are left out of the unit and
  integration measures: an end-to-end test runs the executable and
  proves its `main`.
- **Integration and end-to-end** coverage is measured and reported,
  with no floor: those tests guard behaviour, not lines.
- **Every test layer together: 100%** of each crate's lines.
- **The shape:** more unit tests than integration tests, and more of
  those than end-to-end tests. An inverted pyramid fails the run.

A crate may name its own floors under
`[workspace.metadata.coverage.crates.<name>.floors]`, with the reason
beside them. ENG-11's floors for the product crates (90% for the
security-sensitive ones, 80% overall) join that way as they land. A
misspelled key or crate name in the policy fails the run, so a typo
cannot switch a floor off.

The measure counts shipped code only. Unit-test files (`tests.rs`)
and every `tests/` directory, step bindings included, are left out.
That is why a binding stays thin: the logic it calls lives in a library
crate, where the unit tests reach it. Every cargo the tool starts
carries `PARINN_COVERAGE_MEASURING`, and a run that finds it set refuses
to start, so a test cannot set off a measurement inside a measurement.

CI's `coverage` job writes the table and the test counts into its
summary. It uploads one Codecov flag per test layer. Codecov's
thresholds in [codecov.yml](../codecov.yml) are a soft ratchet on top.

## The executable requirement matrix

The SRS under [docs/srs](srs/index.md) is executable. Every
requirement row, from `REC-01` to `ENG-29`, and every
assumption `ASM-` row, has exactly one Gherkin scenario under
`features/`, one file per SRS section. The scenario carries the row's
id, its priority and its traced invariants as tags:

```gherkin
@REC-03 @P0 @I1 @I10 @pending
Scenario: re-ingesting a transcript source creates no duplicate events
```

The `scenario` crate's gate test, `specification_and_features_agree`,
keeps the two sides in step. A requirement with no scenario fails it.
So does a scenario naming no requirement, an id tagged twice, or a
priority or invariant tag that disagrees with the row. The `srs`
crate's gates check that Appendix B's invariant coverage and
requirement counts match the tables they summarise. They also check
that Appendix C gives every requirement a persona, and that §2.5's
personas match the agents in `.claude/agents`.

A scenario still tagged `@pending` is declared but not written. It
is skipped and counted, never counted as a pass. The `bdd` target in
[tooling/scenario](../tooling/scenario/tests/bdd/main.rs) runs the rest
through cucumber-rs, and a step whose text matches no binding fails
instead of being skipped. It prints `scenarios: N bound, M pending`
first, and CI's test job writes that line into its summary.

The runner reads features with the same gherkin parser as the gate.
That parser is stricter than godog's in two places. A description line
that opens with a Gherkin keyword, such as "Scenarios", reads as that
keyword. And `<name>` in a Scenario Outline must name an Examples
column, so a literal placeholder is written another way, such as
`{id}`.

## Proving the checks: drift injection

A check that keeps the SRS, the scenarios and the records in step can
be weakened as easily as any other code, and a weakened check turns
CI greener, not redder. ENG-27 closes that hole. Every such check has
a registered drift case in
[tooling/drift/src/cases.rs](../tooling/drift/src/cases.rs). A case is
one injection into a copy of the repository, the check to run and the
message it must fail with.

- `cargo test -p drift --test suite -- --ignored` — the drift suite.
  It proves the unedited copy passes every check, then injects each
  case into a fresh copy and requires its check to fail. It needs
  mdsmith on PATH; CI's `drift` job runs it.
- A case the checks miss today is marked `known_gap`. The suite fails
  the day a check starts catching it, so the mark goes with the fix.
- The ENG-27 scenario runs in plain `cargo test --workspace`. It fails
  when a case's injection no longer finds its target, or when a
  non-pending scenario that opens on "the repository checkout" has no
  case guarding its id.

A new check lands with its drift case in the same change. A scenario
put back on `@pending` skips, so its drift cases go uncaught and the
suite fails: re-pending shows up here too.

## Benchmarks and the regression gate

ENG-15 asks for a benchmark suite covering NFR-01..05 and a CI gate
that fails on a statistically significant regression greater than
10%. The `benchmark` tooling crate holds both.

- **The suite** registers each benchmark against the NFR id it
  measures toward, in
  [tooling/benchmark/src/suite.rs](../tooling/benchmark/src/suite.rs).
  A benchmark measures real code: one of an NFR's target covers it,
  and one of a cost the target's subject pays is only a floor. Each
  NFR id ENG-15 names without a benchmark of real code is declared
  awaiting one, with what it waits on.
- **Today** the suite times the core executable's start-up, a floor
  every hook handler pays and not a hook handler. NFR-01..05 await the
  hook handlers, the MCP tools, ingestion and the store, so the ENG-15
  scenario stays `@pending`. Its steps are bound; its first fails
  naming each NFR id and what it awaits.
- **The gate** fails a benchmark when the head's median is more than
  10% above the base's and a one-sided Mann–Whitney U test finds the
  head slower at α = 0.01. Both conditions must hold: significance
  alone would fail a precise 1% slowdown, the threshold alone medians
  a noisy runner pushed apart by chance. The test works on ranks, so
  a spike counts as one rank, and is closed form, so the decision is a
  pure function of the samples. A benchmark the head stopped sampling
  fails unless the suite retired it, and so does one with fewer than
  ten samples a side.
- **The checks:** two gate tests in `engineering`'s `benchmark_gate`
  target hold the suite to every NFR id ENG-15 names and CI to the
  job, each with its drift cases.

- **Accepted regressions:** a change may make ParleyInn slower on
  purpose, with the stakeholder's acceptance. It then records the
  benchmark, a ceiling in percent and the reason in
  [tooling/benchmark/accepted.txt](../tooling/benchmark/accepted.txt).
  The gate passes that benchmark as accepted while the head's median
  stays within the ceiling. It does so only while no base line names
  that benchmark, so an acceptance lasts one change. A drift case proves
  the record's check fails on a malformed line.

`cargo run -p benchmark -- run --core target/release/parinn` samples
the suite; `cargo run -p benchmark -- gate --base FILE --head FILE`
decides from two samples files. CI's `benchmark` job runs both.

The `benchmark` job (ENG-15) compares the head with its first
parent: the `main` a pull request's merge commit sits on, or `main`
before a push. It builds both sides' benchmark suites and cores in
release mode on one runner and samples them interleaved, thirty rounds
of ten samples a side, alternating which side goes first, so a drift
in the runner's speed lands on both. The head's gate then writes its
report into the job summary and fails naming each benchmark that
regressed more than 10% with significance. The suite samples in a
second or two, so it gates every pull request; a benchmark too slow
for that, such as one on a store of 10M events, would run nightly
instead. Until the suite reaches `main`, the base has none, and the
job warns and runs the head's alone.

## Adding or writing a scenario

A new requirement lands in the SRS table and in its section's feature
file in the same change, tagged `@pending`. To write a scenario, drop
`@pending`, make its Given/When/Then concrete, and bind the steps. A
section's bindings live in `tooling/scenario/tests/bdd/<section>.rs`,
the way [engineering.rs](../tooling/scenario/tests/bdd/engineering.rs)
does. Bindings register themselves, so a section adds one module and
one `mod` line in `main.rs`.

cucumber-rs matches a step by its keyword: a `#[given]` binding never
matches a `Then` line, and `And` or `But` take the keyword before them.
A step text two sections both bind fails as ambiguous, so reuse the
existing text. Keep a binding thin: the logic it calls lives in a
library crate with unit tests, as the `engineering` crate holds §10's.

Every step binds on the one `World` a scenario threads. The runner
first runs itself again with `HOME` and `PARINN_HOME` in a fresh
temporary directory. Each world gets its own fresh `HOME` and
`PARINN_HOME` as well, for the processes its steps start (ENG-14).
State a section tracks lives in its own struct, reached through
`World::section::<T>()`, never as a new field on the world.

A step starts the core through `run_core` in `record.rs`, confined, with
the variables earlier steps set through `set_env`. Some exist only for
tests, and a release build never reads them. A development build's hook
handler panics when `PARINN_TEST_FAULT` is `hook-panic`, so the `@ENG-04`
scenario can prove the hook's entry point catches it, and hangs when it
is `hook-hang`, so `@NFR-02` can kill it. `PARINN_TEST_CLOCK` stands the
clock still at a time it names. `PARINN_TEST_HOOK_BUDGET_MS` sets every
hook's budget; `run_core` sets it lenient unless a step sets a hook's
own, so a loaded machine fails no scenario on a budget it does not test.

A benchmark, and a step that holds the core to a time budget, run a
release build. A development build is an order of magnitude slower and
flakes on a loaded runner. The `@NFR-02` row that times a hook handler
starts the core through `run_release_core`. That runs
`testkit::release_core_executable`: the path `PARINN_RELEASE_CORE` names,
else `target/release/parinn`. The test run never builds it. It fails
naming `cargo build --release -p parinn` when the core is absent, so
build it before `cargo test --workspace` or `cargo run -p coverage`.
CI's `test`, `coverage` and release preflight jobs do.

A release build has no seams. That row gives the handler its own
budget, and the rows that kill a hung handler (`PARINN_TEST_FAULT`,
`PARINN_TEST_CLOCK`) stay on the development build. The gate test
`benchmarks_and_timed_steps_run_release_builds` (ENG-15, ENG-27) fails on
a workflow or script line that builds the benchmark suite without
`--release` or points it at a `debug` directory. It also fails on an
integration test or bench function that reads a clock and starts the
development core. A drift case proves each.

The `@NFR-02` budget row times a hook handler against the frozen
`calibrate` workload, one unit of 10 ms by definition, in alternating
pairs under the same confinement. The path `PARINN_CALIBRATE` names, else
`target/release/calibrate`, is found as the core is. Build it with
`cargo build --release -p parinn -p calibrate`, as CI does. The limits
and the freeze are in [testing/hook-budget.md](testing/hook-budget.md).

## Keeping tests off real state

ENG-14 holds in three layers, so a test cannot touch the real `HOME`, a
real ParleyInn home or real Claude Code configuration.

- **The guard.** A test resolves the user's `HOME`, a ParleyInn home or Claude
  Code's configuration through
  [`testkit::guard`](../tooling/testkit/src/guard.rs), which aborts the
  process when the answer is not under the system's temporary directory
  (`..` and symlinks included). The testkit is test-only, so being linked
  arms the guard; no test can opt out of it. The ENG-14 scenario runs
  the scenario runner as a probe test that resolves each place, once
  with a stand-in real `HOME`, which must abort before it prints a path,
  and once with an isolated one, which must resolve.
- **One resolver.** A gate test (`only_the_isolation_guard_reads_the_home`)
  fails when a Rust source reads `HOME`, `PARINN_HOME`,
  `CLAUDE_CONFIG_DIR`, `USERPROFILE` or the standard library's `HOME`
  lookup anywhere but the guard and the scenario runner's own start.
  That is what makes the guard cover every test. A product crate that
  later resolves real state in shipped code joins that list with its plan,
  and the stakeholder approves it (`tooling/` is a code-owner path).
- **The decoy.** CI's `test` job runs the suite with `HOME` pointed at
  an empty directory and fails if anything appears there. The guard
  stops a test that asks for the user's `HOME`. The decoy catches a test
  that writes under the user's `HOME` without going through the guard,
  such as a program a test started without `isolate`. A read, or a write
  through a path built before `HOME` was swapped, leaves no trace there;
  the guard and the one-resolver gate test carry those.

ENG-14 also bars agent instructions that direct a destructive command
at real state. The `engineering` crate's `agent_instructions` scan reads
every `CLAUDE.md` and `AGENTS.md` and everything under a `.claude/`
directory. A statement is flagged when it names a way to delete or
overwrite (`rm`, `purge`, `uninstall`, `git clean`, `-delete`, `tee`,
a `>` redirect and the like) and real state (`~`, `HOME`,
`PARINN_HOME`, `CLAUDE_CONFIG_DIR`), unless prose forbids it
("never", "do not") or the statement first points `HOME` at a temporary
directory (`HOME=$(mktemp -d)`, "a temporary directory"). A code line is
never excused by a word inside it. Paths relative to the repository,
such as `rm -rf target`, are not real state.

## The test-engineer agents

The `test-engineer` agent reviews the pyramid as a whole. Three
specialists go deep on one test layer each: `test-engineer-unit`,
`test-engineer-integration` and `test-engineer-end-to-end`. They read
and report; they never edit or approve.

- The `test-review` skill runs all four on a pull request, plan or
  design, beside the coverage table, and merges what they find.
- The `test-shape` skill goes on from there: it turns each finding into
  a change, written red then green, that adds a missing unit test,
  moves a test to its test layer, or moves logic out of a binding into
  a library its unit tests reach. It then measures again.
