# Domain model: seventeenth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the sixteenth-round decisions were applied (commit a187cc8),
against itself, all of `docs/srs`, the invariants and `features/`,
reading every file from disk. This note keeps the needs-fix findings,
each checked against the files. Each question takes its recommended
option, as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                                                   | Found | Decision                                                                                                                                                                          |
| --- | ------------------------------------------------------------------------------------------ | ----- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | May any person with a seat record a verdict, or only with a pin capability?                | 2/3   | Only a person whose device seat in the room has the pin capability (LANE-22, OWN-27, §9.5).                                                                                       |
| Q2  | What is a "derived view"?                                                                  | 2/3   | Gone: room status is computed where shown from room state, run statuses and freshness marks; the working view is never part of the record.                                        |
| Q3  | Is an unlisted principal act widening, or a gate failure?                                  | 1/3   | Both, for different acts: one no requirement names is widening; one a requirement names must be classified. OWN-18's confirmation and turning on the peer component are widening. |
| Q4  | Do `uninstall`, `backup create`, `receipt make`, `ui` and `launch` record a principal act? | 1/3   | No; `install` records only the personal room's create room act and any risk acceptance made through it (ADM-02, OWN-22).                                                          |
| Q5  | What admits the seat a re-minted seat key starts?                                          | 1/3   | Nothing carries over: it joins as any seat does (LANE-23), and roles and appointments are assigned again.                                                                         |
| Q6  | May the launcher start and control the harness, though ParleyInn never manages it?         | 1/3   | Yes: ParleyInn never manages the harness's transcripts or compaction, changes its configuration only under I7, and only the launcher starts runs in it.                           |
| Q7  | Which rooms' pins restore to a run?                                                        | 1/3   | Every room its run has a seat in, its personal room included (PIN-10), now a relation.                                                                                            |
| Q8  | Must a stamp or an endorsement come from a seat in the room?                               | 1/3   | Yes, as LANE-32 and OWN-08 say; so no stamp reaches a foreign room.                                                                                                               |
| Q9  | Who chooses a branch to compare?                                                           | 1/3   | Any principal with a seat in the room, a neutral act (VIEW-13).                                                                                                                   |
| Q10 | Is Setup a part of Health or its own surface?                                              | 3/3   | Part of Health, as the model says (VIEW's intro).                                                                                                                                 |
| Q11 | Does "rejecting a foreign room stays cut" belong in OWN-11?                                | 3/3   | No: it removes no pin, so the model's list covers it; OWN-11 drops the clause.                                                                                                    |

## 2. Fixes that need no decision

- **Model:** a structural envelope carries §9.7's closed status words
  and marks; Pin candidate no longer bolds "principal"; a token-key-only
  node signs no expire acts either; a run holds a new seat for each seat
  key minted anew; the Harness facts are never changed, not "never
  controlled".
- **SRS and scenarios:** §9.5's `install` "records no act" (ADM-02,
  OWN-22, LANE-01); LANE-23's device-seat join by its node before a
  principal or expire act; LANE-26's stake by its author's principal;
  OQ-14 without "acceptance grant"; 09b's `broken` "a chain or seal
  verification failed"; `parinn ui` and `parinn launch` beside §9.5's rule
  that a command names the component it starts; "derived view" out of
  LANE-09, VIEW-04, OWN-21 and 09b; REC-24, ADM-06 and SEC-27 on the new
  seat's join.

## 3. Optional, not chased

I4's B0 list naming the harness adapter's core part; "at most one"
facilitator against "the room's one facilitator"; "resume" in two
senses; "restored snapshot"; "single source"; the seat certificate rule
in Names; "unqualified request or link"; the outcome window's
presentation; "writer on one node"; `parinn pin remove`; the CLI request
verbs; "channel"; "stage one"; "event kind"; "config"; "operator event"
without backticks; bare "notice"; §4's launcher as the only program
starter; "none is uncertified"; "before B2" as a moment; the routing
relation's wording; Trusted sources without "once PRV-10 ships".

## 4. Invariants

None changes.
