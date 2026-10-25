Feature: Assumptions register (ASM)

  The scenarios for SRS §2.3, one per assumption, tagged with its id. The
  rows carry neither priority nor traces, so a scenario here carries
  its id alone. Each is a contract test (ENG-17) that replays hook
  inputs and transcripts recorded from Claude Code by the M0 spike
  named in the row's "Verified in" column, and is re-run against every
  supported Claude Code release. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the spike's
  recorded fixtures land.

  @ASM-01
  Scenario Outline: hook inputs carry the documented fields (S1)
    Given the hook inputs recorded from Claude Code "supported" in the "<flow>" flow
    When the "<Event>" hook inputs are read
    Then each carries "session_id", "transcript_path", "cwd" and "hook_event_name"
    And the last carries <extra>
    And the last one's "session_id" is <identity> the "startup" flow's

    Examples:
      | flow     | Event            | extra                                                  | identity     |
      | startup  | SessionStart     | "source" set to "startup"                              | the same as  |
      | resume   | SessionStart     | "source" set to "resume"                               | the same as  |
      | continue | SessionStart     | "source" set to "resume"                               | the same as  |
      | fork     | SessionStart     | "source" set to "fork"                                 | other than   |
      | clear    | SessionStart     | "source" set to "clear"                                | other than   |
      | compact  | SessionStart     | "source" set to "compact"                              | the same as  |
      | compact  | PreCompact       | "trigger" set to "manual" and "custom_instructions"    | the same as  |
      | compact  | PostCompact      | "trigger" set to "manual" and "compact_summary"        | the same as  |
      | resume   | UserPromptSubmit | "prompt"                                               | the same as  |

  @ASM-02
  Scenario Outline: additionalContext reaches the model from SessionStart, UserPromptSubmit and PostToolUse, and compaction hooks' output is echoed (S1)
    Given the transcripts recorded from Claude Code "supported" with every hook answering "additionalContext"
    When each line of the recordings is read
    Then the "<Event>" hook's additional context <context> as a hook context attachment
    And the "<Event>" hook's output <echo> in a compact command's output
    And on exit status 1 the "<Event>" hook's stderr shows <stderr>
    And on exit status 0 every hook's stderr shows only in its own hook attachment
    And a harness resume drops a "SessionStart" additional context equal to one since the last compaction, and records it again after one

    Examples:
      | Event            | context     | echo        | stderr                            |
      | SessionStart     | shows       | never shows | only in its own hook attachment   |
      | UserPromptSubmit | shows       | never shows | only in its own hook attachment   |
      | PostToolUse      | shows       | never shows | only in its own hook attachment   |
      | PreCompact       | never shows | shows       | in a compact command's output     |
      | PostCompact      | never shows | shows       | in a compact command's output     |

  @ASM-03
  Scenario: transcripts are one JSONL file per harness session with subagents nested (S3)
    Given a recorded "~/.claude/projects" tree for Claude Code "supported"
    When the recorded tree is walked
    Then each harness session yields one JSONL transcript under "~/.claude/projects/<transcript-dir>/"
    And each subagent transcript lies under "<session_id>/subagents/"
    And each subagent transcript has a ".meta.json" beside it naming its agent type

  @ASM-04
  Scenario Outline: a rewind or a partial compaction appends to the transcript and never rewrites it (S3)
    Given a recorded transcript before and after a "<flow>" flow for Claude Code "supported"
    When the two recordings are compared
    Then the earlier recording is a byte prefix of the later one
    And <shape>

    Examples:
      | flow               | shape                                                                       |
      | rewind             | the first line appended continues from a line before the earlier one's last |
      | partial compaction | a "compact_boundary" line is appended                                       |

  @ASM-05
  Scenario: the harness deletes transcripts after cleanupPeriodDays, but not on a schedule ParleyInn controls (S3)
    Given a recorded "~/.claude/projects" listing before and after harness starts with "cleanupPeriodDays" of 30 for Claude Code "supported"
    When the two trees are compared
    Then a transcript older than 30 days was deleted
    And a transcript older than 30 days was kept
    And no transcript younger than 30 days was deleted

  @ASM-06
  Scenario Outline: compaction hooks carry the main agent's identity whichever agent compacts, and name the compacting agent nowhere until they return (S1, S13)
    Given a recorded hook input for Claude Code "supported"
    And the hook input was captured during <case> compaction
    When the hook "<Event>" runs with the recorded hook input
    Then its "session_id" and "transcript_path" are the main agent's harness session's
    And it carries <extra>
    And it carries no subagent-specific field
    And the additional context of the "SessionStart" hook that follows it lands in the compacting agent's transcript, where the model reads it, and in no other
    And <timing>

    Examples:
      | case                                               | Event        | extra                     | timing                                                                                 |
      | a foreground subagent's                            | PreCompact   | "trigger" set to "auto"   | the compaction's boundary lies in the compacting agent's transcript                    |
      | a foreground subagent's                            | SessionStart | "source" set to "compact" | the compaction's boundary lies in the compacting agent's transcript                    |
      | a foreground subagent's                            | PostCompact  | "trigger" set to "auto"   | the compaction's boundary lies in the compacting agent's transcript                    |
      | a background subagent's                            | PreCompact   | "trigger" set to "auto"   | no transcript of the harness session held the compaction's boundary when its hooks ran |
      | a background subagent's                            | SessionStart | "source" set to "compact" | no transcript of the harness session held the compaction's boundary when its hooks ran |
      | a background subagent's                            | PostCompact  | "trigger" set to "auto"   | no transcript of the harness session held the compaction's boundary when its hooks ran |
      | the main agent's, while a background subagent ran, | PreCompact   | "trigger" set to "auto"   | no transcript of the harness session held the compaction's boundary when its hooks ran |
      | the main agent's, while a background subagent ran, | SessionStart | "source" set to "compact" | no transcript of the harness session held the compaction's boundary when its hooks ran |
      | the main agent's, while a background subagent ran, | PostCompact  | "trigger" set to "auto"   | no transcript of the harness session held the compaction's boundary when its hooks ran |

  @ASM-07
  Scenario Outline: a plugin bundling hooks and an MCP server loads on each target (S4)
    Given the stub ParleyInn plugin recorded for Claude Code "supported", bundling ParleyInn's hooks and a stub MCP server
    When it is installed on "<target>"
    Then the recorded harness session shows every hook the stub plugin registers firing
    And the recorded harness session lists the stub plugin's MCP tools

    Examples:
      | target           |
      | workstation      |
      | runner image     |
      | Agent SDK worker |

  @ASM-08 @pending
  Scenario: self-hosted runners seed ~/.claude into each harness session under one OS user (S4)
    Given a recorded self-hosted runner harness session for Claude Code "supported"
    When the harness session's HOME directory and the OS user of its process are inspected
    Then the harness session's "~/.claude/" was seeded from the runner host's "~/.claude/"
    And every harness session on the runner ran as the same one OS user

  @ASM-09
  Scenario Outline: a PreCompact hook handler exiting 2 blocks compaction, manual and automatic, up to the full context window (S6)
    Given a recorded hook input for Claude Code "supported"
    And a PreCompact hook whose handler exits 2, recorded in the "<recording>" recording
    When compaction is triggered <how>
    Then every "PreCompact" hook input of the recording carries "trigger" set to "<trigger>"
    And no transcript of the recording holds a "compact_boundary" line
    And no "PostCompact" hook fired, nor a "SessionStart" hook with "source" set to "compact"
    And the words the PreCompact handler wrote to stderr <reach>
    And <outcome>
    And the OQ-01 row of SRS §13 records what ASM-09's recordings showed

    Examples:
      | recording   | how                                          | trigger | reach                                                                             | outcome                                                                                                |
      | manual      | manually, by "/compact"                      | manual  | reached the model in the command's output, and the next model reply repeated them | the model replied to the next prompt without a compaction                                              |
      | full window | automatically, on to the full context window | auto    | reached no transcript                                                             | each prompt at the full context window was refused with "Prompt is too long", and no "Stop" hook fired |

  @ASM-10
  Scenario Outline: each hook's harness timeout is its own, and SessionEnd's defaults to about 1.5 s (S1)
    Given the hook timeouts measured on Claude Code "supported"
    When the "<Event>" handler sleeps past its harness timeout in <mode> mode with <timeout>
    Then the harness terminated the handler after about <limit> s
    And no "SessionEnd" handler started in a print-mode harness session whose "Stop" handler reached its harness timeout

    Examples:
      | Event            | mode        | timeout         | limit |
      | SessionStart     | print       | a 4 s timeout   | 4     |
      | UserPromptSubmit | print       | a 4 s timeout   | 4     |
      | PostToolUse      | print       | a 4 s timeout   | 4     |
      | Stop             | print       | a 4 s timeout   | 4     |
      | SessionEnd       | print       | a 4 s timeout   | 4     |
      | SessionEnd       | print       | a 10 s timeout  | 10    |
      | SessionEnd       | print       | no timeout      | 1.5   |
      | SessionEnd       | interactive | a 10 s timeout  | 10    |
      | SessionEnd       | interactive | no timeout      | 1.5   |

  @ASM-11
  Scenario: context the harness adds is written as attachment lines (S3)
    Given a recorded transcript for Claude Code "supported" from a worktree with a CLAUDE.md file, an MCP server and a skill
    When each line of the recording is read
    Then the CLAUDE.md file, the MCP server instructions and the skill listing appear as "attachment" lines
    And each of those lines carries "attachment.type" and "renderedRole"

  @ASM-12
  Scenario: one API message spans several lines, one content block each (S3)
    Given a recorded transcript for Claude Code "supported" with an assistant reply carrying two tool calls
    When each line of the recording is read
    Then each assistant line carries exactly one content block
    And the reply's lines share one "message.id" and repeat its "usage"

  @ASM-13
  Scenario: a tool-result line repeats the output and names its call (S3)
    Given a recorded transcript for Claude Code "supported" with a Bash tool call and its tool result
    When each line of the recording is read
    Then the tool-result line carries the output in "message.content" and again in "toolUseResult"
    And its "sourceToolAssistantUUID" names the line carrying the Bash tool call

  @ASM-14
  Scenario Outline: some transcript line types carry no uuid or no timestamp (S3)
    Given a recorded transcript for Claude Code "supported"
    When each line of the recording is read
    Then every line of type "<type>" carries <missing>

    Examples:
      | type                  | missing                        |
      | last-prompt           | neither "uuid" nor "timestamp" |
      | queue-operation       | no "uuid"                      |
      | ai-title              | neither "uuid" nor "timestamp" |
      | atis-latch            | neither "uuid" nor "timestamp" |
      | cost-state            | neither "uuid" nor "timestamp" |
      | mode                  | neither "uuid" nor "timestamp" |
      | permission-mode       | neither "uuid" nor "timestamp" |
      | file-history-snapshot | neither "uuid" nor "timestamp" |

  @ASM-15
  Scenario: thinking blocks keep their signature but not their text (S3)
    Given a recorded transcript for Claude Code "supported" with extended thinking enabled
    When each line of the recording is read
    Then every thinking block carries a signature and empty thinking text

  @ASM-16
  Scenario: how working directories map to transcript directories is recorded (S3)
    Given a recorded "~/.claude/projects" tree for Claude Code "supported" after one harness session in each of "/w/my_app", "/w/my.app" and "/w/my-app"
    When the recorded tree is walked
    Then the three harness sessions share the transcript directory "-w-my-app"

  @ASM-17 @pending
  Scenario: transcript lines are appended within 1 s of their event while an agent runs (S9)
    Given a recorded harness session from Claude Code "supported" with the time of each hook event
    When the recorded transcript is replayed with the write time of each line
    Then each transcript line was appended within 1 s of its event

  @ASM-18 @pending
  Scenario: the harness prompt stays answerable during PermissionRequest and accepts no decision (S9)
    Given a recorded hook input for Claude Code "supported"
    When the hook "PermissionRequest" runs with a handler that waits and then returns no decision
    Then the recorded harness kept its own permission prompt answerable while the handler waited
    And the recorded harness accepted the handler's exit without a decision and left the choice to its prompt

  @ASM-19 @pending
  Scenario Outline: each agent harness exposes inputs for steer, interrupt and stop (S10)
    Given the recorded input interface of "<harness>"
    When its inputs are listed
    Then it exposes an input for steer, one for interrupt and one for stop

    Examples:
      | harness          |
      | Agent SDK        |
      | ACP              |
      | Codex app-server |

  @ASM-20 @pending
  Scenario: a clone's root commit, HEAD, refs and trees are obtainable inside the core, with no socket and no program start (S11)
    Given a recorded git clone
    When ParleyInn reads the clone with process creation and the network forbidden
    Then it resolves the root commit, HEAD, every ref and the tree of every ref
    And no process was started and no connection was opened

  @ASM-21 @pending
  Scenario: the harness session's MCP server keeps its main run's and its subagents' run-seat keys outside the model's context (S4)
    Given a recorded plugin launch of Claude Code "supported" with its MCP server
    When the harness session's main run starts a subagent
    Then one MCP server serves both runs and keeps the run-seat keys of the main run and the subagent's run
    And each key reached it through a channel the harness keeps out of the model's context
    And no transcript line, hook input or tool result of either run carries a private key

  @ASM-22
  Scenario Outline: the harness puts its attended mark only in its hook handlers' environment, and only in its own terminal interface (S4)
    Given the stub ParleyInn plugin's recording "<recording>" for Claude Code "supported"
    When the environment its hook handlers and its MCP server saw is read
    Then every hook handler saw "CLAUDE_CODE_SESSION_ATTENDED" set to "<attended>" and "CLAUDE_CODE_ENTRYPOINT" set to "<entrypoint>"
    And the MCP server saw "CLAUDE_CODE_SESSION_ATTENDED" <server>

    Examples:
      | recording        | attended | entrypoint | server       |
      | interactive      | 1        | cli        | unset        |
      | workstation      | 0        | sdk-cli    | unset        |
      | Agent SDK worker | 0        | sdk-ts     | unset        |
      | forged-env       | 0        | sdk-cli    | set to "1"   |
