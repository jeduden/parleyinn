# Round 3: platform operator

Target: proposal.md at 77ea1d9. Of 22 round-2 items, 17 resolved and
5 partly; no blocker.

## Round-2 findings

- Resolved: managed policy source (ADM-04); backup and restore of
  segments with new writers (ADM-05, ADM-06); ceilings (SEC-22); holds
  (OWN-06); `parinn-run` listener (SEC-29); uninstall (ADM-02); bridges
  (CON-02, ENG-16, NFR-09); segment count (REC-25); Setup (VIEW-18);
  upgrade; and the earlier #1, #2, #6, #7, #8, #10, #11, #12.
- Partly resolved:
  1. Fleet purge: SEC-31's report has no format or location; ADM-14 is
     per node. A per-node purge receipt an operator can collect is
     still needed.
  2. Rollout: the M7 exit tests one runner image, and plan 2610022338's
     phase 1 has no managed-policy check.
  3. Quotas: ADM-15 still lets a local writer grow without limit.
  4. Cloned images: REC-24 rests on an undefined "node identity";
     containers from one image share `/etc/machine-id`, and the key is
     a `0600` file baked into the image.

## New findings

1. Important: an invalid managed policy has no defined behaviour.
   Draft: when the managed policy file lacks parent-directory
   protection, is unparsable or invalid, every ParleyInn binary MUST refuse
   to start any B1–B3 component or `parinn-run`, MUST keep the core
   recording, and MUST audit and count the condition.
2. Important: purged data returns through backups. Draft: `parinn
   backup` MUST record each backup as an audited event; the purge
   report MUST list every earlier backup; `parinn restore` MUST reapply
   every purge recorded after the backup, or refuse.
3. Important: backup and the pre-migration backup MUST include the
   OPS-02 audit log with its hash chain, and restore MUST verify it.
4. Important: no retention bound. Draft: managed policy MUST be able
   to set a retention window per project; ParleyInn MUST purge past it
   under ADM-07 with tombstones, audited and counted.
5. Important: node identity MUST come from a source outside the home
   that differs between hosts cloned from one image, and an unavailable
   source MUST count as a changed identity.
6. Important: phase 1's RED test MUST start each B1 binary under a
   root-owned managed policy that disables it, and MUST fail unless the
   start is refused, audited and shown by `parinn doctor`.
7. Minor: SEC-29 says `parinn-run` is the only binary allowed
   `os/exec`, but ENG-16 exempts the kernel-worker re-exec.
8. Minor: SEC-30 has empty columns; REC-24, REC-25, VIEW-19 and OWN-21
   sit outside their tables; §10.12 has a stray heading.

Verdict: serves in structure with no blocker; no retention bound (4)
trips "resource use is unbounded", and backups undo purges (2).
