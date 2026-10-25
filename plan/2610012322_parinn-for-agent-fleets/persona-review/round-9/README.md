# Persona review, round 9: blind review of the room model

Round 9 gave all nine personas a self-contained description of the
room model of 4 October 2026, about 600 words drawn from the
[concepts](../../concepts.md) and the
[room security note](../../room-security.md), and nothing else. Each
said whether it would work, what helps most, its gaps and
contradictions, and the one requirement it would insist on.

## Verdicts

| Seat                    | Would it work?              | What helps most                                                          |
| ----------------------- | --------------------------- | ------------------------------------------------------------------------ |
| Fleet developer         | Partly                      | Who waits on whom; outcomes ready for a verdict                          |
| Returning owner         | Partly                      | Outcomes ready for a verdict, with evidence; moderation shown            |
| Reviewer                | No: stops at the verdict    | Outcomes ready for a verdict; links to diff and log lines                |
| Live collaborator       | Not yet: needs sharing      | Nothing another player writes instructs your agent                       |
| OSS maintainer          | Not yet: no lane to receive | Messages, links and other rooms are data, read on request                |
| Multi-machine developer | Not yet: sharing is thin    | Single-writer facts and merges without clocks                            |
| Platform operator       | No: written for one desk    | Nothing another player writes instructs your agent; bars by key          |
| Security officer        | Not yet: three push paths   | Nothing another player writes instructs your agent; claims are not locks |
| Agent                   | Partly                      | Own rules restored word for word; counts without content                 |

The trust split (data, never instructions) is the one part every seat
endorsed.

## Findings the room model must fix

| #     | Finding                                                                                                                                                                                                                                    | Seats                                             |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------- |
| R9-1  | Notices push other players' text: room names, pin diffs, the text of questions addressed to an agent. Notices must carry only ids, versions and counts; everything else is pulled                                                          | agent, security, OSS, collaborator                |
| R9-2  | "The owner provides the players that fill roles" reads as contradicting "only a player's own person adds it": state that the owner provides only their own players, such as their bot                                                      | operator, collaborator, reviewer                  |
| R9-3  | Admission is undefined: nothing says who may join a room, so a person could add themselves anywhere, and a kick is undone by re-adding                                                                                                     | collaborator, fleet                               |
| R9-4  | An owner who leaves deadlocks the room: no operator, pins frozen, nobody to hand over. A successor must be named beforehand                                                                                                                | operator, owner, collaborator, multi-machine      |
| R9-5  | Pins are called information, yet work is "judged against" the intent and leaving speaks of "rules": the wording must stay consistent                                                                                                       | fleet, agent, OSS                                 |
| R9-6  | The etiquette bot can be injected into kicking players, can silence a reviewer when the owner is the author, and can set a room read only over a weekend; its key, connection, scope and rate are unstated                                 | security, reviewer, owner, collaborator, operator |
| R9-7  | "Looks stalled" and "thread digests" read as judgements; both must be mechanical rules, with no model                                                                                                                                      | owner, security, OSS                              |
| R9-8  | Passing something on is undefined: no one-step forward, no record of who forwarded what, no state the writer can see                                                                                                                       | collaborator, agent, security                     |
| R9-9  | Bars by key are evaded by minting a new key; bars must name the person's owner key, covering every key it certified                                                                                                                        | operator, security                                |
| R9-10 | Every room operation that is dropped, refused or fails must be counted and logged, readable without the screen                                                                                                                             | operator                                          |
| R9-11 | Kicks, bars and read only must be told to the player they hit, with the reason; a post refused in read only returns an error                                                                                                               | agent, collaborator                               |
| R9-12 | A harness joining a room receives a participant id, whatever the harness; a subagent run is a child with its own id. ParleyInn suggests joining a session to its workspace's room, and a subagent to its parent's room; the person accepts |                                                   |

## Findings the SRS already answers

The blind text left these out; the SRS holds them, and the room
requirements must keep them:

- **Catch-up across rooms** (VIEW-08) and **search** (VIEW-09).
- **Integrity seals and gaps shown** (VIEW-10, VIEW-11, I6).
- **Exact recall** of an agent's own record (RCL-03).
- **Redaction before storage and on import** (SEC-08).
- **Receiving a lane as a bundle**, verified (LANE-15).
- **Verdicts and approvals**, signed and bound to the head (OWN-27,
  LANE-07).
- **Evidence classes** separating a claim from a recorded run
  (LANE-05).
- **Tracing a landed commit to its lane** (LANE-06).
- **Device and owner keys, enrolment, revocation** (PRV-10, PEER-07).
- **The loopback surface's authentication** (SEC-20).
- **Managed policy** for fleets (SEC-22).
- **Purge and erasure across peers** (SEC-30, SEC-31).
- **Stop, steer and terminal parity** (OWN-03, OWN-14, VIEW-03).
- **Overlap warnings derived from the record** (LANE-13).

## Findings for later phases

- **Sharing transport.** Outbound-only sync through a store the person
  hosts, and copying a sandbox's record off before it is reclaimed
  (multi-machine).
- **The reviewer's gate.** A verdict by the author, or by an agent
  acting for the author, never counts towards a merge gate (reviewer).
  LANE-07 holds approvals; the self-approval rule needs stating.
- **A live view for a joining collaborator,** following the work as
  the owner does (collaborator); the pitch promises it, and sharing
  carries it.

## Stakeholder decisions of 4 October 2026

| Fix   | Decision                                                                                                                                                                                                                                   |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| R9-1  | Open: notices with ids, versions and counts only, explained further                                                                                                                                                                        |
| R9-3  | Yes: admission is a capability of the owner's authorisation layer (invite only, allow list)                                                                                                                                                |
| R9-4  | Yes: the owner names a successor beforehand                                                                                                                                                                                                |
| R9-6  | Yes, for now: the bot's acts are rate-limited and shown to the player they hit; it never acts on the owner or an operator; setting a whole room read only needs a person                                                                   |
| R9-8  | Yes: one-step forwarding, recording writer and forwarder, with a state the writer sees                                                                                                                                                     |
| R9-9  | Yes: bars name the person's owner key, covering every key it certified                                                                                                                                                                     |
| R9-12 | A harness joining a room receives a participant id, whatever the harness; a subagent run is a child with its own id. ParleyInn suggests joining a session to its workspace's room, and a subagent to its parent's room; the person accepts |

R9-2, R9-5, R9-7, R9-10 and R9-11 are clarifications applied to the
room model. The rest go into the requirements with the room model.
