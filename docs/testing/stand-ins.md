---
title: "Stand-ins for the harness"
summary: >-
  The stand-in harness process and the transcript builders the testkit holds, what each starts from, and how the unit tests keep a built line in the shape Claude Code recorded.
---
# Stand-ins for the harness

What a test builds of Claude Code lives in the testkit, so a step file builds
none of its own. The page belongs to [testing.md](../testing.md).

## The stand-in harness

[`testkit::harness`](../../tooling/testkit/src/harness.rs) starts a script
named `claude`, confined as ENG-12 asks, that runs `parinn mcp` and each hook
handler as its children, as Claude Code does (REC-13). A test names the home,
the core executable and the environment; the scenario runner adds its own in
`tooling/scenario/tests/bdd/live.rs`. The unit tests run the harness against a
stub core, so they need no build of the core.

## The transcript builders

[`testkit::transcript`](../../tooling/testkit/src/transcript.rs) starts each
line from one recorded under `testdata/claude-code/2.1.296`. It changes only
the fields a test names. Its unit tests replay every builder's output through
`harness-contract`, the core's own parser and classifier. A built line that no
longer has the recorded shape fails there.

Only the tests depend on `harness-contract`. It depends on the testkit for its
own tests, and cargo allows that cycle between dev-dependencies. The testkit's
build stays free of the core, so a change to the core does not rebuild it.
