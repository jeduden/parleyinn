---
id: 2609292005
title: "Spike S3: catalogue the transcript formats"
status: "✅"
summary: >-
  Collect main, subagent, rewritten and cleaned-up transcripts and
  turn every observed variant into parser fixtures.
model: sonnet
depends-on: []
---
# Spike S3: catalogue the transcript formats

## Goal

Know every transcript shape the parser must read before REC-01 to REC-07
are built on it.

## Context

Resolves ASM-03, ASM-04, ASM-05 and ASM-11 to ASM-16, and informs
OQ-12. ASM-11 to ASM-16 come from a review of sottochat's transcript
parser (SRS §14, reference 17).

## Tasks

1. [x] Collect transcripts from a main agent's harness session and from
   subagents
2. [x] Reproduce a rewind or resume that truncates or rewrites a transcript
3. [x] Confirm deletion after `cleanupPeriodDays`
4. [x] Record attachment records, multi-line assistant replies, tool
   results with `toolUseResult`, lines without `uuid` or `timestamp`,
   and thinking blocks
5. [x] Record how three working directories that differ only in `_`, `.`
   and `-` map to transcript directories
6. [x] Commit a redacted fixture for every observed line type and variant
7. [x] Hand the transcript parser's fuzz target (ENG-07) to M1, which
   builds the parser, with these fixtures as its seed corpus

## Outcome

The [report](report.md) catalogues every line type and variant. The
fixtures are under
[testdata/claude-code](../../testdata/claude-code/2.1.296/manifest.json),
recorded from Claude Code 2.1.296 together with spike S1's.

- ASM-03, ASM-11, ASM-12, ASM-13, ASM-14, ASM-15 and ASM-16 are
  verified. ASM-03 adds the `.meta.json` beside each subagent
  transcript. ASM-14 grows to eight line types without a `uuid`.
- ASM-04 is re-planned: no flow truncated or rewrote a transcript. A
  resume, a compaction, a rewind and a partial compaction each appended,
  a rewind as a branch of the `parentUuid` tree. A fork or `/clear`
  starts a new file, a fork repeating the earlier lines with their
  `uuid`s.
- ASM-05 is re-planned: the harness deleted a 40-day-old subagent
  transcript at a start, but kept 40-day-old harness session transcripts over
  four starts. When its retention sweep runs is not visible to ParleyInn.
- OQ-12 stays moot: the three working directories share one transcript
  directory, which REC-04 no longer depends on, since runs are keyed by
  harness session and agent id.

Deviations from the tasks:

- Task 2 found no flow that truncates or rewrites; the scenario for
  ASM-04 now proves the append instead.
- Task 3 could not make the harness delete a harness session transcript on
  demand; the observation is recorded as data (`cleanup.json`), not as
  a tree.
- The scenarios for ASM-04 and ASM-05 replay the fixtures alone. As
  first drafted they ran `parinn ingest`, which belongs to REC-07's and
  REC-08's own scenarios.

Handed on:

- The transcript parser's fuzz target (ENG-07) moved to M1's task 2
  and its acceptance criteria. On 11 October 2026 the stakeholder
  first kept this spike open for it; since M1 depends on this spike
  and builds the parser, the target lands with the parser instead.
- A fork repeats lines under the same `uuid` in a second transcript.
  M1's ingest (REC-02, REC-04) decides whether a forked run's events
  repeat the originals or point at them.

Requirements re-planned in the SRS: ASM-04 and ASM-05. Scenarios taken
off `@pending`: @ASM-03, @ASM-04, @ASM-05 and @ASM-11 to @ASM-16.

## Acceptance Criteria

- [x] ASM-03, ASM-04, ASM-05 and ASM-11 to ASM-16 each marked
  verified or re-planned in the SRS
- [x] Scenarios @ASM-03, @ASM-04, @ASM-05 and @ASM-11 to @ASM-16
  replay the fixtures and pass
- [x] OQ-12 has a recommendation
- [x] The fixtures are ready as the seed corpus for the
  transcript-parser fuzz target (ENG-07), and M1 carries the target in
  its acceptance criteria
