# Version 3 repair: rulings for generation 2

Input: GA/evals/g2-*.findings.json (core alone through three lenses; the
subsets nobrv, local, localplus, localpeer through one combined lens each).
Follow GA/repair-brief.md and GA/annotate-brief.md. GA/annotated/ is version 2
(snapshot GA/annotated-v2/). Settled by the stakeholder, keep as is: the
principal's own tunnel to the loopback room view and `git push` are the
principal's tools outside I4; room trailers are always on. Do not change
invariant text unless a ruling says so; log any change in
GA/invariant-edits.md.

Core rulings:
K1 Run seat key revocation: revoking a run seat's key while its run continues
   mints a new run seat that names the revoked one without taking it as its
   predecessor; the run's later events go there. Its class: cut, unless it
   stops a pin version from qualifying (a stamped run-seat version), in which
   case it is widening, as for any act that removes a pin from a restore
   block. Fix "An unstamp are the exceptions" so it reads as one exception.
K2 Node clone lineage: a node clone's predecessors' writes are trusted, and
   their pins qualify, only after a widening principal act on the clone
   (accepting that lineage) records it; until then the trust policy reads
   them as another node's. A backup restore keeps its verification rule.
K3 Pin list surface (core): add a core surface, or extend Runs, so the TUI and
   CLI show the personal room's pin list: every pin with its type, author,
   qualifying state and range links, so a person can find a pin to unpin,
   edit or quarantine.
K4 Configuration pins after a device seat revocation: accepting a
   configuration (even unchanged) re-records its pins from the node's current
   device seat, so they qualify again.
K5 Tool results: every result ParleyInn returns to an agent's tool call (an MCP
   tool, or a CLI or TUI call from inside a run) is recall: enveloped and
   recorded as a recall event; replace the "any other output ... outside the
   closed paths" sentence with this, so I2's and I4's channels cover it
   without invariant edits. A recall tool is any tool ParleyInn serves to an
   agent; define it so.
K6 Revoked device seat and predecessors: a revoked device seat no longer
   counts as one of this node's own device seats; only predecessors of its
   current device seats count, and the new seat takes the revoked seat's own
   predecessor as its predecessor.
K7 Settings layers: a settings layer's cut-only change only narrows settings;
   quarantine and key revocation happen only as recorded principal acts.
   Managed policy and repository configuration each override the principal's
   settings only with such changes (fix "alone").
K8 Principal act vs room act: a principal act is told apart from a room act by
   bearing no seat-key signature beyond its writer's seal; it is marked with
   the principal surface it was taken at.
K9 Seat ownership: a seat no device key of this node or of an accepted
   predecessor certified belongs to another principal, kept as theirs and
   untrusted (I8) (principal-keys span keeps the PRV-10 chain rule).
K10 Device: if Device is defined only as "a node" in core, say a device is a
   node or (paired-phone span) a paired phone, so it has one meaning.

Feature rulings (from the subset reviews):
F1 delegation-grants: "an agent on another of its nodes" and a delegate report
   "from another node" need a transport: nest them in segment-exchange.
F2 retention: "an erasure request a retention policy sends" in the routing
   sentence: keep only if purge-retention defines it; otherwise drop it.
F3 Foreign room, visibility, presence/typing hints, focus set "every device":
   each needs another principal's or another node's content; nest them in the
   spans that bring that (shared-rooms, segment-exchange or bundle-import),
   so local candidates do not carry dead concepts.
F4 Sync: a peer receives only the sealed ranges of rooms whose principals
   include its principal, and of its own principal's personal room only for
   that principal's own nodes (peer-sync); cite LANE-17/PEER-02.
F5 Retire a writer: give "Either" a referent or drop it, and state its effect
   (later segments are marked delivered after retirement).
Plus every other needs-fix in the g2 findings files, smallest coherent change;
skip wrong ones.

When done: `python3 GA/check_annotation.py --no-orig` prints OK; assemble
GA/genomes/g2-core.json and g2-localpeer.json into GA/out/v3-core and
GA/out/v3-localpeer and read them. Report per ruling.

Added after the subset reviews:
F6 stamps: Pin version's "What a stamp covers." fragment becomes "A stamp
   covers one version; stamping a later version of the same pin replaces that
   principal's earlier stamp on it."
F7 Relations routing sentence: drop "an erasure request a retention policy
   sends" (the policy appends tombstones); name "the launcher's event about a
   room's branch" as an event kind with structural provenance in Event and
   Provenance (launcher span), or drop it from the routing sentence.
F8 Delegation across nodes (F1 above): in local candidates delegation reaches
   only agents on this node; the cross-node reach and the "from another node"
   delegate report sit in segment-exchange spans.
F9 Pin "Information, never an instruction": say "never an instruction to
   ParleyInn", and the restore block presents qualifying pins to the agent as its
   principal's constraints.
F10 Foreign room's "reject a foreign room" cut act: define it under Foreign
   room (it stops showing the room; it removes no pin from a restore block) or
   drop it from the Cut list.
F11 Bundle before PRV-10: the exporter's device key stands for the bundle's
   principal key until PRV-10 ships (principal-keys span keeps the chain).
F12 Token-key-only nodes and configuration acceptance: add a stand-in, as
   PEER-01's environment variable does for the peer component.
