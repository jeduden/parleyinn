I found four defects in the SRS and scenarios that need fixing, and three gaps in my own file, acts-and-roles.md, which need a stakeholder ruling. None needs new invariant wording.

**Defects to fix**

1. **Joined run seats are assumed to have work.** The LANE-01 and LMK-01 scenarios route a joined run's events to its seat in the room without any role assignment.

  - Role says a seat with no role assignment or appointment is a viewer, apart from the owner's device seats, a run's personal-room seat and a run's seat in a room it created.
  - LANE-16 says "No role MAY follow from joining", and a viewer has no work.
  - As written, both scenarios cannot pass: the events would go to the personal-room seat. lane.feature:17 also implies the default role is something other than viewer.
  - Fix: add an invite or role assignment that gives the joined seat the contributor role.

2. **LANE-19 uses "concurrently" in a sense the model does not define.** It says "When two subagents edit the same file concurrently". In the model, Concurrent means neither event is causally after the other.

  - Two subagents on one node write to writers that record each other's heads, so their edits are almost never concurrent in that sense. The Needs you item would then never fire.
  - What LANE-19 means is that the two edits overlap in time.

3. **"Concurrent" is used for things happening at the same time** in NFR-05, NFR-08, NFR-09, OWN-23, §10's soak test, the M5 pilot text and four scenarios. That gives the word a second meaning beside the model's.

  - The model's own Delegation grant entry already says "how many delegates at once", where OWN-23 says "maximum concurrent delegates".
  - Fix: use "at once" or "in parallel". The SQLite-specific uses (S2's "WAL concurrency", ADR-07's "concurrent connections") belong to SQLite and can stay.

4. **LANE-17 and LANE-16 ask the room view to show every seat's role.** The owner's device seats have no role, and nor does a paired phone's personal-room seat. LANE-22 already handles this with "role, or owner". The principals-and-agents specialist shares this, since Owner is defined there.

**Gaps in my own file, for the stakeholder**

5. **Room merge's coverage list leaves out branch links**, although the same entry gives their conflict rule. LANE-01 also says branch links resolve under LANE-31.
6. **The owner's own runs that join the owner's room default to viewer**, so their events never route there.

  - LANE-16 gives the owner's agents' run seats "only their role and any appointment", and no role follows from joining.
  - A seat id exists only after the join, so the owner would have to assign a role after every join, or invite its own key.
  - Should such a seat get contributor, as a run's seat in a room it created does?

7. **Capability is defined as "What a role lets a seat do".** But the owner's device seats have every capability without a role, and writing a room summary comes from the facilitator's appointment.

**Checked and found consistent**

- The cut, neutral and widening lists match every principal act a requirement names: OWN-11's examples, the class each §9.5 CLI row states, PEER-01/05/06/08/10/11, SEC-12/27/28/30, ADM-02/04/06, LANE-02/10/11/12/13/15/17/18/20/23/26/32, VIEW-11/13, OWN-05/07/13/16/18/21/22/23/25/26/27/28/29 and §9.7.4.
- Expire acts agree with LANE-25, PEER-05, OWN-23 and OWN-26, including that only a grant may set an expiry before PRV-10 ships.
- How the hub routes room, principal and expire acts agrees with LANE-01, OWN-02, OWN-17, §4.3 and §9.5. Join agrees with LANE-23.
- Roles and capabilities agree with LANE-16 and §9.2. Appointments agree with LANE-16 and SEC-32. Bars, kicks and mutes agree with LANE-25 and SEC-32. Pick order agrees with VIEW-22 and LANE-31.
- Handover, succession and stamps agree with LANE-11 and LANE-32. The review step agrees with LANE-10 and SEC-26.
- I2 and I10 read the same in my file, invariants.md and the requirements, including "a pin version that principal stamped", "room state" and "active pins".
- None of the excluded words that map to my concepts appear outside history: operator as a role, owner act, participant, player, read only as a seat state, judge.
- Every requirement id my file cites exists and says what my file says. OQ-41's mark matches where it is used.

```json
[
  {"file": "/home/user/cairn/features/lane.feature", "line": "11, 15, 17", "quote": "a run of \"alice\" on branch \"main\" of repository \"app\", which no room names, that joined room \"R\" ... And the events after the switch to \"feature/x\" went to the writer of the run's seat in \"R\", with no principal act ... And while a role assignment gives the run's seat in \"R\" the viewer role, or a mute covers it", "term": "Role / Work / Join", "breaks": "Role: a seat with no role assignment or appointment is a viewer (only a run's personal-room seat or its seat in a room it created defaults to contributor); LANE-16: 'No role MAY follow from joining'. A joined run seat with no stated role has no work, so its events cannot go to R as line 15 asserts, and line 17 wrongly implies the default role is not viewer.", "fix": "lane.feature:11 add after 'joined room \"R\"': ', whose owner invited \"alice\"'s principal key as contributor,'; line 17: 'while a later role assignment gives the run's seat in \"R\" the viewer role'.", "kind": "needs-fix", "invariant": false, "hard_to_revert": false},
  {"file": "/home/user/cairn/features/landmarks.feature", "line": 13, "quote": "And midway the run joins room \"L1\", which names its current branch, so its later events go to the writer of its seat in \"L1\"", "term": "Role / Work / Join", "breaks": "Same as lane.feature:11. A join gives no role (LANE-16), so the joined seat is a viewer with no work and the run's events stay on its personal-room seat.", "fix": "'midway the run joins room \"L1\", which names its current branch, under an invite that gives its seat the contributor role, so its later events go to ...'", "kind": "needs-fix", "invariant": false, "hard_to_revert": false},
  {"file": "/home/user/cairn/docs/srs/05b-lane-requirements.md", "line": 31, "quote": "When two subagents edit the same file concurrently, ParleyInn MUST raise a Needs you item naming both.", "term": "Concurrent", "breaks": "Concurrent means of two acts or events, neither causally after the other. Two subagents' edit events on one node are causally ordered through the writer heads each records (REC-10), so the trigger is undefined or never fires. The requirement means overlap in time.", "fix": "LANE-19: 'When two subagents that share the worktree both edit the same file while both run, ParleyInn MUST raise ...' (or bound the window by worktree checkpoints, if the stakeholder prefers).", "kind": "needs-fix", "invariant": false, "hard_to_revert": false},
  {"file": "/home/user/cairn/docs/srs/07-non-functional-requirements.md", "line": "19, 22, 23", "quote": "≥ 50 concurrent runs, subagents included, writing | Concurrent appends MUST never corrupt the store | ten concurrent instances", "term": "Concurrent", "breaks": "The model defines Concurrent only for acts and events, in the causal sense, and 'every word one meaning' (hub). These use it for things at the same time. The same group: 05c-principal-and-peer-requirements.md:39 OWN-23 'the maximum concurrent delegates' (the model's Delegation grant says 'how many delegates at once'); 10-engineering-quality.md:29 '50 concurrent writers'; 12-delivery-plan.md:48 'many concurrent agents'; features/non-functional.feature:59, 62, 92, 95; features/engineering.feature:90; features/record.feature:70.", "fix": "In the SRS and scenarios use 'at once' or 'in parallel': '≥ 50 runs writing at once', 'Appends running in parallel MUST never corrupt ...', 'ten instances at once', 'the maximum delegates at once', '50 writers appended in parallel', 'many agents at once'; scenarios likewise.", "kind": "needs-fix", "invariant": false, "hard_to_revert": false},
  {"file": "/home/user/cairn/docs/srs/05b-lane-requirements.md", "line": "29, 28", "quote": "shared with the room's principals, with each principal's petname and the role of each of its seats | The room view MUST show each seat its role and capabilities.", "term": "Role", "breaks": "Role: the owner's device seats have no role (the owner stands beside the roles), and nor does a paired phone's personal-room seat. LANE-17 asks for a role the owner's seats lack; LANE-22 already says 'role, or owner'. Also features/lane.feature:260. Shared with the principals-and-agents specialist (Owner).", "fix": "LANE-17: 'the role of each of its seats, or owner'; LANE-16: 'each seat its role, or owner, and its capabilities'; lane.feature:260 to match.", "kind": "needs-fix", "invariant": false, "hard_to_revert": false},
  {"file": "/home/user/cairn/docs/domain-model/acts-and-roles.md", "line": "181-184, 191-193", "quote": "It covers membership, roles, appointments, pins, pin versions, stamps, mutes, presentations, picks, bars, handovers and their offers, OWN-21's ready and abandoned marks, the CI keys enrolled in the room and whether the git carrier is enabled for it.", "term": "Room merge / Room state", "breaks": "The same entry resolves branch links ('Of two branch links naming one branch, the first in causal order stands'), and LANE-01 sends branch links to LANE-31. But the coverage list, which reads as closed, omits them. So whether the branches a room names are room state is left unsaid.", "fix": "acts-and-roles.md Room merge: '... bars, branch links, handovers and their offers, ...'", "kind": "question", "invariant": false, "hard_to_revert": false},
  {"file": "/home/user/cairn/docs/domain-model/acts-and-roles.md", "line": "113-115", "quote": "A seat with none, other than the owner's device seats, is a viewer; a run's personal-room seat, or its seat in a room it created, is a contributor.", "term": "Role / Work", "breaks": "Together with LANE-16 ('its agents' run seats MUST have only their role and any appointment'; 'No role MAY follow from joining'), the owner's own run that joins the owner's room is a viewer with no work, so LANE-01 never routes its events there. A seat id exists only after the join, so the owner must assign a role after every join, or invite its own key, which nothing addresses.", "fix": "Stakeholder to rule. Option: Role '... a run's personal-room seat, its seat in a room it created, or a seat of the owner's principal's run is a contributor', with LANE-16 to match; or allow the owner to invite its own key.", "kind": "question", "invariant": false, "hard_to_revert": true},
  {"file": "/home/user/cairn/docs/domain-model/acts-and-roles.md", "line": 127, "quote": "**Capability**: What a role lets a seat do.", "term": "Capability", "breaks": "The model elsewhere gives capabilities without a role: the owner's device seats have every room capability without one (Owner), and writing a room summary comes only from the facilitator's appointment (the same entry).", "fix": "acts-and-roles.md: 'Capability: What a role, an appointment or ownership lets a seat do.'", "kind": "question", "invariant": false, "hard_to_revert": false}
]
```

Instruction feedback:

- My instructions don't say whether an ordinary-English use of a word my file defines counts as a finding, such as "concurrent" applied to runs and processes. The hub's one-meaning rule suggests it does; a sentence saying so would help.
- Step 3 asks me to search for "any word the hub excludes in its favour", but the exclusion table doesn't say which entries point to which file. A short mapping would help, for example operator to moderator, read only to mute, participant to seat.
- My instructions don't say how to treat scenarios that leave out a precondition one of my concepts decides, such as a role default. I reported them as needs-fix.
