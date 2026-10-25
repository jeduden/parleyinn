---
title: "4. Reference architecture (non-normative)"
summary: >-
  Non-normative reference architecture: components, event-sourced
  state, the compaction and aggregation scenarios, and design
  decisions ADR-01..10.
---
# 4. Reference architecture (non-normative)

This section describes the intended design so that requirements can be read in
context. It is not binding; §5–§10 are.

## 4.1 Overview

```text
 ┌──────────────────────────── Claude Code / Agent SDK harness ────────────────────────────┐
 │                                                                                           │
 │   hooks (command, stdin JSON)                          MCP client (stdio)                 │
 │        │                                                     │                            │
 └────────┼─────────────────────────────────────────────────────┼────────────────────────────┘
          ▼                                                     ▼
   parinn hook <hook>                                      parinn mcp
   (short-lived process)                                  (process per harness session)
          │                                                     │        spawns
          │                                                     ├──────────────► parinn kernel-worker
          ▼                                                     ▼                (rlimits, hermetic,
 ┌────────────────────────────────────────────────────────────────────────┐     read-only store)
 │                        parinn core (Rust library)                       │
 │                                                                        │
 │  adapter ─► parse ─► provenance ─► redact ─► flag ─► append ─► index   │
 │  (harness)                                          (seq, hash chain)  │
 │                                                                        │
 │  search · recall · envelope · landmarks · pins · quarantine            │
 │  restore builder (TrustedText only) · audit · counters                 │
 └───────────────┬───────────────────────────────────┬────────────────────┘
                 ▼                                   ▼
 ┌─ store ────────────────────────────────────────────────────────────────┐
 │ ┌──────────────────────────────┐    ┌──────────────────────────────┐   │
 │ │ SQLite, WAL                  │    │ payload store (REC-09 names) │   │
 │ │ events · events_fts · spans  │    │ large tool outputs, files    │   │
 │ │ landmarks · pins · quarantine│    └──────────────────────────────┘   │
 │ │ counters · audit · meta      │                                       │
 │ └──────────────────────────────┘                                       │
 └────────────────────────────────────────────────────────────────────────┘
          PARINN_HOME/                         (0700 dirs, 0600 files, one principal's)
```

## 4.2 Components

The components are the closed set of the
[domain model](../domain-model/components-and-surfaces.md)
and §6.3, each inside one network boundary (I4).

| Component                | Responsibility                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Core (B0)                | The hook handlers and the CLI (but for `parinn ui`, `parinn launch` and each B1–B3 component's own entry point), each harness adapter's transcript and hook part among them, the MCP server, the kernel worker, the TUI, the commit hook (LANE-28, run as the CLI), and everything that builds what reaches the model; opens no socket and starts no program but its kernel worker.                                                                                                                                                                                                                                                                                                                                                    |
| Room-view component (B1) | `parinn ui`: serves the browser room view on loopback only (SEC-20) and records the principal acts taken in it, each marked with its principal surface (VIEW-03); off until turned on: the principal's accepted configuration turns it on (ADM-04).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| Launcher (B1)            | `parinn launch`: through each harness adapter's run part, starts, hosts and controls runs, pauses them at the harness prompt and records sandbox state (OWN-22); carries into the harness input only text the core built and recorded, read from the record, recording each carried text's commitment as its turn trigger (LANE-14), and its loopback connection to the room-view component carries no text for the model; any listener it opens is on loopback (SEC-20) or a local endpoint only the same OS user can reach; executes witness checks; the only component but the core, for its kernel worker, that starts another program (SEC-29); off until turned on: the principal's accepted configuration turns it on (ADM-04). |
| Peer component (B2)      | Replicates segments with peers the node's principal enrolled by key and serves paired phones, on each of which it is ParleyInn's only code, with the room view's reduced client inside it; off until turned on: the CLI records the widening act (`parinn peer-component on`).                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Publish component (B3)   | Read-only publishing (SEC-26); off until turned on: the CLI records the widening act (`parinn publish-component on`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| Bridge component (B3)    | Outbound exchange with hosts the node's principal names: the forge bridge reads a linked issue or branch, its associated pull request, reviews and checks only after the read credential owner grants import of that exact target into the room, and sends only a person's explicit issue or review reply (LANE-08, LANE-34, SEC-28), the CI bridge fetches attestations and the notification bridge sends limited notices; off until a bridge is enabled for a host (`parinn bridge on`), the widening act turning it on; it runs while any bridge stands enabled.                                                                                                                                                                    |

Each B1–B3 component's own entry point is started by the person, a service
manager or an ephemeral node's entrypoint, and runs only while the act turning
it on stands; starting `parinn ui` or `parinn launch` records no principal act,
but `parinn ui --phone` records the widening act issuing the phone-scoped
room-view secret (OWN-16).
A configuration turning the room-view component or the launcher off applies
at once (ADM-04, SEC-01).

The core's parts:

| Part of the core       | Responsibility                                                                                                                                                                                                                                                                                                                                       |
| ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `parinn hook <hook>`   | The hook handlers for `SessionStart`, `UserPromptSubmit` and `PermissionRequest`, the hooks through which ParleyInn writes to an agent (§9.1): read hook input from stdin, perform bounded processing within the hook budget, write hook output to stdout, always fail open (I9).                                                                    |
| `parinn mcp`           | The MCP server, one per harness session over stdio, serving its runs (the main run and its subagents' runs) and keeping their run-seat keys (SEC-10); it exposes recall, landmarks, pins, room and kernel tools. It ingests its harness session's transcripts as they grow (REC-13).                                                                 |
| `parinn kernel-worker` | The kernel worker, a child process of `parinn mcp` running the hermetic kernel under OS resource limits with a read-only store handle.                                                                                                                                                                                                               |
| CLI                    | Install, verify, rebuild, quarantine, purge, backup creation and backup restore, migrate, counters, audit, doctor, and principal acts at a terminal (OWN-12).                                                                                                                                                                                        |
| Commit hook            | Run as the CLI from git's commit hook: adds the `ParleyInn-Room:` room trailer and any `ParleyInn-Link:` trailer links to each commit on a branch that belongs to a room (LANE-01, LANE-28).                                                                                                                                                         |
| TUI                    | A reduced client of the room view on a terminal (VIEW-14) and, like the CLI, a principal surface there (OWN-02).                                                                                                                                                                                                                                     |
| Core library           | All logic; no global state; every blocking operation takes a deadline, directly or through a cancellation signal a deadline fires.                                                                                                                                                                                                                   |
| Harness adapter        | Not a component: ParleyInn's code for one harness, split between the core and the launcher. Its transcript and hook part, among the core's hook handlers, MCP server and CLI, parses transcript formats and hook I/O for Claude Code (and later other harnesses), opening no socket and starting no process; its run part sits in the launcher (B1). |

## 4.3 Event-sourced state

The record is authoritative: everything else derives from it. Acts are
themselves events in the record. Principal acts, such as adding, editing or
unpinning a pin of a type that restores written from a device seat a device key
certified (before PRV-10 ships, this node's own device seat), a stamp, a
quarantine, its release and a purge, are `operator` events on the device seat of
the device that signs them (before PRV-10 ships, of the node that records them),
in the room they act on, which that node first joins without admission when its
principal has a member seat there. One that acts on no room or on a room its
principal has no member seat in, such as rejecting a foreign room, or that a
paired phone signs, goes to that device seat in the personal room, naming the
room, and that room shows it by address as it shows a cross-room post (OWN-02,
LANE-29). Once PRV-10 ships, expire acts are `operator` events on the device
seat of the node that signs them. Room acts go to the writer of the seat that
signs them: a post with provenance `post`, a room summary `summary`, never
trusted (LANE-33), every other room act with that seat's pin class, `operator`
for a device seat and `assistant` for a run seat (LANE-31, PRV-01). A purge
under a retention policy is not an act: the node records it naming the policy,
whose setting was the act (ADM-04), and an erasure request it sends peers is
`structural`, where one a principal act sends is `operator`. Every tombstone,
refused-segment event, arrival event, node identity change event, key event and
certificate record is a `structural` event (PRV-01, REC-21, REC-24, PRV-13,
PRV-12). Every derived
table in the store (the search index, spans, landmarks, active pins, quarantine
set, room state, origins, trust levels, run statuses but for their time-relative
freshness marks, integrity statuses, queues, evidence and proof classes, stats)
is a derived artifact whose content `parinn rebuild` reproduces exactly,
whatever
its storage layout, from the writer logs the node holds and the node's own key
set (I10). Purge, the only way stored content is destroyed, removes content but
leaves a tombstone event carrying the removed addresses, counts, reason, the
principal or retention policy that purged, and the commitments of the removed
events, with the structural fields, provenance and recorder of each removed act,
tombstone and event the key set holds (REC-17, ADM-07), never a hash of the
removed content, so rebuilds stay deterministic, no purge undoes an act's
effect, purges stay auditable and nothing retained confirms a guess at what was
purged. Room status and a check's state are computed where shown, never stored.

## 4.4 Key scenarios

### Compaction (the core loop)

1. Context approaches its limit, and Claude Code compacts it. ParleyInn sends it
   nothing for the compaction.
2. Claude Code fires `SessionStart` with `source = compact`. ParleyInn returns
the
   **restore block**: the qualifying pins (PIN-10, verbatim), the landmark
   index, and the recall hint. No transcript names the compacting run yet
   (ASM-06), so INJ-05 decides whose pins it may hold.
3. Once the hook returns, the harness writes the compaction to the transcript.
   The harness session's MCP server, which ingests the transcript as it grows,
   records the compaction and its summary as a `harness_text` event, closes the
   current span and updates landmarks (REC-13).
4. Later, the agent needs a detail that compaction dropped. It calls
   `event_search`, then `event_expand` on the returned address
   range, and receives the exact original inside an untrusted-data envelope.

### Aggregation over large history

1. The agent calls `kernel_exec` with SQL such as
   `SELECT regexp_capture(preview, 'exit code (\d+)', 1) AS code, count(*)
   FROM event_search('timeout') WHERE kind = 'tool_result' GROUP BY code`.
2. The worker keeps the temporary tables and views the SQL creates as the
   run's kernel variables; only the last row-returning statement's rows
   (capped) go back to the agent, tainted by every event the run's
   namespace has read.

## 4.5 Design decisions

| ADR    | Decision                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | Alternatives considered                                             | Rationale                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ADR-01 | **Daemonless core.** Hook handlers and MCP servers, one per harness session, open the store directly. The room-view component and the launcher, which the principal's accepted configuration turns on (ADM-04), and the peer, publish and bridge components are optional and off until turned on, each started by the person, a service manager or an ephemeral node's entrypoint and running only while the act turning it on stands, never on a hook or recall path, and their failure never touches an agent run (I9). | Long-running daemon on loopback (lcm)                               | Removes the port, daemon-credential, stale-daemon, and wedged-daemon failure classes observed in lcm; zero idle footprint; simpler isolation (I8, I9).                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ADR-02 | **Event sourcing.** Every derived artifact, including the effect of every act, is rebuilt from the writer logs a node holds.                                                                                                                                                                                                                                                                                                                                                                                              | Mutable tables                                                      | Makes verification, rollback, and forensic replay possible (I5, I10).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ADR-03 | **Pull-only recall.** Automatic injection accepts only the `TrustedText` type, which can be constructed only from qualifying pins, sanitized structural fields and fixed text ParleyInn ships.                                                                                                                                                                                                                                                                                                                            | Per-prompt memory hints (lcm, claude-mem)                           | Poisoning evidence (§3, item 3). Enforced by the type system, not convention (I2).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ADR-04 | **Hermetic SQL kernel by default**: SQLite, compiled into the core (ADR-07), on a read-only connection to the store; external Python kernel as optional P2.                                                                                                                                                                                                                                                                                                                                                               | Starlark (the prior default), Lua, Python/Jupyter kernel as default | Aggregating the record is what SQL is for, the store is SQLite already, and the kernel adds no dependency. SQLite's authorizer, progress handler and heap limit give the allow-list, the read-only connection, the step budget and the memory limit (CMP-03 to CMP-05); no Python runtime (CON-03). Spike S5 (stakeholder decision, 13 October 2026): in each of Lua and SQL, all 90 of Claude's solutions to S5's 30 aggregation tasks passed, written without running them, and its SQL was half the size of its Lua. Risk: step-by-step logic reads less plainly in SQL; S5's stateful tasks all passed. |
| ADR-05 | **BM25 over SQLite FTS5; no embeddings in v1.**                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | Vector search, hybrid                                               | Deterministic, no model calls at ingest, no extra data leaving the machine; Scroll did well with BM25.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ADR-06 | **Distribution as a Claude Code plugin** (hooks + MCP registration) plus the core executable (CON-02). `parinn install` edits settings only as a fallback.                                                                                                                                                                                                                                                                                                                                                                | Tool rewrites `settings.json` (lcm)                                 | Uses the harness's own extension mechanism; no harness-configuration change in the default path (I7). Depends on ASM-07.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ADR-07 | **SQLite compiled into the core**: through `rusqlite` with SQLite and FTS5 bundled (chosen in [ADR-2610050528](../adr/ADR-2610050528-language-rust-typescript.md); a dependency ADR records the driver); spike S2's FTS5 measurements carry over.                                                                                                                                                                                                                                                                         | Pure-Go drivers (withdrawn with the Go core)                        | CON-02: bundled SQLite links into the core executable. In S2 both pure-Go drivers passed the FTS5, cancellation and WAL checks with 50 concurrent connections and were 10–31% apart at 10M events (ncruces slower); modernc links `os/exec` and `net` (SEC-01) and has no encrypting VFS (OQ-04).                                                                                                                                                                                                                                                                                                           |
| ADR-08 | **Harness adapter interface** for transcript parsing and hook I/O in the core, and for run control in the launcher.                                                                                                                                                                                                                                                                                                                                                                                                       | Claude Code specifics throughout                                    | Contains format churn; enables future harnesses (§1.4).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| ADR-09 | **Redact before storage.** The record is lossless with respect to post-redaction content.                                                                                                                                                                                                                                                                                                                                                                                                                                 | Store raw, redact on read                                           | A secret that is never written cannot leak from backups, indexes, or recall. Trade-off recorded as OQ-05.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ADR-10 | **Deterministic structural landmarks** in v1.                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | LLM-written headlines                                               | Headlines from untrusted free text would reopen the injection path; determinism supports I10. Natural-language headlines for trusted spans (LMK-06, P2) come only from a deterministic template over trusted fields, calling no model (CMP-09).                                                                                                                                                                                                                                                                                                                                                             |
