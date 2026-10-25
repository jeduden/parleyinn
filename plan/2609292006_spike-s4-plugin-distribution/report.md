---
title: "Spike S4 report"
summary: >-
  What Claude Code 2.1.296 and Agent SDK 0.3.296 do with a stub ParleyInn
  plugin on a workstation, a baked runner image, an Agent SDK worker and
  the interactive terminal: every hook fires, one MCP server serves every run of a harness
  session, a tool call names no run, and a key in the harness's
  environment reaches the model's tools.
---
# Spike S4 report: prove plugin distribution

## Method

A stub ParleyInn plugin, in the `record/plugin` kit, registers §9.1's three
hooks, `SessionStart`, `UserPromptSubmit` and `PermissionRequest`, and
one stub MCP server with one tool, `event_search`. Each hook handler
saves its hook input and what its process sees, and answers nothing.
The MCP server logs every message it reads. It binds each tool call to
a run by the transcript that holds the call's tool-use id, and mints a
stand-in run-seat key for each run, kept in its memory and its own log.
A key handed to it at launch in the environment, as `CAIRN_S4_RUN_KEY`,
it logs the same way.

`record.sh` runs Claude Code 2.1.296 under `env -i`, in a scratch `HOME`
and `CLAUDE_CONFIG_DIR` per target (ENG-14), so nothing of an enclosing
harness session reaches the recorded one. Each print-mode target got one
prompt: call the tool, start a foreground subagent that calls it, then
run a Bash command no permission rule allowed. Two more print-mode
recordings of the workstation install probe one question each:
`launch-key` starts the harness with a stand-in key in its
environment, as a hand-over at launch would, and asks the model to
print it; `forged-env` starts it with `CLAUDE_CODE_SESSION_ATTENDED=1`
and `CLAUDE_CODE_ENTRYPOINT=cli` set.

| Target           | How the plugin got there                                                                                              |
| ---------------- | --------------------------------------------------------------------------------------------------------------------- |
| workstation      | `claude plugin marketplace add`, then `claude plugin install`                                                         |
| runner image     | a `~/.claude` baked as an image build would: `settings.json` declares the marketplace and enables the plugin          |
| Agent SDK worker | Agent SDK 0.3.296 `query()` with `plugins: [{type: "local", path}]` and `settingSources: []`                          |
| interactive      | the workstation install, Claude Code's interactive terminal driven through `tmux`, for OQ-06; one prompt, no subagent |

## Findings

| Question                                               | Observed                                                                                                                                                                                                                                                                                                                                     |
| ------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Does the plugin load on each target?                   | Yes. All three hooks fired, and the init message listed `mcp__plugin_cairn_cairn__event_search` with the server `plugin:cairn:cairn` connected, on every target                                                                                                                                                                              |
| Does a baked `~/.claude` need an install command?      | No. `extraKnownMarketplaces` and `enabledPlugins` in `settings.json` loaded the plugin at the first headless start                                                                                                                                                                                                                           |
| Does `PermissionRequest` fire in print mode?           | Yes, for the Bash command no permission rule allowed, with `tool_name`, `tool_input` and `permission_suggestions`; with no decision the harness denied the call                                                                                                                                                                              |
| How many MCP server processes serve a harness session? | One. It served the main run's tool call and the subagent's                                                                                                                                                                                                                                                                                   |
| Does a tool call name its run?                         | No. `tools/call` carries only `_meta` `claudecode/toolUseId` and a `progressToken`; no session, run or agent id                                                                                                                                                                                                                              |
| Can the server find the run?                           | Yes, by the transcript holding the tool-use id: the main transcript or the subagent's under `subagents/`. The line reached it about 0.1 s after the call, once 0.45 s                                                                                                                                                                        |
| Did a minted key leave the server?                     | No. Neither key showed in any transcript line, hook input or tool result                                                                                                                                                                                                                                                                     |
| Does a key handed over at launch stay from the model?  | No. A key in the harness's environment reached the MCP server and every tool the model runs; asked to, the model printed it through the Bash tool, into its transcript (`launch-key`)                                                                                                                                                        |
| What tells the harness's entry point? (OQ-06)          | `CLAUDE_CODE_ENTRYPOINT`: `cli` in the interactive terminal, `sdk-cli` in print mode, `sdk-ts` in the Agent SDK, in every hook handler's and the MCP server's environment                                                                                                                                                                    |
| Is a person present? (OQ-06)                           | Not knowable. `CLAUDE_CODE_SESSION_ATTENDED` is `1` only in the interactive terminal, in hook handlers alone, but this spike's own script drove that terminal and read as attended                                                                                                                                                           |
| Can whatever starts the harness forge them?            | Not for a hook handler. Set to `1` and `cli` in the harness's environment, each hook handler of a print-mode harness session still saw `0` and `sdk-cli`. The MCP server saw `sdk-cli` too, but the harness gives it no `CLAUDE_CODE_SESSION_ATTENDED`, so it inherited the forged `1` (`forged-env`): only a hook handler can read presence |
| What else differs in the interactive terminal?         | Its hook inputs add `scratchpad_dir`, and `SessionStart` adds `model`                                                                                                                                                                                                                                                                        |
| Which OS user ran the hook handlers and the server?    | The OS user that started the harness: root in this cloud environment                                                                                                                                                                                                                                                                         |

## What it means for ParleyInn

- ADM-01 holds on each target: the plugin needs no `settings.json` edit
  by ParleyInn (I7). A runner image bakes it with settings alone.
- SEC-10's run-seat keys can stay in the one MCP server of a harness
  session, as long as that server mints them. A key handed over at
  launch in the environment is open to the model's tools, so ASM-21 is
  re-planned and OQ-55 asks for another channel, or none.
- Signing a room act with the calling run's seat key needs the call
  bound to its run first, and the harness does not name it. Finding
  the run through the ingested transcript waits for the tool-use line;
  OQ-54 put the options to the stakeholder, who chose that, with each
  call logged and the edge cases tested (PRV-14).
- The harness's entry point is reliable; a person's presence is not.
  The stakeholder closed OQ-06: `node.deployment_mode` stays declared,
  and a harness session the harness does not mark attended keeps its
  `user` events untrusted (PRV-04, ASM-22).
- ASM-08 needs a self-hosted runner. Anthropic's cloud environment, the
  only one S4 had, is not one.

## Fixtures

Under `testdata/claude-code/2.1.296/plugin`, one directory per target:
the hook inputs, numbered in firing order, each beside what its handler
saw (`.env`); the MCP server's log (`mcp.jsonl`); the transcripts; the
init message, trimmed to the plugin's view (`init.json`); and the
install output. The manifest names them under `plugin`. Scratch paths
are redacted as the `record` kit's `redact.sh` does, and the plugin's
source path reads `/opt/rec/plugin`.

To record again: `record/plugin/record.sh BASE SDK_DIR`, with the Agent
SDK installed under `SDK_DIR`, then `record/plugin/export.sh BASE
testdata/claude-code/<version>/plugin`.
