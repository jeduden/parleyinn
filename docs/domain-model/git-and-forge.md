---
title: "Git and forge"
order: "08"
summary: >-
  Repositories, branches and commits, the forge, results with their evidence and proof classes, qualified links, comparisons, outcomes and verdicts.
---
# Git and forge

- **Repository**: A git repository, identified by its **repository identity**,
  a bound parentless commit of its default branch, the same on every node with
  a git clone, else a provisional node-local identity bound later by a
  structural event that names the identity it replaces (LANE-02). It has
  remotes and may carry room trailers in its commits and bundles as git refs.
- **Branch**: A git branch, identified by repository identity, remote URL and
  branch name (LANE-01). It belongs to the first of this node's principal's
  rooms whose branch link names it, concurrent links resolving by Room merge
  (LANE-01, LANE-31). So only that room takes a run's events on it, and the
  room trailer of each commit on it names that room (LANE-28). It moves only
  when a room enters or leaves those rooms or a concurrent branch link arrives
  (LANE-01); events recorded and commits made before it moves keep their room.
  Whether a link another principal's seat wrote may win a concurrent tie over
  one a seat of this node's principal wrote is open (OQ-46).
- **Commit**: A git commit.
- **Commit author**: Git's author of a commit, never a seat.
- **Worktree**: A git working tree on a node, where a run edits a branch; its
  repository's **git directory** is git's own store of objects and refs. Its
  repository identity is recorded by a structural event that names it by a keyed
  digest of its root path, so a node rebuilds which identity each worktree is
  bound to from its writer logs (LANE-02).
- **Forge**: The service that keeps remotes, issues, pull requests, reviews
  and branch protection, and approves and lands; its **forge reports** count
  as `asserted` (LANE-08). A **forge import grant** is the credential owner's
  widening principal act authorizing one named room to receive observations
  of one exact linked issue or branch. A branch is identified by repository
  identity, remote URL and branch name; its grant includes the associated
  pull request,
  reviews, branch protection and checks for that branch. The principal sees
  the target, room, forge account and current and future member scope before
  granting it. The grant binds a nonsecret credential handle to that account,
  target and room; the bridge resolves it to the exact credential it uses.
  The room record contains no token or reusable secret-derived verifier.
  A link or bridge enablement alone grants no disclosure. Only the issuing
  credential owner may revoke the grant. A node stops imports when it records
  or applies revocation; an observation recorded on a partitioned node before
  it learns of revocation remains recoverable with a concurrent revocation
  mark after sync. A **forge review item** is a review comment or
  response on a linked pull request, recorded from the forge as an untrusted
  `web` event with its forge, pull request, thread and item ids, asserted
  author, location, time, body and revision. The room shows it in the
  conversation as forge content, never as a post or an act by a room seat. A
  later edit or deletion is another event; earlier versions remain in the
  record, subject to redaction and purge.
- **Forge issue**: The forge's issue object, independent of any room or branch.
  An **issue link** connects a room to one forge issue by forge host,
  repository and issue id; it neither creates an intent nor changes trust or
  recall. A person may start from the issue by explicitly creating or choosing
  a room and adding this link; the issue's text never sets the room's intent.
  A **forge issue item** is a snapshot of the issue's title, body and
  state or one of its comments, with its forge, repository, issue and item ids,
  asserted author, time, body and revision. Under a grant, an importing member
  node records it as an untrusted `web` event in its device seat's writer in
  each authorized room. One issue linked to several rooms yields separate
  events, never a copied event or a reference into another room. A node whose
  principal has no member seat in a room cannot import there. Each room shows
  its own items with its conversation by address, never as posts or acts by a
  room seat. A later edit or deletion is another event; earlier versions
  remain, subject to redaction and purge.
- **Forge discussion item**: A forge review item or forge issue item,
  retaining which kind it is and its forge attribution.
- **Pull request**: The forge's review object for a branch.
- **Check**: A command executed, or expected to execute because one of the
  intent's criteria names it (LANE-20), on a tree, with its exit status once it
  ends; its **check state** is one of §9.7.2's. A check the forge reports, one
  of its **required checks** included, is a forge report, not a result.
- **Result**: What a room's seats established: a check passing or failing on a
  **tree** (git's snapshot of a commit's files), or a claim stated in text. It
  names the intent version and carries one evidence class; it is derived from
  the events it rests on and addressed by the event recording its exit status or
  its claim's text.
- **Evidence**: The checks, CI attestations or text a result rests on.
- **Evidence class**: Of a result, ranked: `claim` < `own check` < `witness
  check` < `CI attested` (LANE-05).
- **Own check**: A check capture recorded from the editing run's transcript on
  that run's node,
  on the latest worktree checkpoint plus the edits recorded since (LANE-05).
- **Witness check**: A check re-run through the launcher, outside any agent
  context, on a fresh checkout of the exact commit, by a node whose **git
  identity** (the author and commit-signing identities its git configuration
  sets) authored no commit on the branch since it left its base (OWN-18).
- **CI attestation**: A check's exit status for the exact commit, signed by a CI
  key the room's owner enrolled and recorded in the room (LANE-05, LANE-22).
- **Landing**: Git or the forge merging commits into the repository's default
  branch or a branch the forge protects; a landing is never a verdict. A branch
  lands when its head lands.
- **Landing link**: The qualified link from a landed commit to a room, carrying
  a proof class, derived from the record alone (LANE-06).
- **Proof class**: Of a landing link. Proven: `same commit`, `same patch`, `same
  tree`. Not proven: `likely`, `asserted`, and `not proven` with a reason
  (LANE-06).
- **Room trailer**: The `ParleyInn-Room:` line ParleyInn adds to the message of
  each commit on a room's branch, naming the room; each `ParleyInn-Link:` line
  is a trailer link (LANE-28).
- **Qualified links**: Every link is named by what it connects: a branch link
  (room to branch), a pull-request link (branch to its pull request, derived by
  ParleyInn from forge bridge events), an issue link (room to forge issue), a
  criterion link (result to criterion), a range link (post or pin to an address
  range), a parent link (subagent run to parent run), a delegation link
  (delegating run to the delegate's run, under a delegation grant), an invite
  link (which connects nothing: it carries an access token), a landing link and
  a trailer link (commit to a pin, post or marked range). "Link" never stands
  alone, except in the link act (Room act).
- **Comparison**: A side-by-side view of two branches: each one's exposure,
  results and evidence. It picks no winner.
- **Outcome**: What a room's runs have produced so far: the heads of the
  branches it names, their results and evidence, as verdicts assess them.
- **Presentation**: What a seat put in the outcome window, such as a dev
  server's URL, a build artifact, a file or a diff, with the seat and branch
  (VIEW-22).
- **Verdict**: A person's `met`, `not met` or `needs changes` on one criterion:
  a `verdict` pin, recorded as a neutral principal act bound to the intent
  version, the heads of every branch the room names on the recording node, and
  the results and evidence shown (OWN-27). It goes stale when one of them
  changes, never because the room names a branch it does not list, on another
  node or later. A newer verdict of the same person on the same criterion
  supersedes it. Service accounts contribute evidence instead.
