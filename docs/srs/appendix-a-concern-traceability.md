---
title: "Appendix A — Concern traceability"
summary: >-
  Every concern raised during research and review, mapped to the
  requirements that answer it.
---
# Appendix A — Concern traceability

Every concern raised during research and review, mapped to ParleyInn's response.

## A. Security review of lossless-claude/lcm

- **A1**: Memory auto-injected at each run's start and on every prompt re-enters
  unrelated runs
  - ParleyInn's response: Pull-only recall; restore blocks limited to
    `TrustedText`; a restore block on a prompt only while
    `restore_block.on_prompt` is on, off by default
  - Requirements: INJ-03, INJ-04, SEC-07
- **A2**: Data leaves the machine by default through the summarizer; privacy
  docs said otherwise
  - ParleyInn's response: No network in the core; no summarizer in core; privacy
    claims backed by tests
  - Requirements: SEC-01, SEC-15, LANE-33, ENG-12, ENG-22
- **A3**: One plaintext archive protected only by regex redaction
  - ParleyInn's response: Redaction before storage with markers that carry no
    hash of the secret; encryption at rest; retention policies; secure purge
  - Requirements: SEC-08, SEC-09, REC-15, ADM-07, SEC-14
- **A4**: Isolation only per OS user; fixed loopback port; credential file
  readable by same-user processes
  - ParleyInn's response: No sockets and no socket credentials in the core; the
    optional room-view component listens on loopback only, on a port chosen at
    launch, behind a room-view secret, a launch secret exchanged once for an
    origin secret no other origin can read; checks that the home belongs to the
    OS user running ParleyInn, and permission checks; home binding
  - Requirements: SEC-01, SEC-02, SEC-03, SEC-20, ADR-01
- **A5**: One FTS5 query froze the daemon for minutes
  - ParleyInn's response: No daemon; literal-term `event_search` query parser
    with limits and cancellation
  - Requirements: SEC-04, ADR-01
- **A6**: Tool rewrites `settings.json`; `doctor` has no read-only mode;
  conflicts with the harness's managed settings
  - ParleyInn's response: Plugin distribution; explicit install with diff; exact
    uninstall; read-only doctor; the harness's managed settings respected
  - Requirements: ADM-01, ADM-02, ADM-03, ADM-10
- **A7**: Silent failures dropped 21,279 events unnoticed
  - ParleyInn's response: Every failure audited and counted; `doctor` fails on
    new failures; completeness verification; canary
  - Requirements: OPS-01..04, ADM-09
- **A8**: Credentials frozen into a daemon's inherited environment
  - ParleyInn's response: No credentials in core beyond its own keys and the
    at-rest key; explicit per-use secret references for every other need
  - Requirements: SEC-10
- **A9**: Repository ships agent skills running destructive commands on the real
  `HOME`; tests reached the real daemon
  - ParleyInn's response: Isolated test homes with guard; no destructive agent
    instructions in the repository
  - Requirements: ENG-14
- **A10**: Maintainer concentration; heavily AI-authored changes
  - ParleyInn's response: Review of every change by an agent other than the one
    that made it, through the reviewer app's required check; stakeholder
    approval of what guides the review; a named security reviewer on invariant
    changes; every reviewer component evaluated against its cases on its model;
    labelled AI changes; ≥ 2 maintainers
  - Requirements: ENG-21, ENG-25, ENG-28, ENG-29, ENG-32
- **A11**: Uninstall leaves all data behind
  - ParleyInn's response: Uninstall offers a recorded purge; status always shows
    data location and size
  - Requirements: ADM-07, ADM-11
- **A12**: Contested memories are still recalled
  - ParleyInn's response: Quarantine removes items from recall immediately
  - Requirements: SEC-12, RCL-06

## B. How poisoning risk scales

- **B1**: Auto-injection multiplies carry-over of poison (64.7% vs 17.4%
  pull-only)
  - ParleyInn's response: Pull-only by design
  - Requirements: INJ-03, INJ-04, ADR-03
- **B2**: Poison accumulates; a lossless store never forgets it
  - ParleyInn's response: Quarantine without deletion; retention windows for
    untrusted payloads; rebuildable derived artifacts
  - Requirements: SEC-12, REC-15, ADM-08
- **B3**: Store growth doesn't dilute poison; manual review stops being feasible
  - ParleyInn's response: Flags set at ingest, visible in recall and queryable;
    flagged text contributes only counts to landmarks
  - Requirements: PRV-07, LMK-04
- **B4**: Compaction as a write channel
  - ParleyInn's response: Compaction summaries untrusted and never restored;
    structural landmarks only
  - Requirements: PRV-03, INJ-09, LMK-02, ADR-10
- **B5**: Shared memory multiplies blast radius
  - ParleyInn's response: Recall defaults to the agent's current run, extends
    only to rooms it has a seat in and the cross-room posts they show, and
    reaches a foreign room only when the call names it; no shared stores in v1
  - Requirements: RCL-05, NG4, MEM-01
- **B6**: Unattended agents act on poisoned content with nobody watching
  - ParleyInn's response: Audit log; recall taint, with an example `PreToolUse`
    hook handler that asks for approval once a run carries it
  - Requirements: OPS-01, OPS-02, SEC-13
- **B7**: Per-prompt search cost and stalls grow with the store
  - ParleyInn's response: No per-prompt search by default; latency targets at
    1M–10M events; bounded queries
  - Requirements: INJ-04, NFR-01, NFR-03, SEC-04
- **B8**: Structural fields (file names, MCP tool names) as an injection vector
  - ParleyInn's response: Allow-list sanitization
  - Requirements: LMK-03
- **B9**: Repository configuration could downgrade security
  - ParleyInn's response: A repository's `.parinn.toml` can only tighten the
    principal's configuration
  - Requirements: SEC-11
- **B10**: Prompts on runners may carry external content (e.g. issue text)
  - ParleyInn's response: `automation` deployment mode by default treats `user`
    events as untrusted
  - Requirements: PRV-04

## C. Context-management research

- **C1**: Summaries lose exact detail (Scroll 73.1 → 19.9)
  - ParleyInn's response: Immutable record; exact `event_expand` and `event_get`
  - Requirements: REC-06, REC-09, RCL-03
- **C2**: Compaction erases constraints, including those stated mid-run
  - ParleyInn's response: Verbatim pins restored after every compaction; pin
    candidate detection
  - Requirements: PIN-01..06, PIN-08, PIN-09, INJ-01
- **C3**: Context rot: effective context is smaller than the window
  - ParleyInn's response: Small, capped restore block; landmarks; recall on
    demand; kernel keeps bulk data out
  - Requirements: INJ-07, LMK-05, CMP-06
- **C4**: Native compaction can't be reliably disabled
  - ParleyInn's response: Coexist in v1, kept after S6 (OQ-01); steering when it
    happens is UC-006
  - Requirements: NG1, OQ-01
- **C5**: Context mutations break prompt caching
  - ParleyInn's response: Byte-deterministic restore blocks; cache-hit rate is
    an acceptance metric
  - Requirements: INJ-06, §11.1
- **C6**: Transcripts deleted after `cleanupPeriodDays`
  - ParleyInn's response: Record independent of transcript sources;
    missing-transcript-source detection
  - Requirements: REC-08
- **C7**: Subagent transcripts carry much of the data; compaction hooks lack the
  harness's agent id
  - ParleyInn's response: Subagent ingestion and parent links; attribution from
    the transcript source; pins-only fallback
  - Requirements: REC-02, REC-04, INJ-05
- **C8**: Model-directed vs deterministic control unsettled
  - ParleyInn's response: Deterministic for safety-relevant state;
    model-directed recall
  - Requirements: ADR-03, ADR-10, §1.3
- **C9**: Published benchmarks uncontrolled and mostly on other models
  - ParleyInn's response: Acceptance measured on production Claude models only
  - Requirements: §11.1
- **C10**: The agent may not use recall tools
  - ParleyInn's response: Recall hint in the restore block; tool-use metric;
    versioned tool descriptions
  - Requirements: INJ-01, §9.2, §11.1
- **C11**: Multi-run reasoning weak for all systems
  - ParleyInn's response: Tracked as known weakness; kernel aggregation
  - Requirements: §11.1, §13.2
- **C12**: Code-based access beats serialized history
  - ParleyInn's response: Hermetic kernel with read-only record access
  - Requirements: CMP-01..07

## D. Deployment and operations

- **D1**: The model runs at the provider, so recalled content leaves the network
  - ParleyInn's response: Stated in I4; redaction before storage; scope limits
    and quarantine bound what can be recalled
  - Requirements: SEC-08, RCL-05, SEC-12
- **D2**: Short harness timeouts, and a `Stop` handler at its timeout
  suppressing `SessionEnd`
  - ParleyInn's response: Only the hooks that write to an agent, with hook
    budgets and start markers; ingest by the MCP server from the transcripts,
    with ingest markers; crash-safe store
  - Requirements: NFR-01, NFR-02, NFR-07, REC-13
- **D3**: Runner state must survive pause and resume
  - ParleyInn's response: Home on durable volumes; verified in S4
  - Requirements: §2.2, S4
- **D4**: Installation on runners without overriding the harness's managed
  settings
  - ParleyInn's response: Plugin or runner-image installation; the harness's
    managed settings respected
  - Requirements: ADM-01, ADM-03

## E. Prior art: sottochat's transcript parser

- **E1**: Context the harness adds, CLAUDE.md included, is written as attachment
  lines whose rendered role can be `user`
  - ParleyInn's response: Provenance from structure, never from the rendered
    role; repository files are `file`
  - Requirements: ASM-11, PRV-08
- **E2**: Most `type: "user"` lines carry tool results, harness metadata or
  command output, not text typed at the harness's input
  - ParleyInn's response: Only text typed at the harness's input is `user`;
    everything else takes its own class
  - Requirements: PRV-08, LMK-01
- **E3**: Markers such as `<system-reminder>` also appear inside tool output,
  where anyone controlling the tool can write them
  - ParleyInn's response: Text markers may lower trust and never raise it
  - Requirements: PRV-08
- **E4**: A tool result is stored twice on its line, and its call may have been
  ingested by an earlier ingest
  - ParleyInn's response: Redaction covers every field; the tool is resolved
    from the stored events
  - Requirements: ASM-13, SEC-08, PRV-08
- **E5**: Some lines carry no timestamp; filling one from the clock makes two
  ingests of one transcript disagree
  - ParleyInn's response: No stand-in timestamp; the hash chain stays
    reproducible
  - Requirements: ASM-14, REC-12, REC-10
- **E6**: A deny-list of SQLite features admits whatever a new release adds
  - ParleyInn's response: Everything the kernel allows is on an explicit
    allow-list
  - Requirements: CMP-03
- **E7**: Distinct working directories may share one harness transcript
  directory
  - ParleyInn's response: Runs are keyed by harness session and the harness's
    agent id, never by directory
  - Requirements: ASM-16, REC-04
