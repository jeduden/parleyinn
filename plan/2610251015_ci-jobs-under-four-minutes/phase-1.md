---
n: 1
title: "The test job counts scenarios from its own run"
status: "🔲"
result: false
---
# Phase 1: the test job counts scenarios from its own run

Requirements. None applies: this phase changes how CI runs, not what
ParleyInn does. It takes no scenario off `@pending`.

BDD coverage: none. The change is in the workflow and reads no
cross-host or lease behavior.

RED spec. Main run 765 shows the test job at 5.3 minutes, of which 1 m
17 s is the "Count scenarios" step running the scenario suite a second
time to read its `scenarios:` line. Record the test job's time from the
`ci-timings` artifact of the current head as the baseline. Any run
where the test job is over four minutes fails this phase.

GREEN sites, in `.github/workflows/ci.yml`:

- the "Run the tests with a decoy HOME" step keeps its full output and
  writes the `scenarios:` line to the job summary, so the count comes
  from the run that already happened;
- the "Count scenarios" step is removed;
- the decoy-home check and the exit status of `cargo test` stay exactly
  as they are.

If the test job is still over four minutes in two consecutive runs,
the plan's task 2 splits it; that is a later phase, declared with the
result.

Gate. The `ci-timings` artifacts of two consecutive runs show the test
job under four minutes, and the job summary still carries the
`scenarios:` line with the same count as before the change.
