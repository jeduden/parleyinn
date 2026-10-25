# Room security: the two warnings from prior art

Draft of 4 October 2026, on the two warnings in the
[room protocol deep dive](room-protocol.md), applied to the
[concepts](concepts.md) as the stakeholder decided them.

## Warning 1: merging membership across peers

### What went wrong elsewhere

Matrix lets several servers each hold a room and merge their views.
Membership and power levels are authorisation state, and when two
servers' histories conflicted, state resolution could pick a branch
that undid a ban or a power change: a "state reset". It took a
coordinated security release and room version 12 in 2025 to fix
([Project Hydra][hydra]).
The root cause is several parties writing the same authorisation
facts, and a merge rule deciding between them.

### Where ParleyInn is exposed

Only once peering exists (P2): rooms shared between nodes, people
working offline, a teammate's node syncing late. Standalone, one node
holds everything and nothing merges.

### How the decided model avoids it

1. **Every fact has exactly one writer.** Each pin is written by its
   author alone, and the intent by the owner; unpins, like bars,
   combine by "any unpin wins". A player's membership is written only
   by that player's own person. No two people ever write the same fact,
   so no merge ever decides between two people.
2. **Keeping a player out is the authorisation layer's job (Q13).**
   A room's owner assigns roles; a role carries permissions; an
   operator may kick a player (remove it now) or bar it (keep it out).
   A player is in the room when its person has added it, that add has
   not been kicked, and no operator's bar stands. A kick revokes the
   current add, so only the player's person can add it again; a bar
   keeps it out until lifted. Bars name a key, never a name. Bars from
   several operators combine by "any bar wins", so a late or reordered
   sync can only make the result
   stricter. A bar is lifted only by the operator who set it or by the
   owner, and an operator whose role the owner revokes writes no
   further bars.
3. **Within one writer, restriction wins.** Only one person's own
   devices can conflict, for example two laptops offline. Then a
   removal beats an add, and a bar beats an unbar, at the same causal
   position. The conflict is recorded and shown (I6). A revocation is
   never undone by a merge, only by a later explicit act.
4. **Ownership moves only by a signed pair.** A handover is the old
   owner's signed event accepted by the new owner's (LANE-11). An
   owner who leaves without handing over freezes the room's pins (Q11).
5. **Equivocation is detected.** A writer who signs two different
   histories is shown as equivocated (PEER-10), and their later events
   stop counting.

### What stays for membership

- **Withheld events.** A peer that withholds a bar delays it on nodes
  that only hear from that peer. The bar's writer's own node applies it
  at once, and sync stops sending to the barred key. A node holding a
  gap shows it.
- **Content already copied.** A barred player keeps what its node
  already holds; barring stops future content, never past.
- **A test that proves it.** The merge must be property-tested: no
  sequence of deliveries, reorderings or duplications re-admits a
  barred player or revives a removed membership.

## Warning 2: other agents' text as instructions

### What goes wrong elsewhere

AutoGen, LangGraph, CrewAI and the OpenAI Agents SDK hand one agent's
words to another as conversation or as task context, which the model
treats as instructions. Prompt Infection showed an injection spreading
from agent to agent that way; naming the sender cut attacks by about
5%, while naming the sender and marking the content untrusted stopped
them in that study ([arXiv 2410.07283](https://arxiv.org/abs/2410.07283)).

### Trust is given by people, not by ParleyInn

ParleyInn is a set of tools, not an authority. Every message in a room,
notices included, is untrusted for an agent unless that agent's own
person has explicitly trusted its poster, in that room or everywhere.
A person may trust a room's operator, a participant or the etiquette
bot; then that poster's messages, pins included, reach the person's
agents as instructions. Without a grant, the most another poster's
text can do is be read. A grant is the person's act, recorded,
revocable and shown in the room.

### What the decided model does

| Path into an agent                    | Treatment                                                                                                                       |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Its own person's rules for agents     | Trusted; restored word for word                                                                                                 |
| Pins of a room it is a participant in | Information: a fixed notice names the room and pin versions; the agent reads them through a tool, inside the untrusted envelope |
| Messages in the room                  | Data: read only when the agent asks, inside the untrusted envelope                                                              |
| Cross-room messages                   | Data, as above; never start a turn, route work or reach another room                                                            |
| Links                                 | Resolved by recall: untrusted envelope, and the session counts as tainted                                                       |
| A room bot's findings                 | Data in the room, like any post; never instructions to anyone's agent                                                           |

The sender is stamped by ParleyInn from the writer's key, never taken from
a name the writer chose, and the content is marked untrusted: the two
together are what the study found effective.

Because room pins are information (Q12), no other person's words
reach an agent automatically, and I2 stays as it is. A room owner
cannot steer a teammate's agents through a pin; the most a pin can do
is be read, as any message can.

## Roles and the etiquette bot

A room's owner configures its roles and provides the players that
fill them (Q16). ParleyInn holds the roles and enforces their permissions
deterministically; it never judges and never calls a model (CMP-09,
I4).

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

- **The etiquette or facilitator bot** is a player of kind bot with
  its own key, run by the owner or an operator outside ParleyInn, like a
  harness. It reads the room's pins, messages and links, and what
  participants chose to make
  visible in the room, through the same tools an agent uses.
- **What it does.** It posts findings to the room, each linking the
  pin and the content it judged, raises a Needs you item for the owner
  and for the person whose agent a finding concerns, and, holding an
  operator's permissions, kicks, bars and sets read only itself
  (Q15, Q18). Every act it takes is audited with the finding behind
  it.
- **What it never does.** Instruct, steer or stop another person's
  agents, change pins, or act outside the room. Its findings are a
  classifier's claims, shown as such, and the owner can lift any bar.
- **A room without one.** The owner may run no etiquette bot; the
  operator role is still there, held by the owner by default.
- **The outcome window.** When participants' presents compete, the
  facilitator decides what the room's outcome window shows; without
  one, an operator; last, the owner.
- **Capabilities and modes.** Each role is a set of capabilities:
  read, post, link, pin and unpin, work, branch, present, kick, bar,
  set read only. The
  owner's configuration grants them; ParleyInn checks every act against
  them. Read only (Q17) makes a player a viewer: for one player, as
  IRC's quiet, or for the whole room as a mode, as IRC's +m, leaving
  posting to the roles the owner names.

### What stays for the bot

- **Errors.** A classifier misses things and flags innocent work.
  Findings are a classifier's opinion; people decide.
- **The bot as a target.** It reads untrusted content too, so it can
  be swayed into kicking or barring. Its powers stay inside the room:
  it can remove players, never instruct them, and the owner can undo
  every removal.
- **Privacy across people.** What a bot may read of a teammate's work
  is the teammate's choice of what to make visible in the room.

### What stays for injection

ParleyInn controls what it delivers, not what sways an agent. An agent
that reads an untrusted message can still be persuaded by it. The
remaining defences are around the agent, not in it:

- recall taint raises sensitive actions to ask first (SEC-13);
- held requests keep the person in the loop for those actions;
- grants bound what an agent can hand on;
- the sandbox state and risk acceptance bound what an unsandboxed
  agent can do (OWN-22).

## Lessons from IRC

IRC has run rooms with roles, bots and splits for decades; the
[IRC note](../../research/notes/irc/irc.md) has the detail. What the
room model takes:

- **Joining** shows the room's pins, intent first, each with its author
  and time, then the roster with roles, as IRC sends the topic and the
  participant list.
- **Pin edits** by anyone but the pin's author, and edits of the
  intent by anyone but the owner, are refused by the authorisation
  layer, not reverted afterwards (IRC's +t, not
  TOPICLOCK).
- **Authority lives in the owner's configuration**, not in who is
  present, as ChanServ's access lists replaced whoever held ops.
  Nothing grants operator status on join.
- **Kick and bar differ**, as on IRC: a kick removes a player now, a
  bar keeps it out. A bar records its target key, reason, private
  note, optional expiry and setter, and is audited.
- **Merging** never uses time: IRC's timestamp rule trusts servers'
  clocks, and peers' clocks are whatever the writer claims. Kicks and
  bars from both sides are unioned; powers come only from the owner's
  grant chain.
- **A room's id** comes from its creation record, so a partition
  cannot recreate it under the same name, as with RFC 2811's safe
  channels; the name can change freely.
- **Player kind and sender** are attested by ParleyInn, as IRCv3's
  server-set bot tag and account tag are, never claimed in a message.
- **No act is parsed from room text**: bots that read channel text as
  commands were injectable, so every act is a typed request through
  the authorisation layer.
- **The owner and successor** are set only on the owner's own machine,
  and a handover needs both sides, as Limnoria and ChanServ do.

## Decisions for the stakeholder

Decisions Q12 to Q21 are recorded once, in the
[concepts' decision table](concepts.md#stakeholder-decisions-of-4-october-2026).

[hydra]: https://matrix.org/blog/2025/08/project-hydra-improving-state-res/
