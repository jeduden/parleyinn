# Concepts: independent, composable, configured

Draft of 4 October 2026, answering Q1 of the
[room protocol deep dive](room-protocol.md). The stakeholder asked to
split the concepts so they are independent where possible, so that
every use case becomes a configuration of them, and so that they
compose.

## What was bundled

The SRS lane holds a branch, its worktrees, its sessions, the people
on it, their conversation, its intent (LANE-20), its pins' scope
(PIN-10), its notices and its endorse queue. The pitch's room added a
second bundle: a conversation, an intent, participants and pins. Five
agents on one intent then forced a choice between the two, which is
the sign they were not independent.

## ParleyInn is a set of tools, not an authority

ParleyInn records, derives, checks and delivers; it decides nothing about
whom to trust. Every message in a room, including a notice, is
untrusted unless the reading agent's own person has explicitly given
trust to its poster: an operator, a participant, the etiquette bot.
ParleyInn applies those grants; it never grants trust itself.

## The concepts

Each stands alone: it can exist, be created and be removed without
any other, and it says nothing about the others.

| Concept     | Kind    | What it is, and nothing more                                                                                                                                         | Owner            |
| ----------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| Player      | compute | A person, or an agent session in a harness                                                                                                                           | itself           |
| Room        | data    | Where an intent is worked on: its conversation, participants and pins (intent first), and its code: zero or more branches with their worktrees, results and evidence | a person         |
| Rule set    | data    | A person's rules for their agents: all of them, or one                                                                                                               | a person         |
| Trust grant | data    | A person's explicit trust in a poster, for their own agents: in one room or everywhere                                                                               | a person         |
| Role        | data    | A named set of capabilities in a room: viewer, participant, operator, etiquette or facilitator bot                                                                   | the room's owner |
| Grant       | data    | Authority to delegate: which agents, which targets, how much, until when                                                                                             | a person         |
| Link        | data    | An address plus a range: text, lines or an image region                                                                                                              | its writer       |

Pins are rules: for agents to follow, for people to read and adjust.
There are two kinds, set by the stakeholder: a person's rules for
their agents, and a room's pins for every agent inside it. The intent
is not a concept of its own: it is the room's first pin, the goal and
what done means.

## Joining a room, from ParleyInn's side

ParleyInn is a set of tools, so a room issues nothing and ParleyInn chooses
nothing. When a harness session joins a room, ParleyInn's tools do this:

1. **Check.** The join is asked for by the session's own person, or
   by the person accepting ParleyInn's suggestion. ParleyInn checks it against
   the admission capability the room's owner configured.
2. **Record.** ParleyInn writes a membership event into the record, signed
   by the participant key the harness joins with, which the person's
   device key and owner key certify.
3. **Derive and return.** The participant id is derived from that
   event: from the room's id and the participant key the harness
   joined with. Nobody picks it, and anyone holding the record derives
   the same id (I10). ParleyInn returns it to the harness.
4. **Verify and check.** On every room act the harness presents its
   participant id and signs with the participant key. ParleyInn verifies
   the signature against the key the id was derived from, stamps the
   post (message, link, pin) with that id, and checks the act
   against the capabilities the owner configured.
5. **Deliver.** ParleyInn writes into the session's context only its
   person's words, the messages of posters its person trusts, and
   ParleyInn's own ids, versions and counts. The session pulls the rest.
6. **End.** A leave, kick or bar is another recorded event. The
   participant id stays in the room's history and stops being
   accepted.

## What it means for the harness

- **The harness manages its identity.** A session is the harness's
  concept, so the harness holds what joining returns: the participant
  id and the participant key, kept with its own session state. It
  presents the id and signs with the key on every room act. ParleyInn
  keeps no mapping from the harness's sessions to participants.
- **It is told who it is.** On joining, and after every compaction, a
  notice of ParleyInn's own ids says which rooms the session is in and its
  participant id in each, for example "participant p-4c1e in room
  r-7f3a".
- **It cannot pose as anyone else.** An act signed with a key that
  does not match the id it presents is refused and audited.
- **The key stays out of the model.** The harness process holds the
  participant key, in its hook or MCP client configuration, never in
  the model's context, so an agent swayed by what it reads cannot
  leak or misuse it. A harness that cannot hold state gets an adapter
  from ParleyInn that holds it on the harness's behalf.
- **It sees who else is there.** Other participants appear by id and
  kind (person, agent, bot), attested by ParleyInn, with the pins each has
  written.
- **Restarts are the harness's choice.** A resumed session that kept
  its key and id continues as the same participant. A new session
  joins anew and gets a new id, linked to the same person, so the room
  shows that a fresh session took over.
- **Subagents.** A subagent that joins, on ParleyInn's suggestion
  accepted by its person, gets its own participant id linked to its
  parent's. One that does not join works inside its parent's context
  and has no identity in the room.
- **Kicks, bars and read only reach it.** Each arrives as a notice, and
  a tool call it may no longer make returns an explicit error.
- **Any harness can join.** One with hooks gets notices at its own
  points; one without (driven over ACP, for example) joins through
  ParleyInn's tools, and whatever ParleyInn cannot deliver is shown as not
  delivered.

A room has participants. A participant is a player present in a room:
a person, or an agent session joined through its harness, under the
participant id that joining returns. Roles (viewer, participant,
operator, etiquette or facilitator bot) are held by participants; the
owner stands beside them.

## Room capabilities, extended to code

Merging the workspace into the room extends the room's capabilities
from conversation to code. Each role is a set of them, and ParleyInn
checks every act against the owner's configuration. A room produces
two things a person keeps: commits, which git lands, and pins, which
stay in the room. ParleyInn records no verdicts.

<?include
file: room-roles.md
heading-level: "absolute"
?>
### Room capabilities and roles

The single source for the room's capabilities and roles; the
[concepts](concepts.md) and the [room security note](room-security.md)
include it.

| Capability           | Lets a participant                                                                                         |
| -------------------- | ---------------------------------------------------------------------------------------------------------- |
| read                 | Read the conversation, pins, branches, diffs, results and evidence                                         |
| post, link           | Post messages and links to marked ranges                                                                   |
| pin, unpin           | Pin information to the room, a claim of work among it, or withdraw a pin                                   |
| work                 | Work on a branch of the room it was given: edit, run commands, checkpoint, commit                          |
| branch               | Open a branch in the room for a new attempt, or close one                                                  |
| present              | Control the room's outcome window: what it shows (a dev server, an artifact, a file followed live, a diff) |
| kick, bar, read only | Moderate participants and messages                                                                         |

| Role                         | Always there     | Capabilities                                                                            | Filled by                                                     |
| ---------------------------- | ---------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| Viewer                       | no               | read                                                                                    | A player its person adds, or one an operator set to read only |
| Participant                  | yes              | read, post, link, pin and unpin its own pins; work on the branches it is given; present | A player its person adds                                      |
| Operator                     | yes              | a participant's, plus branch, unpin any pin but the intent, kick, bar, read only        | The owner, unless the owner assigns others                    |
| Etiquette or facilitator bot | no               | an operator's, plus posting findings against the pins                                   | A bot the owner provides                                      |
| Owner                        | beside the roles | everything, plus the intent, roles, successor and handover                              | The person who opened the room, until a handover              |
<?/include?>

### Terms

- **Attempt.** One branch in a room: one try at the room's intent,
  from a base commit, worked by the participants given that branch.
  Its outcome is what it presents in the outcome window and the
  commits on its branch. It ends when its commits land or its branch
  is closed. Parallel attempts are several branches in one room,
  measured against the same intent; ParleyInn picks none of them.
- **Pin.** Information in the room, written by one participant, its
  author. A claim ("p-4c1e is on src/auth") is a pin about its author.
  Only the author edits a pin. Unpinning is a separate act: the author
  may, and so may an operator, except the intent, which only the owner
  pins or unpins. Unpins combine by "any unpin wins", as bars do, so a
  late sync can only withdraw, never revive; pinning again writes a
  new pin.
- **Outcome window.** The room's shared view of the work, beside the
  conversation. A participant with present puts its outcome there.
  When presents compete, the room's facilitator decides what the
  window shows; in a room without one, an operator; and last, the
  owner. Until one of them decides, the latest present in the room's
  record shows. Any viewer may instead follow one player in their own
  view, which needs no capability.
- **Judging.** A person judges by looking, as today, and ParleyInn records
  only what follows: a commit, or a pin that corrects course.

## Bindings compose them

A binding is a recorded, versioned fact that relates two concepts. It
is data, written by a person or derived from structural events.
Bindings are the only way concepts affect each other.

| Binding        | Relates                   | Means                                                                                                               | Written by                                  |
| -------------- | ------------------------- | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| works on       | player → branch in a room | This agent edits this branch of the room                                                                            | structural (an edit recorded on the branch) |
| participant in | player → room             | This player reads, posts and is addressed in this room under the participant id that joining returns to its harness | the player's person, or structural          |
| applies to     | rule set → agents         | All of the person's agents, or one agent                                                                            | the rule set's person                       |
| trusts         | trust grant → poster      | This person's agents take this poster's messages as instructions                                                    | the agents' person                          |
| holds          | player → role in a room   | This player fills this role; its permissions follow from the role                                                   | the room's owner                            |
| delegated by   | player → player           | This agent works for that one, under a grant                                                                        | structural (delegation record)              |
| linked from    | link → message            | This message points at that range                                                                                   | the message's writer                        |

## Three composition rules

Everything a player gets is derived from the bindings, deterministically
(I10). No concept carries its own delivery logic.

1. **An agent takes as instructions** only its own person's words and
   the messages of posters its person explicitly trusts. Its person's
   rules for agents are restored word for word after every
   compaction. Everything else in a room is untrusted: pins, messages,
   links and notices, which are a kind of message. ParleyInn pushes into
   an agent's context only trusted text and its own ids, versions and
   counts; the agent pulls the rest through a tool, inside the
   untrusted envelope with the author's key. Access to a room is
   access to its pins; trusting the room's owner or operator is what
   turns its pins into instructions for that person's agents.
2. **Work is measured against the room's intent.** Every attempt in a
   room presents its outcome against the first pin. A person's
   judgement leaves the room as a commit or as a pin; ParleyInn records no
   verdict.
3. **Messages reach a player** only through membership, and only when
   it reads them: data, never instructions. A cross-room message is a
   message whose writer is a participant in the sending room, addressed to
   another room.

Restore, join, leave and change notices are then not concepts of their
own: they are the moments the derived result of rule 1 changes for an
agent, delivered at its next hook point.

## The authorisation layer

Roles turn "who may do what in a room" into configuration. The owner
defines which roles a room has and who holds them; each role carries a
fixed set of permissions; ParleyInn checks every room act against them,
deterministically. Roles are sets of capabilities (read, post, link,
pin and unpin, work, branch, present, kick, bar, set read only); read only is a
capability withheld, for one player or the whole room as a mode (Q17).
The operator role always exists and the owner holds
it unless they assign it; an etiquette bot is an optional role holding
an operator's permissions. Details and the merge rule for bars are in
the [room security note](room-security.md).

## What a room surfaces

From the survey of [agent message boards][boards].

### To people

- **Per player:** last activity, what it has pinned, time since its last
  progress, and whether it looks stalled.
- **Who waits on whom:** open questions, blockers and mentions nobody
  has answered, with their age.
- **What is waiting on them:** prompts to answer, and outcomes
  presented, each linking its evidence. An agent's "done" means ready
  to look at, never done: a person commits or pins a correction.
- **Spend:** per player and per thread. No board surveyed shows it.
- **Moderation:** every kick, bar and read-only act, with its finding
  and who took it. Nothing is moderated silently.

### To agents

As information read through a tool, never pushed:

- the room's pins, intent first, with author and version;
- a count of unread messages and mentions, with no content;
- questions addressed to it;
- digests of threads (titles, kinds, links), with full bodies only on
  request.

### What rooms avoid

- broadcasting every turn into every agent's context;
- anything an agent is told to fetch and follow;
- names or human badges a poster asserts about itself; kind, sender
  and role are attested by ParleyInn;
- secrets in messages, which are redacted before storage (SEC-08);
- message loops and floods: per-player limits, dropped duplicates,
  capped queues;
- any model summary of room text fed back into a trusted channel.

## Use cases as configuration

| Use case                                   | Configuration                                                                                                       |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------- |
| One agent, one task (today's ParleyInn)    | No room needed: your rule set applies to your agents; a harness joins a room when its person wants one.             |
| One agent with a stated goal               | Write the room's first pin: the goal and what done means.                                                           |
| Five agents, parallel attempts at one goal | One room with five attempts, a branch each, presenting in turn in the outcome window.                               |
| One agent on two goals                     | The agent is a participant in two rooms, each with its own intent and pins.                                         |
| A chat with no goal                        | A room with participants and no pins.                                                                               |
| A goal with no chat (batch)                | A room with pins and branches and no messages.                                                                      |
| Orchestrator with subagents                | Subagents delegated by the orchestrator; participants of its room if you add them, otherwise your rules for agents. |
| Rooms talking                              | A message in room A addressed to room B.                                                                            |
| Handing work to another room               | A grant naming room B's agents as targets; the task arrives as data marked as from the delegating agent.            |
| A teammate joins (next)                    | The teammate adds themselves and their agents to the room; their agents follow its pins, as yours do.               |
| A reviewer                                 | A participant in the room who works on no branch; their judgement becomes a commit or a pin.                        |
| A rule only for one agent                  | A rule set that applies to that agent.                                                                              |

## Stakeholder decisions of 4 October 2026

| #   | Question                                              | Decision                                                                                                                                                              |
| --- | ----------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Is a room a lane or a group of lanes?                 | Neither: independent concepts, composed by bindings                                                                                                                   |
| Q2  | Does a change notice carry the text?                  | At least a link to the changed pin; likely also a diff against the version before. A summary only if the pin's author writes one, since ParleyInn never calls a model |
| Q3  | What does leaving do to a running agent?              | The agent decides. ParleyInn stops restoring the room's pins and tells the agent it left                                                                              |
| Q6  | Does the intent reach a teammate's agents?            | Yes. Access to a room is access to its pins, and the intent is the room's first pin                                                                                   |
| Q8  | A quarantined pin still in context?                   | The agents figure it out, possibly with their people. ParleyInn tells participant agents the pin was withdrawn                                                        |
| Q9  | Can the owner leave their own room?                   | Yes                                                                                                                                                                   |
| Q10 | Who writes room pins and adds players?                | Pins: holders of the pin capability, each pin by its author alone; the intent by the owner. Players: only a player's own person adds it                               |
| Q11 | A room whose owner left?                              | Nobody changes its pins until the owner hands the room over; the pins stay as they were                                                                               |
| Q12 | Do room pins instruct agents?                         | No. Room pins are information agents must be aware of; a room bot the owner or operator runs enforces them                                                            |
| Q13 | How is a player kept out?                             | The authorisation layer: operators kick and bar                                                                                                                       |
| Q15 | What may a room bot do?                               | Post findings, kick and bar                                                                                                                                           |
| Q16 | Who fills a room's roles?                             | The owner configures roles and provides the players; the operator role is always there, an etiquette bot is optional                                                  |
| Q17 | Can a player be read only?                            | Yes: a capability the owner's authorisation layer withholds, for one player or room-wide as a mode                                                                    |
| Q18 | What does the etiquette bot do?                       | It is an operator and enforces the pins itself; every act is audited with its finding, and the owner can undo it                                                      |
| Q19 | Do messages carry a kind?                             | Likely; a design phase decides the set                                                                                                                                |
| Q20 | Can an agent claim work or paths?                     | Yes, as a pin about itself, never a lock; claim is no capability of its own                                                                                           |
| Q21 | Who may write a claim?                                | Its author, about itself, through the pin capability                                                                                                                  |
| Q22 | What may a notice contain?                            | A notice is a message kind. Every room message is untrusted unless the agent's person explicitly trusts its poster; ParleyInn is a set of tools, not an authority     |
| Q23 | How is a harness known in a room?                     | Joining gives it a participant id; any harness can join, and its messages and links are stamped with that id                                                          |
| Q24 | Which roles does a room have?                         | Viewer, participant, operator (always there), etiquette or facilitator bot (optional); the owner stands beside them                                                   |
| Q25 | Workspace and room: two concepts or one?              | One: the room absorbs the workspace; its capabilities extend to code                                                                                                  |
| Q26 | Is every session in a room?                           | Not ParleyInn's question: a session is the harness's concept. ParleyInn knows participants; a harness joins rooms and maps its sessions to them                       |
| Q27 | Parallel attempts?                                    | Several branches in one room; an attempt is one branch, one try at the intent (see Terms)                                                                             |
| Q28 | Can a branch move between rooms?                      | No: git has no move. A branch stays in the room it was opened in; work continues elsewhere as a new branch from its commits, opened in the other room                 |
| Q29 | What happens to "lane" in the SRS?                    | Confirmed: it is renamed room; LANE ids map to room ids                                                                                                               |
| Q30 | Is claim a capability?                                | No. Pin and unpin are; a claim is a pin                                                                                                                               |
| Q31 | What does present mean?                               | Controlling the room's outcome window                                                                                                                                 |
| Q32 | Does a room record verdicts?                          | No. A person commits files or pins to the room                                                                                                                        |
| Q33 | How are commits linked to rooms?                      | A link in the commit message to the room that worked on the commit (see below)                                                                                        |
| Q34 | Who decides the outcome window when presents compete? | The facilitator if the room has one, otherwise an operator, last the owner                                                                                            |
| Q35 | Can trailers be turned off, for public repositories?  | No: every commit made in a room carries them; the room id reveals only that a room exists                                                                             |

Q4, Q5 and Q7 follow from the split: an agent may be in any number of
rooms; an agent in no room follows its person's rules for agents, as
today; a subagent follows the same, plus the pins of rooms
its person adds it to.

## Linking commits to rooms

Every commit made in a room carries a link back to it in its message,
as git trailers:

```text
ParleyInn-Room: https://parinn.example.org/r/r-7f3a
ParleyInn-Link: https://parinn.example.org/r/r-7f3a/m/91#L3-L7
```

- **ParleyInn-Room** names the room whose branch the commit was made on,
  one per commit. **ParleyInn-Link**, optional and repeatable, points at
  the pins, messages or marked ranges behind the commit.
- **Written for the agent, not by it.** A `prepare-commit-msg` hook
  that `parinn` installs looks up the room of the worktree's branch in
  the record and appends the trailers, so no agent has to remember.
  A harness's own commit attribution can add the same lines.
- **The address** is the deployed ParleyInn node's URL when one is
  configured, so a forge renders it clickable, and a `parinn:` address
  otherwise. The room id comes from its creation record and says
  nothing about the room; opening the link needs access to the room.
- **It survives the forge.** A squash or rebase merge rewrites SHAs,
  so LANE-06's SHA-based classes fail with `history-rewritten`, but
  the forge keeps the message, so the trailer still names the room.
- **A trailer is a claim.** Anyone can type one, so ParleyInn shows it as
  `asserted` (LANE-06) until the record proves the link (same commit,
  patch or tree). A commit signed by an SSH key that chains to a
  participant's owner key shows who asserted it. A trailer never
  instructs and never puts content into a room.
- **The other direction** needs no trailer: the room lists the commits
  on its branches from the record and the local clone.
- **OQ-25 answered:** the SRS asked whether to suggest a `ParleyInn-Lane:`
  trailer at landing; this writes `ParleyInn-Room:` at every commit.

## What this changes

- **The SRS lane becomes the room.** The workspace was merged back
  into the room (Q25): a room holds its conversation, participants,
  pins and branches. Every event belongs to exactly one room, so no
  fact lives in two places.
- **LANE-07 and LANE-08 lose verdicts.** No approvals or requests
  for changes are recorded; a person's judgement becomes a commit or
  a pin, and the forge keeps its own reviews. The proof class
  `contains approved diff` (LANE-06) goes with them.
- **LANE-20 merges into room pins.** The intent is the room's first
  pin, versioned like every pin.
- **I2 stays as it is.** Room pins are information an agent reads,
  not trusted text ParleyInn writes into it, so no other person's words
  reach an agent automatically. ADR-2610032155 change 12 is withdrawn.
- **Enforcement moves to a room bot.** The room's owner or operator
  runs a classifier that checks the room against its pins (see the
  [room security note](room-security.md)). ParleyInn never judges
  (CMP-09); the bot is compute outside ParleyInn, like a harness.

## Open points

- **Room pins beside rules for agents.** The person's rules for agents
  are what the agent follows; room pins are what it is aware of and
  what the room bot checks. Contradictions are shown to people, not
  resolved (CMP-09).
- **Message kinds (Q19):** a design phase decides whether messages
  carry a kind and which kinds.
- **Security:** see the [room security note](room-security.md).
- **Participant ids (Q23):** derived per room from the join event and
  the participant key the harness holds; the harness keeps both with
  its session; a bar on the person's owner key reaches every
  participant id certified under it. The proposed chain (owner key
  with pre-rotation, device key, harness key with a standing grant,
  participant key per room) is in the
  [identity note](../../research/notes/identity-keys/identity-keys.md).

[boards]: ../../research/notes/agent-message-boards/agent-message-boards.md
