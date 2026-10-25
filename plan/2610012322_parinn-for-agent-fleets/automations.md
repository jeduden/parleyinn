# Automations: timers, forge events and merges, in the room model

Draft of 4 October 2026: how automations fit the
[concepts](concepts.md), the pitch and the accepted invariants.
Examples are a nightly timer, a GitHub issue event and a pull request
merged the old way.

## The constraints that shape it

- **ParleyInn is a set of tools, not an authority.** ParleyInn decides
  nothing; a person configures every automation in advance.
- **I2 and OWN-09.** Nothing starts or resumes an agent's turn except
  a person's act or a grant that person recorded. Event text from
  outside (an issue body, a commit message) is untrusted and never
  instructs.
- **I4.** The core opens no socket. Forge events come in through the
  bridge component (B3, outbound only, SEC-28). Only `parinn run`
  starts programs (SEC-29).
- **I10.** Projections read no clock. A timer is compute that writes
  an event; matching and firing are derived from the record.
- **I6.** Every firing, skip and failure is recorded and counted.

## Three parts

### 1. Sources post events

A source is a player of kind bot with its own participant id. It
posts events into a room as messages of kind event: data, untrusted,
stamped by ParleyInn with the source's id.

| Source           | Runs where                                                | Posts                                                                                                                    |
| ---------------- | --------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Timer            | In the running ParleyInn service that holds the rooms     | A tick naming its schedule, for example "nightly 02:00"                                                                  |
| Webhook endpoint | A deployed ParleyInn node, receiving the forge's webhooks | Issue opened, labelled or commented; pull request opened, reviewed or merged; each with a link to a snapshot of the item |
| Git, locally     | A git hook or a fetch the person runs                     | A commit landed on the default branch, matched to its room's branch by the record (LANE-06)                              |
| CI carrier       | the bridge component, B3                                  | A check result for an exact commit                                                                                       |

A merge "the old way", on the forge, is seen twice and agrees: the
bridge posts the forge's event, and the local fetch derives the
landed commit and its room's branch from the record without any network.
The commit's `ParleyInn-Room:` trailer names the room even after a squash,
but only as an assertion; the rule acts on the record's proof.

### 2. Automation rules are a person's grants

An automation is data a person records as their own act: a rule plus
a grant, much like a delegation grant (OWN-23).

| Field    | Meaning                                                                                                                                                          |
| -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Trigger  | A room, a source and an event kind, matched on structural fields only (kind, label, branch, schedule), never on text                                             |
| Action   | One from a closed set: post a notice, open a room from a template, start a session on a new branch of the room, run a recorded command, close a branch or a room |
| Template | Fixed text naming ids and links, never the event's own text                                                                                                      |
| Budget   | Tokens, sessions and how many firings, plus a cap on concurrent runs                                                                                             |
| Expiry   | By count or the person's revocation, never by clock in projections                                                                                               |

### 3. `parinn run` fires them

`parinn run`, the one component allowed to start programs, matches
events against rules as a projection of the record. When a rule
matches, it writes an "automation fired" event whose key is the
triggering event's address, so replaying the record never fires
twice. Then it performs the action within the budget. A started
agent receives the template; it reads the event, the issue or the
diff itself, through a tool, inside the untrusted envelope, and its
session counts as tainted (SEC-13), so its sensitive actions ask
first.

## The three examples

| Example                       | Source posts                                 | Rule                                                                                                                      | What happens                                                                                                        |
| ----------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Nightly dependency update     | Timer: "nightly 02:00" in room "deps"        | Start a session on a fresh branch of the room from the default branch, template "Nightly run for room r-…; read the pins" | The agent works against the room's intent; by morning its outcome waits for a verdict, linked to its evidence       |
| GitHub issue labelled "agent" | Bridge: issue labelled, link to its snapshot | Open a room from a template whose intent pin reads "Resolve issue <link>", start a session serving it                     | The issue's text stays untrusted; the agent reads it on request; the person edits the intent when needed            |
| Pull request merged on GitHub | Bridge: merged; local fetch: commit landed   | Post in the room that the attempt landed, post a cross-room notice to rooms that depend on it, close the branch           | Dependent rooms see a notice with a link to the diff; nothing instructs their agents unless their people pass it on |

## What never happens

- An event's text in a template, a pin or a notice.
- A trust grant to a bridge: its posts carry third parties' text, so
  trusting it would let anyone who can open an issue instruct your
  agents.
- An automation acting outside its person's own agents and rooms, or
  beyond its budget.
- A silent failure: a skipped, refused or failed firing raises a
  Needs you item and a counter.

## Decided on 4 October 2026: a service and a webhook endpoint

- **Rooms need a running ParleyInn service.** Without it there are no
  rooms, so the timer lives in that service, inside the `parinn`
  binary. The core's hooks still need no resident process (NFR-09);
  the room service is its own component.
- **Forge events arrive by webhook.** ParleyInn provides a webhook
  endpoint, so receiving them requires a deployed ParleyInn node the forge
  can reach. That is an inbound listener beyond loopback, a new row in
  the boundary register: off by default, enabled by a person's act,
  lockable by managed policy, and each delivery verified against the
  forge's webhook signature before it is recorded.
- **Webhook payloads are third parties' text**, recorded as untrusted
  events by the endpoint's own participant id, never trusted.

## SRS work

- **OWN-09 and I2's closed list** gain automations under a recorded
  grant, as delegation joined it (ADR-2610032155 change 11). That is
  an invariant change for review.
- **New requirements:**
  - sources as bot participants;
  - the event kind;
  - automation rules and their fields;
  - firing keyed to the triggering event;
  - the closed action set;
  - budgets and counters;
  - a per-room switch off;
  - managed policy able to disable automations (SEC-22).
- **SEC-28** gains an inbound webhook endpoint for issue and pull
  request events, with its own boundary row and signature check; this
  widens I4's boundaries and needs security review.

## Decisions for the stakeholder

| #   | Question                     | Options                                                                                                                                                                                          | Recommendation |
| --- | ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------- |
| A1  | Who runs automations?        | Decided: cron runs in the running ParleyInn service; other events arrive at a webhook endpoint ParleyInn provides, which requires a deployed ParleyInn                                           | —              |
| A2  | Can a person trust a bridge? | Decided: never                                                                                                                                                                                   | —              |
| A3  | Where does the timer run?    | Decided: in the ParleyInn service; without a running ParleyInn service there are no rooms                                                                                                        | —              |
| A4  | How is a merge detected?     | Decided: the webhook triggers, local git confirms; an automation acts only when the landed commit matches the room's branch with a proven class (LANE-06), and raises a Needs you item otherwise | —              |
