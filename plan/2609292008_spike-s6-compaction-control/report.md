---
title: "Spike S6 report"
summary: >-
  What Claude Code 2.1.296 does when a PreCompact hook exits 2: it
  blocks every manual and automatic compaction, and at the full context window
  it refuses every prompt until a compaction is let through.
---
# Spike S6 report: can compaction be deferred

## Method

Two print-mode harness sessions of Claude Code 2.1.296, recorded by
`record/defer.sh` in a scratch `HOME` and `CLAUDE_CONFIG_DIR` under a
clean environment, as in S1 and S4 (ENG-14). The `PreCompact` handler,
`record/block.sh`, saved its hook input, wrote a fresh random word to
stderr and exited 2. Every other hook ran `record/dump.sh`. A word the
model repeats proves it read the stderr; it could not have guessed it.

- Flow `compact-blocked`: one prompt, then `/compact` on `--resume`,
  then a prompt asking whether any message gave a secret word.
- Flow `full-window`: one prompt that reads twelve files of about 12k
  tokens each, one per tool call, with
  `CLAUDE_CODE_AUTO_COMPACT_WINDOW=100000` and
  `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=40`, so the automatic compaction's
  threshold falls at 40k tokens. Then the same question on `--resume`.

The models here run with a 1M-token window, so a recording at the
model's own window would have run to megabytes. The full-window flow
lowers the full context window to 60k tokens with
`CLAUDE_CODE_BLOCKING_LIMIT_OVERRIDE=60000`, the harness's own
override of the limit it enforces there. Reading the 2.1.296 binary
shows that limit is the model's window less 3,000 tokens without the
override, and that the harness takes the same path at it either way.

## Findings

| Question                                          | Observed                                                                                                                                                                                                                                         |
| ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Does exit status 2 block a manual compaction?     | Yes. `/compact` failed; no compaction boundary, no `PostCompact`, no `SessionStart` with `source` = `compact`, no compaction `SubagentStop`                                                                                                      |
| What does a blocked manual compaction record?     | A `system` line of subtype `local_command`, `commandOutcome` `failed`: `<local-command-stderr>Compaction blocked by PreCompact hook: [<command>]: <stderr></local-command-stderr>`                                                               |
| Does the model read it?                           | Yes: asked on the next prompt, the model answered with the word the handler wrote to stderr                                                                                                                                                      |
| Does exit status 2 block an automatic compaction? | Yes. Each `PreCompact` carried `trigger` = `auto`; none was followed by a compaction                                                                                                                                                             |
| How often does the harness try?                   | Each time it calls the model, from somewhat before the threshold on: twice before the full context window in the full-window flow, then once at it                                                                                               |
| What does a blocked automatic compaction record?  | Nothing in the transcript; the stream shows a `compacting` status and then none. The handler's stderr reached no transcript                                                                                                                      |
| What happens at the full context window?          | The harness tries once more (`PreCompact`, blocked), then ends the turn with an assistant line it writes itself, `Prompt is too long`, marked `isApiErrorMessage`, without calling the API; the print-mode process exits 1; no `Stop` hook fires |
| And on the next prompt?                           | `PreCompact` fires at once, is blocked, and the prompt is refused the same way: the harness session stays stuck until a compaction is let through or it is cleared                                                                               |

## Seen, not recorded

- A first recording of the automatic flow, with Sonnet and the 100k window
  but without the blocking override, read sixteen files of about 25k
  tokens in one prompt. `PreCompact` fired sixteen times, each blocked,
  and the context grew to about 416k tokens with no refusal: the
  configured window sets the threshold, not the limit. Its transcript,
  1.2 MB, was not committed.
- A harness session that read one file per prompt on `--resume` kept the context
  near 30k tokens: on a harness resume, the earlier `Read` results did
  not reach the model again, without any compaction or hook. Content
  can leave the agent's context with no compaction for ParleyInn to see.
  SRS §13.2 tracks it as a risk, to be recorded before M2's restore
  relies on compaction alone.

## What it means for ParleyInn

- **ASM-09 holds.** A `PreCompact` hook exiting 2 blocks compaction,
  manual and automatic, on Claude Code 2.1.296. The Morph plugin
  README's "cannot be prevented" does not hold for this version.
- **Blocking only defers.** No hook removes anything from the agent's
  context, so a blocked compaction leaves the context to grow until the
  full context window. There the harness refuses every prompt. ParleyInn would
  stop the agent, against I9, unless it then lets compaction through,
  which is what (a) already does, later and at the same cost.
- **The block is a write to an agent.** On a manual compaction the
  model reads the handler's command and stderr. A ParleyInn handler would
  have to write only fixed text there, as I2 requires, and exit 2,
  against OQ-51 (b), which has hook handlers exit 0 on every path and
  write nothing to stderr.
- **Automatic blocks are silent.** The transcript records none of them,
  so I6 would have ParleyInn count and audit each one itself.

## Recommendation for OQ-01

Keep option (a), coexisting with native compaction, as v1 does, and
NG1 as it stands. Drop option (b): deferring compaction through
`PreCompact` is possible, but without a way to evict it only moves
native compaction to the full context window and blocks the agent
there. Option (c), a proxy, stays outside the core under I4.

The stakeholder closed OQ-01 this way on 13 October 2026, and kept
three options for later under UC-006: a bounded veto, compaction the
launcher asks the harness for, and (c).

## Prior art

How others handle the same harness behaviour, as their public pages
describe it on 13 October 2026:

- [Omnigent][omnigent], Databricks' meta-harness, moved "compaction
  ownership" from its runner back to each wrapped harness in v0.3.0,
  surfaces compaction failures, and restores compaction and transcript
  on resume: it coexists, as (a).
- Memory plugins such as [claude-mem][claude-mem] snapshot in
  `PreCompact` and inject again through `SessionStart` after a
  compaction: also (a).
- The [`PreCompact` veto pattern][veto] blocks only automatic
  compactions mid-task, lets one through at a safe point and forces one
  after three blocks. It warns that an indefinite block exhausts the
  window, as S6 recorded.
- [Hyperhive][hyperhive] turns Claude Code's auto-compaction off through
  `--settings`. After a turn above a model-token watermark, 150k by default,
  it has the agent write its state to files and then sends `/compact`;
  on "Prompt is too long" it compacts at once and retries the turn.

[omnigent]: https://omnigent.ai/releases/0.3.0
[claude-mem]: https://docs.claude-mem.ai/configuration/hooks
[veto]:
  https://agentpatterns.ai/tool-engineering/precompact-hook-compaction-veto/
[hyperhive]:
  https://git.berlin.ccc.de/vinzenz/hyperhive/commit/9cbb05bb862aa4fa60b488c6638455c0849a7887

## Fixtures

Under `testdata/claude-code/2.1.296`: the hook inputs in
`hooks/compact-blocked` and `hooks/full-window`, and the transcripts
under `transcripts/projects/-w-defer` and
`transcripts/projects/-w-defer-full`. The manifest's
`compaction_blocked` names them with the words the handler wrote to
stderr. `record/export-defer.sh` copied them, redacted them with
`record/redact.sh` and stripped them with `record/strip.sh`. @ASM-09
replays both.

## Not covered

- An interactive harness session; both flows ran in print mode.
- A subagent's compaction. The binary logs that an in-process
  subagent whose compaction is blocked carries on uncompacted.
- Exit status 2 with nothing on stderr, and a block given as a JSON
  answer instead of the exit status.
- The full context window at a model's own limit, rather than the lowered one.
