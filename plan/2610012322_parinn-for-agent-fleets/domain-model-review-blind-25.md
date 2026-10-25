# Domain model: twenty-fifth round of blind reviews, merged

Three blind domain-model agents reviewed the model after the
twenty-fourth-round decisions were applied (commit e62913a), as the
stakeholder set: one across the whole model (X), one on principals,
harness facts, places, seats, the record and pins (A), and one on acts,
trust, git and components (B). They read every file from disk. This
note keeps the needs-fix findings, each checked against the files. Each
question takes its recommended option.

## 1. Questions and decisions

| #   | Question                                                                   | Found | Decision                                                                                                                       |
| --- | -------------------------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------ |
| Q1  | Is asking for or accepting a join neutral?                                 | 1/3   | No: widening, since the run's events then go to a room other principals' nodes hold. Earlier rounds kept it neutral.           |
| Q2  | Where do a run's events go when its seat loses work or membership mid-run? | 1/3   | To its personal-room seat, judged from this node's room state when it records the event; routing never refuses an event (I1).  |
| Q3  | How does a person withdraw or change a verdict?                            | 1/3   | Never edited: a newer verdict on the same criterion supersedes it; unpinning one is neutral.                                   |
| Q4  | Who may mark a room ready?                                                 | 1/3   | The owner, or a principal whose device seat in the room is a moderator; abandoned stays the owner's.                           |
| Q5  | Under which recall scope does `delegation_get` fall?                       | 1/3   | A delegate report is recorded on the delegating side, so it stays in `run` scope (RCL-05, OWN-24).                             |
| Q6  | Whose CI keys count toward `CI attested` in a shared room?                 | 1/3   | The owner's, recorded in the room, so every principal sees one class (LANE-22).                                                |
| Q7  | What records a repository's first bind?                                    | 1/3   | A structural event at the first hook event; binding by hand or rebinding is widening.                                          |
| Q8  | Does narrowing visibility or admission stay widening?                      | 1/3   | Yes: every change of visibility, admission or a room setting is widening, whichever way it goes.                               |
| Q9  | What is the harness's own hook limit called?                               | 1/3   | The **harness timeout**, never a budget; "hook budget" means only §9.1's limit.                                                |
| Q10 | Should every cut act have a CLI verb?                                      | 1/3   | Yes: revoking a CI key, stopping publishing a room and turning the publish component off get verbs.                            |
| Q11 | Should writer-label assignment be a requirement?                           | 1/3   | Yes, in RCL-08 with its scenario.                                                                                              |
| Q12 | Which seats does a run have?                                               | 1/3   | Its personal-room seat, its create room and join acts, and the seats that name one of its seats (REC-19, REC-24), per LANE-23. |
| Q13 | May "sealed range" enter the model?                                        | 1/3   | Yes, under Segment.                                                                                                            |

## 2. Fixes that need no decision

- **Model:** the held request's agent keeps waiting at the harness's own
  prompt; a room act is what a seat key signs as the act; the room merge
  covers ready and abandoned marks; Member cites Seat key; the ingest
  seat takes over add and role only; a backup holds no key.
- **SRS and scenarios:** OPS-01's row stops calling an unparsed line
  dropped; REC-23's rows; VIEW-22 breaks ties by the lower commitment and
  says URL; "harness session" in security.feature; `landmark_list`'s
  default scope; §9.4 carries the run's seat ids; "hook budget" only in
  §9.1's sense; OWN-07 "a later allow"; the `m` key's steer; PRV-07
  "chat-role"; §6.1's "Can do" column.

## 3. Optional, not chased

§9.5 rows that leave their class unstated; `room_bar`'s `key` parameter;
`--needs`; the settings key `pin.max_restore_model_tokens`.

## 4. Invariants

None changes. The I4 commit-trailer question of round 23 still waits for
the stakeholder.
