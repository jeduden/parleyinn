---
n: 1
title: "The review-run crate against a fake host"
status: "🔲"
result: false
---
# Phase 1: the review-run crate against a fake host

Requirements. This phase starts the change to ENG-28: a review run by
code. It adds no requirement id yet; the SRS row and its scenario change
in phase 2 with the workflow, so ENG-28's scenario stays as it is here.

BDD coverage. No new scenario: the crate's behaviour is proved by unit
and integration tests against a host double, and the live behaviour by
phase 2's workflow run. The ENG-28 scenario is reworded and rebound in
phase 2.

RED. Write tests for each of these, in order:

- Reading `dispatch.json` and the scopes the bundle writes.
- One run of each dispatched specialist, in parallel up to a bound,
  each with its own deadline.
- One run of each generalist after every specialist has finished, told
  the report files of its specialists.
- A specialist that does not answer, or whose report is missing, empty
  or without its closing outcome line. It is counted and named, and the
  review is still decided.
- The pure verdict function: approve exactly when every dispatched agent
  answered and no kept generalist finding is blocking.
- The outcome JSON in the shape the gate reads.
- The consulted list and `answers.json` as the gate reads them.
- Usage per agent.

GREEN. A \`tooling/review-run\` crate with a library and a thin
executable, depending on \`review-eval\`'s host, runner and log modules
(extract what it needs into a shared module if a dependency on the whole
crate is too wide), no new third-party dependency.

Gate. The crate's unit coverage holds the 99% floor; clippy is clean;
\`review-run\` against a recorded bundle and a fake \`claude\` writes the
files the gate reads, and \`review-gate\` accepts them.
