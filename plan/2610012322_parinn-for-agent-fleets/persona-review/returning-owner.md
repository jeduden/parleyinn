# Persona review: returning owner

Target: pitch v7, plan 2610012322 (plan, traces, ux/) and plan
2610022338, at 618fafb.

1. Blocking: none of my journeys reaches the contract. The proposed
   VIEW family covers live viewing only; catch-up, search, integrity
   and gap display have no ids. Draft: the SRS change MUST give VIEW
   ids to catch-up, operator search, integrity display and gap display,
   each traced to I6 or I10, with a @pending scenario.
2. Blocking: phase 1 does not test my seat. Draft: phase 1's gate MUST
   include a recorded-weekend fixture where catch-up, search-to-replay
   and lane verify pass.
3. Blocking: searching across lanes is forbidden (RCL-05) and only an
   open decision; worktrees of one repository are separate projects
   today. Draft: operator search MUST span every lane the tenant holds,
   separately from agent recall, which stays scoped under RCL-05.
4. Blocking: a lane whose capture failed looks like a quiet lane. Draft:
   catch-up MUST show every lane where capture failed, dropped or
   ingested late after the boundary, separately from lanes with no
   activity.
5. Important: "verified" overclaims until signing lands. Draft: before
   REC-17 is met, the seal MUST say "hash-chained, unsigned" and MUST
   NOT show a signature column.
6. Important: three designs for one screen disagree on ranking, state
   words and names. Draft: one catch-up surface MUST exist, with one
   state vocabulary and one ranking defined in the SRS.
7. Important: the pitch dropped "tells you when anything is missing"
   (backward trace V6).
8. Important: "since last visit" lives on one machine. Draft: the
   catch-up view MUST show the boundary it used and where it came from.
9. Minor: catch-up may lag running sessions; state its frontier.
10. Minor: say whether thinking blocks are searchable.
11. Minor: say who writes stuck-agent marks overnight under NFR-09.

Serves me if made normative: address chips; replay with gaps and
worktree fidelity; result footer counting hidden items; distinct gap,
broken and equivocation states; receipts; claims beside evidence.

Verdict: the history design is close to what I need, but none of my
journeys has an id, a phase or a gate, and silent capture failures can
still hide behind "no activity".
