---
title: "Spike S1 report: the hook contract of Claude Code 2.1.296"
summary: >-
  What Claude Code 2.1.296 sends to each hook, which hook output reaches
  the model, and how long each handler may run, recorded in an isolated
  home and replayed by the ASM scenarios.
---
# Spike S1 report: the hook contract of Claude Code 2.1.296

## Method

Every invocation started `claude` 2.1.296 with `HOME` and `CLAUDE_CONFIG_DIR`
in a scratch directory, never the real `~/.claude` (ENG-14). The
`record` kit beside the fixtures holds the handlers: `dump.sh` saves
each hook input and can answer with `additionalContext`, `tick.sh`
measures how long a handler runs before the harness stops it, and
`nonce.sh`, `plain.sh`, `instr.sh` and `stderr.sh` answer with a fresh
random word, and `fixed.sh` answers the same word at every start but a
compaction.
A word the model repeats afterwards proves it read that output; the
model could not have guessed it. Interactive flows ran in `tmux`.
`redact.sh` rewrote paths, the account's email, organization and
connector names, and `strip.sh` dropped the account's skills from skill
listings and the claude.ai connectors' instructions, before anything was
committed.

Probing by marker strings proved unreliable: the model both invented
and denied markers. Every claim about what the model reads rests on a
transcript line or on a random word, never on a marker the model could
infer.

## Hook inputs

Every input carried `session_id`, `transcript_path`, `cwd` and
`hook_event_name`. The rest, by hook:

| Hook               | Further fields                                                                                                                                             |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `SessionStart`     | `source`, `model`; on `resume` and `fork` also `context_tokens`, `prompt_cache_likely_expired`, `seconds_since_last_response`, `estimated_cache_write_usd` |
| `UserPromptSubmit` | `prompt`, `prompt_id`, `permission_mode`                                                                                                                   |
| `PreToolUse`       | `tool_name`, `tool_input`, `tool_use_id`, `prompt_id`, `permission_mode`, `effort`                                                                         |
| `PostToolUse`      | as `PreToolUse`, plus `tool_response`, `duration_ms`                                                                                                       |
| `Stop`             | `last_assistant_message`, `stop_hook_active`, `background_tasks`, `session_crons`                                                                          |
| `SubagentStart`    | `agent_id`, `agent_type`                                                                                                                                   |
| `SubagentStop`     | `agent_id`, `agent_type`, `agent_transcript_path`, `last_assistant_message`                                                                                |
| `PreCompact`       | `trigger` (`manual`, `auto`), `custom_instructions`                                                                                                        |
| `PostCompact`      | `trigger`, `compact_summary`                                                                                                                               |
| `SessionEnd`       | `reason`: `other` in print mode, `prompt_input_exit` on `/exit`, `clear` on `/clear`                                                                       |

An interactive harness session adds `scratchpad_dir` to most inputs. A
hook fired inside a subagent's run adds `agent_id` and `agent_type`;
`PreCompact`, `SessionStart` and `PostCompact` never do (ASM-06).

`source` took five values. `--resume` and `--continue` give `resume`
and keep the `session_id`. `--fork-session` gives `fork` and a new
`session_id`. `/clear` ends the harness session (`SessionEnd` with
`reason` = `clear`) and starts one with `source` = `clear` and a new
`session_id`. A compaction gives `compact` and keeps it. A rewind fires
no `SessionStart`.

Every compaction also fires a `SubagentStop` with an empty
`agent_type`: the harness summarizes in a hidden agent of its own. The
transcript its `agent_transcript_path` names is never written.

## What reaches the model

| Hook output                                               | Reaches the model                                                                                                                                   |
| --------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `SessionStart` `additionalContext`                        | Yes, every `source`, as a `hook_additional_context` attachment; a harness resume drops one equal to one in the transcript since its last compaction |
| `UserPromptSubmit` `additionalContext`                    | Yes, on every prompt                                                                                                                                |
| `PostToolUse` `additionalContext`                         | Yes, after the tool result                                                                                                                          |
| `SubagentStart` `additionalContext`                       | Yes, in the subagent's context                                                                                                                      |
| `PreCompact`, `PostCompact` `hookSpecificOutput`          | Rejected by the harness's output schema                                                                                                             |
| `PreCompact`, `PostCompact` any output, manual compaction | Yes: the `/compact` command output repeats each hook's output, the rejected JSON included, in a `user` line the model reads                         |
| `PreCompact` plain output                                 | Yes, appended to the compaction's instructions: a nonce it asked for appeared in every summary                                                      |
| `Stop`, `SessionEnd` plain output                         | No                                                                                                                                                  |
| Any hook's stderr, exit status 0                          | No: none of `SessionStart`, `UserPromptSubmit`, `PostToolUse`, `Stop`, `PreCompact`, `PostCompact`, `SessionEnd`                                    |
| `PreCompact`, `PostCompact` stderr, exit status 1         | Yes, in the `/compact` command output; the other hooks' stderr did not                                                                              |

The harness's own validation error lists the hooks whose
`hookSpecificOutput` takes `additionalContext`: `UserPromptSubmit`,
`PostToolUse`, `PostToolBatch`, `Stop` and `SubagentStop`, beside
`SessionStart`. The spike tested only the hooks in the table.

For ParleyInn this means three things under I2:

- The `PostCompact` handler must write nothing to stdout, not even an
  error: whatever it writes reaches the model on a manual compaction.
- The `PreCompact` handler's compaction guidance is plain text, and
  reaches both the summarizer and, on a manual compaction, the model.
  PIN-07 already limits it to fixed text ParleyInn ships.
- A subagent's compaction fires `SessionStart` with `source` =
  `compact` under the main agent's identity, and the
  `additionalContext` it returns lands in the subagent's context.
  ParleyInn cannot tell from the hook input which run compacted, so INJ-05
  applies to every compaction while a subagent of the harness session
  runs.

A harness resume after a compaction recorded a `SessionStart`
context again, though one equal to it lay before the compaction, and
the model read it: the drop on resume only skips a copy still in
context.

The harness waited for a `SessionStart` handler that took 8 s before
the first model call, at startup and on a resume, in print and in
interactive mode.

## Harness timeouts

Each handler slept 15 s, logging a tick every 100 ms. Times are from
the handler's start to its last tick.

| Hook                                                      | Mode        | Configured timeout | Ran    |
| --------------------------------------------------------- | ----------- | ------------------ | ------ |
| `SessionStart`, `UserPromptSubmit`, `PostToolUse`, `Stop` | print       | 4 s                | 3.93 s |
| `SessionEnd`                                              | print       | 4 s                | 3.93 s |
| `SessionEnd`                                              | print       | 10 s               | 9.94 s |
| `SessionEnd`                                              | print       | none               | 1.45 s |
| `SessionEnd`                                              | interactive | 10 s               | 9.93 s |
| `SessionEnd`                                              | interactive | none               | 1.45 s |

In print mode, whenever a `Stop` handler reached its timeout, no
`SessionEnd` handler started, across three trials. A quick `Stop` handler
left `SessionEnd` alone. ParleyInn therefore installs `SessionEnd` with its
own timeout, and its `Stop` handler stays within NFR-02's budget. The
binary also reads `CLAUDE_CODE_SESSIONEND_HOOKS_TIMEOUT_MS`, which the
spike did not test.

## Not covered

- Exit status 2, which the harness documents as blocking for some
  hooks and as feeding their stderr to the model.
- Whether the `PreCompact` and `PostCompact` output reaches the model
  after an automatic compaction. The recorded ones showed no such
  line.
- `PostToolBatch`, `Stop` and `SubagentStop` `additionalContext`.
- A `PreCompact` hook exiting 2 (ASM-09) is spike S6's.

## Raised for the stakeholder

The reviews of this spike raised findings that need a ruling, not a
spike. OQ-51 tracks each of them; none is written into a requirement
here.

From the security officer:

- **Every channel out of a hook handler.** §9.1 bounds stdout and the
  exit status. Draft: hook handlers write nothing to stderr, exit 0 on
  every path, suppress the default panic message, and log only to the
  home's log file; OPS-05 drops stderr for hook handlers.
- **INJ-05 across rooms.** Restoring "the pins of every run it may
  belong to" can restore a pin that qualifies only in the main agent's
  room into a subagent seated elsewhere, an I2 path. Draft: restore
  only pins that qualify for every candidate run, state the rest by
  count, treat each compaction hook as ambiguous while a subagent may
  run (tracked from `SubagentStart` and `SubagentStop`), and trace
  INJ-05 to I2 and I8.
- **A lost `SessionEnd` is silent (I6).** Any `Stop` handler that
  reaches its timeout in print mode, ParleyInn's or another, suppresses
  every `SessionEnd`. Draft: each handler leaves a start marker and
  clears it; the next handler, the MCP server or `parinn status` counts
  and audits the unfinished ones.
- **The compaction's own agent.** Its `SubagentStop` has an empty
  `agent_type` and no transcript. Draft: record it as a hook
  observation of a compaction that starts, seals and counts no run;
  mark a `compact_summary` whose run is ambiguous.
- **Compaction guidance wording.** The summarizer cannot see trust, so
  guidance to keep constraints verbatim also keeps injected text that
  reads like one. Draft: guidance never asks to preserve content by
  its wording and asks to keep tool, web and file content marked as
  data.
- **Harness-added context.** `session_context` carries the account's
  email with role `user`. Draft: every `attachment` line is untrusted
  `harness_text`, and account identifiers are redacted before storage.
- **Hook commands reach the model on failure.** Draft: the commands
  ParleyInn installs carry no secret, home path or record content.
- **Fixture redaction.** Draft: recorded harness fixtures pass a
  residual-identifier check, proven by a drift case (ENG-27). The
  committed fixtures were stripped by hand of the account's skills and
  the connectors' instructions; nothing checks that a re-recording is.

From the domain model:

- **Harness fork.** Does `--fork-session` start a new run, and are the
  lines it repeats new events or the same addresses seen again?
- **Compaction guidance** can now reach the agent's context directly
  on a manual compaction; the definition says only that it goes to
  the harness.
- **The compaction's own agent.** Is it a subagent with a run, or a
  harness internal ParleyInn records no run for?
- **Whose restore block** a `SessionStart` with `source` = `compact`
  builds when a subagent compacted (INJ-01, INJ-05).
