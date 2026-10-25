# Persona review, round 10: blind review of pitch v12

Round 10 gave all nine personas the text of pitch v12 and nothing
else, in up to three loops. Each said whether it would try ParleyInn, what
lands, what is unclear or overclaimed, three rewrites and one cut.

## Loop 1: the first v12 draft

| Seat                    | Try it? | What lands best                                               |
| ----------------------- | ------- | ------------------------------------------------------------- |
| Fleet developer         | Partly  | What needs you, ranked; pins back after compaction            |
| Returning owner         | Partly  | Catch-up lines linked to the record; the ranked queue         |
| Reviewer                | Partly  | Every commit knows its room; links to diff and log lines      |
| Live collaborator       | Partly  | Every message under its agent's name; follow any player       |
| OSS maintainer          | Partly  | Everything else in a room is untrusted                        |
| Multi-machine developer | Partly  | No central service; commit links outlive the room's machine   |
| Platform operator       | Partly  | Shows every change, removes cleanly; budgets                  |
| Security officer        | Partly  | Cross-room posts pull-only; the residual risk stated honestly |
| Agent                   | Partly  | Cross-room posts pull-only; pins back after compaction        |

| #      | Finding                                                                                                   | Seats                         | Change in loop 2                                                                                 |
| ------ | --------------------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| P10-1  | Agents' pins seem to come back to every agent like the person's rules: agent text arriving as instruction | security, agent               | Only your pins come back word for word; agents' pins are notes, marked by author, read on demand |
| P10-2  | The browser reads as the only surface; the terminal sits in the last clause                               | fleet                         | The terminal is named beside the browser at the start                                            |
| P10-3  | "Speak once and every agent hears you" is a trust hole once a guest is in the room                        | collaborator                  | "All your agents hear you"                                                                       |
| P10-4  | Where review happens is unsaid; it could happen twice                                                     | reviewer                      | Review and land on the forge as always                                                           |
| P10-5  | The commit link: to what host, and does a squash keep it?                                                 | reviewer, OSS, owner, machine | The link is in the message, which a squash keeps                                                 |
| P10-6  | No search, no exact recall                                                                                | owner, agent                  | "Find it again" added                                                                            |
| P10-7  | Catch-up is per room                                                                                      | owner                         | One page across all rooms                                                                        |
| P10-8  | Integrity and redaction are not mentioned                                                                 | owner, security, OSS          | Append-only record; secrets redacted before writing                                              |
| P10-9  | "Connects to nothing" beside a localhost screen                                                           | security, operator            | "Opens no connection off your machine; listens on localhost only"                                |
| P10-10 | "Never slows your harness" is an unbacked absolute                                                        | fleet, operator, owner, agent | Every hook has a deadline and fails open                                                         |
| P10-11 | Trust by display name or by key?                                                                          | security, OSS                 | "By key"                                                                                         |
| P10-12 | Own machines peering is missing                                                                           | machine                       | Laptop, server and sandboxes share rooms, next                                                   |
| P10-13 | Overlap warning: before or after?                                                                         | fleet                         | "The moment a second agent edits a file another has edited"                                      |

Not taken, with the reason:

- **Evidence classes on runs** (reviewer, owner): the stakeholder
  dropped the recorded-versus-claim line in v11 (history item 24).
- **Team rollout, tenancy, purge** (operator): the pitch is for the
  developer; the operator's path is in the SRS.
- **Cutting "follow any player"** (reviewer, agent): the stakeholder
  named it the heart of multiplayer.
- **A lane file for strangers** (OSS): export is P2 and not in the
  room model's pitch yet.

## Loop 2: after the loop-1 changes

The agent seat moved to yes; the other eight stayed at partly, each
with fewer and narrower findings.

| #      | Finding                                                                             | Seats                     | Change in loop 3                                                                                      |
| ------ | ----------------------------------------------------------------------------------- | ------------------------- | ----------------------------------------------------------------------------------------------------- |
| P10-14 | "Instruction only when you pass it on" against "posters you trust": which is it?    | agent, security           | "ParleyInn delivers as instructions only your words and those of posters you trust"; the rest is data |
| P10-15 | What does "a post waits" carry? A preview would push untrusted text                 | security, agent           | "Only that a post waits and who sent it"                                                              |
| P10-16 | Recall is marked by author, not as untrusted                                        | agent, OSS                | "Wrapped as untrusted data with who wrote it"                                                         |
| P10-17 | Who creates a room, and how?                                                        | fleet                     | One command or click from the branch you are on; running agents join when you say so                  |
| P10-18 | No stop                                                                             | fleet                     | Steer or stop from the room, wherever the harness allows it (OWN-14, OWN-15)                          |
| P10-19 | Whose run is "the run behind it"?                                                   | reviewer (twice), owner   | Each run as the hook recorded it: command, exit code, commit                                          |
| P10-20 | The commit link is dead for a reviewer until rooms are shared                       | reviewer, OSS, fleet      | "For you now and for whoever you share the room with next"                                            |
| P10-21 | "Append-only" is no proof; redaction overclaims; fail-open looks silent             | owner, security, operator | Hash-chained, missing or altered entries shown; "detected secrets"; every hook failure counted        |
| P10-22 | Catch-up names no failed hooks or redactions, and is not ranked                     | owner                     | Ranked, across rooms and attempts; gaps named: failed hook, redaction, missing entry                  |
| P10-23 | When does a pin change reach an agent, and in what order?                           | agent                     | "Word for word, in order"; a change at the next turn                                                  |
| P10-24 | Peering says nothing of partitions                                                  | machine                   | Each machine works offline and merges on reconnect                                                    |
| P10-25 | How is a teammate's message passed on; no handover                                  | collaborator              | Forward in one step; hand them the room                                                               |
| P10-26 | A localhost page that renders agent content and answers prompts is a browser target | security                  | "Behind a token"; the sandboxing detail stays in the SRS                                              |

Not taken, with the reason:

- **"Multiplayer" in the headline** (collaborator, fleet, operator):
  it says "you and your agents", which the first release delivers;
  the stakeholder set the line.
- **Purge, retention and tenancy** (operator): the operator's path is
  in the SRS, not the developer pitch.
- **Signed room bundles for strangers** (OSS): export is P2.
- **Cutting "follow any agent"** (owner, agent): the stakeholder named
  it the heart of multiplayer; "player" became "agent" to stay defined.

## Loop 3: final

| Seat                    | Try it?             | Blocking                                      |
| ----------------------- | ------------------- | --------------------------------------------- |
| Fleet developer         | Partly, leaning yes | none                                          |
| Returning owner         | Yes                 | none                                          |
| Reviewer                | Partly              | none                                          |
| Live collaborator       | Partly (needs Next) | none                                          |
| OSS maintainer          | Partly              | none                                          |
| Multi-machine developer | Partly (needs Next) | none                                          |
| Platform operator       | Partly              | purge and retention, out of the pitch's scope |
| Security officer        | Partly              | none                                          |
| Agent                   | Yes                 | none                                          |

Wording taken into the final v12: attempts in their own worktree; runs
"not as the agent reported it"; the commit link survives "a squash that
keeps commit messages" and opens "for others once you share the room";
recall "found by search or behind a link"; catch-up "since you left";
peers "enrolled by key"; teammates join by invitation, watch live and
talk with you; forwarded words "marked as theirs and forwarded by you";
entries "missing or altered since it was written".

Carried to the requirements, not the pitch:

- **Trusting an agent's key** (security): trusting one of your own
  agents as a poster would launder what it read; the trust-grant
  requirements must say whether agents can be trusted posters.
- **Forwarded text** (agent, security): forwarding marks the original
  author and the forwarder (R9-8); whether it then counts as the
  forwarder's instruction is the endorsement rule (OWN-08).
- **Integrity before landing** (owner): show whether a room's record is
  whole before a commit from it is kept.
- **Outbound-only peers** (multi-machine): a sandbox that can only dial
  out must join and sync, and its record must outlive it.
- **Retention and purge** (operator): purge as an audited marker the
  chain accepts, not an altered entry.
- **Commit trailers opt-in** (fleet): not taken; the stakeholder decided
  they are always on (concepts Q35).
