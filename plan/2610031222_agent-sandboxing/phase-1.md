---
n: 1
title: "A conformance probe for the residual risks"
status: "🔲"
result: false
---
# Phase 1: a conformance probe for the residual risks

Requirements. None closes: the SRS ids for sandboxing come in task 5.
This phase writes evidence the stakeholder uses to pick the supported
set.

BDD coverage: none yet. The probe's cases become the scenarios task 5
adds as `@pending`.

The probe is a small program in a separate module beside this plan,
like spike S2's, so nothing enters ParleyInn's module graph. It runs as
the agent would, inside the sandbox under test. `HOME` and
`PARINN_HOME` point at a temporary directory (ENG-14). That directory
holds a fixture store, a fixture writer key, a fixture harness
settings file and a stand-in loopback listener. The probe never
touches the machine's real `~/.parinn` or `~/.claude`.

RED: one case per residual risk. Each case is an attempt that must
fail for the risk to count as blocked:

- R1: inject keystrokes into a terminal multiplexer session outside
  the sandbox.
- R2: write the harness settings file or ParleyInn's executables.
- R3: delete or replace `PARINN_HOME`.
- R4: reach the browser profile or a GUI input socket.
- R5: read the fixture store and key.
- R6 and R7: signal ParleyInn's own processes, or reach the loopback
  listener.

The probe fails until it reports each case as blocked or open, with
the evidence.

GREEN sites: the probe, and one runner script per sandbox
technology. Claude Code's own sandbox on Linux comes first, then
OpenShell, then a plain container.

Gate: the matrix of technologies by residual risks is committed beside
this plan. Its runner scripts reproduce it. It covers at least two
technologies.
