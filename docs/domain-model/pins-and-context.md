---
title: "Pins and context"
order: "05"
summary: >-
  What may reach a model: pins and their versions, types and budget, the intent, stakes, posts, room summaries, envelopes, restore blocks, opt-in notices and held requests.
---
# Pins and context

- **Pin**: Verbatim text in exactly one room, with a pin type and numbered pin
  versions, each with one author (a seat); its **pin author** is its first
  version's, and LANE-26 says who may edit or unpin it. Information, never an
  instruction. It restores under PIN-10, to runs that have had a seat in its
  room during the run, or whose agent's earlier runs had one (REC-02), only as
  a **qualifying pin**: a pin of a type that restores, by a version PIN-10 lets
  qualify for the run, trusted for the run's principal on this node by its own
  event or, once PRV-10 ships, by that principal's stamp. PIN-01, OWN-12 and
  OWN-27 name the principal act or room act that adds, edits or unpins a pin
  (LANE-26, LANE-32).
- **Pin version**: One immutable text of a pin, what a stamp covers; each edit
  adds one, unstamped (PIN-04). At most one version of a pin restores to an
  agent, the one PIN-10 chooses.
- **Pin candidate**: Proposed pin text, not yet a pin and so with no pin author,
  that an agent suggested (an `assistant` event, PIN-02) or ParleyInn detected,
  as a
  derived artifact, only in a `user` event trusted on this node, never in an
  ingested or other untrusted one (PIN-05). The confirmation of its principal
  (of the agent that suggested it or the node that detected it), a widening act
  taken only after that principal is shown its exact text, pin type and
  priority, makes it a new pin its device seat authors; for an intent or a
  criterion, only the owner's confirmation, as a new version of the room's
  intent pin (PIN-02, PIN-05, LANE-20).
- **Configuration pin**: A pin the principal's configuration declares, which
  its device seat on that node adds, edits or unpins when that configuration is
  accepted (PIN-01, ADM-04).
- **Pin priority**: An integer the pin's author sets with it, lower first, by
  which PIN-08 orders qualifying pins in the pin budget. A pin keeps the
  priority and pin type it was written with (PIN-04).
- **Budget**: Every budget is named. The **pin budget** is the restore block's
  share of model tokens (PIN-08, ADM-04) and I3's budget; the **hook budget** a
  hook handler's time limit (§9.1, NFR-02); the **step budget** how many steps a
  kernel execution may take (CMP-05); and the delegation budget. A limit keeps
  its own name, such as the **restore block limit** (INJ-07), CMP-05's
  wall-clock and memory limits, SEC-04's query deadline and the output caps
  (RCL-03, CMP-06). I9's defined budgets are these named budgets and limits;
  the latency targets of NFR-01, NFR-03 and NFR-15 are targets, not budgets,
  and the harness's own limit on a hook is the **harness timeout**, never a
  budget.
- **Pin type**: One of `constraint`, `preference`, `decision`, `fact`,
  `episode`, `intent`, `verdict` and `stake`. Only `constraint`, `preference`
  and `intent` pins restore (PIN-06).
- **Unpin**: The act, of a pin's author or its principal (LANE-26), that ends
  the pin: it takes the pin off its room's **pin list** (its pins neither
  unpinned nor taken off by a list removal, the intent first) and stops it
  restoring but for a stamped version (LANE-32), while its versions stay in the
  record (I1).
- **List removal**: A moderator's or the owner's room act taking a pin, never
  the intent or a verdict, off the pin list without unpinning it (LANE-16); what
  it raises and what keeps restoring after it, LANE-26 and LANE-32 say.
- **Intent**: A room's lead pin, of type `intent`: a goal, its criteria and
  optionally the paths it is meant to change. Only the owner's principal act
  changes it (LANE-20, LANE-26). But for a stamped version, it restores only by
  its newest version (PIN-10); after a change of ownership, until the new owner
  revises it, it restores only to its stampers' agents (LANE-11).
- **Criterion**: One acceptance condition of an intent, with a stable id.
- **Stake**: A pin of type `stake` stating what its author works on (LANE-26).
- **Title, labels**: Room state the owner's device seat or a moderator sets: a
  display name and tags, reaching a model only through recall, enveloped
  (LANE-30, VIEW-15).
- **Assignment**: Room state asking a seat to work on a branch. It makes
ParleyInn
  refuse no seat's act; a role assignment is always named so.
- **Post**: Text a seat writes to a room's conversation, with provenance `post`,
  reaching an agent only by recall, endorsement or a trust grant (OWN-29). A
  post may carry range links; a **comment** is a post on a **marked range**, the
  address range a range link names. A seat's unsent post is a **draft**,
  ephemeral like a presence hint (PEER-09). A **cross-room post** is a post a
  seat sends to another room, which shows it by address (LANE-29). A post
  responding to a forge review or issue item stays in the room whether or not
  its author's separate forge reply act succeeds (LANE-08, LANE-34).
- **Directed post**: A post directed to one agent, waiting in its principal's
  Needs you queue for an endorsement (LANE-12).
- **Room summary**: A facilitator's summary of a room, always untrusted, linked
  by address to the events it covers, which an agent reads only through
  `room_summary_get` and asks for through `room_summary_request` (LANE-33,
  §9.3, CMP-03). A restore block's **room summary pointer** names the latest
  room summary by its id (LANE-33).
- **Compaction summary**: The harness's summary at compaction, recorded as
  untrusted `harness_text`, never restored (INJ-09).
- **Envelope**: The **untrusted-data envelope** of I2, the one name beside
  "envelope": the wrapper that marks recalled content as untrusted historical
  data; the only way recalled content reaches a model. An envelope **marked
  structural** carries only structural fields, §9.7's closed status words and
  marks, and ParleyInn's ids (`room_get`, VIEW-15).
- **Envelope warning**: The fixed sentence at the head of every envelope (field
  `warning`).
- **Trusted text** (`TrustedText`): Text only the core's restore-block code
  constructs, the only text the **restore builder** accepts, built only from
  qualifying pins, sanitized structural fields and fixed text ParleyInn ships
  (INJ-03): restore blocks, opt-in notices and the fixed templates of OWN-04
  and OWN-07.
- **Restore block**: Deterministic trusted text ParleyInn injects after
compaction
  (INJ-01), at a run's start or harness resume (INJ-02), and on a prompt while
  `restore_block.on_prompt` is on (INJ-04): qualifying pins, a landmark index, a
  recall hint and what else INJ-01, PIN-08, PIN-10, PIN-11, PEER-06, LANE-30 and
  LANE-33 add.
- **Landmark index**: The current run's tiers of landmark blocks (LMK-05),
  each landmark with its address range, as a restore block lists them (INJ-01).
- **Recall hint**: The one fixed line in a restore block saying that the recall
  tools reach the full history (INJ-01).
- **Opt-in notice**: Trusted text, only fixed text ParleyInn ships and what
LANE-30
  allows, saying posts or a delegate report wait, or a room changed, sent only
  under the owner's notice allowance and the agent's principal's notice opt-in
  (INJ-10). The only meaning of "notice".
- **Model**: The language model behind an agent; "the model" means it wherever
  the domain model's files, or the domain-model agent's instructions, are not
  speaking of the domain model. Its output in a turn is a **model reply**; a
  **model token** is its unit of text for budgets, counted by §8.4's estimate;
  spend counts the model's own count.
- **Working view**: Whatever is currently in the model's context window. Never
  part of the record, never authoritative.
- **Held request**: A permission request, question or hand-off with a stable id,
  answerable from any principal surface within its scope (OWN-05). Its **hold
  window** is the longest ParleyInn keeps the agent waiting on it before its
  away
  policy, if one is on, answers the agent (OWN-06, OWN-07). A **reply** answers
  a held request that is a question, with principal-typed text.
- **Qualified requests**: A permission request (the harness's, recorded as a
  held request), a role request (a viewer's room act asking for a wider role), a
  join request (a room act of a run's personal-room seat naming the room,
  LANE-23), an erasure request (a node's purge sent to its peers, PEER-11), a
  purge request (a neutral principal act asking the room's owner to purge what
  its seats wrote, SEC-30), a quarantine request (to peers) and a summary
  request (to the facilitator). "Request" never stands alone.
- **Notification**: A signal to a person: in-page, desktop or terminal on the
  same device, or through the notification bridge to a service the node's
  principal names. It carries no room text and never reaches a model (VIEW-17,
  SEC-28).
