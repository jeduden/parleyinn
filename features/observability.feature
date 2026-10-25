Feature: Observability (OPS)

  The scenarios for SRS §5.10, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @OPS-01 @P0 @I6 @pending
  Scenario Outline: every degraded operation is audited and counted
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a"
    When <operation>
    Then an audit entry records "<expected>"
    And the counter "<counter>" increases by 1

    Examples:
      | operation                                                                                   | expected  | counter              |
      | the hook "UserPromptSubmit" runs with a malformed JSON object                               | rejected  | hook_rejected        |
      | the transcript contains an AWS secret key and is ingested                                   | redacted  | redactions           |
      | the hook "SessionStart" runs past its 150 ms hook budget                                    | timed-out | hook_budget_exceeded |
      | the agent calls the MCP tool "event_expand" with a range above 8,000 estimated model tokens | truncated | recall_truncated     |
      | the hook "SessionStart" runs with an unreadable transcript path                             | failed    | hook_failed          |

  @OPS-02 @P0 @I6
  Scenario: the audit log is an append-only hash-chained JSONL file that rotates with chain continuation
    Given an isolated ParleyInn home
    And an audit log that has reached its rotation size
    When the person runs "parinn counter ack" at a terminal
    Then the file "audit/audit-000002.jsonl" is created with mode 0600
    And its first entry's previous hash equals the hash of the last entry in "audit/audit-000001.jsonl"
    And every entry is RFC 8785 canonical JSON whose SHA-256 chain verifies with "parinn verify"

  @OPS-03 @P0 @I6
  Scenario: doctor fails while a failure counter has risen since the counters were last acknowledged
    Given an isolated ParleyInn home
    And the person has acknowledged the counters at a terminal with "parinn counter ack", a neutral principal act
    When the hook "UserPromptSubmit" runs with a malformed JSON object
    And the person runs "parinn doctor"
    Then the command exits 1
    When the agent's tool call runs "parinn counter ack" with its standard input and output not a terminal
    Then the command exits 1
    When the person runs "parinn doctor"
    Then the command exits 1
    When the person runs "parinn counter ack" at a terminal
    And the person runs "parinn doctor"
    Then the command exits 0

  @OPS-04 @P1 @I6 @pending
  Scenario: the canary writes through the hook path and recalls through MCP
    Given an isolated ParleyInn home
    And deployment mode "automation"
    When the person runs "parinn canary"
    Then the command exits 0
    And the canary event was written through capture's ingest path and recalled through the MCP server's tool "event_search"
    Given the MCP recall path is broken
    When the person runs "parinn canary"
    Then the command exits 1

  @OPS-05 @P1 @I2 @I4 @pending
  Scenario Outline: structured JSON logs go to a local file or stderr and nowhere else, and a hook handler logs only to the home's log file
    Given an isolated ParleyInn home
    And logging is configured to <target>
    When the person runs "parinn ingest --all"
    Then every line written to <target> is a JSON object with "time", "level" and "msg"
    And the process opened no network connection
    When the hook "SessionStart" runs with an unreadable transcript path
    Then the hook handler writes nothing to stderr and opens no network connection
    And the home's log file records the failure as a JSON object with "time", "level" and "msg"

    Examples:
      | target                    |
      | the file "logs/parinn.log" |
      | stderr                    |

  @OPS-06 @P1 @I6 @pending
  Scenario Outline: every failure of a component outside the core reaches the home's audit log and a named counter
    Given an isolated ParleyInn home
    And "<component>" is running
    When an operation of "<component>" is <expected>
    Then the home's audit log records the operation as "<expected>"
    And a named counter for it increases by 1
    And "parinn status" shows the failure outside any browser

    Examples:
      | component           | expected  |
      | room-view component | rejected  |
      | room-view component | coalesced |
      | launcher            | timed-out |
      | peer component      | dropped   |
      | publish component   | failed    |
      | bridge component    | failed    |
