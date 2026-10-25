# Persona review: live collaborator

Target: pitch v7, plan 2610012322 (plan, traces, ux/) and plan
2610022338, at 618fafb.

## Blocking

1. Lane handover is one sentence and contradicted elsewhere: the
   proposed glossary fixes the owner at creation, each agent has one
   principal, pins restore only under the owner key, there is no accept
   step and no rule for concurrent handovers in a partition. Draft: a
   lane handover MUST be a signed event accepted by the new owner, MUST
   state what happens to the old owner's agents, pins and pending
   requests, and MUST resolve concurrent handovers by a documented
   deterministic rule.
2. My words can reach the owner's agent without the owner knowing:
   agents recall posts as relevant, and no surface shows it. Draft:
   every recall that returns another participant's post MUST appear in
   the lane timeline, visible to the lane owner and the post's writer.
3. Nothing for my seat reaches the contract or a phase: no ids for
   invites, roles, endorse, request status, presence or handover;
   plan 2610022338 has no multiplayer task. Draft: the SRS change MUST
   give requirement ids to lane invites, participant roles,
   endorsement, request status, presence and handover.

## Important

4. The two UX files describe incompatible multiplayer models (Adopt as
   instruction with no notice, versus Endorse with co-authors driving
   their own agents and a notice per turn). One must win before the
   SRS change.
5. My request can be ignored with no sign. Draft: each request MUST
   show its writer one state: delivered, seen by the owner, endorsed
   (with any edit shown) or dismissed.
6. Joining is not "within seconds": enrolment first, a port or VPN off
   the LAN, no role on the invite, no joiner's view. Draft: an invite
   MUST name the invitee's role, and a joiner's first view MUST show
   the lane's goal, pins, state, pending requests and latest results
   before the full sync completes.
7. Pairing talk is mixed with requests to agents. Draft: the lane MUST
   separate discussion addressed to people from requests addressed to
   an agent, and only requests MUST appear in the owner's endorse
   queue.
8. A watcher has no way to contribute. Draft: a watcher MUST be able to
   send the owner an in-product request for a wider role.

## Minor

9. Presence can be forged by any peer. Draft: presence hints MUST be
   attributed to the peer key that authenticated the connection.

Serves me already: one principal per agent; "To Sam's agent" hint;
waiting-on and Nudge; the role table; hunk attribution; the partition
band; no central service.

Verdict: the review-multiplayer model serves a live collaborator well,
but handover is unusable, the owner is never told when an agent reads
my words, and none of my journeys is in the contract or a phase.
