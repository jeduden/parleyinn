# ParleyInn domain model v2 (proposed)

A proposal for the stakeholder, dated 6 October 2026. It includes your
[decisions 1][d1] to [19][d19] and would replace [docs/domain-model.md][model],
which now also holds the [definitions][16] the SRS glossary held.

Each entry gives a **name**, a definition, and why the concept exists or how
it differs from its neighbours. Every word has one meaning. Where everyday
English uses one word for several things, each sense gets its own qualified
name.

**Verb rule.** These verbs each have one job:

- A principal *owns* a room.
- A seat *is a member of* a room.
- A principal *has a seat in* a room through any of its seats, including its
  agents' run seats.
- A node *holds* logs and rooms.

"Holds a room" never means owning a room or being a member of it.

## Decisions this model builds in

The stakeholder took these decisions on 5 and 6 October 2026. The
model below builds them in; they are not reopened here.

### D1: Owner is a room relation

"Owner" means only a room's owner. A person, an agent's principal or a service
account may own a room, with full owner rights over room governance: roles,
admission, appointing moderators, successor and handover.

### D2: Service account replaces bot

A service account is a non-human principal with its own key, certified and
revocable by another service account or by managed policy. It can own rooms,
hold seats and start agents. The facilitator is a service account appointed
moderator.

### D3: Principal is a person or a service account

A service account is principal of its own agents exactly as a person is: its own
pins, stamps and trust grants apply to its own agents only. Room governance is
separate from [I2][inv] trust.

### D4: Run means an agent run only

The run component becomes the launcher (`parinn launch`). The evidence classes
`own run` and `witness run` become `own check` and `witness check`. A witness
check runs on a node whose principal authored no change in the range.

### D5: Attempts are dropped

A room names branches in any repositories. A branch is identified by repository
identity, remote URL and branch name. A branch with a pull request links to it.

### D6: Link stays generic, always qualified

Each use of "link" carries a qualifier: criterion link, branch link, landing
link and so on.

### D7: Member and roles

A member is anyone holding a seat in a room. The roles are viewer, contributor
and moderator; the owner stands beside them. Co-author is dropped;
[LANE-15][511]'s outside party is the pull-request author.

### D8: Author replaces actor

The author of an event or pin is the seat that wrote it; its principal follows
from the seat. `--actor` becomes `--author`.

### D9: Node and device

A node is one ParleyInn home on one machine, container or sandbox, for one
principal. A device is anything holding a device key. "Sandbox" keeps only its
confinement meaning ([OWN-22][513]).

### D10: Foreign room means no seat held

A foreign room is a room held on this node in which this principal holds no
seat. Joined rooms are never foreign. A principal's own transcripts from outside
the configured roots are imported runs in its personal room.

### D11: Acts split by who signs

Room acts are signed by a seat key and checked against the role's capabilities.
Principal acts are signed by a device key at an authenticated surface and
classed cut, neutral or widening. Expire acts are recorded by a node: the
setter's node first, then a moderator's, then the owner's. Events with no run go
to the person's device seat. A room's create act is the first act of the
creating seat's log.

### D12: One key per seat

The seat key is also its writer key. A run's seat key lives only in its MCP
server's memory.

### D13: Outcome is defined

The outcome is what a room's work has produced so far: the heads of the branches
it names, their results and evidence, as verdicts judge them. The outcome window
shows part of it.

### D14: Directed post and held request

[LANE-12][511]'s post addressed to one agent is a directed post. [OWN-05][513]'s
kind is a held request, never shortened. Other requests are always qualified.

### D15: Mute replaces read only

A muted seat or muted room keeps only read.

### D16: Operator is an event class only

`operator` stays only as the event class in [I2][inv]. The room role is
moderator. The person running a node is the node's principal.

### D17: A pin's author is a seat

Restore rules speak of the author's principal.

### D18: One merge rule for all room state

Title, labels and assignment merge under [LANE-31][511]'s rule too. Room status
is a derived view, not state.

### D19: Earlier decisions

ParleyInn defines no slash commands. Verdicts are recorded by any person in the
room, bound to the intent version, the heads of every branch the room names and
the results and evidence shown. Recall defaults to the agent's own run. Every
pin belongs to exactly one room. One personal room per person per node.
Principals and agents create rooms; ParleyInn never does.

## 1. Principals and actors

| Name                | Definition                                                                                                                                                                                | Why it exists / how it differs                                                                                                               |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| Principal           | A person or a service account. It holds a principal key, signs principal acts, and is the principal of the agents its nodes start.                                                        | Only an agent's own principal widens what reaches that agent ([I2][inv]). A principal is not the same as an owner, which is a room relation. |
| Person              | A human principal. No one certifies a person's principal key.                                                                                                                             | The root of human trust. Only a person records a verdict.                                                                                    |
| Service account     | A non-human principal with its own principal key. Another service account or managed policy certifies it and can revoke it, so its chain of certifiers may end without a person.          | Replaces bot. It can own rooms, hold seats, and be the principal of its own agents (CI, runners), just as a person can.                      |
| Managed policy      | Root-owned configuration that an organisation sets on a machine. ParleyInn never overrides it, and it may certify service accounts.                                                       | The only authority that is not a principal. It can turn off boundaries B1 to B3 and forbid risk acceptance.                                  |
| Harness             | The external program that runs agents, such as Claude Code or the Agent SDK. It owns their sessions and transcripts, compacts their context, and reports to ParleyInn through hooks.      | ParleyInn records the harness but never manages it. "Session" and "compaction" remain the harness's words.                                   |
| Agent               | A worker a harness runs for exactly one principal: the principal of the node that started it.                                                                                             | Receives restore blocks and recalls history. An agent is never a principal.                                                                  |
| Run                 | One agent run as the harness reports it, on one node. It holds run seats and carries recall taint, sandbox state, its seat keys, a kernel namespace and spans.                            | "Run" has only this meaning ([decision 4][d4]).                                                                                              |
| Subagent            | An agent that another agent's harness started for it. It has its own run, tied to the parent's run by a parent link.                                                                      | Takes work without a delegation grant. A delegate needs one.                                                                                 |
| Imported run        | A run ingested from a transcript the hooks did not watch. It is marked `imported` and untrusted. A principal's own transcripts from outside `transcript_roots` land in its personal room. | Keeps a principal's own history out of foreign rooms ([decision 10][d10]).                                                                   |
| Facilitator         | A service account appointed moderator. It may also post findings in its own words and write room summaries.                                                                               | Replaces "facilitator bot". Its posts reach an agent as trusted text only through a trust grant.                                             |
| Author              | The seat that wrote an event or a pin. The author's principal follows from the seat.                                                                                                      | Replaces "actor", so "who wrote it" has one word.                                                                                            |
| Member              | Anyone with a seat in a room.                                                                                                                                                             | Replaces "participant" and "co-author". The member's role says what it may do.                                                               |
| Owner               | The one principal who owns a room: its intent, roles, admission, appointments, successor and handover. Ownership changes only by handover.                                                | Applies only to rooms ([decision 1][d1]). The owner stands beside the roles rather than holding one.                                         |
| Pull-request author | The outside party whose commits a foreign room's bundle describes. ParleyInn matches them through their commit-signing identity ([LANE-15][511]).                                         | Replaces "co-author" in [LANE-15][511]. May have no seat at all.                                                                             |

## 2. Places

| Name              | Definition                                                                                                                                       | Why it exists / how it differs                                                                          |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------- |
| Home              | The directory holding all of one principal's ParleyInn state on a node (`PARINN_HOME`), owned by one OS user.                                    | The unit of isolation ([I8][inv]).                                                                      |
| Node              | One home on one machine, container or sandbox, for one principal. Its device key signs what it records.                                          | One principal per node.                                                                                 |
| Device            | Anything holding a device key: a node, or a paired phone.                                                                                        | Groups a principal's seats by where they act from.                                                      |
| Paired phone      | A device with no home, limited to reading and to allowing or denying held permission requests.                                                   | The only device that is not a node.                                                                     |
| Sandbox           | A confinement around a run that blocks some of the residual risks ([OWN-22][513]).                                                               | Keeps only its confinement meaning. A node running inside a sandbox is still a node.                    |
| Room              | Where an intent is worked on. It holds at most one intent, a conversation, seats, pins, and branches in any repositories, named by branch links. | The unit ParleyInn records, shows and shares. Principals and agents create rooms; ParleyInn never does. |
| Personal room     | A person's private room on each of their nodes. Every run sits in it from its first event, without a join.                                       | The default room for events and pins that belong to no other room. See open choice 2.                   |
| Foreign room      | A room this node holds in which this principal has no seat, such as an imported bundle or a peer's room.                                         | Untrusted and outside widened recall. A room where the principal has a seat is never foreign.           |
| Principal's rooms | The rooms a principal owns or has a seat in.                                                                                                     | Replaces "rooms the person holds" ([VIEW-09][512]).                                                     |
| Writer            | One seat's append-only log on one node, named by its seat key.                                                                                   | The storage side of a seat.                                                                             |
| Record            | The set of writer logs a node holds.                                                                                                             | The source of truth; everything else is derived from it ([I10][inv]).                                   |
| Peer              | Another node, enrolled by key for replication over B2.                                                                                           | Exchanges segments, but is never trusted for content.                                                   |
| Blind peer        | A peer that stores and serves a room's segments encrypted to the members' keys, and reads none of them.                                          | Storage without membership or authority.                                                                |
| Room view         | The local browser, terminal UI and CLI surfaces that show rooms.                                                                                 | A client of the record, never its source.                                                               |
| Outcome window    | The pane beside a room's conversation that shows one presentation.                                                                               | Shows part of the outcome.                                                                              |
| Rooms overview    | The surface listing the principal's rooms and their live runs.                                                                                   | Replaces "Fleet".                                                                                       |
| Needs you         | The one queue of items waiting on a principal ([VIEW-05][512]).                                                                                  | Different from notifications, which only signal.                                                        |
| Catch up          | The one surface that answers "what happened since a boundary" ([VIEW-08][512]).                                                                  | A projection with stated boundaries.                                                                    |

## 3. Content

**Record mechanics.** These meanings do not change:

- **Event**: one immutable entry in a writer.
- **seq**: an event's gap-free position in its writer.
- **Payload**: a large event's content, stored apart from the event itself.
- **Segment**: a sealed range of one writer; the unit peers exchange.
- **Seal**: a seat key's signature over a writer's chain head.
- **Commitment**: a keyed commitment to content, erased together with the
  content.
- **Span**: a contiguous range of one run's events.
- **Landmark**: a structural headline over a span.
- **Kernel**: the hermetic environment for computing over the record.

These exist for losslessness and integrity ([I1][inv], [I10][inv]). None of them
is a room concept.

| Name               | Definition                                                                                                                                                                                   | Why it exists / how it differs                                          |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| Address            | (writer, seq), written in short, range or full `parinn:` form.                                                                                                                               | The only meaning of "address".                                          |
| Node URL           | A deployed node's URL, used in room trailers.                                                                                                                                                | Not an address. It opens nothing without access to the room.            |
| Peer address       | The network endpoint of an enrolled peer.                                                                                                                                                    | Not an address.                                                         |
| Pin                | Verbatim text in exactly one room, with one author, a pin type and numbered pin versions.                                                                                                    | Information, never an instruction. It restores only under [PIN-10][53]. |
| Pin version        | One immutable text of a pin. Each edit adds a new version.                                                                                                                                   | What a stamp covers.                                                    |
| Pin candidate      | An inactive pin that an agent proposed or that ParleyInn detected.                                                                                                                           | Becomes active only through a principal's confirmation or a stamp.      |
| Pin types          | `constraint`, `preference`, `decision`, `fact`, `episode`, `intent`, `verdict`, `stake`.                                                                                                     | Only `constraint`, `preference` and `intent` pins restore.              |
| Intent             | A room's lead pin, of type `intent`: a goal, its criteria and optional paths. Only the owner's principal act changes it.                                                                     | Verdicts and results bind to a specific intent version.                 |
| Criterion          | One acceptance condition of an intent, with a stable id.                                                                                                                                     | What criterion links and verdicts point at.                             |
| Stake              | A pin of type `stake` stating what its author is working on. Only its author writes it, and it locks nothing.                                                                                | Replaces "claim of work", so "claim" names only an evidence class.      |
| Title, labels      | Room state the author chooses: a display name and tags.                                                                                                                                      | Never reach a model. They merge under room merge.                       |
| Assignment         | Room state asking a seat to work on a branch.                                                                                                                                                | Locks nothing. Others set it, unlike a stake.                           |
| Post               | A message a seat writes to a room's conversation. Its provenance is `post` and it is always untrusted.                                                                                       | Reaches an agent only through recall, an endorsement or a trust grant.  |
| Directed post      | A post addressed to one agent. It goes into that agent's principal's endorse queue ([LANE-12][511]).                                                                                         | Replaces [LANE-12][511]'s "request".                                    |
| Room summary       | A facilitator's summary of a room, linked by address to the events it covers.                                                                                                                | Read only through `room_summary_get`.                                   |
| Compaction summary | The harness's summary at compaction, recorded as untrusted `harness_text`.                                                                                                                   | Never re-injected.                                                      |
| Envelope           | The wrapper that marks recalled content as untrusted historical data.                                                                                                                        | The only way record content reaches a model.                            |
| Envelope warning   | The fixed sentence at the head of every envelope (field `warning`).                                                                                                                          | Replaces the envelope's `notice` field.                                 |
| Restore block      | Deterministic trusted text injected after compaction: qualifying pins, a landmark index and a recall hint.                                                                                   | Never carries untrusted text.                                           |
| Opt-in notice      | Trusted text built only from ids, versions, counts, key fingerprints and addresses. It is sent only while the owner's notice allowance and the agent's principal's notice opt-in both stand. | The only meaning of "notice".                                           |
| Gap mark           | A tombstone mark, quarantine mark or truncation mark shown where content is missing.                                                                                                         | Replaces the other uses of "notice".                                    |
| Quarantine         | Recorded, reversible exclusion of content from recall and injection.                                                                                                                         | Removes content from circulation without deleting it ([I5][inv]).       |
| Purge              | Deletion of content, leaving a tombstone.                                                                                                                                                    | The only thing that destroys content ([I1][inv]).                       |
| Bundle             | A signed, reviewed export of a room, carried as a file or a git ref.                                                                                                                         | Importing one makes a foreign room.                                     |
| Held request       | A permission request, question or hand-off with a stable id, which can be answered from any principal surface ([OWN-05][513]).                                                               | Never shortened to "request".                                           |
| Qualified requests | Role request (a viewer asks for a wider role), join request (a run asks to join), erasure request and quarantine request (both to peers), summary request (to the facilitator).              | "Request" never stands alone.                                           |
| Notification       | An in-page, desktop or terminal signal to a person on the same device. It carries no room text.                                                                                              | Not a notice: it never reaches a model.                                 |

## 4. Acts and roles

### Seats and keys

| Name                 | Definition                                                                           | Why it exists / how it differs                                                              |
| -------------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------- |
| Seat                 | One run, one device or one service account in one room.                              | The unit of membership, signing and authorship.                                             |
| Run seat             | A run's seat. Its key lives only in the memory of that run's MCP server.             | Agents act only through run seats.                                                          |
| Device seat          | A person's seat for one device. Events that belong to no run go here.                | Groups a person's direct acts by device.                                                    |
| Service-account seat | A service account's own seat, used without a run.                                    | The facilitator's kind of seat.                                                             |
| Seat id              | A seat's id, derived from the room id and the seat key.                              | Nobody chooses it.                                                                          |
| Seat key             | The one key each seat has. It signs the seat's room acts and seals its writer.       | Replaces "writer key" ([decision 12][d12]).                                                 |
| Device key           | A device's key. A principal key certifies it, with a scope and a maximum rule level. | Signs principal acts and expire acts.                                                       |
| Principal key        | A principal's root key, which certifies its device keys.                             | Replaces "owner key".                                                                       |
| Seat certificate     | A device key's signature over a seat key, scoped to the seat's room.                 | Replaces "writer certificate". The chain runs from principal key to device key to seat key. |

### Act kinds

Room state is derived from these three kinds of act only, and each act belongs
to exactly one kind.

| Kind          | Signed by                                                                                                                   | Acts                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| ------------- | --------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Room act      | A seat key, checked against the capabilities of the seat's role                                                             | join, leave, post, link, pin, edit (a pin), unpin, present, pick, kick, bar, unbar, mute, unmute                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Principal act | A device key, at a principal surface. Classed cut, neutral or widening ([OWN-11][513]); an unlisted act counts as widening. | **Room governance:** create room, role assignment, admission, appointment, successor, handover, stamp, trust grant, verdict, intent. **Agent and node acts:** answer a held request, run controls, endorsement, correction, retry, delegation grant, acceptance grant, delegation cancel, rule change, away policy, risk acceptance, quarantine and release, purge, export, foreign-room reject, repository bind, ready mark, notice allowance, notice opt-in, enrolment and revocation, token mint, witness-check start, backup restore |
| Expire act    | A node's device key                                                                                                         | Ends the expiry set on a bar, a mute or a handover offer. The setter's node records it first; if that fails, any moderator's node; then the owner's node.                                                                                                                                                                                                                                                                                                                                                                                |

Room acts never widen what reaches an agent, because room governance is not
[I2][inv] trust. [OWN-11][513]'s classes therefore cover principal acts only.

What the room and principal acts do:

- **Join** enters a room. A run joins only when its principal asks for the join
  or accepts it.
- **Leave** ends one's own seat.
- **Kick** revokes a seat's current add. Only that seat's principal may add it
  again.
- **Bar** names a principal key and covers every key that chains to it, until an
  unbar or an expire act.
- **Mute** withdraws every capability except reading, from one seat or from the
  whole room. It replaces "read only".
- **Present** puts a presentation in the outcome window.
- **Pick** chooses which presentation the window shows.
- **Admission** sets the room to invite only or to an admission list of keys. An
  invite (a key plus a role) and an invite link are admission acts.
- **Appointment** is a principal act by the owner or a moderator. It makes a run
  seat or a service-account seat a moderator. The appointer or the owner can
  revoke it.
- **Successor** is named in advance and accepts after the owner leaves.
- **Handover** transfers ownership by an offer and an acceptance.
- **Stamp** makes one pin version restore to the stamping principal's own agents
  only.

| Name        | Definition                                                                                                                                                       | Why it exists / how it differs                                                                                                 |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Room merge  | The one rule that derives all room state from the three act kinds in causal order. When acts conflict, the more restrictive act wins, then the lower commitment. | Covers membership, roles, appointments, pins, stamps, mutes, presentations, picks, bars, offers, title, labels and assignment. |
| Concurrent  | Said of two acts when neither is causally after the other.                                                                                                       | Defined without reference to any clock.                                                                                        |
| Room status | A derived view of a room: Running, Quiet, Ready for review and so on.                                                                                            | Not state; no act sets it ([decision 18][d18]).                                                                                |

### Roles

| Capability                                                            | Viewer | Contributor | Moderator |
| --------------------------------------------------------------------- | ------ | ----------- | --------- |
| Read                                                                  | yes    | yes         | yes       |
| Role request                                                          | yes    | no          | no        |
| Post, link, present                                                   | no     | yes         | yes       |
| Pin, edit and unpin own pins; work on the room's branches             | no     | yes         | yes       |
| Unpin any pin except the intent; kick, bar, unbar, mute, unmute, pick | no     | no          | yes       |

- **Owner:** stands beside the roles, with every capability plus room
  governance.
- **Work:** a capability, not an act. A run's events go to its run seat while
  the run works on one of the room's branches.
- **Appointed moderator:** stays within [SEC-32][62]'s limits, which forbid
  changing pins, muting the whole room, unbarring and unmuting.

## 5. Git and forge

| Name                | Definition                                                                                                                                                                                                                                                                                                 | Why it exists / how it differs                                                                                                                        |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Repository          | A git repository, identified by a bound root commit, together with its remotes.                                                                                                                                                                                                                            | A git fact. It holds no ParleyInn state.                                                                                                              |
| Branch              | A git branch, identified by repository identity, remote URL and branch name.                                                                                                                                                                                                                               | Rooms name branches directly; attempts are gone.                                                                                                      |
| Commit              | A git commit.                                                                                                                                                                                                                                                                                              | What landing links and checks bind to.                                                                                                                |
| Forge               | The service that holds remotes, pull requests, reviews and branch protection.                                                                                                                                                                                                                              | Its reports count as `asserted`. The forge approves and lands; ParleyInn does neither.                                                                |
| Pull request        | The forge's review object for a branch.                                                                                                                                                                                                                                                                    | Separate from the room.                                                                                                                               |
| Worktree            | A git working tree on a node where a run edits a branch.                                                                                                                                                                                                                                                   | Not a room.                                                                                                                                           |
| Worktree checkpoint | An event recording a worktree's commit, branch, and redacted diff since the previous checkpoint ([REC-20][51]).                                                                                                                                                                                            | Ties checks to specific trees.                                                                                                                        |
| Check               | A command and its exit status, bound to a tree.                                                                                                                                                                                                                                                            | What backs a result.                                                                                                                                  |
| Result              | An outcome of a room's work: a check passing or failing on a tree, or a stated outcome. It names the intent version and carries one evidence class.                                                                                                                                                        | Different from a check, which is the observation, and from a delegate report.                                                                         |
| Evidence            | The checks, attestations or text a result rests on.                                                                                                                                                                                                                                                        | Its evidence class ranks it.                                                                                                                          |
| Evidence class      | `claim` (text only) < `own check` < `witness check` < `CI attested`.                                                                                                                                                                                                                                       | Replaces `own run` and `witness run`.                                                                                                                 |
| Own check           | A check a hook recorded on the room's own node, run on the latest checkpoint plus the recorded edits. Otherwise it is marked `unbound` and counts as a `claim`.                                                                                                                                            | The lowest class backed by an actual run of the command.                                                                                              |
| Witness check       | A check re-run through the launcher on a fresh checkout of the exact commit, by a node whose principal authored no change in the range.                                                                                                                                                                    | Independent re-execution.                                                                                                                             |
| CI attested         | A check result for the exact commit, signed by a CI key.                                                                                                                                                                                                                                                   | The highest evidence class.                                                                                                                           |
| CI key              | A key the principal enrolled to sign CI check results.                                                                                                                                                                                                                                                     | Signs attestations only.                                                                                                                              |
| Landing             | Git or the forge merging commits into a protected branch. ParleyInn never lands anything.                                                                                                                                                                                                                  | A landing is never a verdict.                                                                                                                         |
| Landing link        | The link from a landed commit to a room, carrying a proof class.                                                                                                                                                                                                                                           | Derived by ParleyInn, not asserted.                                                                                                                   |
| Proof class         | Proven: `same commit`, `same patch`, `same tree`. Not proven: `likely`, `asserted`, and `not proven` with a reason.                                                                                                                                                                                        | Grades landing links only.                                                                                                                            |
| Room trailer        | The `ParleyInn-Room:` line on a commit made on a room's branch. Each `ParleyInn-Link:` line is a trailer link.                                                                                                                                                                                             | Counts as `asserted` until proven.                                                                                                                    |
| Qualified links     | Each link is named by what it connects: **branch link** (room to branch), **pull-request link** (branch to its PR), **criterion link** (result to criterion), **range link** (post or pin to an address range), **parent link**, **delegation link**, **invite link**, **landing link**, **trailer link**. | "Link" never stands alone ([decision 6][d6]).                                                                                                         |
| Comparison          | A side-by-side view of two branches: their exposure, outcome and evidence.                                                                                                                                                                                                                                 | Shows the difference but picks no winner.                                                                                                             |
| Outcome             | What a room's work has produced so far: its branch heads, results and evidence, as verdicts judge them.                                                                                                                                                                                                    | The outcome window shows part of it.                                                                                                                  |
| Presentation        | What a seat put in the outcome window, such as a dev server, an artifact, a file or a diff, together with the seat and branch.                                                                                                                                                                             | Replaces "present" used as a noun.                                                                                                                    |
| Verdict             | A person's `met`, `not met` or `needs changes` on one criterion. It is a `verdict` pin bound to the intent version, the heads of every branch the room names, and the results and evidence shown. It goes stale when any of these changes.                                                                 | Persons only, but any member may record one: a verdict is human judgement, now that judges are removed. Service accounts contribute evidence instead. |

## 6. Trust and flow

| Name                                          | Definition                                                                                                                                                                                                                                       | Why it exists / how it differs                           |
| --------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------- |
| Provenance                                    | An event's class, from a closed set. `operator` is the class of the events this node's principal writes through a principal surface.                                                                                                             | `operator` survives only as this event class.            |
| Trust level                                   | `trusted` or `untrusted`, derived from an event's provenance and writer.                                                                                                                                                                         | The basis for what may reach a model.                    |
| Taint                                         | A derived artifact is untrusted when any of its sources is untrusted.                                                                                                                                                                            | Applies to derived artifacts.                            |
| Recall taint                                  | A run's mark after it recalls untrusted content.                                                                                                                                                                                                 | Tightens the run's rule levels.                          |
| Trusted boundary                              | This node's `operator` and structural events, plus principal acts signed by a device key that the agent's principal certified, within that key's scope.                                                                                          | The [I2][inv] line.                                      |
| Principal surface                             | An authenticated surface for principal acts: the room view under [SEC-20][62], the CLI at a terminal, or a paired phone within its scope.                                                                                                        | Replaces "owner surface".                                |
| Presence check                                | A hardware-backed, user-verified proof bound to one widening act.                                                                                                                                                                                | Ties a widening act to a person physically present.      |
| Cut, neutral, widening                        | The three classes of principal act. A widening act needs [OWN-22][513] and, where configured, a presence check.                                                                                                                                  | Decides what each principal act requires.                |
| Recall                                        | An agent's tool call that returns enveloped content.                                                                                                                                                                                             | Pull-only, and defaults to the agent's own run.          |
| Recall scope                                  | `run`, `room` or `rooms`, or a foreign room named in the call.                                                                                                                                                                                   | Bounds what a recall may reach.                          |
| Endorsement                                   | A principal act that sends a post's displayed text to one of the principal's own agents, in a fixed template.                                                                                                                                    | One way a post reaches an agent.                         |
| Trust grant                                   | A principal's widening act that trusts another principal's key for its own agents, in one room or everywhere. It covers only that principal's device-seat and service-account-seat posts and pins, never run seats or relaying service accounts. | Replaces "trusting a poster".                            |
| Delegation                                    | Work one agent hands to another.                                                                                                                                                                                                                 | The general act; the entries below are its parts.        |
| Delegate                                      | The agent that receives a delegation.                                                                                                                                                                                                            | Keeps its own principal.                                 |
| Delegated task                                | The text of the task, delivered in [OWN-24][513]'s template.                                                                                                                                                                                     | What the delegate receives.                              |
| Delegate report                               | What a delegate returns, read through `delegation_get` and enveloped.                                                                                                                                                                            | Replaces "delegate's result".                            |
| Delegation grant                              | A principal act naming who may delegate, to which targets, at what rule level, within what budget and until when.                                                                                                                                | Required for a delegation to another principal's agent.  |
| Acceptance grant                              | Another principal's act accepting delegations from this principal.                                                                                                                                                                               | The receiving side of a delegation grant.                |
| Permission grant                              | The grant that lets an approved, parked action pass again ([OWN-07][513]).                                                                                                                                                                       | Separate from trust and delegation grants.               |
| Notice allowance, notice opt-in               | The owner's per-room allowance, and the agent's principal's opt-in. An opt-in notice needs both.                                                                                                                                                 | The two switches behind opt-in notices.                  |
| Rule level, action class                      | For each action class, one of: act without asking, act when told, ask first, hand off.                                                                                                                                                           | How far an agent may go on its own.                      |
| Away policy                                   | What happens to an unanswered held request: keep going, pause or stop.                                                                                                                                                                           | Covers the principal being away.                         |
| Residual risk, sandbox state, risk acceptance | The risks listed in [§6.1][61]; what blocks them for a run; and the principal's act accepting the ones left open.                                                                                                                                | Widening acts need each risk either blocked or accepted. |
| Hand-off, hand-back                           | An agent passes control to its principal as a held request. The principal returns control with a note and a worktree checkpoint.                                                                                                                 | The two halves of handing control back and forth.        |
| Run controls                                  | Steer, interrupt, pause, resume and stop: principal acts on a run.                                                                                                                                                                               | Direct control over a run.                               |
| Correction, retry                             | After a verdict: principal-typed text in a fixed template, or a new run from a checkpoint.                                                                                                                                                       | The two ways to act on a verdict.                        |
| Petname                                       | The name the viewer's own contacts bind to a key.                                                                                                                                                                                                | Never a name sent by a peer.                             |

## 7. Interfaces and naming rules

### Components and interfaces

- **Core (B0):** the hooks, the MCP server, the kernel worker and the CLI.
- **Launcher (B1):** `parinn launch`, which starts and hosts runs.
- **Room view component (B1), peer component (B2), publish component (B3).**
- **Boundary:** one of B0 to B3.
- **Hook:** the harness's callback into ParleyInn.
- **MCP server:** the agent's tools.
- **CLI:** `parinn`.
- **Harness skill:** may call MCP tools or the CLI. ParleyInn defines no slash
  commands, and a skill's call is the agent's own act, never a principal act.

### Naming rules

- An MCP tool for a room act is `room_<act>`. Every other MCP tool is
  `<concept>_<verb>`, with the reading verbs `get`, `list` and `search`.
- A CLI command is `parinn <concept> <verb>`.
- Flags name concepts: `--author`, `--seat`, `--writer`, `--run`, `--room`.
- Configuration keys are `<concept>.<setting>`, with the concept in the
  singular.
- The attested seat kinds are `run`, `device` and `service account`.

| Old                                                  | New                                                                          |
| ---------------------------------------------------- | ---------------------------------------------------------------------------- |
| `room_pin` op add / edit / unpin                     | `room_pin`, `room_edit`, `room_unpin`                                        |
| `room_present` op show / pick                        | `room_present`, `room_pick`                                                  |
| `room_moderate` op kick / bar / read_only            | `room_kick`, `room_bar`, `room_mute`                                         |
| `room_pins`, `pins_list`                             | `pin_list` (`room` optional)                                                 |
| `pins_propose`                                       | `pin_propose`                                                                |
| `room_delegate` / `room_result`                      | `delegation_start` / `delegation_get`                                        |
| `room_summary`                                       | `room_summary_get`                                                           |
| `room_status` †                                      | `room_show`                                                                  |
| `search`, `expand`, `get`, `landmarks`, `stats` †    | `event_search`, `event_expand`, `event_get`, `landmark_list`, `record_stats` |
| `parinn run -- <harness>`                            | `parinn launch -- <harness>`                                                 |
| `parinn steer\|interrupt\|pause\|resume\|stop <run>` | `parinn run steer\|interrupt\|pause\|resume\|stop <run>`                     |
| `--actor`; envelope `actor`                          | `--author`; `author`                                                         |
| envelope `notice` †                                  | `warning`                                                                    |
| `parinn room read-only … on\|off`                    | `parinn room mute\|unmute`                                                   |
| `parinn room present\|pick`                          | unchanged (room acts)                                                        |
| `parinn room stamp\|unstamp`                         | `parinn pin stamp\|unstamp`                                                  |
| `parinn pin remove`                                  | `parinn pin unpin`                                                           |
| `parinn verdict <room> …`                            | `parinn verdict record <room> …`                                             |
| `parinn trust add`                                   | `parinn trust grant`                                                         |
| `parinn answer`                                      | `parinn held-request answer`                                                 |
| `parinn endorse`                                     | `parinn post endorse`                                                        |
| `parinn correct`                                     | `parinn correction send` (`--retry-from`)                                    |
| `parinn reject`                                      | `parinn room reject`                                                         |
| `parinn rooms` (Fleet)                               | `parinn room list` (Rooms overview)                                          |
| start a witness run                                  | `parinn check witness`                                                       |
| `notices.allow.<room>` / `notices.opt_in.<room>`     | `notice.allowance.<room>` / `notice.opt_in.<room>`                           |
| `away.<room>`; "fleet default"                       | `away_policy.<room>`; node default                                           |
| `own run`, `witness run`                             | `own check`, `witness check`                                                 |
| room role operator; operator appointment             | moderator; appointment                                                       |
| seat kinds person / agent / bot                      | `device`, `run`, `service account`                                           |
| owner act, owner key, owner surface, owner-typed     | principal act, principal key, principal surface, principal-typed             |
| writer key, writer certificate, bound human          | seat key, seat certificate, principal key chain                              |
| claim of work                                        | `stake` pin                                                                  |
| [LANE-12][511] request; Q3 "co-author request"       | directed post; role request                                                  |
| `/pin`, `/unpin`, `/intent`                          | removed                                                                      |

† These renames follow only from the naming rules; no decision of yours asked
for them.

### No longer ParleyInn concepts

- bot, co-author, actor, poster, bound human
- attempt, claim of work, fleet, read only
- run component
- operator (as a role or as a person)
- owner act, owner key, owner surface
- session, project
- participant, player
- judge, approval gate
- hide
- lane, which survives only in the LANE and VIEW requirement ids
- tenant, which survives in [I4][inv] and [I8][inv] until the ADR rewords them

## 8. Invariant wording to change (for the ADR)

| Invariant | Current words                                                                                                   | Proposed                                                                                             |
| --------- | --------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| [I1][inv] | "data an operator explicitly purges"                                                                            | "the node's principal"                                                                               |
| [I2][inv] | "another writer, node or person"                                                                                | "another writer, node or principal"                                                                  |
| [I2][inv] | "owner acts signed by a device key the owner certified"                                                         | "principal acts signed by a device key the agent's principal certified"                              |
| [I2][inv] | "Without an owner act"; "On an owner act"                                                                       | "Without a principal act"; "On a principal act"                                                      |
| [I2][inv] | "owner-typed text"                                                                                              | "principal-typed text"                                                                               |
| [I2][inv] | "a person may also trust a poster … that poster's posts … the person's own text"                                | "a principal may trust another principal by key … that principal's posts … the principal's own text" |
| [I4][inv] | "nodes the owner enrolled"; "off until the tenant turns it on"; "hosts the owner names"; "the tenant turned on" | "the node's principal" in each place                                                                 |
| [I5][inv] | "a request that node's operator applies"                                                                        | "a quarantine request that node's principal applies"                                                 |
| [I6][inv] | "visible to operators"                                                                                          | "visible to the node's principal"                                                                    |
| [I8][inv] | "Isolation follows the tenant"; "one tenant's home"; "another tenant or node"; "this tenant's agents"           | "principal" in each place                                                                            |

[I3][inv], [I7][inv], [I9][inv] and [I10][inv] need no change. `operator` in
[I2][inv] stays, as the name of the event class.

## 9. Open choices

1. **Create room.** Decision 11 lists it as a principal act; [decision 19][d19]
   lets agents create rooms.

  - (a) Make it the first room act of the new seat, signed with the new seat
    key, so agents can create rooms.
  - (b) Keep it a principal act only; an agent's `room_create` then becomes a
    request its principal confirms.
  - **Recommendation: (a)**, removing create room from the principal-act list.

2. **Personal room for service accounts.** Decision 19 gives personal rooms to
   people only; [decisions 2][d2] and [3][d3] let service accounts run agents.

  - (a) One personal room per principal per node.
  - (b) Service-account runs start with no personal room.
  - **Recommendation: (a)**, because event routing needs a fallback seat.

3. **Naming a branch in a room.** Decision 7 drops the old "branch" capability.

  - (a) Adding a branch link is part of the contributor's link capability.
  - (b) Only moderators may add branch links.
  - **Recommendation: (a).**

4. **What [SEC-32][62]'s limits attach to.**

  - (a) To the appointment.
  - (b) To any seat that is not a person's.
  - **Recommendation: (a).** An owner who deliberately gives a service account
    the moderator role gets an unlimited moderator, and a service account that
    owns a room keeps full owner rights.

5. **A branch with no remote.**

  - (a) A provisional node-local identity, rebound when the branch is pushed,
    the way [LANE-02][511] already handles repositories.
  - (b) The branch cannot be linked until it is pushed.
  - **Recommendation: (a).**

[16]: ../../docs/domain-model/index.md
[51]: ../../docs/srs/05-functional-requirements.md#51-record-rec
[511]: ../../docs/srs/05b-lane-requirements.md#511-room-lane
[512]: ../../docs/srs/05b-room-view-requirements.md#512-room-view-view
[513]:
  ../../docs/srs/05c-principal-and-peer-requirements.md#513-principal-acts-own
[53]: ../../docs/srs/05f-pins-recall-and-restore-requirements.md#53-pins-pin
[61]: ../../docs/srs/06-security.md#61-threat-model
[62]: ../../docs/srs/06-security.md#62-security-requirements-sec
[d1]: #d1-owner-is-a-room-relation
[d10]: #d10-foreign-room-means-no-seat-held
[d12]: #d12-one-key-per-seat
[d18]: #d18-one-merge-rule-for-all-room-state
[d19]: #d19-earlier-decisions
[d2]: #d2-service-account-replaces-bot
[d3]: #d3-principal-is-a-person-or-a-service-account
[d4]: #d4-run-means-an-agent-run-only
[d6]: #d6-link-stays-generic-always-qualified
[inv]: ../../docs/srs/invariants.md
[model]: ../../docs/domain-model/index.md
