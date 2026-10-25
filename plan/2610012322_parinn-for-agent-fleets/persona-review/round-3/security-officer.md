# Round 3: security officer

Target: proposal.md at 77ea1d9. Most round-2 findings resolved, four
partly; no give-up condition tripped as written.

## Round-2 findings

- Resolved: confirmable content (PIN-03, ADM-07, REC-17); steering
  (OWN-12); boundaries (rows 25 and 26, `parinn-peer` in B2 only,
  `parinn-bridge` in ENG-16); presence (OWN-11); review gate (ENG-29,
  plan 2610022338's phase 1); binds (SEC-24); CLI owner acts; the
  closed set of automatic writes (I2); `parinn-run` children; B3
  credentials, `lane_status` fields, random lane ids, discovery.
- Partly resolved:
  1. Purge: SEC-31 exists, but copies remain (findings 3 to 5).
  2. Tamper evidence: the VIEW-10 receipt goes outside `PARINN_HOME`,
     where a same-user agent can still rewrite it; nothing requires it
     off the node.
  3. Revocation backdating: delegated keys escape the rule (finding 2).
  4. Lane sharing: LANE-10 reviews and PEER-08 encrypts, but PEER-02
     still serves any enrolled peer (finding 6).

## New findings

1. Important: the OWN-12 code can be recorded from the `parinn-run`
   banner, and `script -c` fakes a terminal. Draft: the code MUST name
   the exact verb and arguments, MUST never be written to the record, a
   log or a file; `parinn status` MUST say when OWN-11 acts rest on
   OWN-12 alone; managed policy MUST be able to require an
   authenticator.
2. Important: a revocation MUST extend to every writer key and
   certificate the revoked key issued that the node did not hold before
   the revocation.
3. Important: purge MUST erase the purged range from every backup
   ParleyInn made, or list each one in the purge report.
4. Important: SEC-31 MUST carry ADM-07's priority, P0.
5. Important: every purge MUST replicate, and a peer that applies it
   MUST run SEC-31 on every copy it holds in any writer's log.
6. Important: a node MUST serve a lane's segments only to keys that
   are members of that lane.
7. Important: `parinn export` MUST be an OWN-12 owner act, MUST pass
   SEC-26's review whenever it includes another writer's events, and
   MUST be audited.
8. Minor: plan 2610022338's phase 1 MUST state that neither harness
   receives the other's events except through enveloped recall.
9. Minor: terminal signals MUST go only to the terminal, never into
   hook output the harness adds to context.
10. Minor: `parinn-publish` MUST bind only to configured addresses, none
    by default.
11. Minor: row 25 says "outside", against SEC-19's one-of-B0-to-B3;
    row 26 is P2 while OWN-18 is P1.
12. Minor: minting an enrolment token MUST require OWN-11.
13. Minor: the stray heading in §10.12; REC-24, REC-25, VIEW-19 and
    OWN-21 outside their tables; SEC-30 without Ver or Inv.
14. Minor: the pitch claims "tamper-evident" outright while sealing is
    P1 and the key can be a readable file.

Verdict: serves me; the closest give-up conditions are erasure copies
(3 to 5) and a readable confirmation code (1).
