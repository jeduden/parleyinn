# Room protocol deep dive: join, participate, leave

Deep dive of 4 October 2026 into the room protocol of the
[entity model](entity-model.md). Three threads fed it: what each
harness lets ParleyInn do at each step, how existing room and multi-agent
systems handle membership, and a stress test against the invariants.
Sources are cited inline; claims no source supported are marked
unverified.

## Findings in brief

1. **The harnesses can carry the protocol.** Claude Code and Codex
   offer every step through hooks, including a start hook for
   subagents. Gemini CLI, OpenCode, Amp and Cursor each lack one step;
   ACP lacks compaction entirely.
2. **The SRS has two gaps the protocol exposes.** The intent is not on
   I2's closed list of writes, and no path delivers a changed pin or
   intent at the next turn. Both need an I2 change and a security
   review (ENG-29).
3. **A room groups lanes.** Five agents on one intent work on five
   branches, so the intent, room pins and notices move from the lane
   to the room.
4. **Prior art warns twice.** Merging membership across servers is the
   hardest part (Matrix needed a security release in 2025), and every
   multi-agent framework passes other agents' text in as instructions,
   which is how cross-agent injection spreads.
5. **Nine decisions are the stakeholder's**, listed at the end.

## The protocol, revised

| Step        | What happens                                                                                                                                                                                                                                                                                              |
| ----------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Join        | The agent's own person adds it, or it starts in a lane the room holds, or a delegation names the room. At its next start hook it gets the intent in force, its own person's pins for the room, and a fixed notice naming the room and players by id and key fingerprint only.                             |
| Participate | Restore after every compaction, startup, resume and clear. A changed pin or intent reaches it at its next hook point as a fixed notice carrying that verbatim text. It reads messages, cross-room messages and links when it asks, marked untrusted, and posts its own. Only its own person instructs it. |
| Leave       | Its person removes it, its session ends, or a delegate returns. A running agent removed from a room keeps that room's rules until its next startup or clear, unless the removing act lifts them and says so. Everything it did stays in the record.                                                       |

Membership is written by a person (add, remove) or derived from
structural events (session start and end, delegation records); no
agent can change it through a tool. Every delivery is recorded with
the rooms and versions it carried, so pending notices are derived from
the record alone, with no clock (I10).

### Membership states of one session in one room

| State          | Meaning                                                                                         |
| -------------- | ----------------------------------------------------------------------------------------------- |
| none           | Not a participant                                                                               |
| pending        | Membership recorded; the join text not yet delivered                                            |
| participant    | Join delivered; restores active                                                                 |
| notice pending | The record holds a newer pin or intent version than the last delivery                           |
| undelivered    | The harness offers no delivery point; shown as "rules not delivered" with a Needs you item (I6) |
| left, carrying | Removed while running; rules kept until its next startup or clear                               |
| ended          | Session over; a session that died without an end event stays a participant, shown "unconfirmed" |

## What the harnesses offer

S supported, P partial, U unsupported, ? unverified.

| Harness     | Join | Restore | Next-turn notice | Read and post (MCP) | Leave | Subagent |
| ----------- | ---- | ------- | ---------------- | ------------------- | ----- | -------- |
| Claude Code | S    | S       | P                | S                   | S     | P        |
| Codex       | S    | S       | S                | S                   | P     | S        |
| Gemini CLI  | S    | P       | S                | S                   | S     | ?        |
| OpenCode    | S    | S       | S                | S                   | P     | ?        |
| Amp         | P    | P       | S                | S                   | P     | P        |
| Cursor      | S    | U       | P                | S                   | S     | P        |
| ACP         | P    | U       | P                | S                   | S     | U        |

- **Claude Code**
  ([hooks](https://code.claude.com/docs/en/hooks),
  [subagents](https://code.claude.com/docs/en/sub-agents)):
  - Join and restore use `SessionStart`, with the sources startup,
    resume, clear, compact and fork.
  - `SubagentStart` adds context to a subagent; a subagent's own
    compaction mid-run is unverified.
  - A next-turn notice arrives when the person types, or at the next
    tool result through `PostToolUse`. Reaching an idle session needs
    an `asyncRewake` hook, the channels preview, or the Agent SDK.
- **Codex** ([hooks](https://learn.chatgpt.com/docs/hooks),
  [app-server](https://learn.chatgpt.com/docs/app-server)):
  - Every step is covered, including `SubagentStart`.
  - The app-server's `thread/inject_items` adds context without
    starting a turn.
  - `SessionEnd` reports only "other". Hooks are skipped until trusted
    by hash, so an upgrade can silently stop restores.
- **Gemini CLI** has no post-compression event; restore must be
  emulated on the next `BeforeAgent`
  ([hooks](https://geminicli.com/docs/hooks/reference/)).
- **Cursor** cannot restore after compaction, and `subagentStart` can
  only allow or deny ([hooks](https://cursor.com/docs/agent/hooks)).
- **Amp and OpenCode** need an in-process JavaScript plugin shim, the
  latter on experimental hooks.
- **ACP** has no instructions field and no compaction signal; text
  arrives as user turns ([prompt turn][acp]).

### Limits that bind the restore block

- Claude Code shows hook text over 10,000 characters only as a file
  path and a preview, so pins would silently stop being verbatim.
- Codex caps hook text at about 2,500 tokens.
- Hooks run in parallel, so another plugin's output may sit beside
  ParleyInn's.
- A start hook that times out counts as no output, so a join can be
  lost without a trace unless ParleyInn records every delivery.
- An administrator's switch can disable all hooks. ParleyInn needs a
  detectable heartbeat, so the loss shows (I6).

## What prior art teaches

- **Matrix**
  ([room v12](https://spec.matrix.org/v1.19/rooms/v12/),
  [Project Hydra][hydra]):
  - Membership is a signed state event, authorised by power levels.
  - History visibility is fixed when each event is added, never
    widened later.
  - Merging state across servers produced "state resets" that rolled
    back removals, fixed in 2025 with room v12.
  - Lesson: keep one authority per room, let revocations win, and
    fail closed.
- **XMPP group chat**
  ([XEP-0045](https://xmpp.org/extensions/xep-0045.html)):
  - Lasting affiliation is kept apart from session presence.
  - A joiner gets occupants, then history marked as delayed, then the
    room subject, in a fixed order, and can bound the history it
    receives.
- **Multi-agent frameworks** (AutoGen, LangGraph, CrewAI, the OpenAI
  Agents SDK) pass other agents' text into a joining agent's context
  as conversation or as free-text context. The OpenAI Agents SDK folds
  history into one block, with an input filter, which is the nearest
  thing to an envelope. A2A treats agents as opaque, but analyses find
  no injection defence in it.
- **Research:**
  - [Prompt Infection](https://arxiv.org/abs/2410.07283): tagging the
    sender alone cut attacks by about 5%; tagging plus marking content
    untrusted stopped them in that study.
  - [Triedman et al.](https://arxiv.org/abs/2503.12188): web content
    hijacked orchestrators into running code in 58–90% of trials.
  - Unit 42's "agent session smuggling" (2025, link below): a remote
    agent smuggled instructions between turns.
  - [CaMeL](https://arxiv.org/abs/2503.18813) separates control from
    data, the strongest support for "data never instructs".
- **Lessons for ParleyInn:**
  - History is pulled, never pushed (Slack bots work this way).
  - The room's purpose is delivered as its own item.
  - Routing (which agent runs, which room is reached) comes only from
    a person's act.

## Edge cases that shape the requirements

| Case                                         | Outcome                                                                                                           | Decided by      |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- | --------------- |
| An agent in two rooms                        | One section per room in the restore block, ordered by room id; ParleyInn never reconciles two intents             | I3, I10, CMP-09 |
| A pin change racing a compaction             | Both built from the record at the hook's causal position; a notice already carried by the restore is suppressed   | I10, I3         |
| A notice that cannot be delivered            | "Rules not delivered" with the pending versions and a Needs you item; never written into CLAUDE.md or settings    | I6, I7          |
| An urgent tightening mid-task ("never push") | A notice is too late; the rule engine or an interrupt enforces it                                                 | I2, I9          |
| Removed while waiting on a permission prompt | Removal does not answer, deny or cancel the request                                                               | I6, I9          |
| Two people's pins conflict in one room       | Each person's pins reach only their own agents; the view shows both; ParleyInn flags overlap, never meaning       | I8, CMP-09      |
| A teammate's agent joins                     | The owner's intent reaches it only after the teammate adopts that version                                         | I8, OWN-01      |
| An agent posts text that looks like a pin    | Stays untrusted, never styled like a pin; becomes one only by an owner act showing the exact text                 | I2              |
| A room renamed                               | Display only; names never reach an agent, ids and key fingerprints do                                             | I2              |
| Rooms merged or split                        | References resolve to the surviving id; both intents restore until the owner adopts one; nobody is moved silently | I3, I10         |
| A peer removes my agent                      | A request, as quarantine is; only an agent's own person changes its membership                                    | I8, I5          |
| The restore block overflows                  | Intent first, then pins by priority, none truncated; omissions stated by room id and count, audited               | I3, I6          |
| A pin quarantined while an agent holds it    | A fixed "withdrawn" notice, and a clear offered, since the text is still in the context window                    | I5, I6          |
| The person leaves their own room             | Not as such: they close it or hand it over                                                                        | I3, OWN-01      |

## SRS changes this needs

- **I2 and INJ-03:** add the intent in force, the change notice and
  the join notice to the closed list, built only from the session's
  own person's text, ids and version numbers. Needs an ENG-29 review.
- **INJ-04:** carve out the change notice from "prompt-time injection
  off by default", or accept that changes wait for the next restore.
- **PIN-10, LANE-20, LANE-22, OWN-07, INJ-10, LANE-12:** move scope
  from the lane to the room where the stakeholder decides so.
- **OWN-08:** split endorsement within one person (P1, needed to pass
  a cross-room message on) from endorsement across people (P2).
- **New family ROOM:** room identity and owner; membership written by
  a person or derived from structural events, in causal order with no
  clock; no tool changes membership; removal keeps rules until a safe
  point; removal leaves held requests alone; a dead session stays
  "unconfirmed"; merge and split rules; one restore section per room.
- **New INJ rows:**
  - INJ-11: the change notice, verbatim, audited, never starting a
    turn.
  - INJ-12: every delivery recorded with its rooms and versions.
  - INJ-13: notices name rooms and players by id and fingerprint only.
  - INJ-14: "rules not delivered" shown, never simulated.
  - INJ-15: an intent size cap and the overflow order.
  - INJ-16: a withdrawn notice for a quarantined or ended pin.
- **New OWN rows:** speaking once reaches each of the speaker's own
  agents as one act naming each; a teammate's agents get the owner's
  intent only after adopting it.
- **New VIEW row:** agent and peer posts render distinctly from pins,
  intents and the person's own messages.
- **New assumption and spike:** the harness gives a subagent its own
  start hook (Claude Code and Codex document one); a spike checks
  restore after a subagent's own compaction.

## Decisions for the stakeholder

The answers the stakeholder gave are recorded once, in the
[concepts' decision table](concepts.md#stakeholder-decisions-of-4-october-2026);
this table keeps the options weighed.

The stakeholder decided these on 4 October 2026; the decisions and
what follows from them are in [concepts](concepts.md). Q2 stays open.

| #   | Question                                           | Options                                                                                                     | Recommendation                                                |
| --- | -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| Q1  | Is a room a lane or a group of lanes?              | Answered: split into independent concepts composed by bindings; see [concepts](concepts.md)                 | —                                                             |
| Q2  | Does the change notice carry the text?             | (a) ids only, recalled as untrusted; (b) verbatim, on for participants; (c) adapter only                    | (b): otherwise the agent's own rule reads as untrusted        |
| Q3  | What does leaving do to a running agent?           | (a) lift its rules at once; (b) keep them until a safe point; (c) stop the agent                            | (b) by default, (a) as an explicit choice                     |
| Q4  | Can a session be in several rooms?                 | (a) one at a time; (b) many, equal; (c) one active, earlier rules carried                                   | (c), as pins already accumulate across lanes                  |
| Q5  | How does a plain session join?                     | (a) explicit add only; (b) automatically from its lane's room; (c) only via `parinn run`                    | (b): joining only tightens, and a session must not lack rules |
| Q6  | Does the owner's intent reach a teammate's agents? | (a) after the teammate adopts it; (b) never as text; (c) trusted by role                                    | (a)                                                           |
| Q7  | How do private subagents get the rules?            | (a) ParleyInn rewrites the task; (b) their own start hook, else "not delivered"; (c) the parent copies them | (b): Claude Code and Codex offer `SubagentStart`              |
| Q8  | A quarantined pin still in context?                | (a) withdrawn notice; (b) also offer a clear; (c) wait for compaction                                       | (a) and (b)                                                   |
| Q9  | Can the owner leave their own room?                | (a) no, close or hand over; (b) leaving closes it; (c) ownerless rooms                                      | (a)                                                           |

[acp]: https://agentclientprotocol.com/protocol/prompt-turn
[hydra]: https://matrix.org/blog/2025/08/project-hydra-improving-state-res/

| Source                           | Link                                                                                                               |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Unit 42, agent session smuggling | [unit42.paloaltonetworks.com](https://unit42.paloaltonetworks.com/agent-session-smuggling-in-agent2agent-systems/) |
