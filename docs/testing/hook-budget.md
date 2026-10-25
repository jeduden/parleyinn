---
title: "The hook budget row"
summary: >-
  How the NFR-02 scenario holds a hook handler to its hook budget in units of a frozen calibration workload, why that reference, the limits and their derivation, and where the absolute budget is held.
---
# The hook budget row

How the `@NFR-02` scenario holds a hook handler to its hook budget, and
why it compares times instead of reading the clock. The page belongs to
[testing.md](../testing.md).

## Why the row is relative

The core runs under `strace` in a network namespace (ENG-12). That adds
25 to 30 ms to every run. A hook handler needs 5 to 10 ms, so a 50 ms
budget was overrun on a loaded machine. The core stops a handler at its
own budget and counts `hook_budget_exceeded`. That happened in two runs
of thirteen idle, and in three of six under load.

The row now times 11 pairs of runs under that same confinement. Each
pair is the calibration workload, then the hook handler on its
recorded-shape input. The order swaps every round
(`testkit::relative::interleaved`). Whatever slows one run slows the
other.

`within_multiple` then compares the two medians. The hook's median must
be at most a limit times the calibration workload's. Each run must also
exit 0, and a hook run must give an empty answer with no stderr. The
slowest hook must stay below its harness timeout. The start-marker steps
are unchanged.

## The reference is the calibration workload

The reference is the `calibrate` executable (`tooling/calibrate`), not a
run of the core. A run of the core, such as `parinn version`, is the code
base itself. A change that slows the core would slow the reference too,
and the ratio would hide it.

The calibration workload has none of that. It imports no crate and
reads nothing of ParleyInn. It does a fixed amount of work with the same
shape as a hook's, so the confinement taxes both alike:

- **A computation.** A fixed seed run through 1,200,000 rounds of
  integer mixing. The result is printed, so it cannot be optimised away.
- **System calls.** 50 rounds that each create a new file, write 64
  bytes, close it, open and read it, look up its size, ask for the
  process id and the clock, and remove it. That is about 400 calls,
  which is what a hook handler makes (220 to 470 by `strace -c`).
  `strace` stops the program at every call, traced or not, so the
  confinement's cost grows with the call count in the workload as it
  does in a handler. Each round makes a new file, as rewriting a file
  makes the file system flush it on close and times the disk instead.

One calibration unit is a nominal 10 ms, by definition. The two counts
were tuned once, on a release build on the development machine, so that
a run takes about that long (10 to 12 ms measured on a loaded machine).
The unit does not follow the machine: a slower machine makes a unit take
longer, and the hook with it. The number never changes after the commit
that recorded it.

## Frozen

Three things hold the workload constant:

- **The record.** `engineering::calibration` holds the 64-bit FNV-1a
  hash of every file in `tooling/calibrate/src` and the three constants
  (seed, mixing rounds, file rounds). The gate test
  `the_calibration_workload_is_frozen` fails when the source or a
  constant differs from the record.
- **The result.** A unit test in the crate pins the value the frozen
  workload returns.
- **The drift cases.** Two ENG-27 cases change the workload, one a
  constant and one the mixing, and must make the gate fail.

`tooling/` is stakeholder-owned in CODEOWNERS, so a change to the
workload or its record waits for the stakeholder's approval. Changing
the unit is a change to what every hook limit means.

## The limits

A limit is the hook's budget (§9.1) divided by the 10 ms unit, by
definition and not tuned:

| Hook              | Budget | Limit | Largest ratio seen | Headroom |
| ----------------- | ------ | ----- | ------------------ | -------- |
| SessionStart      | 150 ms | 15    | 1.63               | 9x       |
| UserPromptSubmit  | 50 ms  | 5     | 1.43               | 3.5x     |
| PermissionRequest | 50 ms  | 5     | 1.41               | 3.5x     |

The ratio is the hook's median over the workload's, both confined. The
largest ratio is the worst row of three conditions. They ran on the
4-core development machine:

| Condition          | Rows | SessionStart | UserPromptSubmit | PermissionRequest |
| ------------------ | ---- | ------------ | ---------------- | ----------------- |
| Idle               | 12   | 0.77 to 1.07 | 0.72 to 0.89     | 0.57 to 0.75      |
| Six busy loops     | 8    | 0.98 to 1.40 | 0.67 to 0.90     | 0.54 to 1.17      |
| Sixteen busy loops | 5    | 0.91 to 1.63 | 0.94 to 1.43     | 0.91 to 1.41      |

Both sides ran in the same pairs each time, which is why a loaded
machine moves the ratio so little. A hook gets slower against the
reference only when its own work grows. One that took 3.5 times its
present time under confinement would still pass; one that took 5 times
the reference's would fail. A hook that took 5 units unconfined would
use all of its 50 ms budget.

## Where the absolute budget is held

The confined row cannot also require that no overrun was counted. The
confinement alone can reach the core's own 50 ms stop. The absolute
budget is held on the unconfined release core instead. The benchmark
suite's hook samples (`hook_session_start`, `hook_user_prompt_submit`)
fail when the handler counted `hook_budget_exceeded` in the sample's
home (ENG-15). A sample whose handler overran is taken once more on a
fresh home, because a stall of the runner under one sample is not the
handler's; a handler that overruns twice in a row fails the benchmark.

## What proves the check

A unit test with timing doubles shows a slowed hook fails the relative
check and a healthy one passes. A drift case (ENG-27) shows the test
fails when the check is loosened.
