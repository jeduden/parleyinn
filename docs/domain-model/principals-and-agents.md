---
title: "Principals and agents"
order: "01"
summary: >-
  Who acts in ParleyInn: principals, persons and service accounts, managed policy, agents, runs and subagents, the facilitator, authors, members, the owner and delegates.
---
# Principals and agents

- **Principal**: A person or a service account. It has a principal key, signs
  principal acts with a device key at a principal surface once PRV-10 ships,
  each recorded on a device seat, and is the principal of every agent whose runs
  its nodes record. ParleyInn counts principals by principal key: every device
  key,
  token key and seat key that chains to one principal key through device, token
  or seat certificates belongs to that principal, each certificate counting
  only once the key it certifies countersigns it; a key whose countersigned
  certificates chain it to two principal keys chains to neither, and a chain
  never passes through another principal key (PRV-10). No key chains to a
  principal key before PRV-10 ships, so until then a room admits no seat of
  another principal, and admission, invites, bars and the owner's protection,
  which name principal keys, apply only once it ships (OQ-42). A certified
  service account is still its own principal; its **certifier** is whoever
  certified it.
- **Person**: A human principal. No one certifies a person's principal key, and
  only a person records a verdict (OWN-27).
- **Service account**: A non-human principal with its own principal key, which a
  person, another service account or managed policy, by listing it, certifies,
  marking whether it relays text others wrote, and only that certifier revokes
  (PRV-10). A service-account certificate counts only once the certified
  principal key countersigns it, and each managed-policy listing and each
  removal of one is recorded as a structural event that enters the key set of
  the node that recorded it from its machine's managed policy, and another
  node's only once the listed key countersigns it, as a certificate; a removal,
  needing no countersignature, enters wherever the listing it ends did
  (PRV-10); how a listing or a removal reaches another principal's node rests
  on OQ-41. A key marked as relaying, or listed unmarked, stays relaying once
  what marked or listed it is revoked or removed; and a revocation of any
  certificate of a key, or a removal of any listing of it, makes every trust
  grant naming that key that does not causally follow that act cover nothing
  (Trust grant). It can own rooms, have seats and be the principal of its own
  agents, such as CI or runner agents.
- **Managed policy**: Settings belonging to root that an organisation sets on a
  machine, overriding every other settings layer (ADM-04). ParleyInn never
  overrides
  it (I7). Among its powers are SEC-22's and certifying service accounts by
  listing their principal keys (PRV-10). The harness's own managed settings,
  which ParleyInn also never writes (ADM-03), are part of the **harness
  configuration**: the harness's settings, hooks and MCP registrations (I7).
- **Agent**: A worker a harness runs for exactly one principal: the principal of
  the node that records its runs (OWN-01). It receives restore blocks and
  recalls history; an agent is never a principal. It is identified by its node,
  its harness and the harness's agent id across harness sessions; where the
  harness gives no stable agent id, each run is its own agent, as is each
  subagent's run (REC-02). Its later runs inherit its rooms for recall (the
  `rooms` scope), for restore and for its directed posts, never a seat, add or
  role, each as REC-02 sets: a new run joins only as Join says.
- **Run**: One agent's execution within one harness session, on one node, keyed
  by the harness session and the harness's agent id: a harness session's
  transcripts contain its main agent's run and one per subagent, a **harness
  resume** that keeps its harness session continues the run, while a harness
  resume that starts a new harness session without marking itself a fork, a
  **harness clear**, or a **harness
  fork** (a new harness session, marked by `source` = `fork`, whose transcript
  repeats an earlier one's lines)
  starts a new run (REC-02), of the same agent only where the harness gives a
  stable agent id; without one, whether it is the resumed run's agent is open
  (OQ-43), and until then it is its own agent. It has run seats and carries its
  recall taint (SEC-13), sandbox state (OWN-22), seat keys (SEC-10), kernel
  namespace (CMP-02) and spans (LMK-01). As a noun, "run" has no other meaning.
- **Subagent**: An agent the harness started for another agent. It has its own
  run, tied to its parent's run by a parent link, with its own seats and
  writers, among them a seat in each room its parent's run seat is a member of
  when it starts, joined then with that seat's role and no appointment, under
  its parent's join (LANE-23). It takes a delegated task without a delegation
  grant. Its run records that task as untrusted `harness_text`, never `user`
  (PRV-08).
- **Ingested run**: A run ingested from a transcript no MCP server ingested as
  it grew, with origin `ingested`, whose events are untrusted (REC-22).
- **Facilitator**: The service account, at most one per room, whose device seat,
  on its own node, the owner appoints as the room's facilitator, making that
  device seat an appointed moderator (SEC-32); its program, neither a ParleyInn
  component nor an agent, acts through that node's CLI (`parinn room-summary
  write`). It writes room summaries, answering summary requests (LANE-33). It
  posts only in its own words, pointing by range link to any marked range it
  names and never quoting or embedding it, so it relays no third party's text
  (SEC-32); every pin version it writes is likewise in its own words, and its
  program writes no pin that restores unstamped (OWN-12). An agent on that node
  can act through its CLI as the program does: §6.1's residual risk R7, which
  only a sandbox removes.
- **Author**: The seat that wrote an event or a pin. The author's principal
  follows from the seat.
- **Member**: A seat whose add stands and that no bar covers, or one LANE-25
  makes a member with no add, such as a run's personal-room seat (Seat, Seat
  key, Run seat); the seat's role says what it may do. A principal is never a
  member; **the room's principals** are those with a member seat, and text for
  people speaks of the room's principals. A personal-room seat can neither
  leave nor be kicked.
- **Owner**: The one principal who owns a room: its intent, roles, admission,
  appointments, successor and handover. Ownership changes only by handover or
  succession, and stays with the owner after all its seats leave (LANE-11).
  The owner stands beside the roles rather than having one: each of its device
  seats in the room, but a paired phone's, has every room capability but
  writing a room summary, within LANE-16's limits, while its agents' run seats
  have only their role and any appointment; its act joining its own run assigns
  that seat contributor (LANE-16). No mute, a whole-room one included, covers
  its device seats (LANE-16). Once PRV-10 ships, no one may kick, bar or mute
  the owner or any key that chains to its principal key (LANE-25).
- **Pull-request author**: The outside party whose commits a foreign room's
  bundle describes, matched through their commit-signing identity and a
  **binding statement** that identity signs, naming the bundle's principal key
  (LANE-15).
- **Delegate**: Any agent that receives a delegation, a subagent included
  (OWN-24). It keeps its own principal.
