---
id: 2609292003
title: "Spike S1: record the hook contract"
status: "✅"
summary: >-
  Record real hook payloads, output handling and budgets for every
  hook ParleyInn uses, including subagent compaction.
model: sonnet
depends-on: []
---
# Spike S1: record the hook contract

## Goal

Pin down what Claude Code actually sends to and accepts from each hook,
so §9.1 is built on recorded fixtures rather than documentation.

## Context

Resolves ASM-01, ASM-02, ASM-06 and ASM-10, and informs OQ-02. The
recorded payloads seed the ENG-17 contract tests.

## Tasks

1. [x] Record stdin payloads for SessionStart (all four sources),
   PreCompact, PostCompact, PostToolUse, Stop, UserPromptSubmit and
   SessionEnd
2. [x] Record a subagent compaction and note which identity its hooks carry
3. [x] Test whether `additionalContext` is honored per hook
4. [x] Measure the effective timeout of each hook, SessionEnd included
5. [x] Commit the fixtures under testdata and write the contract into
   [docs/srs/09-interfaces.md](../../docs/srs/09-interfaces.md) if it
   differs

## Outcome

The [report](report.md) holds the observations. The fixtures were
recorded from Claude Code 2.1.296 in an isolated home and redacted.
They live under
[testdata/claude-code](../../testdata/claude-code/2.1.296/manifest.json),
with the hook handlers that recorded them in its `record` kit. The
`harness-contract` crate reads them, and the `@ASM-*` scenarios replay
them through it.

- ASM-01 is verified, with one more `source`: `fork`, from
  `--fork-session`, under a new harness session.
- ASM-06 is verified: a subagent's compaction fires `PreCompact`,
  `SessionStart` with `source` = `compact` and `PostCompact` with the
  main agent's identity and no subagent field. The `additionalContext`
  that `SessionStart` returns lands in the subagent's context, so INJ-05
  decides every compaction while a subagent runs.
- ASM-02 is re-planned. `SessionStart` (every `source`),
  `UserPromptSubmit` and `PostToolUse` honour `additionalContext`;
  `PreCompact` and `PostCompact` reject it. On a manual compaction the
  harness repeats both hooks' output, rejected or plain, where the
  model reads it, and appends `PreCompact`'s plain output to the
  compaction's instructions.
- ASM-10 is re-planned. Each hook's configured harness timeout holds,
  `SessionEnd`'s included; without one `SessionEnd` gets about 1.45 s.
  In print mode a `Stop` handler that reaches its harness timeout keeps
  every `SessionEnd` handler from starting.
- OQ-02 is answered: `PreCompact`'s plain output steers the summary, a
  main agent's and a subagent's alike.
- §9.1 now lists `fork`, has `PreCompact` answer in plain text and
  `PostCompact` write nothing, and installs every hook with its own
  harness timeout.

Deviations from the tasks:

- Task 1 recorded five `SessionStart` sources, `fork` among them, and
  `PreToolUse`, `SubagentStart` and `SubagentStop` beside the hooks
  named.
- Spikes S1 and S3 share one recording and one fixture tree; the
  transcripts S3 catalogues carry this spike's `additionalContext`
  evidence.

Handed on:

- INJ-05's implementation tells a subagent's compaction from its
  main agent's only by a `SubagentStart` with no `SubagentStop` yet; M2
  binds that rule.
- A hook's stderr stayed out of the model's context on exit status 0
  and reached it from `PreCompact` and `PostCompact` on exit status 1.
  M2's hook handlers exit 0 on every path, a caught panic included.
- On a harness resume the harness drops a `SessionStart` context
  equal to one in the transcript since its last compaction, and a
  resumed or forked `SessionStart` carries `context_tokens`. M2's
  restore budget (PIN-08) may use the latter.

Requirements re-planned in the SRS: ASM-02 and ASM-10. Scenarios taken
off `@pending`: @ASM-01, @ASM-02, @ASM-06 and @ASM-10.

## Acceptance Criteria

- [x] ASM-01, ASM-02, ASM-06 and ASM-10 each marked verified or
  re-planned in the SRS
- [x] Scenarios @ASM-01, @ASM-02, @ASM-06 and @ASM-10 replay the
  recorded fixtures and pass
- [x] OQ-02 has an answer or a stated follow-up
