---
title: "Acts and roles"
order: "07"
summary: >-
  The three act kinds and what they may do: room acts, principal acts and their classes, expire acts, roles, appointments, joins, moderation, handover, stamps and the room merge.
---
# Acts and roles

Room state derives from three kinds of act only. Seats that are members with no
add, and the seat ingest starts, stand beside them (Seat). Each act belongs to
exactly one kind (LANE-31).

- **Room act**: An act signed by a seat key, a device seat's taken only as
  Device seat says. It is checked as LANE-16 and LANE-24 say. The room acts are
  create room, join, a join request, leave, a role request, post, link, pin,
  edit, unpin, list removal, present, pick, kick, bar, unbar, mute and unmute;
  set title, labels or an assignment; and write a room summary or a summary
  request. A link act adds one range link, branch link, criterion link or issue
  link. A leave, kick or bar never drops a pin from any restore block, even
  where it makes the room foreign (Foreign room). Room acts are governance,
  not I2 trust: they carry no class and on their own never widen what reaches
  an agent (LANE-24).
- **Principal act**: An act of a kind OWN-11 classes, taken at a principal
  surface, signed by a device key once PRV-10 ships and recorded as OWN-02 says.
  A widening act lets more reach an agent, or more act or leave the node; a cut
  act only stops, narrows or undoes a widening; a neutral act does neither.
  Every change of visibility, admission or a room setting is widening,
  whichever way it goes. Its classes:
  - **Cut:** deny, interrupt, pause or stop a run, cancel a delegation, end a
    permission grant, reject a foreign room, decline a join request, tighten a
    rule level, revoke a role assignment, revoke a CI key, stop publishing a
    room, quarantine content without removing anything from a restore block,
    withdraw a risk acceptance, revoke a trust grant, end a delegation grant or
    an acceptance grant, revoke a forge import grant, turn an away policy
    off, withdraw a named successor,
    revoke an appointment, unstamp a pin version, turn notices off, reject an
    inception no confirm or `recovered` rotation has counted, a node identity
    change event or a void key event (PRV-12), decline a handover, withdraw as
    successor, dismiss a directed post, revoke an access token, a seat key or a
    service account's certificate unless that removes a pin from a restore block
    or stops principal acts arriving (SEC-27, PEER-06), turn capture on, and
    turn off a B1 to B3 component or a bridge, refusing no act (Component).
  - **Neutral:** mark a room ready, its **ready mark**, or abandoned, its
    **abandoned mark** (OWN-21), acknowledge an overlap, record a `met`, `not
    met` or `needs changes` verdict, unpin a verdict, open the forensic view,
    make a purge request, refuse a quarantine request, an erasure request or a
    purge request, choose a branch to compare, acknowledge counters, dismiss a
    Q3 or Q4 item other than a directed post, unpin a pin its own agent's run
    seat wrote, and add a room to or remove it from the focus set.
  - **Widening:** allow, answer a hand-off (a hand-back), reply, forge reply,
    forge import grant,
    steer, confirm
    a steer for a later turn (OWN-13), send a correction, retry from a worktree
    checkpoint, set or revise an intent, resume, record a delegation grant or an
    acceptance grant, add, edit or unpin a pin of a type that restores written
    from a device seat a device key certified (before PRV-10 ships, this node's
    own device seat), confirm a pin candidate, change a room's visibility,
    invite a principal key, issue an invite link, choose a fork, turn on the
    peer or publish component, issue a phone-scoped room-view secret, accept a
    handover or succession, ask for a device seat's join to a room its principal
    has no member seat in, ask for or accept a join, endorse, loosen a rule
    level, turn on or change an away policy other than turning it off,
    quarantine removing a pin, pin version or landmark from a restore block,
    release a quarantine, purge or apply an erasure request, export, bind a
    repository identity by hand or rebind it, accept open residual risks
    (OWN-22), certify a service account's principal key, assign a role, set a
    room's admission, appoint a moderator or the facilitator, stamp a pin
    version, name a successor, offer a handover, record a trust grant, allow
    notices for a room or opt in to them, enable a bridge, set a room setting,
    publish, apply a purge request, change a retention policy, accept
    configuration (ADM-04), enroll a CI key, rotate a device key, write a
    recovery key, confirm an inception, choose which key event of a duplicity
    to keep, retire a writer, turn capture off or pause it, enroll or revoke a
    device, peer or authenticator, mint or rotate an access token, start a
    witness check
    (OWN-18), and a backup restore. Revoking a device or a peer stays widening
    although it undoes an enrollment, since it can drop pins and stop principal
    acts arriving.
  Applying a quarantine request takes the class of the quarantine it applies
  (PEER-11). Every principal act a requirement names is classed here, and
  OWN-11 follows and classes the rest.
- **Forge reply**: A widening principal act of a person, taken after a
  principal surface shows the exact post their own device seat wrote, the
  target forge review or issue item, and the forge account that will send it.
  It authorizes the forge bridge to send that post's text to the review thread
  or as a comment on the linked issue, using the person's own forge credential
  on the enabled host (LANE-08, LANE-34, SEC-28). The act is not a forge
  approval or landing request; the post alone authorizes no send.
- **Expire act**: Once PRV-10 ships, an act a node records, signed with its
  device key, ending only an expiry its original act set. It ends a bar, a mute
  or a handover offer (LANE-25), an access token (PEER-05), or a delegation
  grant or an acceptance grant (OWN-23, OWN-26).
- **Role**: A named set of room capabilities: viewer, contributor or moderator.
  The owner gives a seat its role by a **role assignment**, which an invite, an
  invite link or a handover also records (LANE-10, LANE-11); a seat with none
  has the role LANE-16 gives it.
  - **Viewer:** read, a role request and a summary request.
  - **Contributor:** read and a summary request; post, link and present; pin,
    edit and unpin its own pins, its **pin capability**, a device seat's own
    pins being every pin its principal wrote from a device seat within that
    seat's device scope; and work.
  - **Moderator:** a contributor's capabilities, plus a list removal of any pin
    but the intent or a verdict, kick, bar, unbar, mute, unmute, pick, and set
    title, labels and assignments.
- **Capability**: What a role or an appointment lets a seat do, from the closed
  set LANE-16 lists.
- **Work**: A capability, not an act: to edit, execute commands, write worktree
  checkpoints and commit (LANE-16); an assignment only asks. As a capability,
  "work" means nothing else; what one agent hands another is a delegated task.
- **Appointment**: A principal act that makes a run seat, or a device seat of a
  principal other than the owner, an **appointed moderator**: a moderator within
  SEC-32's limits. LANE-16 says who appoints and revokes.
- **Join**: The act that adds a seat to a room under its admission, or without
  it for a device seat whose principal has a member seat there. A run joins only
  when its principal asks for the join or accepts it, a subagent under its
  parent's (Subagent, LANE-23). That covers the branches the room names later by
  branch links, whose runs' events then go there (LANE-01), so a branch link
  stays a room act. **Leave** is a seat's room act ending its own add.
- **Kick**: Revokes a seat's current add (LANE-25).
- **Bar**: Once PRV-10 ships (Principal), names a principal key and keeps every
  key that chains to it out, until an unbar or an expire act (LANE-25).
- **Mute**: Withdraws every capability but read from one seat or the whole room
  (LANE-16).
- **Present**: Puts a presentation in the outcome window.
- **Pick**: Chooses which presentation the outcome window shows.
- **Admission**: Whether a room is invite only or admits a list of principal
  keys, once PRV-10 ships; until then a room admits no other principal's seat
  (Principal). An invite names a principal key and a role (LANE-10, LANE-18).
  The owner's principal key always satisfies admission (LANE-23).
- **Review step**: The step before an invite, an export or a publish takes
  effect. It shows the acting principal what will take effect or leave, SEC-08
  applied (LANE-10, SEC-26).
- **Successor**: A principal the owner names in advance, who accepts ownership
  once all the owner's seats have left the room. That acceptance is a
  **succession** (LANE-11).
- **Handover**: Transfers ownership by an offer and an acceptance.
- **Stamp**: The act of a principal with a seat in the room on one pin version
  it was first shown in full. Once PRV-10 ships, a stamped version of a type
  that restores restores to that principal's own agents only, as PIN-10 says; no
  room act ends a stamp (LANE-32).
- **Active pin**: A pin on its room's pin list (I10).
- **Room merge**: The one rule deriving all room state from the three act kinds,
  in causal order (LANE-31). It covers membership, roles, appointments, pins,
  pin versions, stamps, branch links (Branch), issue links and active or
  revoked forge import grants (Forge), mutes, presentations, picks,
  bars, handovers and their offers, OWN-21's ready and abandoned marks and the
  CI keys enrolled in the room. It covers the successor, title, labels,
  assignment, visibility, admission, the notice allowance and the **room
  settings**: whether drafts show live, whether typing shows, the roles a
  whole-room mute leaves posting to, and the **appointment rate**. That rate is
  how many kicks, bars and mutes an appointed moderator may set per period
  (PEER-09, SEC-32). Concurrent picks resolve by **pick order**: the
  facilitator's seat, then any other moderator, then the owner (VIEW-22). Acts
  with no member seat: open (OQ-41).
- **Concurrent**: Of two acts or events: neither causally after the other;
  **causal order** puts each after every act or event it saw.
- **Room state**: Everything the room merge derives (LANE-31).
