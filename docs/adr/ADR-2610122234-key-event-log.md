---
id: ADR-2610122234
title: "A node's keys form a key event log anchored by recovery keys"
status: accepted
summary: >-
  PRV-12 makes a node's device keys, or a token-key-only node's token
  keys, a hash-linked key event log whose keys commit to an offsite
  recovery key, closing OQ-52. A process that can append to the store,
  but can neither use the device key, drive ParleyInn's CLI at the
  principal's terminal, nor produce a presence proof, can no longer make
  its own writers `witnessed` or strip the principal's pins. The
  security reviewer approved it on 13 October 2026 (ENG-29).
---
# ADR-2610122234: A node's keys form a key event log

## Context

[ADR-2610112004](ADR-2610112004-derived-artifacts-by-content.md) made
origin derive from the record (PRV-13). An event reads `witnessed` when a
key of the node's own certified its writer's seat, and SEC-10's key events
say which keys those are. A key event is signed only by the keys it names,
so a process that can append to the store can record one naming its own
key and make its writers trusted (I2). OQ-52 tracked that gap.

A first design had a key event log with recovery keys. It followed
KERI's pre-rotation ([KERI][keri], [identity-keys research][ik]). The
domain-model agents, the personas and the test-engineer reviewed it.
Their findings and the stakeholder's rulings of 12 October 2026 shape
this record:

- A key lost and minted anew keeps the earlier writers `witnessed`;
  only the new key's writers wait.
- A headless node's inception can be confirmed from another device of
  its principal, or by a confirmation key managed policy names, without
  a terminal on the node.
- An inception awaiting a confirm, a duplicity or a recovery key still
  on the node shows in the Needs you queue, Catch up and `parinn doctor`,
  not only in `parinn status`.

[keri]: https://trustoverip.github.io/kswg-keri-specification/
[ik]: ../../research/notes/identity-keys/identity-keys.md

## Decision

PRV-12 is added. Key events form a key event log in which each key
commits, by digest, to a recovery key. The stakeholder's rulings of 13
October 2026 on its review shape it:

- **Inceptions and rotations.** An inception (`minted`,
  `token key adopted`, `minted anew`) starts a key event log. A rotation
  (`rotated`, `recovered`) carries the committed recovery key, is signed
  by it and names a freshly minted key, never the recovery key itself.
  "Current" and "newest" come from hash links, never from a clock or
  arrival order.
- **An inception counts only with proof.** Other than the genesis log's,
  an inception replaces nothing and certifies no `witnessed` writer
  until a `recovered` rotation joins it, a confirm names it, or, for a
  token key, the same issuing device key made its token certificate.
- **Confirm, choose and reject need a proof the forger lacks.** Each
  counts only with an OWN-11 presence proof bound to its digest, or a
  signature by a key that counts independently of the key event log it
  names: another device of the principal once PRV-10 ships, or a
  confirmation key a recorded managed-policy listing names. The key an
  inception replaces may sign a reject, or a choose keeping its own
  side, never a confirm. Each takes effect whatever origin its own
  writer derives, and a concurrent confirm and reject leave the reject
  holding until a choose. Every signature names its kind, so one kind
  never verifies as another.
- **Reject.** `parinn key-event-log reject`, a cut act, voids an
  inception no confirm or `recovered` has counted, voids a node identity
  change event the key it replaces signs it for, and acknowledges a void
  key event. Before a confirm, choose, reject or `recovered`, the
  principal sees which seat recorded the inception, whether the key it
  replaces is still in the key store, and which pins would change; a
  `recovered` names its `minted anew` by digest.
- **Lost keys keep their writers.** A key a counting inception replaced
  stays a key of the node's own for its certificate records at or
  before the chain head that replaced it. A `minted anew` whose chain
  head is at a lower seq than a certificate record of the key it
  replaces is void and reported as tampering.
- **Node identity changes and revocations are anchored.** A node
  identity change event carries the chain head of the key event log it
  leaves, and a reject the replaced key signs voids it. A `revoked`
  counts only when the revoked key, its recovery key or its principal
  key signed it. A backup restore's effects on origin wait until its
  `minted anew` counts.
- **The recovery key comes from off the node when it can.** Where an
  authenticator is enrolled, its public half comes from there and its
  private half never touches the node. Where managed policy names a
  recipient key, ParleyInn writes each new recovery key at once, encrypted
  to it. Otherwise `parinn recovery-key write` writes it once, encrypted
  under a passphrase, and `parinn device-key rotate` writes the new one
  in the same act. A token key's stays with the issuing node, which signs the
  rotation a later access token carries.
- **Robustness.** Checking for a missing key and recording a key event
  happen under one lock, re-read under it, never past a hook's budget.
  A locked key store is never a missing key.
- **Surfacing, by milestone.** In M1, `parinn status`, `parinn verify` and
  `parinn doctor` show an inception awaiting a confirm, a duplicity, a
  void key event and a stalled key store, each counted. There are no
  presence proofs before M7, so in M1 a confirm needs a managed-policy
  confirmation key, or a `recovered` rotation brings a key back. The
  Needs you
  queue (reject offered before confirm), Catch up and restore blocks
  that name withheld pins follow with VIEW-05, VIEW-08 and PIN-10.

SEC-10, SEC-22, SEC-27, REC-19, REC-24, PRV-01, PRV-13, ADM-06, ADM-07,
PEER-06, ENG-08, the CLI table, ENG-29's list and the domain model
follow. OQ-52 is closed, and OQ-53 tracks a token-key node resumed
without its issuing node.

The stakeholder, `@jeduden`, is to approve this record as security
reviewer (ENG-29) by a comment on the pull request that adds it, naming
the record's id and the pull request's head commit. The approval covers
PRV-12 and the requirements listed above as worded there.

The stakeholder approved this record on 13 October 2026 by a [comment
on the pull request that adds it][approval], naming the record's id.
The comment came at head commit `de59777` and names no commit, so this
record binds the approval to that commit's wording of PRV-12 and the
requirements listed above.

Approval: [given][approval].

[approval]: https://github.com/jeduden/parleyinn/pull/39#issuecomment-6149850310

### Security analysis (author)

This is the author's analysis for the security reviewer, not the review
itself. The attacker it assumes can append to the store but can neither
use the device key, drive ParleyInn's CLI at the principal's terminal, nor
produce a presence proof. The PRV-12 scenario, still `@pending`, is to
prove each row.

| Attack                                                          | What answers it                                                                      |
| --------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| forge a rotation to add one's own key                           | a rotation needs the recovery key, off the node where an authenticator gives it      |
| append a `minted anew` and a confirm for it                     | the confirm needs a presence proof or an independent key, so it counts for nothing   |
| sign that confirm with the key the inception replaces           | the replaced key may reject, never confirm                                           |
| append a node identity change event to demote every writer      | a reject signed by the key it replaces voids it                                      |
| append a `revoked` to stop later certificate records counting   | a `revoked` counts only signed by the revoked key, its recovery key or principal     |
| append a backup restore act to demote reinstated writers        | the restore's effects on origin wait until its `minted anew` counts                  |
| lure the principal into a `recovered` joining a forgery         | `recovered` names its `minted anew` by digest, after ParleyInn shows what it changes |
| adopt another node's token key from the same issuer             | a token certificate must name the token key it succeeds and this personal room       |
| use the same forgery to strip the principal's pins              | an uncounted inception replaces nothing; earlier writers keep counting               |
| backdate a `minted anew`'s chain head to demote later seats     | a chain head below a certificate record of the replaced key voids it                 |
| keep a node clone's source writers `witnessed`                  | a node identity change starts a new key event log that counts only when confirmed    |
| certify new seats with a replaced or revoked key                | certificate records count only at or before the chain head that replaced it          |
| deliver key events naming this node's key from a peer or bundle | only key events this node recorded count                                             |
| cause a duplicity by starting ten processes at once             | the missing-key check and the key event are recorded under one lock                  |
| turn a locked key store into a lost key                         | a locked store is never a missing key; appends wait at an ingest marker              |

What stays open: a process that can use the device key, read the key
store or drive ParleyInn's CLI at the principal's terminal, residual risks
R3 and R7 (§6.1). It can certify seats directly, sign as the replaced
key, and, where the recovery key was minted on the node, read it until
it leaves. M1 leaves R3 open for every run, so until sandboxes ship
this record raises the bar for an append-only attacker, not for a
process that runs as the principal. An authenticator closes the
recovery-key window; only a sandbox closes the rest.

## Alternatives

- **Simple key events, as SEC-10 had them.** Any process that can append
  could add its own key (OQ-52).
- **Confirm at the terminal alone.** It leaves nothing in the record a
  forger cannot also write.
- **Demote every earlier writer after a lost key.** One stray process or
  a locked key store would strip every agent's pins.
- **Always mint the recovery key on the node.** It would sit beside the
  device key after every rotation.
- **Resolve duplicity by any new inception.** Anyone who can append
  could choose which side wins.

## Consequences

- M1 ships the device key's key event log, its certificate records, its
  recovery key, `parinn recovery-key write`, `parinn device-key rotate`
  and `parinn key-event-log confirm|choose|reject`, surfaced through
  `parinn status`, `parinn verify` and `parinn doctor`.
- M3 brings the backup restore's `minted anew`, M8 token keys, and
  PRV-10 confirmation from another device; VIEW-05, VIEW-08 and PIN-10
  bring the Needs you, Catch up and restore-block clauses.
- Property tests prove that the order a rebuild reads key events in
  never changes the key event logs, and that key events no committed
  recovery key signed, or confirms, chooses and rejects without a proof
  the forger lacks, never add a `witnessed` writer (ENG-08).
