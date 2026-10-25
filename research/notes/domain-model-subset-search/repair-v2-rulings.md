# Version 2 repair: rulings for generation 1

Input: GA/evals/g1-repair-input.json (findings on g1-core and g1-all, both
severities). Follow GA/repair-brief.md and the marker rules in
GA/annotate-brief.md. GA/annotated/ is version 1 (snapshot GA/annotated-v1/).
Settled by the stakeholder, keep as is: the principal's own tunnel to the
loopback room view (`parinn ui --phone`, phone-scoped room-view secret) and
`git push` are the principal's tools outside I4; room trailers are always on.

Core rulings:
C1 Create room in core is only the device seat's creation of its node's
   personal room at ParleyInn's first start. `room_create`, creating any other
   room and every other create room use go to work-rooms.
C2 Range links: the room merge covers range links, and a recall that returns
   a pin returns its range links (so the link act has a use in core).
C3 Revoking this node's own current device seat's key mints a new device seat,
   certified by the node's device key, that names the revoked seat but does
   not take it as a predecessor.
C4 Clone chain: a seat certified by this node's device key, or by the device
   key of any node its home descends from by node clone or backup restore,
   however far back, belongs to this node's principal; use "however far back"
   wherever predecessors are named. Define node identity change in Node (it
   makes a node clone of the earlier node) or drop the term.
C5 Managed policy may override the principal's settings only with changes a
   cut act could make, and declares no pins.
C6 Every agent-facing output that is not recall and not a closed path (the
   result of a non-recall MCP tool, of any CLI call from inside a run that is
   not a read) carries only fixed text ParleyInn ships, structural fields and
   ids, never record text.
C7 A pin qualifies only while neither its creating event nor the version that
   restores was written by a seat whose key is revoked; revoking a device
   seat's key stops its pins and the pin versions it wrote qualifying.
C8 Capture: pausing turns capture off until the principal resumes it; in I1
   the capture-gap clause reads "..., the latter each shown as a capture gap"
   so redaction is not called a capture gap. Log the I1 wording change in
   GA/invariant-edits.md (E1 update).
C9 Minor core uselessness: give Flag a use (shown with the event on Runs and
   inside the envelope of any recall that returns it); drop the Work
   capability from core if nothing gates on it (keep it in work-rooms if a
   feature uses it).

Feature rulings:
F1 browser-room-view: `parinn ui` started at a terminal that no run's tool
   process started is a principal surface; started from a run's tool process
   it mints no room-view secret (covers `--phone`).
F2 service-accounts: a service account whose certificate or listing,
   standing or revoked, marks it as relaying, whose listing is unmarked, or
   that no standing certificate or listing covers, is refused by a trust grant.
F3 conversation / work-rooms: a cross-room post from a run seat shows in the
   target room only while a seat of the same run is a member there that may
   post.
F4 access-tokens: before PRV-10 ships, an access token's expiry may be set and
   takes effect only as a refusal by the node that minted it (Expire act).
F5 Subagent (core or delegation-grants, wherever the subagent text sits): a
   subagent's run joins each room its parent run has a member seat in, under
   the parent's join, from its first event (work-rooms span).
F6 Agent identity: each harness session's main agent and each subagent is its
   own agent; a harness clear or a new harness session starts a new agent.
F7 Backup restore (store-protection): a backup restore names a writer as
   predecessor only once it verifies to its newest seal against a head receipt
   the principal kept apart; events past that seal are read as another node's.
F8 trust-grants: a trust grant covers another principal's posts and pins only
   within the certifying device key's device scope (I2 wording via spans; log
   any invariant text change).

Then the other findings: repair core ones with the smallest coherent change;
feature ones when small and clearly right; skip wrong ones and ones about a
mechanism the candidate already cites. Never weaken an invariant.

When done: `python3 GA/check_annotation.py --no-orig` prints OK; assemble
GA/genomes/g1-core.json and g1-all.json into GA/out/v2-core and GA/out/v2-all
and read the core text. Report per ruling, then the other findings.
