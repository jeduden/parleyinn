---
n: 1
title: "The standalone lane experience, with no network"
status: "🔲"
result: false
---
# Phase 1: the standalone lane experience, with no network

Requirements. None closes yet: plan 2610012322's SRS change names the
lane, lifts NG5 and scopes peering first. This phase fixes the
experience and the test approach.

BDD coverage: a scenario per behavior once the SRS ids exist.

Experience first. One view holds three things side by side: the
harness of the agent being worked with, live (prompts, tool calls,
permission requests, output); the work results; and the other
harnesses on the machine, each a live tile that opens full size. The
lane's timeline shows the conversation, each edit as a diff, each
tool run and its result, and who did what: agent or person. Background
agents appear as child threads you can open, and steer where the
adapter allows; a control it lacks is shown as unavailable (OWN-15).
A result shows its evidence class, `claim` or `own run`, because a
completed run is not a verified one; witness runs and CI attestation
come later (LANE-05). Words and marks follow the proposal's §8.

Standalone means ParleyInn sends nothing off the machine. The UI is a
web page in the browser, served on a loopback-only port with a token
per launch and Host and Origin checks. Harnesses write to the same
store with no UI running, and the UI joins later and shows the same
lanes. Neither harness receives the other's events except through
enveloped recall, when it asks. The view is built against recorded
lanes first, so the design can be judged before any live harness is
wired in.

RED: a test runs in a sandbox that denies all but loopback, with `HOME` and
`PARINN_HOME` in a temporary directory (ENG-14). It starts two
harnesses on separate worktrees of one clone. It fails until both
lanes, their edits and their results appear in one view, the view
updates as each harness writes, and no socket is reachable from off
the machine and no outbound connection is made. The same test holds
every NFR-01 hook budget with the view open and ten harnesses writing.
It also checks these cases:

- Every result shows its LANE-05 class, and a result known only from
  agent text or tool output shows `claim`.
- Neither harness's events reach the other's context except through
  enveloped recall. After a harness switches branch and compacts, its
  restore block holds the pins of both the old and the new lane, word
  for word, and names both lanes.
- A same-user process cannot complete an owner act that widens what
  an agent may do, by any vector T21 or §11.3 of the proposal names.
- Under a root-owned managed policy that disables `parinn ui`, starting
  it is refused, audited and shown by `parinn doctor`.
- A recorded-weekend fixture runs through Catch up, search to replay
  and lane verify, in the view and through the CLI alone. It holds a
  failed capture, a session in no lane, a missing segment, a late
  ingest, a quarantine, a purge, an unsynced writer and an altered
  segment, and the test fails when Catch up omits any of them.

GREEN sites: the lane view, the local event feed it reads, and the
recorded-lane fixtures.

Gate: the RED test passes against the built binaries. The stakeholder
walks through the view and signs off the design. The listening socket
stays a prototype, provably absent from release artifacts,
until the ENG-29 security review of I4 and SEC-01 is accepted. Only
then does phase 2 add peer to peer.
