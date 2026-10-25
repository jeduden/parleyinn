# Persona review: multi-machine developer

Target: pitch v7, plan 2610012322 (plan, traces, ux/) and plan
2610022338, on branch claude/upbeat-goodall-fbh451 at 618fafb.

1. Blocking: the same repo on different machines becomes different
   projects, so lanes split or duplicate. trace-forward.md C6 keys a
   project by the hash of the canonical repository path, which differs
   per node; LANE-01 builds lane identity on the project and RCL-08
   forbids reaching another project. Draft: a project's identity MUST
   be the same on every node that holds a clone of the same repository,
   independent of its local path.
2. Blocking: a reclaimed sandbox loses its tail, and the plan accepts
   that (ux/users-onboarding-surfaces.md, "last 3 min not received").
   PEER-04 ships only closed segments; REC-18 signs at close; nothing
   bounds how long a segment stays open; C10 defers reclaimed sandboxes
   to M9. Draft: a node marked ephemeral MUST close, sign and offer its
   open segment to a connected peer on every Stop, SubagentStop and
   SessionEnd, and at most every N seconds while a session runs. Move
   outbound sandbox sync from M9 into M8.
3. Important: no requirement bounds how live a remote lane is. UI-02's
   2 s covers only local transcripts; PEER-04's 5 s counts from segment
   close. Draft: an event written on any connected peer MUST appear in
   every connected peer's view within N seconds of being written.
4. Important: no owner key hierarchy, so relayed sandbox segments have
   no trust path. SEC-22 accepts segments only from enrolled peers; a
   sandbox enrolled at the home server is unknown to the laptop. Draft:
   every node and device key MUST be certified by the tenant's owner
   key; a peer MUST accept a relayed segment whose origin key chains to
   an owner key it trusts; revocations MUST propagate as signed events.
5. Important: peer traffic has no confidentiality requirement. Draft:
   all B2 traffic MUST be encrypted and mutually authenticated with
   enrolled keys.
6. Important: concurrent metadata edits need manual repair in the UX,
   contradicting LANE-05's conflict-free rule. Apply LANE-05's rule
   automatically and show the losing value in history.
7. Important: no unattended way to start the peer in a sandbox; PEER-01
   requires an explicit tenant action. Draft: an environment setting
   the tenant writes once MUST count as the explicit action that starts
   parinn-peer in an ephemeral sandbox.
8. Important: the sandbox path depends on one inbound-reachable node
   with no fallback. Make the git carrier the sandbox fallback and
   schedule it with sandbox sync, not last.
9. Minor: the sandbox token cannot continue an existing lane.
10. Minor: sandbox trust marking in the UX disagrees with C5.
11. Minor: the sandbox token is a bearer secret on vendor
    infrastructure; audit its rotation and revocation (I6).
12. Minor: the two traces give REC-18 different meanings (origin
    signing in the forward trace, worktree checkpoints in the
    backward trace).

Give-up list: central service passes; partitions are right in design,
but findings 1 and 6 break merging; sandbox history fails (findings 2
and 8); outbound-only peering is blocked by finding 7.

Verdict: the architecture fits, but all four journeys are post-v1, and
two blockers would make the record split, duplicate or vanish in
exactly this setup.
