# Domain model: ninth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the eighth-round decisions were applied (commit 2292e9f), against
itself, all of `docs/srs`, the invariants and `features/`, reading
every file from disk. This note keeps the needs-fix findings, each
checked against the files. All three found no excluded term outside
where the model allows it. Each question takes its recommended option,
as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                                                 | Found | Decision                                                                                                                                                      |
| --- | ---------------------------------------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Is terminal takeover a principal act?                                                    | 3/3   | No: the principal typing in the harness's terminal the launcher hosts is the harness's own channel (OWN-02, OWN-19); run controls are the other five.         |
| Q2  | Which class does a post take, when "a room act takes its seat's pin class"?              | 2/3   | `post`; every other room act takes its seat's pin class, `operator` for a device seat and `assistant` for a run seat.                                         |
| Q3  | Who confirms an intent or criterion candidate?                                           | 3/3   | Only the owner, making a new version of the intent pin its device seat authors; any other candidate its principal.                                            |
| Q4  | May the owner's device seat unpin another seat's pin?                                    | 1/3   | Yes, as a moderator may, with the same exception: a device-seat pin keeps restoring to every agent it restored to until its author's principal unpins it.     |
| Q5  | Who revises the intent after a handover?                                                 | 1/3   | The new owner: its revision adds a version its device seat authors.                                                                                           |
| Q6  | Do room acts widen what reaches an agent, given trust-granted posts?                     | 1/3   | Not on their own: only principal acts such as a trust grant, an endorsement or a stamp widen it.                                                              |
| Q7  | Which class are applying a quarantine request, dismissing a Q3 or Q4 item and hand-back? | 2/3   | Applying a quarantine request takes the class of the quarantine it applies; dismissing a Q3 or Q4 item is neutral; a hand-back answers a hand-off (widening). |
| Q8  | Which structural events does I2 trust?                                                   | 1/3   | Events of provenance `structural`; a hook observation keeps only structural fields, and LANE-05 and VIEW-04 derive from structural fields.                    |
| Q9  | Is repository configuration a concept?                                                   | 2/3   | A qualified configuration: a repository's `.parinn.toml`, which may only tighten the principal's configuration (ADM-04).                                      |
| Q10 | How does a witness node show its principal authored no commit?                           | 1/3   | By its **git identity**: the author and commit-signing identities its git configuration sets authored no commit in the range.                                 |
| Q11 | I2's untrusted list leaves out stamped pin versions; I7's title says "Configuration".    | 2/3   | Reword by ADR-2610062400: I2 excepts a pin version the agent's principal stamped; I7 reads "Harness configuration"; I10 says stats.                           |
| Q12 | With two owners, who acknowledges an overlap?                                            | 1/3   | Each room's owner, for its room (LANE-13).                                                                                                                    |

## 2. Fixes that need no decision

- **Model:** restore block lists omitted pins' ids and count; backup is
  a copy of the store; pin list, pin class, key set, device
  certificate, canary and canary event defined; a component is a part
  ParleyInn plays; an agent has a seat through its run's member seats;
  principal acts are signed by a device key once PRV-10 ships; device
  scope covers the posts and pins its seats may write.
- **SRS:** SEC-26's binding statement is signed by the pull-request
  author's commit-signing identity (LANE-15); SEC-32 "the marked range
  it names" for a facilitator's "flagged" content; §9.5 `parinn room
  present|pick` lets any seat with the present capability present;
  `parinn room leave` leaves with a device seat only; `parinn
  join-request make` becomes `parinn run join <run> <room>`, the
  principal's neutral ask, after which the run's MCP server records its
  join request (LANE-23); OQ-19 "forked repositories"; CON-02 and §6.3
  "part" for a component; "device certificate" (SEC-10) and OPS-04's
  "canary event" now model terms.

## 3. Optional, not chased

"ephemeral node", "rule-setter", "payload store", "managed lock",
"pause ingestion", "web service", "login", OS "home", "stated
outcome", "holding a clone", the `room_` prefix, `recall.default_scope`,
"branch assignment", "writer log", "the abandoned run", OQ-33's
"automations", the review tool's `verdict` field, Kernel's "Claude
runs code", and the domain-model agent's own "Check".

## 4. Invariants

I2, I7 and I10 change, recorded in ADR-2610062400.
