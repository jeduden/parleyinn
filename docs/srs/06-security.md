---
title: "6. Security"
summary: >-
  The normative security requirements SEC-01..32, SEC-28 in 06b. The
  threat model is in 06a, and the boundary register in 06b.
---
# 6. Security

The threat model, §6.1, is in [06a](06a-threat-model.md), and the boundary
register, §6.3, in [06b](06b-boundary-register.md).

## 6.2 Security requirements (SEC)

### SEC-01

- Pri: P0
- Ver: T, I
- Traces: I4

The core (B0) MUST NOT open any socket or make outbound connections.
Every ParleyInn process MUST run exactly one component, fixed when the process
starts, and a core process MUST NOT start a B1–B3 component, in-process or as a
child.
The CLI MUST record the act turning a B1–B3 component on (for the room-view
component and the launcher, the configuration acceptance, until a configuration
turning it off applies, which `parinn configuration accept` records as the cut
act, ADM-04; for the bridge component, a bridge enabled for a host, while any
stands); its own entry point, started by the person, a service manager or an
ephemeral node's entrypoint, MUST run only while that act stands.
Packaging (one executable or several) MUST NOT change any boundary.
The room-view component (B1) MAY listen only on loopback (SEC-20), the launcher
only on loopback or on a local endpoint only the same OS user can reach, and
both MUST connect nowhere else.
Build-time evidence MUST show, per component, that no code the component can
execute, dependencies and code that runs as the process starts included, opens a
socket beyond its boundary or starts a program outside the launcher, save the
core's own kernel worker (CMP-05); CI MUST fail when that evidence is missing or
shows a violation, and suites MUST run each component confined to its boundary
(ENG-12, ENG-16).

### SEC-02

- Pri: P0
- Ver: T
- Traces: I8

ParleyInn MUST create directories `0700` and files `0600`, and MUST refuse to
operate if `PARINN_HOME` or any store file does not belong to the running UID or
has looser permissions.

### SEC-03

- Pri: P0
- Ver: T
- Traces: I8

If a home id is configured (`home.id`), ParleyInn MUST refuse to open the home
unless the home id supplied by the environment matches.

### SEC-04

- Pri: P0
- Ver: T
- Traces: I9

The code that builds a search query MUST treat all caller text as literal terms
(escaping FTS5 syntax), expose boolean and phrase operators only through
structured parameters, limit queries to 16 terms of at most 64 characters,
forbid leading wildcards, and cancel any query exceeding its deadline (default 2
s).

### SEC-05

- Pri: P0
- Ver: T
- Traces: I9

ParleyInn MUST enforce input limits: hook stdin ≤ 1 MiB, JSON nesting depth ≤
64, and transcript lines above the payload threshold MUST be streamed to the
payload store rather than kept in memory.

### SEC-06

- Pri: P0
- Ver: T
- Traces: I2

Recalled content MUST be carried as JSON string values inside the envelope, so
no stored content can alter the envelope's structure, and each envelope MUST
state that its contents are historical data and not instructions.
At a terminal, the CLI and the TUI MUST show every control character but the
line feed, and every zero-width, bidirectional and tag character, in text other
than fixed text ParleyInn ships as a visible placeholder naming its code point,
never written raw, so no such text can alter what the principal reads or
confirms there.

### SEC-07

- Pri: P0
- Ver: I, T
- Traces: I2

The `TrustedText` guarantee MUST be covered by a static check in CI that fails
if any code outside the `restore_block` crate constructs restore content, or
builds a `TrustedText` from anything INJ-03 excludes.

### SEC-08

- Pri: P0
- Ver: T
- Traces: I1

Redaction MUST apply a maintained, versioned rule set for common secret formats,
plus patterns the principal defines in its own configuration, never a
repository's (§9.6), to every text field, payload, and hook input before
anything is written.
Redaction MUST remove the harness account identifiers a harness adds to its
transcripts, such as the account's email address and organization id.
Redacted text MUST be replaced by `[REDACTED:<rule>]`, which names the rule and
carries no hash or other value derived from the secret, in everything stored,
replicated, exported or published.
Correlation of repeated secrets, if kept, MUST live only in a node-local index
keyed under this node's own at-rest key (SEC-09, SEC-10), so that no one off
this node can confirm a guess at a secret from it; that index MUST NOT be
replicated, exported, backed up off the node or published, and its entries MUST
be erased with their event on purge (SEC-31).
Redaction MUST write nothing past the **redaction scan limit**.

### SEC-09

- Pri: P1
- Ver: T, I
- Traces: —

ParleyInn SHOULD support encryption at rest for the store, its payload store
included, with keys obtained through a secret reference (SEC-10).
Deployment documentation MUST require encrypted volumes where this is not
enabled.

### SEC-10

- Pri: P0
- Ver: I
- Traces: I4

The core MUST handle no credential other than its seat keys (REC-18, SEC-27),
the node's device key and, until it leaves the node, its recovery key (PRV-12),
on an ephemeral node the token key its access token carries (PEER-06), on a node
that issued an access token its token key's recovery key, the device, token and
seat certificates of PRV-10 and PRV-11, the per-prompt keys of prompt
commitments (PRV-08), and, when encryption at rest is on, the at-rest key
(SEC-09); who signs and seals what another component records is open (OQ-40).
A witnessed run's run-seat private key, but for the seat ingest starts, MUST
live only in the memory of its harness session's MCP server, which keeps its
main run's and subagents' run-seat keys and, once REC-18 ships, seals their
writers (REC-19), every one minted in that server's memory, the main run's
included, outside the model's context (ASM-21), and none handed over at launch
(OQ-55), and MUST NOT be written to disk, a log, the record, a backup or any
output.
Every other seat key the node uses, whose writers the core seals once REC-18
ships (an ingested run's, which ingest mints, included), and the device key MUST
be generated on the node, kept in a platform key store the core reaches from
inside the core (no socket, no program start) where one exists, named per OS, in
an entry named per home, so two homes of one OS user never share a key, or else
in a file only the principal's OS user can read (SEC-02), never inherited from
the environment unless a configuration setting under ADM-04 opts in, and never
logged, exported, backed up or placed in a segment; a token key MUST be kept the
same way; keys kept in a platform key store, and a run-seat key in its MCP
server's memory, are bound to the home (I8); `parinn status` MUST say when a key
is a file an unsandboxed agent of the same OS user could read.
A seat key missing from where it is kept MUST be audited and counted and minted
anew under SEC-27.
Every device key or token key the node starts using MUST be recorded as a
structural key event in its device seat in its personal room, carrying only what
PRV-12 names for its kind (`minted`, `token key adopted`, `minted anew`,
`rotated`, `recovered` or `revoked`); these key events form PRV-12's key event
logs.
The at-rest key and every credential of another component (access tokens, apart
from the token key the core handles, forge and notification credentials) MUST be
resolved per use from an explicit secret reference (a file only the principal's
OS user can read, or the platform key store) and MUST NOT be inherited from the
environment unless such a setting opts in, nor logged, exported, backed up or
placed in a segment; the at-rest key MUST NOT be persisted elsewhere, not in the
store or a derived artifact; a credential of another component MUST be loaded
only by the component that uses it, never by a core process.

### SEC-11

- Pri: P0
- Ver: T
- Traces: I2, I7

A repository configuration MUST only be able to tighten security-relevant
settings of the principal's configuration: it MUST NOT enable any injection,
change the trust policy (PRV-05) or make a change ADM-04 holds for acceptance.

### SEC-12

- Pri: P0
- Ver: T
- Traces: I5

Quarantine MUST be available by room, run, span, author seat, writer, address
range, time window, provenance class, flag and derived artifact, and a seat or
writer selector of a run seat the seat `parinn ingest` starts beside it, never a
seat that names it under SEC-27; a quarantine that removes a pin, a pin version
or a landmark from a restore block MUST be a widening principal act, and one
that removes nothing from a restore block a cut act; MUST be recorded as an
`operator` event with the stated reason; MUST take effect immediately on the
node that records it, keeping what it covers, and what first matches its
selector later, off every path carrying content to an agent or exporting it as
trusted, as a purge (ADM-07) keeps what it removes: recall, kernel reads,
landmark text, restore blocks and trusted-only exports leave it out (RCL-06,
CMP-07, LMK-04, PIN-10, ADM-12), but a pin version or landmark recorded after a
cut quarantine MUST leave a restore block only through a widening one, and any
other write to an agent carrying it, such as an endorsement (OWN-08), a trust
grant's post (OWN-29) or a delegated task (OWN-24), or principal act making a
pin of it, such as a pin candidate's confirmation (PIN-05, LANE-20), MUST be
refused, audited and counted; and release, a widening principal act, MUST be
equally recorded, one carried over (ADM-06) taking no effect.
A quarantine and its release MUST NOT change room state or the key set, but for
which pin version PIN-10 restores.
A quarantine reaches peers only as a quarantine request that each peer's
principal applies (PEER-11).

### SEC-13

- Pri: P1
- Ver: T
- Traces: I2

ParleyInn SHOULD maintain each run's recall taint, queryable by
`parinn recall-taint show --run R`, and SHOULD ship an example `PreToolUse`
hook, the principal's own and outside the hooks ParleyInn installs (§9.1), that
requires approval for the action classes it configures as sensitive once the run
carries recall taint.
Where maintained, recall taint MUST be set on a run once untrusted content has
been returned to it (RCL-04), and MUST pass from run to run, whatever agent each
belongs to (REC-02): forward through a harness resume or a harness clear, a
delegated task, a subagent's included (OWN-24), or the input a run's tool call
gives a harness session (PRV-08), into each run so started from its first event,
the run reached carrying recall taint from its first event wherever its node
cannot tell the run it comes from or derive that run's recall taint from the
writers it holds; and back through a delegate report (OWN-25) or that tool
call's result.

### SEC-14

- Pri: P0
- Ver: T, I
- Traces: I5

Content removed by purge MUST NOT stay recoverable from storage the purge freed,
on the node that ran it: neither from the store nor from the payload files it
removed.
Documentation MUST state that physical erasure on SSDs is not guaranteed without
encryption at rest.

### SEC-15

- Pri: P0
- Ver: I, T
- Traces: I4

ParleyInn MUST NOT include telemetry, crash reporting to remote services, or
update checks.

### SEC-16

- Pri: P0
- Ver: T
- Traces: I6, I9

Hook inputs MUST be validated against a schema; invalid inputs MUST be rejected
fail-open (no injection) and audited.

### SEC-17

- Pri: P0
- Ver: I
- Traces: —

A threat-model document MUST be maintained in the repository and reviewed at
every minor release.
It MUST cover every row of the boundary register and the threat sources acting
across each, and MUST compare ParleyInn control by control with Zed Delta on
record signing, trust in other principals, central-service dependence and key
custody.

### SEC-18

- Pri: P0
- Ver: T
- Traces: I8

Every path received from hook inputs, transcripts, or configuration MUST be
resolved (including symlinks) and MUST lie within an allowed root: the
configured transcript roots (`transcript.roots`), `PARINN_HOME`, or, for reading
only, the run's worktree, found from the `cwd` of the `SessionStart` hook input
its MCP server holds the session binding for (REC-13), never a transcript
line's, up to the repository's top level after resolving symlinks, and its
repository's git directory, the one that worktree names.
The one exception is a path the principal names on `parinn ingest --path` at a
terminal, an explicit widening principal act (OWN-11) allowed once resolved,
symlinks included (REC-22); a path from hook input or a transcript line never
has it.
A path outside every root and not so named, or one that leaves its root once
symlinks are resolved, MUST be rejected and audited.
ParleyInn MUST NOT write inside the run's repository except where ADM-13 allows.

### SEC-19

- Pri: P0
- Ver: T, I
- Traces: I4

Every ParleyInn component, process and protocol MUST be assigned to exactly one
boundary, B0 to B3, in a register kept in the repository (§6.3).
Every B1, B2 and B3 component, the launcher included, MUST be off until the
node's principal turns it on (SEC-01).
A ParleyInn feature whose purpose is to reach off the machine through a tool the
principal runs outside ParleyInn, such as its own tunnel, MUST have its own §6.3
row, be off by default and be turned on by a widening principal act.
Room trailers and trailer links (§6.3 row 28, always on, LANE-28) are not such a
feature, nor is a head or purge receipt, which carries only digests (VIEW-10).
CI MUST fail when a component's build-time reach evidence is missing, when that
evidence or a test of the component confined to its boundary shows more reach
than the register grants, or when a process exists that the register does not
list.

### SEC-20

- Pri: P1
- Ver: T
- Traces: I4, I6, I8

The room-view component MUST listen only on a loopback network address, on a
port chosen at launch, and MUST require a room-view secret: a launch secret with
at least 128 bits, never sent to the server in an HTTP request line, and never
in argv, an environment another UID can read, a log or a referrer.
It MUST exchange the launch secret once for an origin secret that only its own
origin, port included, can read or send; a page served from another loopback
port MUST NOT be able to obtain or replay it.
Only that origin secret makes the browser room view a principal surface, within
the narrower scope of a phone-scoped room-view secret (open: OQ-39), which
serves a browser on the principal's phone through the principal's tunnel
(OWN-16): it MUST permit enveloped reading and cut or neutral principal acts,
and widening ones only under OWN-11; it lasts until that room-view component
stops.
It MUST reject an HTTP request whose `Host` or `Origin` is not its own, allow no
cross-origin HTTP request, and audit every rejection.

### SEC-21

- Pri: P1
- Ver: T
- Traces: I2, I4

The room view MUST render record content as inert text, never interpreting HTML,
script, or Markdown links and images from it.
A Content-Security-Policy MUST forbid any resource from outside its own origin.
Untrusted content MUST be visibly marked.

### SEC-22

- Pri: P0
- Ver: T
- Traces: I4, I7, I6

Managed policy MUST be able to disable each of B1, B2, B3 and the launcher per
host, set quotas once ADM-15 ships, fix the deployment mode, cap each action
class's rule level, disable away policies, hook permission decisions and keeping
held requests waiting, require an authenticator for widening acts, name a
confirmation key and a recipient key with the directory recovery keys are
written to, each recorded as a structural listing that enters the key set
(PRV-12), forbid risk acceptance, and set a retention policy per room and
provenance class, past whose window, counted in the node's time marks (REC-15),
the node purges under ADM-07, leaving structural tombstones, each purge recorded
naming the policy, audited and counted.
Managed policy turns no component on; ParleyInn MUST refuse to start a component
managed policy disables and MUST audit the refusal.
Until presence proofs ship (OWN-11, OWN-12), ParleyInn MUST refuse every
widening act while managed policy requires an authenticator, and audit and count
each refusal.

### SEC-23

- Pri: P1
- Ver: T
- Traces: I7

The room view MUST NOT write harness configuration, the principal's
configuration, the deployment mode or any boundary's state.
It MAY show the diff and the CLI command that would; each change MUST go through
the CLI with a shown diff (ADM-02).

### SEC-24

- Pri: P2
- Ver: T
- Traces: I4, I8

B2 traffic MUST be encrypted and mutually authenticated with each peer's or
paired phone's device key, or a token-key-only node's token key.
The peer listener MUST bind only to the network addresses its configuration
names, none by default, and MUST refuse a wildcard network address.
B2 MUST carry only sealed ranges (PEER-03), in both directions whichever side
dialled, and the ephemeral, signed presence hints of PEER-09.
Local discovery MUST advertise only a random per-boot instance id and a port,
never a principal, host or room name.

### SEC-25

- Pri: P2
- Ver: T
- Traces: I2, I6, I8

The peer component (B2) MUST accept segments only from peers enrolled by key and
sealed by seat keys that chain to a principal key in its key set, MUST verify
each segment's seal (REC-18) and chain continuity before accepting it, MUST
refuse and audit a segment failing either, recording it as REC-21 says, and MUST
store received events under the local home, each untrusted unless PRV-02
classifies it trusted; being a peer MUST NOT make content trusted.

### SEC-26

- Pri: P2
- Ver: T, I
- Traces: I2, I4

Every export, the trusted-only export included (`parinn export`, `--rendering`,
`--trusted-only`, B0), and every publish (B3) MUST be a principal act under
OWN-12, MUST be audited, and MUST pass a review step that shows included and
withheld content by class, MUST apply SEC-08 plus a publish-only rule set that
redacts absolute paths, user names, host names and email addresses, MUST fail
closed on an unresolved secret-scan hit, and MUST sign a manifest of included
and withheld ranges, and a bundle with its exporter's device key, which chains
to the bundle's principal key.
A bundle MUST keep the chained header of every withheld or redacted event, so
the receiver verifies the chain across it, and MAY carry a binding statement,
signed by the pull-request author's commit-signing identity, naming the bundle's
principal key (LANE-15).
A bundle MUST be shareable as a plain file or a git ref with no host, account or
sync.
The publish component's listener MUST serve bundles read-only with no write
path, and MUST bind only to the network addresses its configuration names, none
by default.

### SEC-27

- Pri: P1
- Ver: T
- Traces: I6, I10

A seat-key rotation MUST be a signed event in the seat's writer, signed by the
old key and the new key; the seat MUST keep its id and writer, and what the old
key sealed MUST stay verifiable.
Any other key minted in place of a seat's key, on a node identity change, a node
clone or a backup restore (REC-24, ADM-06), or for a key missing from where
SEC-10 keeps it, as a run-seat key is once lost with its MCP server on a harness
resume or a restart (open: OQ-43), MUST start a new seat and writer, audited,
which names the old one, inherits no add, role or appointment, and is a member
only as LANE-23 and LANE-25 say.
Device keys MUST support rotation, each recorded as a key event (PRV-12), and
seat and device keys revocation, as signed events; revoking a seat key MUST be a
cut principal act unless it removes a pin from a restore block or stops
principal acts arriving, either of which makes it widening (OWN-11).
Events a revoked key sealed before its revocation MUST stay verifiable; other
events by it MUST be refused under PRV-10's rule.
A head receipt of every writer's chain head MUST be checkable on another node
with no network.

### SEC-29

- Pri: P1
- Ver: T, I
- Traces: I4, I2

The launcher MUST be a separate component, off by default, the only one allowed
to start another program (the core starting its own kernel worker, CMP-05, is
the one exception), with no network beyond loopback to the room-view component,
its own register row and its own threat review.
That loopback connection MUST carry no text for the model, and the launcher MUST
carry into the harness input only text the core built and recorded, read from
the record.
Any listener it opens MUST meet SEC-20 or be a local endpoint only the same OS
user can reach, refusing a connecting process of another UID.
The processes it starts MUST each have their own register row (§6.3).
It MUST run only commands the person typed or confirmed under OWN-18.

### SEC-30

- Pri: P2
- Ver: T
- Traces: I5, I6

A purge MUST travel as an erasure request (PEER-11) naming its tombstone, so a
peer that applied it shows a tombstone and a peer that suppressed events shows a
missing range.
A principal with a seat in a room MUST be able to send the room's owner a signed
purge request, a neutral principal act, for the events its own seats wrote,
applied by the owner's widening principal act or refused by a neutral one,
either audited.

### SEC-31

- Pri: P0
- Ver: T
- Traces: I1, I5, I6

Purging an event, by the node's principal, under a retention policy or by an
applied erasure request (PEER-11), MUST erase or tombstone every stored copy of
its content this node holds in any writer, found by address or commitment: the
text an endorsement, steer or hand-back sent, recall results naming its address
in an agent's recorded tool results (RCL-07, RCL-11), worktree checkpoints and
payloads that contain it, and search index entries.
Purge MUST also erase, with their keys, every prompt commitment whose
`prompt_id` a purged event's line carries, matched or not, and every one
recorded inside the purged scope or time window, and the purge receipt MUST
count them.
However stored content is removed, by purge, under a retention policy, by an
applied erasure request or by a backup restore that reapplies a purge, no file
ParleyInn or its storage engine writes, temporary storage included (§8.1), MAY
afterwards contain, in any encoding, the removed events' content, their
commitment keys or a search index entry naming their addresses: free pages,
unused cell space, the write-ahead log and rollback and statement journals
included.
Erasure from a backup MUST meet this for that backup's files, or list the
backup.
No audit entry or log line MAY carry event content, only addresses, commitments,
counts and typed reasons.
A purge killed or timed out before this state MUST NOT report the range erased;
it MUST audit, count and name each residue in the purge receipt and stay
pending, its residue remaining, by design, until the next purge, ingest or
rebuild completes it outside hook budgets (ADM-07), then write a further
receipt, and while it is pending `parinn doctor` MUST exit non-zero, whatever
counters were acknowledged, and the Needs you queue and Catch up MUST show it
(VIEW-05, VIEW-08).
ParleyInn MUST record each purged line's position and length in its transcript
source and refuse, with an audit entry, to ingest them again from a harness
transcript (OQ-57 names what the refusal and the zeroing miss).
It MUST recompute the source's consumed-prefix hash with the bytes of each
purged transcript line zeroed, or, where its transcript cannot be read or no
longer holds what the stored hash covers, store one that confirms nothing, so
that ingest starts a new transcript generation of the source (ADM-07, ADM-09),
and verify and resume MUST zero the same bytes, so that source's hash confirms
no guess at the purged bytes and the source still resumes: only each purged
line's position, length and the harness's own `uuid` remain.
Purge changes only ParleyInn's own record of a source, its ingest position's
hash and recorded positions, never the harness's transcript file.
Purge MUST erase the range from every backup ParleyInn made, or list each such
backup.
Each node MUST write a signed purge receipt the node's principal can collect:
the scope, the ranges, the copies erased, and every copy ParleyInn knows of and
cannot erase (harness transcript files, exported bundles, backups, and peers
that refused or are unreachable).

### SEC-32

- Pri: P1
- Ver: T
- Traces: I2, I6

Every appointed moderator (LANE-16) MUST stay within these limits; a room's one
facilitator MUST be the service account whose device seat, on its own node, only
the owner appoints; the program that writes for it is neither a ParleyInn
component nor an agent and acts only through that node's CLI, never MCP tools,
signed with that device seat.
Each kick, bar or mute an appointed moderator sets MUST be within the
appointment rate, a room setting the owner's principal act sets (LANE-31), an
act beyond it refused and counted; MUST be audited naming by address the post
behind it, which names the act's target by id and carries a range link to the
marked range; MUST be shown in the room view and named by id in the error the
hit seat's next refused call returns (LANE-25); MUST raise a Needs you item for
the owner, its appointer and the principal whose seat it concerns; and each bar
and mute MUST be undoable by the owner or its appointer (unbar, unmute); a
kicked seat only its own principal adds again.
It MUST NOT act on the owner or another moderator, MUST NOT mute the whole room,
MUST NOT change another seat's pin or make a list removal, MUST NOT unbar or
unmute, MUST NOT act outside its room, and MUST NOT appoint.
Its posts MUST be in its own words only, shown as its own statements, that point
by range link to the marked range each names and never quote or embed it, so it
relays no third party's text; they MUST NOT reach as trusted any agent whose own
principal has recorded no trust grant for its principal (OWN-29).
Every pin version the facilitator writes MUST likewise be in its own words only,
quoting or embedding no marked range, so no trust grant restores a third party's
text through it (OWN-29).
