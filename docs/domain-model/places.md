---
title: "Places"
order: "03"
summary: >-
  Where ParleyInn's state lives and is shared: homes, nodes, devices and paired phones, sandboxes, rooms and their kinds, visibility, peers and blind peers.
---
# Places

- **Home**: The directory with one principal's ParleyInn state on a node
  (`PARINN_HOME`, default `~/.parinn`), belonging to one OS user, holding the
  keys SEC-10 binds to it, and optionally bound to a **home id** (`home.id`)
  that the environment must supply to open it (SEC-03). The unit of isolation
  (I8).
- **Node**: One home on one machine, container or cloud environment, for one
  principal, with its own device key; an **ephemeral node**, one in a
  short-lived cloud environment, may have only a token key, a **token-key-only
  node**, which signs no principal act (Token key). Minting its access token is
  also its principal's ask for each of its runs to join, at the run's start,
  every room the token names to continue, as that room's admission permits, and
  the token carries that ask and those rooms' qualifying pins (PEER-06), so
  those pins restore to its runs from their first restore block. Its **node
  identity** is a value outside the home that a cloned image or a restored
  snapshot cannot carry over (REC-24). A **node identity change event** is the
  structural event a node whose node identity changed records in its new device
  seat in its personal room, carrying only the fingerprints of the device key or
  token key it replaces and of the new one and the chain head of the key event
  log it leaves, followed by the new key's inception; both count only as PRV-12
  says, and a reject the replaced key signs voids them (REC-24, PRV-12). A
  **node clone** is a copy of a home started where its node identity differs: a
  new node, with new keys and seats, that shares the carried-over personal room
  (REC-24). On any node whose node identity changed, and after a backup restore,
  the record from before carries over only as ADM-06 says (open: OQ-48).
- **Device**: A node or a paired phone.
- **Paired phone**: A device with no home, limited to reading and to allowing or
  denying held permission requests within its device scope, reaching its node
  over B2; the node it pairs with holds its device seat's writer (OWN-17; open:
  OQ-39).
- **Sandbox**: A confinement around a run that blocks some residual risks
  (OWN-22).
- **Room**: Where an intent is worked on: at most one intent, a **conversation**
  (its ordered posts and the forge discussion items shown with them),
  seats, pins, issue links, and branches in any repositories, each named by a
  branch link. The unit ParleyInn shows and shares.
- **Personal room**: A principal's private room on each of its nodes, created as
  ADM-02 says, or carried over to a node clone, which shares it (LANE-01,
  LANE-17). Every run its node records has a seat in it from its first event,
  and every event or pin that belongs to no other room goes there. Unlike a
  node clone, a backup restore of a copy another node made does not share that
  node's personal room (ADM-06; open: OQ-45).
- **Foreign room**: A room this node holds, other than only as a blind peer,
  that its principal neither owns nor has a seat in, such as the room of an
  imported bundle or a room every seat of its principal has left or lost to a
  kick or a bar. Recall reaches it only when the call names it, and marks every
  item from it untrusted (RCL-10). Yet a leave, kick or bar that makes a room
  foreign drops no pin: PIN-10 says which of its pins keep restoring, and to
  which runs.
- **Principal's rooms**: The rooms a principal owns or has a seat in.
- **Visibility**: Whether a room is private, shared with the room's principals,
  published or stored on blind peers (LANE-17). Changing it is a widening
  principal act of the owner.
- **Peer**: Another node this node's principal enrolled by key and exchanges
  sealed ranges with through the peer component (B2). Being a peer never makes
  content trusted (PRV-02). PEER-02 says which sealed ranges a node serves to
  which peer (open: OQ-41). Exchanging the sealed ranges each may receive is
  **sync**.
- **Blind peer**: A peer that holds a room's segments without any key they are
  encrypted to, so it stores and serves them encrypted and reads none of them
  (PEER-12).
