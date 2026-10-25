# Domain model: twelfth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the eleventh-round decisions were applied (commit c116717),
against itself, all of `docs/srs`, the invariants and `features/`,
reading every file from disk. This note keeps the needs-fix findings,
each checked against the files. Each question takes its recommended
option, as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                                                   | Found | Decision                                                                                                                                                                                                 |
| --- | ------------------------------------------------------------------------------------------ | ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Who seals a paired phone's writer?                                                         | 2/3   | The phone, with its device seat's key; the core seals every other writer but a run seat's (REC-19).                                                                                                      |
| Q2  | Does a trust grant cover a trusted principal's token-key-only device seats?                | 2/3   | No: only device seats a device key of that principal certified (I2, OWN-29, ADR-2610062700).                                                                                                             |
| Q3  | May a principal edit its restoring pin from another of its devices?                        | 1/3   | Yes: the author's principal edits a device-seat pin from any of its devices; each version keeps its own author, and the pin's author is its first one's.                                                 |
| Q4  | By which act does a principal unpin its own agent's run-seat pin?                          | 2/3   | A neutral principal act.                                                                                                                                                                                 |
| Q5  | Which class do revoking an access token, a seat key or a service account take, and others? | 1/3   | Cut: revoking an access token, a seat key or a service account's certificate, turning capture on, turning off the peer component, a bridge or the git carrier. Certifying a service account is widening. |
| Q6  | What role does a run seat get in a room its run created?                                   | 1/3   | Contributor.                                                                                                                                                                                             |
| Q7  | What origin do CLI, MCP and launcher events take?                                          | 1/3   | `witnessed`: recorded live on this node; `ingested` is from a transcript the hook handlers did not watch. Trust derives from origin too.                                                                 |
| Q8  | Is the TUI a principal surface?                                                            | 1/3   | Yes: the CLI or TUI at a terminal.                                                                                                                                                                       |
| Q9  | May a moderator by role assignment appoint the facilitator?                                | 1/3   | No: only the owner appoints the facilitator.                                                                                                                                                             |
| Q10 | Should a device seat post, link and ask for a summary from the CLI?                        | 1/3   | Yes: `parinn room post\|link` and `parinn room-summary request`.                                                                                                                                         |

## 2. Fixes that need no decision

- **Model:** the pin list leaves out pins a list removal took; a user
  turn is trusted where witnessed; an opt-in notice may carry fixed
  text ParleyInn ships; "token certificate" defined; witnessed
  `harness_meta` is trusted in either mode; the owner's device seat
  edits and unpins only pins it wrote; a fixed template may carry the
  principal-typed, endorsed or delegated text its requirement names;
  "holds" covers whatever a node stores; role assignment defined once.
- **SRS and scenarios:** PRV-02 "its witnessed `harness_meta` events,
  and its witnessed `user` turns while … `interactive`"; PRV-03 and
  §9.4 except stamped versions PRV-02 trusts; OWN-01 adds endorsed
  posts and stamped pin versions; §9.7.6 "unless a principal stamped
  them"; `parinn ui --phone`; `parinn access-token revoke`; 12's
  "concurrent branch links naming one branch"; §9.1 "the rule levels
  and permission grants the principal signed"; the keymap's `x`
  acknowledges an overlap.

## 3. Optional, not chased

"resume" in two senses; OWN-07's "was approved"; `parinn hook <event>`;
LANE-22's "petname and role"; `parinn.event_search` in §4; Kernel's
"Claude runs code"; the harness adapter's place in the Component list;
"summary store" in MEM-01; mute states in lane.feature's role column;
the domain-model agent's "check" as a verb.

## 4. Invariants

I2 changes, recorded in ADR-2610062700.
