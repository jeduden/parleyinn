# Persona review: platform operator

Target: pitch v7, plan 2610012322 (plan, traces, ux/) and plan
2610022338, at 618fafb.

1. Blocking: a tenant can open a LAN listener and I cannot lock it off;
   SEC-19 only makes B2 and B3 off by default. Draft: managed policy
   MUST be able to disable each boundary B1–B3 per host, and every
   ParleyInn binary MUST refuse to start a component its policy disables,
   auditing the refusal.
2. Blocking: the browser can change the trust mode and start peering,
   contradicting "the UI server never writes agent configuration".
   Draft: the UI server MUST NOT write ParleyInn configuration, the trust
   mode or the boundary state; each change MUST go through the CLI with
   a shown diff, and managed policy MUST be able to pin it.
3. Blocking: purging one user's data has no fleet-wide path; ADM-07 has
   no actor or writer scope, and a refusing peer keeps its copy. Draft:
   `parinn purge` MUST accept an actor or writer-key scope and MUST send
   the purge to every enrolled peer as a signed operator event, with
   each peer's applied, refused or unreachable state audited and
   counted.
4. Blocking: the proposal demotes my role; every journey is an
   individual developer's. Keep the platform team a named primary
   stakeholder and add a fleet-rollout journey covering parinn-ui and
   parinn-peer.
5. Important: peers can fill disks without limit (PEER-02 full copies,
   sandbox tokens with no lane limit, uncapped checkpoints). Draft:
   ParleyInn MUST enforce a configurable disk quota per writer, per peer
   and per checkpoint, refusing and auditing anything over it.
6. Important: the new resident processes have no resource bounds.
   Draft: NFR-09 MUST state peak RSS and steady-state CPU bounds for
   parinn-ui and parinn-peer.
7. Important: failures in the new components show only in a browser.
   Draft: parinn-ui and parinn-peer MUST audit and count every failure
   under OPS-01, and `parinn status` and `parinn doctor` MUST report their
   state, sync lag and any writer chain ended without a closed segment.
8. Important: the two traces assign conflicting ids (SEC-19..21,
   REC-17/18 swapped, tenant-keyed versus lane-keyed hash). Reconcile
   before the SRS change lands.
9. Important: the upgrade and restore path is undefined. Drafts: a
   segment MUST carry a format version, and a peer MUST refuse and
   audit an unsupported one; a change in a directory's project identity
   MUST be detected and audited, never start a new store silently.
10. Important: the UI token can leak between users through argv. Draft:
    the UI token MUST NOT appear in any argv or environment another UID
    can read.
11. Minor: the phone path serves a UI over B2, which the boundary table
    does not allow.
12. Minor: budgets stop at the lane; there is no fleet-wide cap.

Right already: the core stays B0, hooks fail open, rejections are
audited, the UI cannot write settings.json, no telemetry or relay.

Verdict: partly served. The standalone core is sound, but peering and
the UI arrive without operator policy locks, quotas, fleet-wide purge
or observability outside the browser.
