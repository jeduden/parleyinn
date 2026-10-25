# Re-review: live collaborator

Target: proposal.md at ff6fa8e.

## Earlier findings

- Resolved: #3 ids (LANE-10..12, OWN-08, PEER-09, RCL-11), #4 Endorse
  chosen (§8.10), #5 request states (LANE-12, Q3), #7 discussion versus
  requests (LANE-12), #8 watcher request (LANE-10), #9 presence
  attribution (PEER-09, SEC-24).
- Partly resolved:
  - #1 handover (LANE-11): the former owner's role is unstated, so its
    running agents' events could be refused on every peer; who decides
    re-sign versus end, and which pins restore meanwhile, is open;
    handover lands in M9 after joining in M8. Draft: after a handover
    the former owner MUST hold the co-author role, and their agents'
    events MUST stay accepted.
  - #2 recall visibility (RCL-11): recorded but not required to be
    shown. Draft: the lane view MUST show, beside each post, every
    agent that recalled it, with the recall's address, to both the lane
    owner and the post's writer.
  - #6 joining: no time budget for the first view; off-LAN still needs
    a reachable port or VPN. Draft: a joiner's first view MUST paint
    within 5 s† of enrollment completing.
- Not resolved: none.

## New findings

1. Important: OWN-08 ("exactly the text rendered") contradicts LANE-12
   ("endorsed, with any edit shown"). Draft: an edited endorsement MUST
   record both the original and the sent text and show their diff to
   the post's writer.
2. Important: "seen by the owner" has no recorded source. Draft: seen
   MUST be recorded as an explicit owner act, or the state dropped.
3. Important: only the lane owner can endorse. Draft: any principal
   MUST be able to endorse a post to agents they are principal of.
4. Important: no M8 exit criterion exercises these journeys. Draft:
   M8's exit MUST show an invited co-author's first view, one request
   through every LANE-12 state, and an RCL-11 recall visible to both.
5. Minor: nothing requires presence or typing to be displayed.
6. Minor: the watcher's role request has no queue class or state.
7. Minor: no "handover lost" state; OQ-18 leaves Endorse presence open.

Verdict: serves a live collaborator in contract terms; handover,
recall visibility, the seen state, edited endorsements and co-author
endorse each need one more rule.
