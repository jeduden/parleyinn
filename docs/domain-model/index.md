---
summary: >-
  ParleyInn's domain model: the closed set of concepts with their
  definitions, how they relate, the terms that are not ParleyInn concepts,
  and how names in code, docs and UI follow the model. The SRS links
  here for every term, and the domain-model agent reviews against it.
---
# Domain model

ParleyInn speaks in a small, closed set of concepts. Requirements, scenarios,
code,
documentation and every screen use only them. These files are their one source;
the [specification](../srs/index.md) defines no terms and links here. Every
ParleyInn concept has one name and every word one meaning. Where English uses
one
word for several things, each sense gets a qualified name, such as a branch link
or a held request. An outside thing keeps its own name when its domain qualifies
it: a network or email address, a command-line flag, a code owner or a persona
name.

These verbs each have one job:

- A principal *owns* a room.
- A seat *is a member of* a room as Member defines.
- A principal, an agent or a run *has a seat in* a room through any of its seats
  that is a member, its agents' run seats included; an agent has a seat in a
  room through its runs' seats that are members.
- A node *holds* writer logs, rooms and whatever else it stores. "Owns" is said
  of rooms and "holds" of nodes ("held request" and "hold window" are names);
  "holds a room" never means owning a room or being a member of it.

## Concepts

The concepts live in one file per group. Read this hub first, then the
group a term belongs to.

<?catalog
glob: "*.md"
where: 'order: string'
sort: order
header: ""
row: "- [{title}]({filename}) — {summary}"
?>
- [Principals and agents](principals-and-agents.md) — Who acts in ParleyInn: principals, persons and service accounts, managed policy, agents, runs and subagents, the facilitator, authors, members, the owner and delegates.
- [Harness facts](harness-facts.md) — The harness's own facts that ParleyInn records and names: the harness, harness sessions and transcripts, hooks, turns and compaction.
- [Places](places.md) — Where ParleyInn's state lives and is shared: homes, nodes, devices and paired phones, sandboxes, rooms and their kinds, visibility, peers and blind peers.
- [Record](record.md) — The record and what derives from it: writers, events, addresses, segments, seals, provenance and origin, spans and landmarks, quarantine and purge, receipts, backups, bundles and exports.
- [Pins and context](pins-and-context.md) — What may reach a model: pins and their versions, types and budget, the intent, stakes, posts, room summaries, envelopes, restore blocks, opt-in notices and held requests.
- [Seats and keys](seats-and-keys.md) — Seats and the keys behind them: run and device seats, room and seat ids, seat, device, principal and token keys, key events and key event logs, recovery keys, key stores, certificates, access tokens, authenticators and CI keys.
- [Acts and roles](acts-and-roles.md) — The three act kinds and what they may do: room acts, principal acts and their classes, expire acts, roles, appointments, joins, moderation, handover, stamps and the room merge.
- [Git and forge](git-and-forge.md) — Repositories, branches and commits, the forge, results with their evidence and proof classes, qualified links, comparisons, outcomes and verdicts.
- [Trust and flow](trust-and-flow.md) — Trust and how content reaches an agent: trust levels and sources, recall, endorsement, trust grants, delegation, rule levels, away policies, fixed templates, sandbox state and risk acceptance, quotas and spend, counters, canary and stats, run status and room status, the Needs you queue and focus set, overlap, forks, presence hints and petnames.
- [Components and surfaces](components-and-surfaces.md) — ParleyInn's components inside their network boundaries, the room view's surfaces and the outcome window.
<?/catalog?>

## Relations

- A principal owns any number of rooms; an agent's principal follows from its
  node.
- A run has its personal-room seat from its first event, plus a run seat per
  room it joined or created, a new one per seat key minted in place of a lost
  one, and the seat ingest starts beside any of them (Run seat).
- A restore block carries the qualifying pins of every room its run or its
  agent's earlier runs had a seat in, and a leave, kick or bar drops none
  (PIN-08, PIN-10, REC-02).
- Ingest splits one transcript by run.
- Every seat belongs to one principal and one room; every writer to one seat.
- Each event goes to exactly one seat's writer: a run's to one of its run seats,
  any other to a seat of the device that signs or records it, as LANE-01, OWN-02
  and REC-19 route each (open: OQ-41).
- A run's history spans its seats' writers, tied together by the run; a room
  contains only the events routed to its seats and shows others by address
  (LANE-01).
- A purge destroys content, never an act's effect: its tombstone keeps the
  fields ADM-07 names, so every derived artifact derives as before but for the
  purged content (ADM-07, REC-17; open: OQ-44).
- Principals and agents create rooms, an agent's owned by its principal;
ParleyInn
  never does on its own initiative, a personal room coming from its principal's
  install or a node clone (LANE-01, ADM-02).
- Recall defaults to the calling agent's current run, through every writer of
  its seats, whatever room each belongs to. It extends only to the principal's
  rooms the agent has a seat in, with the cross-room posts they show, or to a
  foreign room the call names (RCL-05, RCL-10; open: OQ-45).

## Names follow the model

Code, CLI verbs, MCP tools, settings keys, events, documentation, UI copy,
errors and logs use the model's words.

- An MCP tool for a room act is `room_<act>`, but `room_create` for create room.
  Every other MCP tool is `<concept>_<verb>`, with the reading verbs `get`,
  `list`, `search` and `expand`.
- A CLI command that acts on a concept is `parinn <concept> <verb>`. Node-wide
  utilities keep one word: `install`, `uninstall`, `status`, `doctor`, `verify`,
  `rebuild`, `migrate`, `licenses`, `ingest`, `import`, `export`, `purge`,
  `audit`, `canary`, `ui`, `why`, `open`, `launch`, `hook`, `mcp` and
  `kernel-worker`.
- An option that selects a concept is named for it: `--author`, `--seat`,
  `--writer`, `--run`, `--room`.
- A **settings key** names one setting in a settings layer:
  `<concept>.<setting>`, or `<concept>.<room>.<setting>` or
  `<concept>.<room>.<qualifier>` per room, the concept singular and in snake
  case (`node.deployment_mode`). A multi-word concept is snake case in MCP tools
  and settings keys, kebab case in CLI commands.
- Layout words (such as tab, panel, pane, gutter, sheet, stack, tile, pill,
  rail, composer and command palette) name parts of a screen, never concepts.
- ParleyInn defines no slash commands. A harness skill may call MCP tools or the
  CLI; a skill's call is the agent's own tool call, never a principal act, and
  acts only from its run seat (Device seat).

## Not ParleyInn concepts

| Term                                                                              | Why                                                                                                         | Where else it may appear                                                                                                     |
| --------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| session                                                                           | It belongs to the harness; ParleyInn speaks of runs.                                                        | As "harness session", or inside a harness's own names, such as `SessionStart`, `session_id` and `allow-session`.             |
| project                                                                           | A room names branches in repositories; nothing is scoped to a project.                                      | The harness's `~/.claude/projects` path, the harness settings scope `--scope project`, and ParleyInn's own software project. |
| compaction guidance                                                               | Withdrawn by the stakeholder (OQ-51): ParleyInn sends the harness nothing for a compaction.                 | Nowhere else.                                                                                                                |
| tenant                                                                            | Replaced by principal.                                                                                      | Nowhere else.                                                                                                                |
| operator, as a role or a person                                                   | The role is moderator; the person running a node is the node's principal.                                   | Only as the `operator` provenance class.                                                                                     |
| owner act, owner key, owner surface                                               | Replaced by principal act, principal key and principal surface.                                             | Nowhere else.                                                                                                                |
| bot, co-author, actor, poster, bound human                                        | Replaced by service account, pull-request author, author (for actor and poster) and principal.              | Nowhere else.                                                                                                                |
| writer key, writer certificate                                                    | Replaced by seat key and seat certificate.                                                                  | Nowhere else.                                                                                                                |
| attempt, claim of work, read only (a seat state)                                  | Replaced by branch, `stake` pin and mute.                                                                   | Nowhere else; "read-only" as an ordinary adjective (read-only publishing) stays.                                             |
| run component; own run and witness run as evidence classes                        | Replaced by launcher, own check and witness check.                                                          | Nowhere else.                                                                                                                |
| away mode, room alias, child agent                                                | Replaced by away policy, petname and "a subagent or a delegate".                                            | Nowhere else.                                                                                                                |
| trusted boundary                                                                  | Replaced by trusted sources; "boundary" means a network boundary.                                           | Nowhere else.                                                                                                                |
| room board                                                                        | Replaced by conversation.                                                                                   | Nowhere else.                                                                                                                |
| participant, player                                                               | Replaced by seat and member.                                                                                | Nowhere else.                                                                                                                |
| lane                                                                              | Renamed to room.                                                                                            | LANE requirement ids and file names such as lane-view.feature.                                                               |
| judge, approval gate                                                              | Removed by the stakeholder; a person records a verdict.                                                     | Nowhere else.                                                                                                                |
| service-account seat                                                              | Replaced by device seat.                                                                                    | Nowhere else.                                                                                                                |
| import of a transcript or a peer's segments; imported run                         | Only a bundle is imported.                                                                                  | Nowhere else.                                                                                                                |
| presence check; unqualified token; Needs you as a run or room status; work marker | Replaced by presence proof, access token or model token, Asking, and ingest marker.                         | Nowhere else.                                                                                                                |
| hide                                                                              | Removed by the stakeholder; say the UI does not show it.                                                    | The cryptographic term "hiding commitment" (REC-17).                                                                         |
| git carrier                                                                       | Removed by the stakeholder; peers exchange segments, and ParleyInn writes no room segments to a git remote. | Nowhere else.                                                                                                                |
| next key; pre-rotation; fork or branch of a key event log                         | KERI's names; ParleyInn says recovery key, rotation and duplicity.                                          | Only in ADR-2610122234's citation of KERI and in research notes.                                                             |

Every excluded term may still appear in **historical records**: the SRS change
log, accepted ADRs and plan records, which keep the words of their time.
Excluded words, and words the model defines, such as `verdict`, may also appear
in that tool's meaning in files an outside tool writes and maintains, or that
name its fields. Examples are frit's plan skills and plan files, and the
repository's own engineering tooling, such as ENG-28's review gate and the
requirement naming its fields. The domain-model agent skips them.

Build and packaging words have no ParleyInn meaning: crate, lockfile, release
build, executable and release target. They are engineering-tooling words.
Requirements may use them in their ordinary sense wherever they name a
built artifact, as ADM-01 does with "core executable" and the `licenses`
row of the command-line section does.

## Changing the model

A new concept, a renamed one or a new relation is a stakeholder decision. It
lands here before anything uses it. The model keeps every concept defined, those
the invariants name included, and §12.2 decides when each one's requirements
ship. The domain-model agent reviews every change to these files and every
proposal to change it. It names the invariants whose wording would change, and
every use the change makes stale or that runs ahead.
