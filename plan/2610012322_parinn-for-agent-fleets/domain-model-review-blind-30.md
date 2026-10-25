# Domain model: thirtieth round of blind reviews, merged

Three blind domain-model agents reviewed the model after the
twenty-ninth-round decisions were applied (commit d105bd1), in the
review workflow: two skeptics, on text accuracy and on whether the
defect is real, checked every needs-fix finding. Of 9, 8 survived, two
of them the same; the one refuted (the seat ingest starts beside a
personal-room seat) another reviewer found again and both skeptics
confirmed. Each question takes its recommended option, but the one in
section 4.

## 1. Questions and decisions

| #   | Question                                                                    | Found | Decision                                                                                                                                 |
| --- | --------------------------------------------------------------------------- | ----- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Is the seat ingest starts beside a personal-room seat a member?             | 2/3   | Yes, while the run seat it names is; it shares that seat's add where it has one (Seat, Run seat, LANE-25, REC-19).                       |
| Q2  | What is the one name for how far ingest has read a transcript source?       | 2/3   | **Ingest position**; "cursor" keeps only recall paging, the **continuation cursor**; **transcript roots** defined.                       |
| Q3  | Is a room a node holds only as a blind peer foreign?                        | 1/3   | No; "a peer's room" goes, and a room every seat left or lost to a kick or a bar is foreign.                                              |
| Q4  | Does a witness check deny the OS user's home directory?                     | 1/3   | Yes: `HOME` and the principal's home, each denied by default, and the room view says which it cannot deny (OWN-18).                      |
| Q5  | Is the free text of a cut or neutral principal act trusted on this node?    | 1/3   | No: untrusted whatever its event's trust level, keyed on the act's class (OWN-11, PRV-02).                                               |
| Q6  | Which act turns the bridge component on?                                    | 1/3   | Enabling a bridge for a host (`parinn bridge on`); it runs while any bridge stands enabled.                                              |
| Q7  | Is each room-view surface an optional client?                               | 2/3   | No: the room view is the optional client; unqualified "surface" means one of its surfaces, and acts are taken at a principal surface.    |
| Q8  | Does the owner's principal key pass its own room's admission?               | 1/3   | Yes, for its device and run seats alike; roles still come only from role assignment.                                                     |
| Q9  | Should `pin.max_restore_model_tokens` name the pin budget?                  | 1/3   | Yes: `pin_budget.max_model_tokens`, before anything depends on it.                                                                       |
| Q10 | Does a seat certificate mark a service account?                             | 1/3   | No: the mark goes; the service-account certificate or managed-policy listing in the key set decides it.                                  |
| Q11 | Where does the text the launcher carries come from?                         | 1/3   | Only text the core built and recorded, read from the record; its loopback link to the room-view component carries no text for the model. |
| Q12 | Which verb records the cut turning off the room-view component or launcher? | 1/3   | `parinn configuration accept`, at a terminal with nothing more asked; the component stops as soon as the configuration applies.          |
| Q13 | Does retiring a writer on token expiry read a clock?                        | 1/3   | No: the expire act ending the access token, recorded by the node that minted it, names the writer's last accepted seq (I10, PEER-05).    |
| Q14 | May a stamper unstamp while every seat of the owner has left?               | 1/3   | Yes; every other pin act stays frozen (LANE-11, LANE-32).                                                                                |
| Q15 | Is a principal's own cross-room post untrusted in the target room?          | 1/3   | Trust stays per event: untrusted unless PRV-02 trusts the event for the reader, never through a trust grant (LANE-29).                   |
| Q16 | Which component is ParleyInn's code on a paired phone?                      | 1/3   | The peer component (B2), with the room view's reduced client inside it (§6.3 row 17).                                                    |
| Q17 | Does accepting a join cover branches the room names later?                  | 1/3   | Yes, and the acceptance says so, so a branch link stays a room act (LANE-23).                                                            |
| Q18 | Does `parinn uninstall` stop recording without a principal act?             | 1/3   | No: it needs a terminal and records the widening act turning capture off before it removes the hook registrations (ADM-02, OWN-11).      |

## 2. Fixes that need no decision

- **Model:** the Relations entry names the seat ingest starts; **publish**
  defined beside Export; "pause or stop a run" in the cut class; recall
  taint is the delegating run's and tightens the sensitive action
  classes; Home's home id is one the environment must supply; Member
  says "left"; the pin author is defined once; Retire a writer moved to
  the record's file and Room status beside Run status, for the token
  budget.
- **SRS and scenarios:** the assets "Closed paths" and "Recall";
  `parinn needs-you dismiss` leaves directed posts to `parinn post
  dismiss`; change-log row 2.6 names `parinn list-removal make`; the
  fixture `new-harness-session-run`; "a loopback network address"; "a
  connecting process of another UID"; the plugin's first-run message;
  OWN-06's hold window; OWN-10's sensitive action classes; `TrustedText`
  in PIN-07 and OWN-04; the persona-reviewer agent says "commit author".

Before the round was applied, three files were split for their token
budgets, unchanged and keeping their numbers: §5.14 (PEER) moved to
05d-peer-requirements.md, §6.3's boundary register to
06b-boundary-register.md, and §9.5's command-line interface to
09a-command-line-interface.md.

The adversarial check found two problems, both fixed: a PRV-02 example
row set a cut act's whole event untrusted, now a step making only its
free text untrusted; and PEER-05 no longer obliged anyone to record the
expire act ending an access token, now the minting node MUST once its
clock passes the expiry. LANE-29's "for the reader" and LANE-11's
"always" came back; LANE-16 no longer lists what work covers, which the
model defines, and LANE-11 drops a sentence LANE-31 already states.

## 3. Optional, not chased

Entry-point verbs for the peer, publish and bridge components; a closed
set of refusal reasons for §9.2; moving harness configuration, harness
resume and harness clear into the harness-facts file.

## 4. Invariants and reviews

No invariant changes. Waiting for the stakeholder:

- the I4 commit-trailer wording;
- the signing and sealing of what the room-view component, the launcher
  and the bridge component record (SEC-10, SEC-20);
- I10's "statuses";
- new, marked hard to revert: whether I4's last sentence ("Data leaves
  the machine only as …") allows §6.3 row 11, where record content
  leaves through the principal's own tunnel from the B1 room-view
  component. A reviewer recommends rewording it to "ParleyInn's components
  send data off the machine only as …" through an ADR with a named
  security reviewer, and marking row 11 as resting on §1.4's reading
  until then.
