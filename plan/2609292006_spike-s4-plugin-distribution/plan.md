---
id: 2609292006
title: "Spike S4: prove plugin distribution"
status: "✅"
summary: >-
  Install a stub ParleyInn plugin on a workstation, a runner image and
  the Agent SDK, and check how runners seed ~/.claude.
model: sonnet
depends-on: []
---
# Spike S4: prove plugin distribution

## Goal

Show that ParleyInn can ship as a Claude Code plugin on every target ADR-06
names, so ADM-01 and NFR-12 rest on a working install.

## Context

Resolves ASM-07, ASM-08 and ASM-21, and informs OQ-06.

## Tasks

1. [x] Package a plugin that registers a no-op hook and a stub MCP server
2. [x] Install it on a workstation, a self-hosted runner image and an
   Agent SDK worker
3. [x] Check how the runner seeds `~/.claude` and which OS user it runs as
4. [x] Check whether the `interactive` deployment mode can be detected
   (OQ-06)

## Outcome

The [report](report.md) holds the observations. The stub plugin, its
recorder and its exporter live in the `record/plugin` kit under
[testdata/claude-code](../../testdata/claude-code/2.1.296/manifest.json);
the fixtures, recorded from Claude Code 2.1.296 and Agent SDK 0.3.296
in isolated `HOME` directories and redacted, under its `plugin` directory. The
`harness-contract` crate reads them, and @ASM-07 replays them.

- ASM-07 is verified: the plugin's hooks fired and its MCP tools were
  listed on a workstation, on a `~/.claude` baked as a runner image would
  bake it, with settings alone, and in an Agent SDK worker.
- ASM-21 is re-planned: one MCP server served the main run and a
  subagent, and the keys it minted reached no transcript, hook input or
  tool result. But a key handed over at launch in the harness's
  environment reached the Bash tool, and the model printed it, so
  OQ-55 asks how SEC-10's hand-over at launch keeps a key from the
  model. A tool call names no run, so OQ-54 asks how the server binds
  a call to its run. @ASM-21 stays `@pending` as written.
- ASM-08 is re-planned: no self-hosted runner was available.
- OQ-06 is closed by the stakeholder on S4's findings: the deployment
  mode stays declared, and a `user` event of a harness session the
  harness does not mark attended stays untrusted (PRV-04). ASM-22
  records the mark, and @ASM-22 replays it.
- OQ-54 is closed by the stakeholder: the MCP server finds a tool call's
  run through the ingested transcript, logging each call and testing
  the edge cases (PRV-14, @PRV-14 `@pending`).

Deviations from the tasks:

- Task 1 registered §9.1's three hooks, not one, so every hook ParleyInn
  installs is shown firing.
- Task 2's runner image is a `~/.claude` baked with settings alone, as
  an image build would bake it, not a self-hosted runner.
- Task 3 could not run: S4 ran in Anthropic's cloud environment. ASM-08
  waits for a recording on a self-hosted runner.
- ASM-21 joined the spike from the delivery plan's S4 row.

Handed on:

- ASM-08: record a self-hosted runner before ADM-01 ships there.
- PRV-14 and PRV-04's attended mark: implemented with M1's MCP server
  and hook handlers, taking @PRV-14 and @PRV-04 off `@pending`.
- OQ-55: the stakeholder and a security review pick the channel for a
  run-seat key at launch, before SEC-10's run-seat keys ship. Then
  @ASM-21 can come off `@pending`.

## Acceptance Criteria

- [x] ASM-07 and ASM-08 marked verified or re-planned in the SRS
- [ ] Scenarios @ASM-07 and @ASM-08 pass against recorded install
  evidence: only @ASM-07 does; @ASM-08 and @ASM-21 stay `@pending`
  with their re-plans, as the deviations say
- [x] OQ-06 has an answer or a stated follow-up: closed into PRV-04
  and ASM-22
