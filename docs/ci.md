---
summary: >-
  What ParleyInn asks of its CI: a fast build, a stable build that does
  not flake, and failure output that tells the author what to do, with
  the ci-engineer agent and ci-review skill that check it.
---
# CI expectations

How the pipelines run is in [development.md](development.md). This page
says what they must be. The `ci-engineer` agent reviews them against it,
and the `ci-review` skill measures them.

## Fast

- The `CI` job waits for every other job, so the slowest chain of jobs
  sets the wall clock. Keep that chain short.
- Jobs that can run side by side do. A job names only the jobs it needs.
- Every job restores a cache keyed on the pins, and installs tools once. A
  key that changes every run, or a cache nothing reads, is waste. The one
  exception is the release build, which takes no branch-writable cache;
  the cache holds only the tools, so the first release under the cap is
  the measurement that confirms the build fits without it.
- No job repeats another's build, download or install. Work too slow for a
  pull request moves to the nightly job.
- Every job in `ci.yml` and `build.yml` sets `timeout-minutes: 5`. A job
  that cannot finish in five minutes fails, and that failure is the signal
  to speed it up or split it, not to raise the limit. The other workflows
  keep limits sized to their work.
- A sharded job takes its shard from its place in the matrix
  (`strategy.job-index` of `strategy.job-total`), not from a number the
  matrix lists, so the shards cover the work once whatever the list says.
  The scenarios divide by a hash of the feature file and scenario name
  (`--shard K/N`, or `PARINN_BDD_SHARD` for `cargo test -p scenario`),
  the drift cases and benchmark rounds by position. The `test` job's
  shards each run the crates of the package list at their position, and a
  gate test fails when a crate is in no list or in two.
- The `CI` job records each job's duration and result in its job summary
  and uploads them as the `ci-timings` artifact. Read those; do not
  measure again.
- A review's timing and cost per agent run are in its `review-reports`
  artifact, the run report `report.html` and the run list `agent-runs.jsonl`;
  read those before timing a review again.

## Feedback time

The loop is edit, check here, review here, push, CI, fix. Each turn of it
is short, because a finding found late costs a whole round.

- Review here before every push: `scripts/review-local.sh` runs the
  review against `origin/main`, or with `--base REF` against the commit
  last reviewed, so a fix cycle reviews only what moved. A small change
  reviews in about a minute. The agents are the head's definitions, what
  main has after the merge; `--agents base` runs main's, as CI does.
  `--resume` reruns only what the last run left without an accepted
  report and each agent whose answer key changed, so a slip costs one run.
  `review-run --answers DIR` also reuses every stored answer whose key is
  unchanged; no workflow gives it a store yet.
  A review that aborts on a mass failure of its specialists says so
  apart from any finding: fix the outage or prompt, then `--resume`.
- A pull request's reviews share one cost cap, ENG-28's, 5 USD counted
  over all of them: each review keeps what it spent in a
  `review-spend-pr-<N>-a<attempt>` artifact, and the next sums every one
  into `review-run --spent-before`, up to 1000 of them, so reviews run
  side by side, cancelled or rerun never erase each other's. A run
  starts only once its tier's cap, 0.30 USD for a specialist and 1.50
  for a generalist, fits beside
  what was spent and the caps of the runs in flight, else it waits; a run
  paid for that left no result, killed at its deadline, crashed or
  cancelled, is charged its tier's whole cap, and one whose result
  reported no cost is charged nothing. A review the cap stopped
  lists every agent it did not run, `over budget:`, in
  `review/not-reviewed.md` and requests changes. A run cut off by its
  turn budget or deadline is not a failure and never runs again: its
  log is read as its report, and the `Read` calls in it decide whether
  it read its whole diff, each unread range a `partial:` gap. Deleting the pull
  request's `review-spend-pr-<N>-a*` artifacts resets its budget, and
  `--max-cost-usd` overrides the cap.
- CI ends within ten minutes of a push. The slowest chain of jobs sets
  that; a job that grows toward its five-minute limit is split before it
  fails.
- `review-eval plan` prints how many runs, minutes and dollars an
  evaluation is expected to take. A plan beyond 150 runs is a reason to
  split the change, or to keep its shared-section edit apart from the
  rest.
- A workflow or tool change that makes any step of this loop slower is a
  finding, with the measured time before and after.

## Run CI's gates here

Review is not the only gate a push meets. The coverage floors, the drift
suite, the supply-chain checks and the rest run in CI alone unless they
run here first, and a defect they find costs a round. So CI's gates are
defined once, in [scripts/ci-gates.txt](../scripts/ci-gates.txt): each
line names a gate, whether it fits a small machine, what it needs, its
scope and its command. A job of `ci.yml` runs a gate through
`scripts/ci-gate.sh NAME`; `scripts/ci-local.sh` reads the same list. A
check (ENG-27) fails CI when `ci.yml` writes a gate's command inline,
calls a gate the list does not define, or leaves a gate without a job.
A gate whose job is its definition, because its shards, decoy `HOME` and
artifacts are the job's own (`test`, `scenarios`, `eval`, `benchmark`,
`miri`), is listed with `@job:NAME`, so the runner can still report it.

`scripts/ci-local.sh` runs the fast gates on what the diff against
`origin/main` touches: formatting, clippy, the tests and the unit-line
coverage floor of each touched crate, `mdsmith check .`, Taplo, zizmor
`--offline`, the srs and scenario gate tests, the scenarios of the
requirement ids the diff names, the ENG-34 line limits, and the drift
cases whose injection files the diff touches. `--full` adds the gates
that need disk or the network, `--gate NAME...` runs the ones named, and
`--list` prints the list. Every gate is reported ran, skipped or failed,
with the reason for a skip: a gate that needs a CI runner (the end-to-end
coverage shards, the benchmark, Miri), a core not built, or too little
free disk. The exit status is 1 when a gate failed.

CI's coverage merge wants every line of a crate covered by the unit,
integration and end-to-end layers together. The unit-line floor is only
the unit layer's share, so a line the unit layer misses can still pass
here and fail there. Locally only the unit layer runs. After each
crate's unit coverage `scripts/uncovered-added.sh` reads the report and
the diff and lists the lines the diff adds that the unit layer leaves
uncovered, as a `WARNING` in the summary: "CI also counts the
integration and e2e layers; these added lines are uncovered by the unit
layer". It never fails the gate, because the runner cannot tell a line
an integration test covers from a line nothing covers. Cover each listed
line with a unit test unless a test of another layer takes it.

Builds go to `target/ci-local` with `CARGO_PROFILE_DEV_DEBUG=0` and
`CARGO_INCREMENTAL=0`: debug information and the incremental cache are
most of a checkout's build size, and a sandbox's disk is what runs out.
A build step is refused, and reported skipped, below
`PARINN_CI_MIN_FREE_GB` of free space. A heavy gate needs 8 GB, a fast one
half that.

`scripts/review-local.sh` is the other half of the loop. With
`PARINN_REVIEW_CI_LOCAL=1` it runs `scripts/ci-local.sh` first. It stops
when a gate failed.

## Stable

- Every action is fixed to a commit SHA and every tool to its pin.
- No test sleeps to wait, reads the network, or depends on order, a clock
  or unseeded randomness. A step that downloads retries within a bound and
  names the host when it fails.
- A flake has a root cause. No retry, `continue-on-error`, `|| true`,
  skipped test or quarantine turns a failure green. One rerun may confirm
  a failure that is not the change's.
- No job touches the real `HOME`, `PARINN_HOME` or Claude Code
  configuration (ENG-14), and each workflow holds the least `permissions:`
  it needs, and zizmor stays green.
- No step that runs an agent holds the Claude Code OAuth token: one step
  in each agent job stages it outside the agents' reach, and what an
  agent job keeps is scrubbed of it before any upload (ENG-28).

## Actionable

- A failure names the requirement id, scenario, file and line, what was
  expected and what was found.
- It gives the command that reproduces the failure locally, or repairs it,
  such as `mdsmith fix .`.
- The first failing line is near the top of the job summary. A failing job
  uploads what a person needs to debug it.
- The `CI` job names the jobs that failed.
- A gate that reads the repository's records has a drift case (ENG-27).
- A file past its kind's line limit fails CI unless the size list,
  `tooling/engineering/oversize.txt`, lists it, and so does a listed
  file whose lines are not exactly those listed (ENG-34). A file that
  grew must be split; a file that shrank yet stays past its limit must
  have its listed lines lowered, so a regrowth cannot hide behind an old
  number. The size list only shrinks: a listed file that fits, or is
  gone, must leave it. To fix a failure, split the file, or move a
  workflow `run:` body into a script under `scripts/`, then lower or
  drop the file's line in the size list.

## Checking it

The `ci-review` skill measures the pipelines against this page and hands
the numbers to the `ci-engineer` agent, which reports and never approves.
Workflow changes are CODEOWNERS-protected, so they land in their own pull
request.
