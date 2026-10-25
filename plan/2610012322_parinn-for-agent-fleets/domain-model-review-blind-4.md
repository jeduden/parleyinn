# Domain model: fourth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the third-round decisions were applied (commit 24aacee),
against itself, all of `docs/srs`, the invariants and `features/`,
with identical prompts and no shared context. All three found no
excluded term outside where the model allows it. The count after
each finding says how many found it on their own.

## 1. Questions for the stakeholder

| #   | Question                                                                                                                                                                                                                                                                                                                                                                                                                 | Found |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----- |
| Q1  | **Trust per node or per principal.** The trusted sources are "this node's" events and `user` turns, and I2 and I8 distrust "another node", yet the trust level is the same on every node of the principal, and PRV-10 and PIN-11 trust its other devices' acts and pins (provenance.feature:61).                                                                                                                         | 3/3   |
| Q2  | **The creating seat.** A member needs an add or a run's personal-room seat; the seat whose create room act made a room, a personal room's device seat and a device seat that joins have neither, yet `room_create` calls the creator a member. Join is defined for runs only.                                                                                                                                            | 3/3   |
| Q3  | **The owner's room acts.** Room acts are checked against the seat's role, and the owner has none; LANE-16 gives it "every capability", the model does not. Title and labels are "the owner's choice", yet moderators set them.                                                                                                                                                                                           | 3/3   |
| Q4  | **Seats without a device.** A node with only a token key has no device key, so no device seat, personal room or place for events with no run; the facilitator "runs outside ParleyInn" yet has a device seat; a paired phone's writer is sealed by the node without a named key.                                                                                                                                         | 3/3   |
| Q5  | **Second senses.** One word, two meanings: *work* (capability, delegated work, result), *restore* (pins, a backup), *check* (Check, presence check), *token* (credential, model tokens), *widening* (acts, recall scope), *holds* (nodes, held request, the adapter, the home), *Needs you* (surface, run status, room status).                                                                                          | 3/3   |
| Q6  | **Invariant wording.** I1 lets only the node's principal remove data under a retention policy, but managed policy sets one too; I4 says only the core reaches the model, but the launcher delivers steers and delegated tasks; B1 is loopback only, but SEC-01 and §6.3 allow a same-user local endpoint, and a paired phone travels over B2, which reaches only nodes; I7 says managed policy, ADM-03 managed settings. | 3/3   |
| Q7  | **Agent-written pins.** A pin candidate becomes active by confirmation or a stamp, a run seat's pin by a stamp only (LANE-32 against PIN-05 and LANE-20). Who authors a candidate ParleyInn detected?                                                                                                                                                                                                                    | 2/3   |
| Q8  | **Acts of no kind.** The role request, the join request (`room_join`) and the purge request belong to no act kind, so they default to widening.                                                                                                                                                                                                                                                                          | 2/3   |
| Q9  | **"Has a seat in"** still counts kicked and barred seats, and is said of agents and runs, though the model defines it for principals only.                                                                                                                                                                                                                                                                               | 2/3   |
| Q10 | **Verdict pins.** Adding a device-seat pin is widening, yet a verdict is a device-seat pin recorded by a neutral or cut act, and viewers may record one.                                                                                                                                                                                                                                                                 | 1/3   |
| Q11 | **Owner leaves.** What ends an owner's ownership, and how does LANE-11's frozen room ever reach the handover it waits for?                                                                                                                                                                                                                                                                                               | 1/3   |
| Q12 | **Room state.** Visibility, admission, successor and the notice allowance are missing from the room merge's list. Are they room state?                                                                                                                                                                                                                                                                                   | 1/3   |
| Q13 | **Work.** Does a contributor need an assignment before it works on a branch, when event routing needs none?                                                                                                                                                                                                                                                                                                              | 1/3   |
| Q14 | **Certifier.** Who certifies a service account where no managed policy exists? May a person?                                                                                                                                                                                                                                                                                                                             | 1/3   |
| Q15 | **Own posts.** Is a principal's own device-seat post trusted for its own agents? Post says untrusted; I2's "own text" implies trusted.                                                                                                                                                                                                                                                                                   | 1/3   |
| Q16 | **Link act and trust grant scope.** Which qualified links does the link room act add, given criterion links, invite links and derived links? LANE-33 lets a trust grant cover a room summary; the model covers posts and pins only.                                                                                                                                                                                      | 1/3   |
| Q17 | **Configuration keys.** `deployment.mode` and `retention.<room>.<provenance>` break `<concept>.<setting>`. Widen the rule or rename?                                                                                                                                                                                                                                                                                     | 1/3   |

## 2. Fixes that need no decision

- **Proof mark** → proof class (LANE-15, VIEW-06, lane.feature:189,
  lane-view.feature:63).
- **Integrity status:** VIEW-10's `hash-chained, unsigned` →
  `unsigned`.
- **Trust level stored:** §8.2's `events` table and
  provenance.feature:38 store it; it is derived per principal.
- **Record** includes the node's own key set (I10); the derived
  artifact list matches I10 and drops room summaries' range links,
  which the facilitator writes; "stats" names I10's statistics.
- **Segment:** define the open segment (REC-19, CON-05, ADM-06,
  PEER-03, PEER-05 and three scenario files).
- **"Source"** unqualified in the model, §4.3 and the invariants'
  summary; the hook field `source` joins the harness names.
- **Quota scopes:** ADM-15 and the model list the same scopes.
- **Components:** CON-02's "room view" → room-view component; ADM-16
  and OPS-06 name the publish component.
- **Kernel built-ins:** CMP-03's `parinn.search` against §4.4's
  `parinn.event_search`.
- **Author** for people: §6.1, SEC-11, U5 and the witness check say
  commit author; RCL-11 and LANE-12 show a post's author's principal.
- **Seat key:** REC-18's "writer's public key".
- **Succession:** lane.feature:147-149 say a successor "completes the
  handover"; it accepts ownership by succession.
- **Comparison:** VIEW-13 and lane-view.feature:141 choose a winner;
  the act is to choose a branch to compare.
- **Working view:** §1.4's layer 3 names features, not the view.
- **Trust marks** (§9.7.6) cover items a trust grant covers; "revoked
  node" → a revoked device.
- **Delegate report:** OWN-25 exempts subagents from `delegation_get`.
- **Harness strip** for "status line" (§6.3 row 5, VIEW-15).
- **Counter:** add truncated and coalesced operations (OPS-01, OPS-06).
- **"pin"** meaning fix or lock (SEC-22, ENG-01, ENG-10, OQ-35, three
  scenario files) and **"harness"** meaning the crash-test rig
  (ENG-06, NFR-07, §11.1).
- **Qualify:** "end a grant" → a permission grant; "linked to the old
  one" (REC-24, ADM-06, SEC-27, LANE-23); "exchange writer logs" →
  segments; "another run of the same principal" → agent.
- **CLI:** `parinn trust grant|revoke` → `parinn trust-grant
  record|revoke`; `room unappoint` → `appointment revoke`; `parinn
  risk`, `room role`, `room successor`, `room handover`, `peer token`
  named after concepts; `parinn purge` gains ADM-07's and ADM-14's
  scopes; keymap `t` leaves the leave act alone.
- **Smaller drift:** LANE-22's "correction" is a post; LANE-03's
  subagent events go to the subagent's own seats; SEC-31's retention
  policy purges, not expires; visibility "shared with members" → the
  room's principals; OQ-25 proves a landing link; the OWN intro
  matches the model's principal act; Result and Outcome stop defining
  each other; the event kinds list worktree checkpoints, rotations and
  tombstones; the persona name "Returning owner", kept by the
  stakeholder, joins the model's allowed uses of "owner".
- **CLAUDE.md** says the product is written in Go; CON-01 and
  ADR-2610050528 say Rust, with Go for repository tooling.

## 3. Undefined terms

Found by all three: **turn** (LANE-14, OWN-13), **drafts** and live
drafts (PEER-09), **configuration pin** and its author (PIN-10),
**delegation link** endpoints, **hold** and hold window (OWN-06,
OWN-07, NFR-01), **open room** (LANE-13), **exposure** (VIEW-13).

Found by one or two: model and Claude (the LLM), work marker, audit
log, spend, capture, pin priority, surface, phone and login
credentials, launch credentials, controlling terminal, terminal
takeover, frontier, generation, rule engine, classifier, readable
report, publisher (SEC-26), appointed moderator.

## 4. Invariants that would need new wording

- I1: retention set by managed policy (Q6).
- I2: "this node's" and "another writer, node" against per-principal
  trust (Q1).
- I4: what "reaches the model" means; B1's same-user local endpoint;
  the paired phone over B2; the harness adapter's core part (Q6).
- I7: managed policy and managed settings; "agent configuration".
- I8: "another node wrote" against the principal's other devices (Q1).
- I10: "statistics" (fixable in the model alone).

## 5. Stakeholder decisions, 6 October 2026

The stakeholder decided Q1 to Q4 and asked for the recommended option
on every other question, so Q5 to Q17 take the recommendation.

1. **Trust (Q1):** unsigned trusted sources (this node's `operator`,
   `harness_meta` and structural events, its interactive `user` turns)
   count only on the node that recorded them; the principal's signed
   acts, device-seat pins, stamps and trust grants count alike on
   every node. I2 and I8 change by ADR.
2. **Creating seat (Q2):** an add is the act that placed a seat, its
   create room act or a join accepted under the room's admission;
   `parinn install`'s create is the device seat's add in the personal
   room; any seat joins, a run on its principal's ask or acceptance.
3. **Owner (Q3):** a room act signed by a device seat of the room's
   owner has every room capability; the owner's agents' run seats get
   only what role assignment or an appointment gives them; title and
   labels are set by the owner's device seat or a moderator.
4. **Seats without a device (Q4):** a token key certifies a
   token-key-only node's device seat, which signs no principal or
   expire acts; the facilitator is a service account with its own
   node; a paired phone seals its own writer; a run's MCP server seals
   its run seat's writer, and ingest keeps an ingested run's seat key
   like any non-run seat key.
5. **Second senses (Q5):** work is only the capability; "backup
   restore" (`parinn backup create|restore`); presence proof; model
   token and access token; widening only for acts; "holds" only for
   nodes; the run and room status Needs you becomes Asking.
6. **Invariants (Q6), by ADR:** I1 names managed policy's retention;
   I4 says only the core builds what reaches the model, B1 allows a
   same-user local endpoint, B2 reaches paired phones; I7 says harness
   configuration.
7. **Agent-written pins (Q7):** a pin candidate is proposed text, not a
   pin; confirmation makes a new pin of the principal's device seat; a
   run seat's pin restores only once stamped.
8. **Acts of no kind (Q8):** the role request and the join request are
   room acts; the purge request is a neutral principal act.
9. **Has a seat in (Q9):** through a seat that is a member; an agent
   has a seat through its run's seats.
10. **Verdict pins (Q10):** widening covers device-seat pins of a type
    that restores; a verdict is its own principal act (OWN-27).
11. **Owner leaves (Q11):** every seat of the owner's principal has
    left; ownership stays, and the owner may still hand over or name a
    successor, which ends LANE-11's freeze.
12. **Room state (Q12):** visibility, admission, successor and the
    notice allowance are room state.
13. **Work (Q13):** on any branch the room names; an assignment only
    asks.
14. **Certifier (Q14):** a person, another service account or managed
    policy certifies a service account.
15. **Own posts (Q15):** a principal's own device-seat posts are
    trusted for its own agents.
16. **Links (Q16):** the link room act adds range, branch and criterion
    links; a trust grant never covers a room summary.
17. **Configuration keys (Q17):** the concept in snake case:
    `node.deployment_mode`, `retention_policy.*`.
