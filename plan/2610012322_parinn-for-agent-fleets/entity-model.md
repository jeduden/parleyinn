# Entity model: data, and compute with data

Draft of 4 October 2026, from the stakeholder's ask to sort out which
entities are data and which compute and hold data. The pitch's rooms,
links, messages between rooms and delegation rest on this split.

## The split

**Data** is inert. It can be stored, read, linked and shown. It never
acts and never instructs. Every piece of data has an address in the
record and a writer.

**Compute with data** acts: it reads data, decides or executes, and
writes new data. It holds state of its own (a context window, a
process, a running app). Only compute entities act, and only one kind
of them, a person, can instruct an agent.

## Data

| Entity             | Written by                                        | What it is                                                                                                                                                                           |
| ------------------ | ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Event              | the compute entity behind it                      | One entry of the record: a message, an edit, a command, a result                                                                                                                     |
| Intent             | the room's owner                                  | The room's first pin: the goal and what done means                                                                                                                                   |
| Pin                | a participant, its author                         | A rule: a person's rule for their agents, which they follow, or a room's pin, which participant agents are aware of and a room bot checks; a claim of work is a pin about its author |
| Room message       | a player, person or agent                         | A post addressed to one room; every player in it can read it                                                                                                                         |
| Cross-room message | a player in another room                          | A post from one room to another, arriving as data in the target room                                                                                                                 |
| Mark and link      | a person or an agent                              | An address plus a range: characters, lines or an image region                                                                                                                        |
| Snapshot           | ParleyInn, from a compute entity                  | A screenshot of the app, a diff, a file version, a test log                                                                                                                          |
| Presentation       | a participant with present                        | What the room's outcome window shows: a dev server, an artifact, a file followed live, a diff                                                                                        |
| Checkpoint         | ParleyInn, from the worktree                      | The state of a worktree at a point                                                                                                                                                   |
| Commit trailer     | the commit's author, through ParleyInn's git hook | `ParleyInn-Room:` and `ParleyInn-Link:` lines linking a commit to the room that worked on it; asserted until the record proves it                                                    |
| Grant              | a person                                          | A delegation budget: which agents, which targets, how much, until                                                                                                                    |
| Room               | a person                                          | One intent's shared space: its players, messages, links and pins, and its code: the branches worked on for the intent                                                                |
| Membership         | a person, or structural                           | A player joining or leaving a room: added or removed by its person, or derived from a session start or end or a delegation                                                           |
| Role and holder    | the room's owner                                  | A room's roles, their capabilities and who fills each: viewer, participant, operator, etiquette bot; the owner beside them                                                           |
| Kick and bar       | an operator                                       | Removing a player now, or keeping it out, under the authorisation layer                                                                                                              |
| Branch in a room   | derived by ParleyInn                              | A branch and its worktrees, results and evidence, as the record holds them; what the SRS called a lane                                                                               |
| Status and queue   | derived by ParleyInn                              | What needs whom, rebuilt from the record (I10)                                                                                                                                       |

## Compute with data

| Entity                  | Holds                     | Acts by                                                   | Trust                                                                                      |
| ----------------------- | ------------------------- | --------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Person (player)         | keys, judgement           | writing intents, messages, pins, commits, grants          | The only source of instructions to their own agents                                        |
| Agent                   | a context window          | calling tools through its harness                         | Swayed by what it reads; ParleyInn cannot stop that, only limit it                         |
| Harness                 | a process, its transcript | running the agent's tool calls                            | Its own input channel is the person's                                                      |
| Subagent and delegate   | a context window          | as an agent, within its parent's grant                    | No looser than its parent                                                                  |
| Etiquette bot           | a classifier, its key     | posting findings, kicking and barring players in one room | An optional role the room's owner provides, run outside ParleyInn; its findings are claims |
| Running app, dev server | a process, its pages      | executing code an agent wrote                             | Untrusted; its screens are untrusted data                                                  |
| Command and test run    | a process, briefly        | executing, then exiting with a status                     | Its output is untrusted data; its exit status is structural                                |
| ParleyInn               | the record, derived state | recording, deriving, delivering, displaying               | Deterministic, no model, never judges (I10, CMP-09)                                        |
| Peer node               | another person's record   | syncing data by key                                       | Its data arrives as theirs, untrusted                                                      |
| Model provider          | outside ParleyInn         | serving the harness's model calls                         | Outside every ParleyInn boundary                                                           |

## Rules that follow

1. **Data never instructs.** Only a person's words reach an agent as an
   instruction, through the harness's input (I2). Messages, links,
   snapshots and results reach an agent only when it asks, marked
   untrusted.
2. **Compute writes data, attributed.** Everything a compute entity
   produces enters the record as data under its writer: an agent's
   message, a command's output, a screenshot of a running app.
3. **Links point at data, never at compute.** You cannot link to a
   running app; marking it takes a snapshot first, and the link points
   at the snapshot. A link resolves the same way forever.
4. **ParleyInn computes but never judges.** It records, derives and
   displays deterministically. Judging an outcome is a person's act;
   presenting it is the agent's.
5. **Delegation is the one bounded path from compute to compute.** An
   agent hands a task to another agent only under a grant a person
   wrote; the task arrives as data marked as from the delegating agent,
   within the grant's budget.
6. **Untrusted compute is isolated.** A running app executes code no
   person reviewed; its pages stay out of the agent's context unless an
   agent fetches them as untrusted data, and out of the lane view's own
   origin.

## Room protocol: join, participate, leave

Restoring pins is part of being in a room. A restore is ParleyInn writing
trusted text into one agent's context at a point the harness offers
(INJ-01): after compaction, at startup, resume or clear. The room
protocol says when each participant gets what.

**Join.** A person adds a player to a room, recorded as a membership
event: an agent they run, a subagent or delegate an agent starts in
the room, or, later, a teammate. An agent joining gets, word for word:
the intent in force, its own person's pins for the room, and a fixed
notice naming the room and its players by id and key fingerprint, never
by a name someone chose. A subagent gets the same through its own start
hook where the harness offers one (Claude Code and Codex do); without
one it is shown as "rules not delivered".

**Participate.** While a participant, an agent:

- has the intent and its person's pins restored after every
  compaction, at startup, resume and clear;
- gets a changed intent or pin at its next turn, as a fixed notice
  naming the new version; the change is the person's act, so the
  notice is a write I2 allows;
- reads room messages, cross-room messages and links when it asks,
  marked untrusted, and posts its own;
- takes instructions only from its own person.

**Leave.** A person removes a player, the agent's session ends, or a
delegate returns its result. The membership event records it. ParleyInn
stops restoring the room's pins to that agent; everything it did stays
in the record.

In a shared room each person's pins reach only that person's agents.
A teammate who joins sees every pin in the room, and their own agents
restore their own person's pins.

## Where this settles the open questions

- **Messages in a room** are data: every player in the room can read
  them; only the person's own words reach their agents as
  instructions.
- **Messages between rooms** are data: an agent may write one, and it
  instructs no one until a person passes it on.
- **Handing tasks to another room** is rule 5: possible only under a
  grant, not by message.
- **Pinning in a room** follows the room protocol above. What stays
  open is harness support per harness and the stakeholder's decisions
  in the [room protocol deep dive](room-protocol.md).
