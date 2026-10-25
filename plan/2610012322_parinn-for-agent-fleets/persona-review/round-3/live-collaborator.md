# Round 3: live collaborator

Target: proposal.md at 77ea1d9. Three round-2 findings resolved, the
rest partly; two important findings still trip a give-up condition.

## Round-2 findings

- Resolved: the "seen" state (dropped from LANE-12); co-authors endorse
  (OWN-08, §8.10); the watcher's request enters Needs you as Q3; the M8
  exit criteria, in text.
- Partly resolved:
  1. Handover (LANE-11): states and the concurrency rule are in. Left:
     the former owner's agents lose lane rights and their events are
     refused under LANE-10; the former owner's own role is unstated;
     nobody says who chooses whether pins are re-signed or ended; an
     unanswered offer never expires; LANE-11 is M9 while joining is M8.
  2. Recall visibility (RCL-11): "so that … see in the timeline" is a
     purpose, not a requirement to show the recall and its address.
  3. Joining: the first view has no time budget.
  4. Edited endorsement: OWN-08 records both texts, but "send exactly
     the text rendered" still reads as forbidding edits, and so does
     the pitch.
  5. LANE-12 routes requests to "the owner's endorse queue", not to the
     target agent's principal.
- Not resolved: presence and typing display (PEER-09 limits hints, but
  nothing requires showing them).

## New findings

A. Important: M8's exit needs a request in the `endorsed` state, but
   OWN-08 is in M9. Draft: OWN-08 MUST ship in the same milestone as
   LANE-12, or M8's exit MUST NOT require the endorsed state.
B. Important: roles have names but no rights; the matrix lives only in
   the non-normative UX draft. Draft: the SRS MUST list, for each lane
   role, the event kinds and owner acts it permits, and the lane view
   MUST show a joiner their role and its rights.
C. Important: after a handover the former owner's in-flight agents are
   refused. Draft: the former owner MUST hold the co-author role unless
   the new owner changes it, and their agents' events MUST stay
   accepted under it.
D. Minor: the pitch still says "the owner adopts a post".
E. Minor: §10.12 breaks "(U7 #4)" into a stray heading.
F. Minor: the lane view MUST show each connected participant's
   presence, and typing where their role allows, attributed per
   PEER-09.

Verdict: serves in contract terms; B and C trip "know clearly what I
can do" and journey 4; A blocks journey 2 in M8.
