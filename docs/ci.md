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

## Feedback time

The loop is edit, check here, review here, push, CI, fix. Each turn of it
is short, because a finding found late costs a whole round.

- Review here before every push: `scripts/review-local.sh` runs the
  review against `origin/main`, or with `--base REF` against the commit
  last reviewed, so a fix cycle reviews only what moved. A small change
  reviews in about a minute.
- CI ends within ten minutes of a push. The slowest chain of jobs sets
  that; a job that grows toward its five-minute limit is split before it
  fails.
- `review-eval plan` prints how many runs, minutes and dollars an
  evaluation is expected to take. A plan beyond 150 runs is a reason to
  split the change, or to keep its shared-section edit apart from the
  rest.
- A workflow or tool change that makes any step of this loop slower is a
  finding, with the measured time before and after.

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

## Actionable

- A failure names the requirement id, scenario, file and line, what was
  expected and what was found.
- It gives the command that reproduces the failure locally, or repairs it,
  such as `mdsmith fix .`.
- The first failing line is near the top of the job summary. A failing job
  uploads what a person needs to debug it.
- The `CI` job names the jobs that failed.
- A gate that reads the repository's records has a drift case (ENG-27).

## Checking it

The `ci-review` skill measures the pipelines against this page and hands
the numbers to the `ci-engineer` agent, which reports and never approves.
Workflow changes are CODEOWNERS-protected, so they land in their own pull
request.
