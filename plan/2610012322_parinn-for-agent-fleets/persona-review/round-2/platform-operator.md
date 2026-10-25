# Re-review: platform operator

Target: proposal.md at ff6fa8e. Eight of twelve earlier findings
resolved, four partly; ten new, two blocking.

## Earlier findings

- Resolved: #1 policy lock (SEC-22, once N1 is fixed), #2 browser config
  (SEC-23, VIEW-18), #6 process bounds (NFR-09), #7 CLI observability
  (OPS-06, ADM-16), #8 ids (§9), #10 token argv (SEC-20, parinn-ui only),
  #11 phone (row 11, R2), #12 fleet cap (OQ-26, accepted).
- Partly resolved:
  - #3 fleet purge: ADM-14 is per node; PEER-11 needs peering. Needed: a
    per-node purge receipt in a stable format an operator can collect.
  - #4 rollout journey: M7's exit shows one lock on one image, not a
    fifty-runner rollout; workstations are now co-primary.
  - #5 quotas: a local writer over quota only raises a counter nobody
    sees on a headless runner. Needed: a managed retention setting that
    bounds the local store.
  - #9 upgrade and restore: backup and restore not done (N2).

## New findings

N1. Blocking: managed policy has no source; ADM-04's layers are
    unchanged. Draft: ParleyInn MUST read managed policy from a path the
    tenant cannot write, MUST let it override tenant and project config,
    and MUST refuse to start a B1–B3 component if that file is writable
    by the tenant.
N2. Blocking: backup and restore no longer fit the record (ADM-05/06
    still use SQLite backup; writer keys are never backed up). Draft:
    backup, restore and the pre-migration backup MUST cover segments,
    payloads and derived state; a restore MUST start a new writer and
    audit it, never reuse or lose a seq.
N3. Important: no fleet ceiling on rule levels. Draft: managed policy
    MUST be able to cap every action class's rule level, disable away
    policies and disable hook permission decisions.
N4. Important: holds stall headless runners. Draft: ParleyInn MUST NOT hold a
    request unless an owner surface is permitted and running, and managed
    policy MUST be able to disable holds.
N5. Important: parinn-run's loopback listener is outside SEC-20. Draft:
    every B1 listener MUST meet SEC-20 or use a 0600 Unix socket in
    PARINN_HOME, and MUST refuse peers of another UID.
N6. Important: cloned images share a writer key. Draft: ParleyInn MUST detect
    a home copied to another host and MUST mint and audit a new writer
    key before appending.
N7. Important: uninstall regressed. Draft: `parinn uninstall` MUST list and
    offer to remove every artifact any ParleyInn binary created, and MUST
    audit what it left.
N8. Minor: bridges sit outside CON-02, ENG-16 and NFR-09.
N9. Minor: about 2,880 segment files a day per writer with no compaction
    requirement.
N10. Minor: Setup offers import from a view VIEW-03 calls read-only.

Verdict: partly served; rollout (N1, N6), restore (N2) and fleet control
(N3, N4) still do not work for a fleet.
