---
title: "5. Functional requirements: pins, recall and restore"
summary: >-
  Normative functional requirements for pins, recall, landmarks,
  restore and the kernel (PIN, RCL, LMK, INJ, CMP). Part of §5.
---
# 5. Functional requirements: pins, recall and restore

## 5.3 Pins (PIN)

### PIN-01

- Pri: P0
- Ver: T
- Traces: I2, I3

Qualifying pins MUST come only from a principal (`parinn pin add` at a principal
surface, its accepted configuration or its stamp), never a harness skill's call.
Adding, editing or unpinning a pin MUST be the act OWN-11 and OWN-12 name for
its type and author; each but a neutral unpin MUST need its seat's pin
capability.

### PIN-02

- Pri: P0
- Ver: T
- Traces: I2

An agent MUST NOT be able to make any pin restore.
Text `pin_candidate_propose` proposes MUST be stored as a pin candidate with
provenance `assistant`, and every list of pin candidates MUST show it with that
provenance, marked untrusted.

### PIN-03

- Pri: P0
- Ver: T
- Traces: I3, I5

Each pin MUST store: verbatim text (≤ 1,000 characters†, holding none of
INJ-08's delimiter sequences), pin type, pin priority, room, the address of its
creating event, each version's author, and the creating event's commitment,
never a bare hash of its text.

### PIN-04

- Pri: P0
- Ver: T
- Traces: I10

Pin versions MUST be immutable; an edit MUST add a new, unstamped pin version,
keeping every earlier one and the pin's type and priority; changing a pin's
type, priority or room MUST be an unpin then a new pin.

### PIN-05

- Pri: P1
- Ver: T
- Traces: I2, I3

ParleyInn SHOULD detect constraints stated in trusted `user` events as pin
candidates, each a derived artifact over its creating `user` event that holds
its text by address, with no provenance of its own (I10).
A pin candidate MUST become a pin only by its principal's confirmation, taken
only after that principal is shown its exact text, pin type and priority, the
pin's room (its creating event's room, or its principal's personal room where
that principal has no member seat left there) and, for one an agent suggested,
its provenance `assistant`, marked untrusted, as a new pin that principal's
device seat authors, or, for an intent or criterion, only by the owner's, as a
new version of that room's intent its device seat authors, never by
configuration in `automation` deployment mode.
A principal MAY dismiss a pending pin candidate by a neutral principal act,
which leaves its creating event in the record.

### PIN-06

- Pri: P0
- Ver: T
- Traces: I3

Pin types MUST be `constraint`, `preference`, `decision`, `fact`, `episode`,
`intent`, `verdict` and `stake`.
Only `constraint`, `preference` and `intent` pins MUST be restored automatically
in v1.

### PIN-08

- Pri: P0
- Ver: T
- Traces: I3, I6

If qualifying pins exceed the pin budget (`pin_budget.max_model_tokens`, default
1,000 model tokens†), ParleyInn MUST inject first the pins of the rooms the run
has had a seat in during the run, its personal room included, then those of
rooms only its agent's earlier runs had a seat in (PIN-10, REC-02), each with
intents first (LANE-20), then by pin priority, then by their creating events'
addresses (writer, then `seq`), MUST NOT truncate text within a pin, MUST name
the omitted pins in the restore block by id and count, audited.

### PIN-09

- Pri: P1
- Ver: T
- Traces: —

`parinn doctor` SHOULD warn when a pin duplicates text in the worktree's
`CLAUDE.md` files.

### PIN-10

- Pri: P0
- Ver: T
- Traces: I3, I2

Only a version of a pin of a type that restores MAY qualify for a run: one
written from a device seat a device key of the run's principal, or, outside a
foreign room, of a principal its trust grant covers, certified (before PRV-10
ships, this node's own device seat), its own event trusted on this node under
its recorded deployment mode (for the intent, only its newest version, and as
LANE-11 says after a change of ownership), or, once PRV-10 ships, one the run's
principal stamped, its stamp so trusted.
Of a pin's versions that qualify for a run, only the newest MAY restore, but one
qualifying only through a trust grant MUST NOT displace one the run's principal
stamped.
A version MUST count as absent wherever this requirement chooses which version
of a pin restores, the intent's included, while it is quarantined (SEC-12), once
it is purged, however purged (SEC-31), its tombstone never restoring, or if the
act adding it takes no effect on this node (PRV-02, REC-24, ADM-06).
Any other pin of such a type in a room whose pins restore to the run MUST be
stated only by count, room id and key fingerprint, but for what PRV-12 names,
and audited.
Every pin MUST belong to exactly one room, by default its principal's personal
room on the node that wrote it.
A restore block MUST contain the qualifying pins of every room its run has had a
seat in during the run, its personal room included, and of every room its
agent's earlier runs had a seat in (REC-02), name those rooms by id and name
each pin it omits as PIN-08 does, with its room id.
A branch switch, a leave, a kick or a bar MUST NOT drop a pin from any restore
block: where every seat of the run's principal in a room has left it or lost it
to a kick or a bar, what qualified there for that principal's agents, a trust
grant's pins included, MUST keep restoring as it stood, in causal order, before
the act that ended its last member seat there, until that principal's own act
changes it, or until a certificate it qualified through is revoked (PRV-10) or
the trust grant it qualified by covers nothing (OWN-29), as in any other room;
after a change of ownership, the intent MUST restore in such a room only as
LANE-11 says.

### PIN-11

- Pri: P2
- Ver: T
- Traces: I3, I6

When a pin that qualifies on another of the principal's nodes does not qualify
on this node, because its seat certificate or token key does not cover the pin's
room, a certificate it chains through is revoked or its event has not arrived,
the restore block MUST state how many such pins exist and the reason, without
their text, audited.

## 5.4 Recall (RCL)

### RCL-01

- Pri: P0
- Ver: T
- Traces: I1

ParleyInn's MCP server MUST expose the recall tools `event_search`,
`event_expand`, `event_get`, `landmark_list`, `pin_list`, and `stat_list`, and
`mcp_server_get`, which is no recall tool: it returns whether the MCP server
holds its session binding, whether its search index is readable, the count of
its timed-out searches and its version, as closed words and counts and no
recalled content, for a call whether or not a run is found for it, and records
no recall event.

### RCL-02

- Pri: P0
- Ver: T
- Traces: —

`event_search` MUST accept filters for run, time range, exact provenance class
(`provenance`), provenance class family (the versioned `kind` filter, where
`tool_result` matches every `tool_result:<tool>`), and trust level, and return
at most `k` hits (default 10, maximum 50) ranked by BM25 with length
normalization and per-run damping.

### RCL-03

- Pri: P0
- Ver: T
- Traces: I1

`event_expand` and `event_get` MUST return exact post-redaction content,
resolving payload references, subject to a per-call cap (default 8,000 model
tokens) with a continuation cursor.

### RCL-04

- Pri: P0
- Ver: T
- Traces: I2

All recalled content MUST be returned inside the envelope (§9.3).
Outside it and I2's closed paths, whatever ParleyInn returns to a tool call, its
errors and a CLI verb's output that is not a terminal included, MUST carry only
fixed text ParleyInn ships, ids, counts and structural fields sanitized under
LMK-03.

### RCL-05

- Pri: P0
- Ver: T
- Traces: I8

Recall scope MUST default to `run`: the calling agent's current run, through
every writer of its seats, which record its delegate reports.
Extending it to `room` (one of its principal's rooms the agent has a seat in,
through the writers this node holds) or `rooms` (every such room), each with the
cross-room posts it shows, MUST take an explicit `scope` parameter and be
logged.
A foreign room MUST be reached only under RCL-10, never through the `room` or
`rooms` scope.
Whether a call may name a room of its principal's that every run seat of the
agent has left or lost to a kick or a bar, which neither scope reaches, is open
(OQ-45).

### RCL-06

- Pri: P0
- Ver: T
- Traces: I5

A quarantined address MUST return a quarantine marker, and a purged range a
tombstone (SEC-12).

### RCL-07

- Pri: P0
- Ver: T
- Traces: I6

Every recall MUST be recorded as an event (provenance `assistant` for the query,
with the addresses of the returned events).

### RCL-08

- Pri: P1
- Ver: T
- Traces: I1

Every address form ParleyInn shows, and the ASCII input form, MUST be accepted
wherever an address is taken, on the CLI and by `event_get` or `event_expand`,
resolving to the same events, a tombstone or a quarantine marker.
Writer labels MUST be unique per node, each writer-label assignment a structural
event in the assigning device seat's personal-room writer.

### RCL-09

- Pri: P1
- Ver: T
- Traces: I2, I6

Every recalled item MUST carry its writer, its author, its trust level, its
origin (`witnessed`, `ingested`, `bundle` or `peer`) and its integrity status;
an item from a foreign room carries `untrusted` in place of its trust level
(RCL-10).

### RCL-10

- Pri: P2
- Ver: T
- Traces: I2, I8

Recall MUST reach a foreign room only through a `room` parameter naming it in
that call, and log the call.
Recall results MUST be enveloped, every item marked untrusted whatever its trust
level or trust grant covers the foreign room's keys, and taint the calling run
under SEC-13; PRV-02 still keeps its principal's device-seat pins and stamped
versions there, and the pins PIN-10 keeps after a leave, kick or bar, trusted
for restore (PIN-10).

### RCL-11

- Pri: P2
- Ver: T
- Traces: I6

Every recall that returns a post another seat wrote MUST be recorded in the
writer of the recalling run's seat and shown, with that recall event's address,
to the room's owner and the principal of the post's author.

## 5.5 Landmarks (LMK)

### LMK-01

- Pri: P0
- Ver: T
- Traces: I10

ParleyInn MUST split each run into spans, starting one at every user turn, every
turn whose input a run's tool call supplied (PRV-08), compaction, subagent start
or end, each new transcript generation (REC-07), and whenever the run's events
move to another seat's writer; a run's last span MUST close at its end, which
the MCP server records as a structural event when a subagent's report
(`subagent_result`) or the task notification naming it reaches its parent's
transcript or the harness session ends.
Span boundaries MUST be a deterministic function of the record.

### LMK-02

- Pri: P0
- Ver: T
- Traces: I2

ParleyInn MUST produce one landmark per closed span containing only structural
fields: address range, turn range, event counts by kind, tool names with call
counts, touched file paths, and an error indicator.

### LMK-03

- Pri: P0
- Ver: T
- Traces: I2

Every structural field untrusted input can influence (file paths, tool names,
MCP server names) MUST be sanitized to an allow-listed character set
(`A–Z a–z 0–9 . _ / @ : + -`) and a maximum length (default 80†); non-conforming
values MUST be replaced with `<field:HASH8>`.

### LMK-04

- Pri: P0
- Ver: T
- Traces: I2, I5

Flagged and quarantined events MUST contribute only to counts, never to text
fields.

### LMK-05

- Pri: P0
- Ver: T
- Traces: I10

Landmarks MUST roll up into tiers of at most `k` landmark blocks (`k` ≥ 3,
default 8†): a full tier keeps its newest in full detail and collapses the older
ones, one line each, into one landmark block of the next tier, bounding the
landmark index of a run of n spans to O(k · log_k n) landmark blocks.

### LMK-06

- Pri: P2
- Ver: T
- Traces: I2

ParleyInn MAY generate natural-language headlines for spans whose every event is
trusted, only by a deterministic template over trusted fields.

## 5.6 Restore and injection (INJ)

### INJ-01

- Pri: P0
- Ver: T
- Traces: I3

On `SessionStart` with `source = compact`, ParleyInn MUST return a restore block
containing: qualifying pins, PIN-10's statement of other pins, the current run's
landmark index, the recall hint and any room summary pointer, separated by fixed
text ParleyInn ships.

### INJ-02

- Pri: P0
- Ver: T
- Traces: I3

On `SessionStart` with `source` = `startup`, `resume`, `clear` or `fork`,
ParleyInn MUST by default return qualifying pins and, if a room the run has had
a seat in during the run has history, the recall hint.
Including the landmark index MUST be configurable
(`restore_block.landmarks_on_start`).

### INJ-03

- Pri: P0
- Ver: I, T
- Traces: I2

The restore builder MUST accept only values of type `TrustedText`. `TrustedText`
MUST be constructible only inside the `restore_block` crate, only from
qualifying pins, sanitized structural fields and fixed text ParleyInn ships,
enforced by private fields and a `pub(crate)` constructor, with no public method
or trait implementation that builds one, except by copying an existing
`TrustedText`, or changes one.

### INJ-04

- Pri: P0
- Ver: T
- Traces: I2

ParleyInn MUST inject a restore block on `UserPromptSubmit` only while
`restore_block.on_prompt` is on, off by default, holding what INJ-02 returns,
built only from `TrustedText` and audited.

### INJ-05

- Pri: P0
- Ver: T
- Traces: I2, I3, I8, I9

If ParleyInn cannot determine unambiguously which run a hook belongs to, it MUST
inject only the qualifying pins that qualify for every run it may belong to
(PIN-10), within PIN-08's budget, and name each other pin that qualifies for one
of them by id and room id, with their count.
A `SessionStart` with `source` = `compact` MUST be treated as ambiguous while,
by the record, any subagent of its harness session may be running, from the tool
call of a run of that harness session that started it until its tool result, or
for a background subagent the task notification naming it, is recorded, and
while ingest of that harness session's transcripts has not reached their end,
the candidate runs then including every run of that harness session in the
record, among them each subagent run whose transcript the MCP server found under
its session binding and recorded as a structural event: no hook input or
transcript names the compacting run while its hooks run (ASM-06).
While any run on the node leaves R3 or R8 open (PRV-08), no transcript line ends
that ambiguity: a compaction stays ambiguous from a subagent's start to the end
of its harness session.
When every candidate run qualifies for the same pins, the restore block MUST
hold them all and omit only the landmark index.

### INJ-06

- Pri: P0
- Ver: T
- Traces: I10

For an identical record, the restore block MUST be byte-identical: stable
ordering, no timestamps, no absolute paths, no random identifiers.

### INJ-07

- Pri: P0
- Ver: T
- Traces: I9

The restore block MUST respect a total limit (`restore_block.max_model_tokens`,
default 2,000 model tokens†), which bounds all but what PIN-08 and PIN-10 make
it carry.
When over it, ParleyInn MUST remove landmark detail deterministically, coarsest
tier last, and audit and count a block those alone keep over it.

### INJ-08

- Pri: P0
- Ver: T
- Traces: I2

The restore block MUST use fixed delimiters, and no injected field may contain
the delimiter sequences, by construction.

### INJ-09

- Pri: P0
- Ver: T
- Traces: I2

Content with provenance `harness_text` (including compaction summaries) MUST NOT
appear in any restore block.

### INJ-10

- Pri: P1
- Ver: T
- Traces: I2, I6

Opt-in notices MUST be off by default and reach an agent only under its room
owner's notice allowance and its principal's notice opt-in, for that room or
every room.
An opt-in notice MUST be `TrustedText` containing only fixed text ParleyInn
ships and what LANE-30 allows, audited, never starting or resuming a turn, and
returned only on `UserPromptSubmit` or `SessionStart`.

## 5.7 Compute kernel (CMP)

### CMP-01

- Pri: P1
- Ver: T
- Traces: —

ParleyInn SHOULD expose MCP tools `kernel_exec`, `kernel_variable_list`, and
`kernel_reset`, backed by hermetic SQL (ADR-04).

### CMP-02

- Pri: P1
- Ver: T
- Traces: —

Each run MUST have a persistent kernel namespace of temporary tables and views
that survives across `kernel_exec` calls until `kernel_reset`, worker restart,
or the run's end.

### CMP-03

- Pri: P1
- Ver: T
- Traces: —

The kernel MUST provide read-only table functions `event_search`,
`event_expand`, `event_get`, and `landmark_list`, mirroring the same-named MCP
tools, a `summary` event only as its address, plus JSON, regular-expression,
math, and date functions.
Each call MUST record a recall event (RCL-07).
Everything the kernel allows, SQLite's built-ins included, MUST be on an
explicit allow-list.

### CMP-04

- Pri: P1
- Ver: I, T
- Traces: I4

The kernel MUST have no access to the filesystem, network, processes,
clock-driven nondeterminism beyond allow-listed date functions, or environment
variables.

### CMP-05

- Pri: P1
- Ver: T
- Traces: I9

Each execution MUST run in a `parinn kernel-worker` child process with a step
budget, a wall-clock limit (default 10 s†), and a memory limit (default 512
MiB†).
Exceeding a limit MUST return an error, restart the worker, and state that the
namespace was lost.

### CMP-06

- Pri: P1
- Ver: T
- Traces: I2

Only the last row-returning statement's rows MUST return to the agent, capped
(default 8,000 model tokens†) with a truncation marker stating that kernel
variables persist.
The output MUST be enveloped and tainted by every event its namespace has read.

### CMP-07

- Pri: P1
- Ver: T
- Traces: I5, I8

Kernel reads MUST honour quarantine and recall scope exactly as MCP recall does;
a quarantine or purge MUST reset each namespace that read what it covers
(CMP-05).

### CMP-08

- Pri: P2
- Ver: T
- Traces: I4

ParleyInn MAY support an external Python kernel as an opt-in, confined,
network-less kernel worker (CMP-05) with read-only store access, inside the
core's boundary (§6.3 row 3).

### CMP-09

- Pri: P2
- Ver: I
- Traces: I4

Parallel model sub-calls over record data MAY be supported only through harness
mechanisms.
ParleyInn MUST NOT call model APIs itself.
