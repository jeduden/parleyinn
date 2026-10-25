# Invariant edits in versions 1 to 4 of the annotated model

Text changes to GA/annotated/invariants.md beyond feature spans. Each entry
gives the text before and after with all features present (markers removed),
the core-only reading where it differs, and why. Span-only moves are listed
last. Entries E1 to E4 come from version 1 (GA/repair-v1-rulings.md); E1's
update and E5, E6 from version 2 (GA/repair-v2-rulings.md); E7 and E8 from
version 4 (GA/repair-v4-rulings.md). E8 moves E2, E3 and E5 into the new
`node-clone` feature's spans, so in a candidate without it they no longer
apply.

## E1 — I1: capture off is an exception (R12)

Before:

> The only exceptions are secrets removed by redaction before storage or on
> import, and data the node's principal explicitly purges, or removes under a
> retention policy that principal or managed policy sets. Every exception is
> recorded.

After:

> The only exceptions are secrets removed by redaction before storage or on
> import, data the node's principal explicitly purges, or removes under a
> retention policy that principal or managed policy sets, and the events of a
> run while its principal had capture off or paused, each shown as a capture
> gap. Every exception is recorded.

Core only: "The only exceptions are secrets removed by redaction before storage
and the events of a run while its principal had capture off or paused, each
shown as a capture gap."

Why: turning capture off or pausing it is a principal act every candidate has
(widening, OWN-11), and the model defines a capture gap as the part of a run
capture did not record. I1 named redaction as the only exception, so I1 and
capture off could not both hold (findings #2, #74). "Or paused" is added to the
ruling's wording because the act list classes "turn capture off or pause it"
together; without it a pause would still contradict I1. The exception is
recorded and shown, so I6 still holds.

Update in version 2 (ruling C8): "each shown as a capture gap" becomes "the
latter each shown as a capture gap".

Before (version 1, all features):

> ... and the events of a run while its principal had capture off or paused,
> each shown as a capture gap. Every exception is recorded.

After (version 2, all features):

> ... and the events of a run while its principal had capture off or paused,
> the latter each shown as a capture gap. Every exception is recorded.

Core only: "The only exceptions are secrets removed by redaction before storage
and the events of a run while its principal had capture off or paused, the
latter each shown as a capture gap."

Why: "each" took in every exception, so a redacted secret, or purged data, was
called a capture gap, which the model defines as part of a run its capture did
not record (g1 findings #6). "The latter" limits the capture-gap display to the
events capture did not record; every exception is still recorded, so I1's
protection and I6 are unchanged. Capture now says that pausing turns capture
off until the principal turns it on again, so "off or paused" names one state
reached two ways.

## E2 — I2: a predecessor's device-seat writing is not "another node's" (R6)

Before (parenthetical):

> and anything another node or principal produced, except the agent's
> principal's acts signed by a device key it certified, posts and pins written
> from a device seat such a key certified, a pin version that principal
> stamped, or what a trust grant of that principal covers

After:

> and anything another node or principal produced, except what a predecessor of
> this node's own device seats wrote, the agent's principal's acts signed by a
> device key it certified, posts and pins written from a device seat such a key
> certified, a pin version that principal stamped, or what a trust grant of
> that principal covers

Before (trusted sources):

> The trusted sources are this node's own `operator` and structural events, the
> `harness_meta` events its hook handlers recorded, ...

After:

> The trusted sources are this node's own `operator` and structural events and
> those the predecessors of its own device seats wrote, the `harness_meta`
> events its hook handlers recorded, ...

Core only: the parenthetical ends "anything another node or principal
produced, except what a predecessor of this node's own device seats wrote".

Why: R6 makes a pin qualify when written from this node's own device seat or
from a predecessor (the device seat a newly minted device seat names after a
node identity change, a node clone or a backup restore). A qualifying pin's
creating event must be trusted on this node, and a node clone is a new node,
so without this edit the carried-over constraints either stop restoring or
contradict I2 (findings #4, #20, #69, #97). The carve-out is narrow: only what
the principal's own earlier device seats, named by a seat the node itself
minted, wrote; nothing from another principal, and nothing a run seat wrote.

## E3 — I8: the same carve-out (R6)

Before:

> Content another principal wrote, or another node wrote other than this
> principal's acts signed by a device key it certified and its posts and pins
> written from a device seat such a key certified, is kept as theirs

After:

> Content another principal wrote, or another node wrote other than what a
> predecessor of this node's own device seats wrote, this principal's acts
> signed by a device key it certified and its posts and pins written from a
> device seat such a key certified, is kept as theirs

Core only: "Content another principal wrote, or another node wrote other than
what a predecessor of this node's own device seats wrote, is kept as theirs".

Why: as E2; I8 otherwise marks the carried-over device-seat pins untrusted.

## E4 — I4: one self-contained sentence per boundary component (R8)

Before:

> B1, machine: a component may listen only on loopback or on a local endpoint
> only the same OS user can reach, and connect nowhere else; the launcher
> carries into the harness's input only text the core built. B2, peer:
> encrypted, mutually authenticated connections to nodes and paired phones the
> node's principal enrolled by key, off until that principal turns it on. B3,
> public: read-only publishing and outbound exchange with hosts the node's
> principal names, off until turned on. Managed policy can disable B1, B2 and
> B3. No component sends telemetry or depends on a central or third-party
> service. ParleyInn's components send data off the machine only as recalled
> content an agent receives through a tool call, or as what ParleyInn writes to an
> agent through I2's closed paths, both of which the harness sends to its
> model, or through a B2 or B3 component the node's principal turned on;
> whatever such a component brings in is trusted only as I2 allows.

After:

> B1, machine: the launcher may listen only on loopback or on a local endpoint
> only the same OS user can reach, connects nowhere else, and carries into the
> harness's input only text the core built; managed policy can disable it. B1,
> machine: the room-view component may listen only on loopback or on a local
> endpoint only the same OS user can reach, and connects nowhere else; managed
> policy can disable it. B2, peer: encrypted, mutually authenticated
> connections to nodes and paired phones the node's principal enrolled by key,
> off until that principal turns it on and while managed policy disables it;
> what the peer component brings in is trusted only as I2 allows. B3, public:
> the publish component's read-only publishing and outbound exchange with hosts
> the node's principal names, off until turned on and while managed policy
> disables it; what it brings in is trusted only as I2 allows. B3, public: the
> bridge component's outbound exchange with hosts the node's principal names,
> off until turned on and while managed policy disables it; what it brings in
> is trusted only as I2 allows. No component sends telemetry or depends on a
> central or third-party service. ParleyInn's components send data off the machine
> only as recalled content an agent receives through a tool call, or as what
> ParleyInn writes to an agent through I2's closed paths, both of which the harness
> sends to its model, or through the peer component, or through the publish
> component, or through the bridge component.

Core only (unchanged in substance): "B0, the core (...), opens no socket and
makes no outbound connection. No component sends telemetry or depends on a
central or third-party service. ParleyInn's components send data off the machine
only as recalled content an agent receives through a tool call, or as what
ParleyInn writes to an agent through I2's closed paths, both of which the harness
sends to its model."

Why: spans can only add text when a feature is present, never when one of two
features is. Each boundary's rule therefore sits in a sentence of its own,
inside the span of the feature whose component it bounds: the launcher's and
the room-view component's (B1), the peer component's (B2), the publish
component's and the bridge component's (B3). Before, B1 sat only in the
launcher's span (a room-view component without the launcher had no rule),
managed policy's power to disable B1 to B3 sat in the launcher's span, B3's
rule sat only in the git carrier's span (a bridge without the git carrier had
none), and the off-machine clause sat in segment-exchange, which left "a
component the node's principal turned on" dangling in candidates with segment
exchange and no transport (findings #73, #87). No protection is lost: every
component keeps its listening and connection limits, is off until turned on
where it was, can be disabled by managed policy, and has what it brings in
trusted only as I2 allows. The B1 and B3 sentences repeat when both of their
features are present; each names its component.

## E5 — I2 and I8: predecessors however far back (ruling C4)

Before (I2, parenthetical):

> except what a predecessor of this node's own device seats wrote, ...

After:

> except what a predecessor of this node's own device seats, however far back,
> wrote, ...

Before (I2, trusted sources):

> The trusted sources are this node's own `operator` and structural events and
> those the predecessors of its own device seats wrote, ...

After:

> The trusted sources are this node's own `operator` and structural events and
> those the predecessors of its own device seats, however far back, wrote, ...

Before (I8):

> Content another principal wrote, or another node wrote other than what a
> predecessor of this node's own device seats wrote, ...

After:

> Content another principal wrote, or another node wrote other than what a
> predecessor of this node's own device seats, however far back, wrote, ...

Core only: the same three readings, without the principal-keys clauses that
follow them.

Why: Pin already let a device seat's pins qualify from "one of their
predecessors, however far back", while the hub gave a principal only to seats
certified by this node's device key or that of the node its home was cloned
from, and I2 and I8 left open whether a predecessor's predecessor counts. On a
clone of a clone the personal room's creating seat then belonged to no
principal, and its still-restoring constraint pins were trusted by Pin but
"another node's" by I8 (g1 findings #1, #11, #15, #21). The ruling makes the
chain explicit everywhere: the hub now gives this node's principal every seat
certified by the device key of any node its home descends from by node clone
or backup restore, however far back. The carve-out stays as narrow as E2's:
only the principal's own earlier device seats, reached through seats this
node's own seats name as predecessors, and never a run seat or another
principal. A backup restore takes a seat as predecessor only once its writer
verifies against a head receipt the principal kept apart (ruling F7), and a
revoked seat's pins and pin versions stop qualifying (ruling C7), so the longer
chain adds no unverified or revoked writer to the trusted sources.

## E6 — I2: a trust grant's restoring pins stay within the device scope (ruling F8)

Before (all features):

> the pins that principal wrote from device seats one of its device keys
> certified then restore to those agents, and its posts reach them inside the
> fixed template of OWN-29.

After:

> the pins that principal wrote from device seats one of its device keys
> certified, within that key's device scope, then restore to those agents, and
> its posts so written reach them inside the fixed template of OWN-29.

Core only: unchanged (the sentence sits in the trust-grants span; "within that
key's device scope" is nested in principal-keys, which trust-grants requires).

Why: the agent's own principal's device-seat posts and pins are trusted only
within the certifying key's device scope, but a trust grant covered another
principal's posts and pins from any device seat its device keys certified, so
a key that principal scoped to write no pins or posts still put trusted pins in
the grantor's restore blocks and posts in front of its agents (g1 finding #28).
The edit tightens I2: a grant covers less than before, never more. Trust grant,
Pin and Intent carry the same limit.

## E7 — I1: capture off, without "or paused" (version 4, ruling R6)

Before (version 3, all features):

> ... and the events of a run while its principal had capture off or paused,
> the latter each shown as a capture gap. Every exception is recorded.

After (version 4, all features):

> ... and the events of a run while its principal had capture off, each such
> stretch shown as a capture gap. Every exception is recorded.

Core only: "The only exceptions are secrets removed by redaction before storage
and the events of a run while its principal had capture off, each such stretch
shown as a capture gap."

Why: Capture defined pausing as turning capture off until the principal turns
it on again, so the model had two names for one act, and "the latter" could be
read as showing only paused stretches as capture gaps (g3-core #18, g3-nobrv #3).
Ruling R6 drops "pause" everywhere (I1, Capture, the Widening list) and
makes the gap visible in the record: turning capture off (widening) and on
(cut) are principal acts in the device seat's personal-room writer, and when
capture comes back on the hook handlers append a structural capture-gap marker,
naming both acts, to each affected run's writer; Runs shows capture gaps. The
exception is the same one, still recorded and now shown per stretch, so I1's
protection and I6 are unchanged.

## E8 — I2 and I8: the predecessor carve-outs move into node-clone sentences (version 4, ruling §1)

Before (version 3, I2 parenthetical, all features):

> and anything another node or principal produced, except what a predecessor
> of this node's own device seats, however far back, wrote, the agent's
> principal's acts signed by a device key it certified, posts and pins ...

After (version 4, all features; the repository text):

> and anything another node or principal produced, except the agent's
> principal's acts signed by a device key it certified, posts and pins ...

Before (version 3, I2 trusted sources):

> The trusted sources are this node's own `operator` and structural events and
> those the predecessors of its own device seats, however far back, wrote, the
> `harness_meta` events its hook handlers recorded, ...

After (version 4; the repository text):

> The trusted sources are this node's own `operator` and structural events, the
> `harness_meta` events its hook handlers recorded, ...

Added to I2, after the trusted-sources sentence, inside a `node-clone` span:

> The `operator` and structural events a predecessor of this node's own device
> seats, however far back, wrote, up to the last seq this node took in from it,
> count among this node's own.

Before (version 3, I8, all features):

> Content another principal wrote, or another node wrote other than what a
> predecessor of this node's own device seats, however far back, wrote, this
> principal's acts signed by a device key it certified and its posts and pins
> written from a device seat such a key certified, is kept as theirs

After (version 4; the repository text):

> Content another principal wrote, or another node wrote other than this
> principal's acts signed by a device key it certified and its posts and pins
> written from a device seat such a key certified, is kept as theirs

Added at the end of I8, inside a `node-clone` span:

> The `operator` and structural events a predecessor of this node's own device
> seats, however far back, wrote, up to the last seq this node took in from it,
> are not another node's content here: they count among this node's own (I2).

Core only: I2 and I8 read exactly as the repository's invariants without the
features' clauses ("anything another node or principal produced)"; "The
trusted sources are this node's own `operator` and structural events, the
`harness_meta` events ..."; "Content another principal wrote, or another node
wrote, is kept as theirs"). Every candidate without `node-clone` carries no
predecessor carve-out at all.

Why: ruling §1 makes node clones, predecessors and lineage acceptance the
optional `node-clone` feature. Spans can only add text, and the carve-out sat
inside lists whose connective ("except", "other than") the principal-keys
clauses also need, so the carve-out is now a sentence of its own in each
invariant rather than a list element. The new sentences are narrower than E2,
E3 and E5, never wider:

- Only `operator` and structural events count, not "what a predecessor
  wrote": in version 3 the parenthetical exempted a predecessor's interactive
  `user` and `harness_meta` events while the trusted-sources sentence admitted
  only its `operator` and structural ones, so those events were both inside and
  outside the trusted sources (g3-core #5).
- Only events up to the last seq this node took in: a node clone's earlier node
  usually keeps running, and in version 3 everything it wrote later and that
  reached the clone was trusted there (g3-nobrv #15). The lineage acceptance
  (or, with store-protection, the backup restore) names that seq for each
  predecessor's writer, so trust stays a function of the writer logs (I10).
- Predecessors are device seats only (Node: "A run seat is never a
  predecessor"), so no agent-written run-seat pin on the earlier node becomes a
  restoring pin on the clone (g3-core #9, g3-nobrv #13); a backup restore takes
  only device seats the copied node's device key certified (g3-nobrv #14).

## Span-only moves (no text change)

- I2: "." after "closed set of paths" and the words "Without a principal act"
  now sit in the launcher's span, so a candidate without the launcher reads
  "closed set of paths: restore blocks ..." and the qualifier is kept only
  where the launcher's "On a principal act recorded at that time" paths
  contrast with it (finding #77).
