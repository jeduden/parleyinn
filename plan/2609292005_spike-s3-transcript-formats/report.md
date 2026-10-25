---
title: "Spike S3 report: the transcript formats of Claude Code 2.1.296"
summary: >-
  Where Claude Code 2.1.296 keeps transcripts, every line type it
  writes, how a resume, fork, rewind or compaction changes a file, and
  what its cleanup deletes.
---
# Spike S3 report: the transcript formats of Claude Code 2.1.296

## Method

The transcripts come from the same isolated invocations as spike S1's
[report](../2609292003_spike-s1-hook-contract/report.md), redacted the
same way. The fixture tree under
[testdata/claude-code](../../testdata/claude-code/2.1.296/manifest.json)
keeps them in the harness's own layout; its `manifest.json` names each
by role.

## Layout

A harness session writes
`projects/<dir>/<session_id>.jsonl`. A subagent writes
`projects/<dir>/<session_id>/subagents/agent-<agent_id>.jsonl`, with a
`.meta.json` beside it naming `agentType`, the `description`, the
calling `toolUseId`, `spawnDepth` and whether it ran in the
foreground. Each of its lines carries the main agent's `sessionId`, its
`agentId` and `isSidechain` = `true`.

`<dir>` is the working directory with every character other than a
letter or digit replaced by `-`: `/w/my_app`, `/w/my.app` and
`/w/my-app` all write to `-w-my-app` (ASM-16).

## How flows change a file

| Flow                            | Effect on the transcript                                                                                    |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Resume, continue                | Appends to the same file                                                                                    |
| Compaction, manual or automatic | Appends a `compact_boundary` line and the summary as a `user` line with `isCompactSummary`                  |
| Rewind ("restore conversation") | Appends; the new prompt's `parentUuid` points at the line before the abandoned prompt, a branch of the tree |
| Rewind ("summarize from here")  | Appends a partial compaction: a `compact_boundary` whose `logicalParentUuid` points into the earlier chain  |
| Fork (`--fork-session`)         | A new file under a new `session_id`, repeating the earlier lines with their `uuid`s and the new `sessionId` |
| `/clear`                        | A new file under a new `session_id`                                                                         |

In every flow the earlier file stayed a byte prefix of the later one;
no flow truncated or rewrote a transcript (ASM-04). The live
conversation is the path from the newest line back through
`parentUuid`, not the file's order.

## Line types

Lines in the committed fixtures:

| `type`                  | Lines | `uuid` | `timestamp` |
| ----------------------- | ----- | ------ | ----------- |
| `attachment`            | 278   | yes    | yes         |
| `assistant`             | 56    | yes    | yes         |
| `user`                  | 47    | yes    | yes         |
| `system`                | 38    | yes    | yes         |
| `last-prompt`           | 34    | no     | no          |
| `atis-latch`            | 27    | no     | no          |
| `queue-operation`       | 22    | no     | yes         |
| `mode`                  | 17    | no     | no          |
| `cost-state`            | 14    | no     | no          |
| `file-history-snapshot` | 12    | no     | no          |
| `permission-mode`       | 10    | no     | no          |
| `ai-title`              | 7     | no     | no          |

`system` lines carry a `subtype`: `compact_boundary`,
`stop_hook_summary`, `turn_duration`.

Attachments carry `attachment.type`, and those the model reads carry a
`renderedRole` (ASM-11): CLAUDE.md files as `instructions` with role
`user`; MCP server instructions as `mcp_instructions_delta`, skills as
`skill_listing`, hook output as `hook_additional_context`, all with
role `system`. `session_context` and `remote_session_change` carry
role `user`; `hook_success`, `prompt_snapshot` and `credential_org`
carry none.

An assistant line holds one content block; one API message spans as
many lines as blocks, all with its `message.id` and the same `usage`
(ASM-12). A tool result repeats its output in `toolUseResult` and names
its call's line in `sourceToolAssistantUUID` (ASM-13). The shape of
`toolUseResult` differs by tool: `stdout` and `stderr` for Bash, the
content array for an MCP tool. Every thinking block kept its signature
and no text (ASM-15).

## Cleanup

Transcripts were planted with old modification times and the harness
started four times, the last with `cleanupPeriodDays` = 1. The first
start deleted a 40-day-old subagent transcript; no start deleted a
40-day-old harness session transcript, and the sentinel file `.last-cleanup`
was not written again. The binary's retention sweep checks conditions
the spike could not observe from outside. ParleyInn should treat a
transcript as deletable at any harness start (ASM-05, REC-08). Git
keeps no modification times, so the fixture is a listing,
`cleanup.json`, of the planted paths with their ages and of the paths
left after the starts, not a tree; @ASM-05 replays that listing.

## OQ-12

Moot, as the stakeholder ruled on 5 October 2026. The directory
mapping is many-to-one, as the spike confirmed, and REC-04 keys runs by
harness session and agent id, which the layout above carries on every
line.
