# Round 4: open-source maintainer

Target: proposal.md at ca209ea. Seven of nine round-3 findings
resolved, one partly, one not; no blocker.

## Round-3 findings

- Resolved: real versus fabricated (LANE-15 binding statement; foreign
  writers chain to the owner key); the sharing path (SEC-26 covers
  every export); local flags and `parinn reject`; ingest trusts a
  stranger (REC-22, PRV-02); rows back in their tables; phase 1's gate;
  purge while a link exists.
- Partly resolved: the pitch names bundles as next, but reads as if a
  contributor must turn on peer to peer to hand one over, while SEC-26
  needs no host, peering or account. Split the sentence.
- Not resolved, important: REC-23 still refuses any bundle whose lane
  or writer this node holds, so an updated bundle and the next pull
  request are refused; §10.13 dropped the finding. Draft: import MUST
  accept a bundle that continues a held foreign writer's chain under
  the same owner key, MUST refuse one that forks it or claims a lane or
  writer of this tenant, and MUST audit both outcomes.

## New findings

N1. Important: the REC-23 finding above blocks journey 1 on every
    update.
N2. Important: nothing says how the binding statement is signed; the
    core cannot start `ssh-keygen` or gpg or reach their agents. Draft:
    `parinn export` MUST write the binding statement as a file the
    contributor signs with their own git tooling, and MUST attach and
    verify that detached signature without starting a process or
    opening a socket; verification MUST name any signature kind it
    cannot check, rather than report "no match".
N3. Minor: OWN-12's list MUST include `reject`, with its own §8.8 row.
N4. Minor: a foreign lane from an ingested transcript MUST be shown as
    `imported`, with no publisher key and every writer `unbound`, from
    the release that ships REC-22.

Verdict: serves all four journeys in the P2 design; N2 is closest to a
give-up condition.
