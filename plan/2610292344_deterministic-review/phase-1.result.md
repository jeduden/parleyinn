---
n: 1
title: "The review-run crate against a fake host"
status: "✅"
result: true
summary: >-
  The review-run crate runs every dispatched specialist, then every
  generalist, as claude --agent through the evaluation's runner and
  process host, and writes the outcome, consulted list, answers and
  usage the review gate reads; unit tests cover every line, and an
  end-to-end test has the gate approve its output.
---
# Phase 1 handoff

## Done

- `tooling/review-run`, a library and a thin executable:
  `review-run --workdir DIR [--parallel N] [--deadline SECONDS]`,
  exit 0 decided, 1 could not run, 2 usage. It reads
  `review/dispatch.json` and the bundle's printed output in
  `class.txt`, runs the specialists eight at once, each with the
  evaluation's 900-second deadline, then the generalists told their
  specialists' report files, and writes `review/outcome.json`,
  `review/consulted.json`, `review/answers.json` and
  `review/usage.json`, with each run's log under `review/logs/`. It
  prints `verdict`, `dispatched`, `consulted`, `findings` and `cost`.
- Each run is `runner::command` of `review-eval`, so the tools, the
  write rule, the turn budget from the agent's front matter, the
  prompts and the nested harness session scrub are the evaluation's. The
  process host moved from `review-eval`'s executable into its library,
  shared by both executables.
- The verdict is a pure function: approve exactly when every
  dispatched agent answered, every specialist's report passes the
  gate's own report check (now public in `review-gate`), and no
  generalist kept a blocking finding. A silent agent or a refused
  report is named in the summary and in a blocking finding; the
  consulted list leaves a silent agent out, so the gate still refuses
  to post such a review, as ENG-28 asks.
- Unit tests cover every line of the crate (lcov); clippy is clean;
  the end-to-end test runs the executable on a recorded bundle with a
  stand-in `claude` and the review gate approves its output, or, with
  a specialist past its deadline, refuses to post.

## Decisions

- An empty dispatch requests changes with a blocking finding at
  `review/`: no reviewer read the change, so code does not approve it.
- A generalist's line is dropped when it starts with `dropped` or names
  `dropped` as its severity; another severity than `nit` or `blocking`
  is read as blocking.
- A report file left from before a specialist's run is removed first.
- No `--model` is passed: each agent runs on its front matter's model.

## Next

- Phase 2: `review.yml` installs the pinned `claude` and runs
  `review-run --workdir .` after the bundle step (which already saves
  `class.txt`); the consulted and answers steps read
  `review/consulted.json` and `review/answers.json`, the usage step
  `review/usage.json`, and the job output `outcome` reads
  `review/outcome.json`.
- The generalists' front matter names `sonnet`; plan task 4 moves them
  to Opus.
- `docs/agents/generalist-report.md` should say a dropped finding's
  line starts with `dropped —`, so the reader excludes it.
- The gate's message still says "the orchestrator did not consult".
