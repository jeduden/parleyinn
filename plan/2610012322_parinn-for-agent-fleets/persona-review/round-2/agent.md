# Re-review: Claude (the agent)

Target: proposal.md at ff6fa8e.

## Earlier findings

- Resolved: #1 owner-voice text (OWN-03, OWN-04, D4, OQ-17, new I2), #2
  owner devices and pins (§6.0 rule 1, PRV-10, OWN-17, PIN-11), #3
  notice (INJ-10, off by default), #4 envelope origin (RCL-09, §9.3), #5
  pin scope (PIN-10, P0), #7 watchdog (OWN-09, reopened by A), minors
  #8–#11.
- Partly resolved: #6 holds (NFR-01 row, OWN-06); a later approval on
  the plain-hook path never reaches the agent (B).

## New findings

A. Important: LANE-14 lets an away-policy template or a resume start a
   turn, against OWN-09 and I2. Draft: an away policy MUST act only as
   the reply to the agent's own pending request, and MUST NOT start or
   resume a turn.
B. Important: a parked approval is lost on the plain-hook path. Draft:
   where the harness has no input interface, a parked request's approval
   MUST reach the agent as fixed TrustedText naming the request id on
   its next UserPromptSubmit or SessionStart, or OWN-15 MUST list its
   absence at install.
C. Important: lane_status fields are undefined; co-author titles could
   look trusted. Draft: lane_status MUST return only a closed, listed set
   of structural fields, including the count of waiting posts and
   requests and the state of the agent's held requests by id; no field
   may hold text any writer chose.
D. Important: endorsed text arrives in the owner's voice with no
   source. Draft: an endorsement MUST reach the agent inside a fixed
   template naming the post's local petname and its recall address,
   around the endorsed text.
E. Minor: a lane id MUST be opaque and never chosen by a writer.
F. Minor: every restore block MUST name the lane whose pins it holds;
   PIN-10 must scope tenant-config pins.
G. Minor: RCL-09 and VIEW-10 use two vocabularies for chain state;
   `imported` cannot tell a first-run import from a foreign bundle.
H. Minor: PIN-03 still stores a bare seq, not the (writer, seq)
   address.

Verdict: now serves me; two untracked paths for text (A, D) and the
lane_status fields (C) remain.
