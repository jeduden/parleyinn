---
n: 7
title: "The core confined in the end-to-end suite (ENG-12)"
status: "✅"
result: false
---
# Phase 7: the core confined in the end-to-end suite (ENG-12)

Requirement. ENG-12 (P0, Ver T): the end-to-end suite runs each
component's entry point confined to its boundary and fails on any
socket outside the boundary the register assigns.

Only the core exists. This phase runs every end-to-end test of the
core with all network denied, and makes any socket the core opens
fail the suite naming the component and the test that called it. The
room-view, launcher, peer, publish and bridge rows come with M7 to M9,
so `@ENG-12` stays `@pending`; this phase never retags it.

RED: a probe that opens a socket from the core's process runs under
the suite and is not caught. GREEN: the confinement, the socket
check, a drift case.

Gate: the probe fails the suite naming the component and the caller;
CI runs the confined suite.
