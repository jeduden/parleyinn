---
title: "Components and surfaces"
order: "10"
summary: >-
  ParleyInn's components inside their network boundaries, the room view's surfaces and the outcome window.
---
# Components and surfaces

- **Component**: A part ParleyInn plays, inside exactly one network boundary
(I4).
  The set is closed, listed here and in §6.3; how the components ship is open
  (OQ-32), and a new one is a model change and a §6.3 row.
  - **Core (B0):** the **hook handlers** (answering hooks), the **CLI**
    (`parinn`, but for `parinn ui`, `parinn launch` and each B1 to B3
    component's
    own entry point, SEC-01), each harness adapter's transcript and hook part
    among them, the **MCP server** (the MCP tools of §9.2, one per harness
    session, serving its runs and ingesting their transcripts once it holds its
    **session binding**: the harness session id and transcript path a
    `SessionStart` hook handler of its own harness process recorded, REC-13),
    the **kernel worker** (running the kernel's executions), the **TUI** (the
    terminal room view), the **commit hook** (LANE-28, run as the CLI), and
    everything that builds what reaches the model.
  - The **harness adapter** is no component: ParleyInn's code for one harness,
  its
    transcript and hook part in the core and its run part in the launcher.
  - **Room-view component (B1):** serves the browser room view on loopback only
    (SEC-20) and records the principal acts taken in it (VIEW-03). A
    **room-view secret** makes the browser room view a principal surface
    (SEC-20): a **launch secret** it mints per launch, exchanged once for an
    **origin secret** only its own origin can read or send; a phone-scoped one
    serves a browser on the principal's phone (OWN-16).
  - **Launcher (B1):** `parinn launch`, which starts, hosts and controls runs
    through each harness adapter's run part and runs witness checks; the only
    component that starts programs, but for the core's own kernel worker
    (SEC-29, CMP-05).
  - **Peer component (B2):** exchanges segments with peers (sync) and serves
    paired phones; on a paired phone it is ParleyInn's only code, with the room
    view's reduced client inside it. Turning it off, like any component, is cut:
    unlike a revocation, which makes a node refuse the revoked key's events it
    does not yet hold (PRV-10), it refuses none and drops no pin (PEER-01).
  - **Publish component (B3):** read-only publishing (SEC-26).
  - **Bridge component (B3):** outbound exchange with hosts the node's principal
    names, through three **bridges**: the **forge bridge** (reads linked issues
    and branches, associated pull requests, reviews and checks only under the
    room's exact forge import grant, records forge issue and review items, and
    sends person-confirmed responses to issue discussions and pull-request
    review threads), the **CI bridge**
    (fetches CI attestations) and the **notification bridge**.
- **Boundary**: One of B0 core, B1 machine, B2 peer and B3 public (I4).
  Unqualified, "boundary" means a network boundary; any other boundary is
  qualified, such as a span boundary or a crate boundary.
- **Room view**: What shows rooms to a person: the browser, through the
  room-view component, the one client that shows every surface, and the TUI, the
  CLI, the paired phone and the **harness strip** (a status line the harness
  shows) as reduced clients (VIEW-14). A client of the record. Without the
  room-view component, and before the room view's requirements ship (§12.2),
  the CLI shows the record at a terminal (§5.12); `parinn status` shows Health's
  counters and store locations (ADM-11). The room view's surfaces are a closed
  set, and unqualified, "surface" means one of them; where principal acts are
  taken is a principal surface:
  - **Fleet:** every live and recorded run of the principal's rooms, grouped by
    room.
  - **Room page:** one room, with the tabs Timeline, Review and Replay, the
    Replay tab's **context lens** showing what the model's context window
    contained at an event; the verify and why panels, the comparison and the
    **quarantine list** (the room's quarantine set), with its **forensic view**
    of quarantined content, open from it, and a foreign room opens in it
    (LANE-15).
  - **Catch up:** the one surface answering "what happened since a starting
    point" the principal picks (VIEW-08).
  - **Needs you:** the one queue of items waiting on a principal (VIEW-05).
  - **Health:** counters, failures and store locations (I6).
    - **Setup:** configuration and harness configuration, every change shown as
      a diff (I7, SEC-23).
  - **Peers:** enrolled peers and their state.
- **Outcome window**: The part of the Room page beside a room's conversation
  that shows one presentation (VIEW-22), never the outcome itself.
