---
title: "Spike S13 report"
summary: >-
  What Claude Code 2.1.296 shows a hook handler when a background
  subagent or the main agent compacts: identical hook inputs, and no
  compaction boundary in any transcript until the hooks return.
---
# Spike S13 report: can a hook tell which run compacted

## Method

Two print-mode runs of Claude Code 2.1.296, in a scratch `HOME` and
`CLAUDE_CONFIG_DIR` as in S1 (ENG-14), with
`CLAUDE_CODE_AUTO_COMPACT_WINDOW=100000` and
`CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=40`. Every hook ran `record/snap.sh`.
It saves the hook input and, beside it, a snapshot of every transcript
of the harness session at that moment: lines, compaction boundaries
and the last line's type. Each `SessionStart` answered with a fresh
nonce, so its landing place is a fact, not an inference.

- Flow `subagent-compaction-background`: a background subagent
  (`run_in_background`) read large files and compacted three times.
- Flow `compaction-during-background`: the main agent launched a
  background subagent, then read large files and compacted three times.
  The subagent was still running at the first compaction.

## Findings

| Question                                                       | Observed                                                                                                                                                              |
| -------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Do the compaction hooks' inputs differ by compacting run?      | No. `PreCompact`, `SessionStart` (`compact`) and `PostCompact` carry the main agent's `session_id` and `transcript_path`, no subagent field, in both flows            |
| Do they carry anything new?                                    | `prompt_id`, the id of the main agent's prompt; a subagent's lines carry the same `promptId`, so it does not tell the runs apart                                      |
| Does a transcript name the compacting run while the hooks run? | No. The `compact_boundary` line reaches the compacting agent's transcript only after `SessionStart` and `PostCompact` have returned                                   |
| Where does `SessionStart`'s `additionalContext` land?          | In the compacting agent's context: the background subagent's transcript in the first flow, the main transcript in the second                                          |
| When does a background subagent's tool call get its result?    | At launch ("Async agent launched"); the main agent's turn can end while the subagent runs                                                                             |
| How does its end show?                                         | A `queue-operation` and a `user` line carrying `<task-notification>` with the subagent's task id, `promptSource` `system`, `turnOrigin` `task_notification`           |
| Does `UserPromptSubmit` fire for that notification?            | Yes, with the notification as `prompt`; the hook input does not mark it as the harness's own                                                                          |
| Does a transcript mark a turn's end without a `Stop` hook?     | Yes: the assistant line's `message.stop_reason` is `end_turn`, from the API                                                                                           |
| How does a `user` line mark a prompt's origin?                 | `promptSource`: `typed` for a prompt typed interactively, `sdk` for print mode, `system` for the harness's own; the compaction summary's line inherits a `turnOrigin` |
| Does each compaction fire a `SubagentStop`?                    | Yes, with a fresh `agent_id` and an empty `agent_type`, as S1 found (ASM-06, R4)                                                                                      |

## What it means for ParleyInn

- A compaction is ambiguous whenever a subagent of the harness session
  may be running. For a foreground subagent that is from its tool call
  until its tool result; for a background one, until the task
  notification naming it. A rule built only on open tool calls, as
  first drafted, misses background subagents.
- The record tells afterwards which run compacted, from where the
  boundary lands, but a hook handler must decide before that.
- Trusting a `user` line needs more than the `UserPromptSubmit` input:
  the harness submits its own notifications through it. The line's
  `promptSource` tells a typed prompt from the harness's own.
- Turn ends need no `Stop` hook (REC-19, REC-20, PEER-05).

## Fixtures

Under `testdata/claude-code/2.1.296`: the hook inputs and snapshots in
`hooks/subagent-compaction-background` and
`hooks/compaction-during-background`, and both harness sessions'
transcripts under `transcripts/projects/-w-bg`. They were redacted with
`record/redact.sh` and stripped with `record/strip.sh`. @ASM-06 replays
both flows beside S1's foreground case.

## Not covered

- A manual `/compact` while a background subagent runs.
- Several foreground subagents started in one message.
- How long the harness takes to flush a line; spike S9 measures it.
