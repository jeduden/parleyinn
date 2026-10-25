Feature: Compute kernel (CMP)

  The scenarios for SRS §5.7, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @CMP-01 @P1 @pending
  Scenario: the kernel tools are listed by the MCP server
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a"
    When the MCP server "parinn mcp" is asked to list its tools
    Then the tool list contains "kernel_exec", "kernel_variable_list" and "kernel_reset"
    And each kernel tool runs SQL on a read-only connection to the store

  @CMP-02 @P1 @pending
  Scenario: kernel variables persist across executions until reset
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a"
    When the agent calls the MCP tool "kernel_exec" with code "CREATE TEMP TABLE x AS SELECT 41 AS v"
    And the agent calls the MCP tool "kernel_exec" with code "SELECT v + 1 FROM x"
    Then the result is one row holding 42
    When the agent calls the MCP tool "kernel_reset" with no arguments
    And the agent calls the MCP tool "kernel_variable_list" with no arguments
    Then the namespace lists no kernel variables

  @CMP-03 @P1 @pending
  Scenario: read-only table functions return rows
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a" containing the word "migration"
    When the agent calls the MCP tool "kernel_exec" with code "CREATE TEMP TABLE hits AS SELECT * FROM event_search('migration'); SELECT count(*), min(address) FROM hits"
    Then the result is one row holding a count and an address (writer, seq)
    And the table functions "event_expand", "event_get" and "landmark_list", the JSON, math and date functions and "regexp" and "regexp_capture" are callable
    And every table, view, function and statement kind the kernel allows, SQLite's own built-ins included, is on the kernel's allow-list
    And a function SQLite provides outside the allow-list is unavailable to kernel code
    And the record gains one recall event for each table-function call the execution made, and no event other than a recall event

  @CMP-04 @P1 @I4 @pending
  Scenario Outline: the kernel has no access to host resources
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a"
    When the agent calls the MCP tool "kernel_exec" with code "<code>"
    Then the tool returns an error naming "<resource>" as unavailable
    And no file, socket, process or environment variable was touched by the worker

    Examples:
      | code                                 | resource   |
      | ATTACH '/etc/passwd' AS p            | filesystem |
      | VACUUM INTO '/tmp/copy.db'           | filesystem |
      | SELECT load_extension('/tmp/x.so')   | extension  |
      | PRAGMA temp_store_directory = '/tmp' | pragma     |
      | SELECT date('now')                   | clock      |
      | SELECT random()                      | randomness |
      | DELETE FROM events                   | record     |

  @CMP-05 @P1 @I9 @pending
  Scenario Outline: exceeding a kernel limit restarts the worker and reports the lost namespace
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a"
    And the agent has set the kernel variable "kept" with "kernel_exec"
    And the kernel's step budget is <budget>
    When the agent calls the MCP tool "kernel_exec" with code "<code>"
    Then the tool returns an error stating the <bound> was exceeded and the namespace was lost
    And the "parinn kernel-worker" child process has been restarted
    And an audit entry records "kernel <bound> exceeded"

    Examples:
      | budget      | code                                                                                                                  | bound                 |
      | the default | WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x + 1 FROM c) SELECT count(*) FROM c                                | step budget           |
      | unlimited   | WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x + 1 FROM c) SELECT count(*) FROM c                                | 10 s wall-clock limit |
      | unlimited   | WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x + 1 FROM c) SELECT group_concat(printf('%.1048576c', 'x')) FROM c | 512 MiB memory limit  |

  @CMP-06 @P1 @I2 @pending
  Scenario: only the last row-returning statement's rows return to the agent, capped, enveloped and tainted
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a" containing an untrusted "web" event
    When the agent calls the MCP tool "kernel_exec" with code "CREATE TEMP TABLE r AS SELECT * FROM event_search('web'); WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 10000) SELECT r.* FROM r, n"
    Then the kernel output is wrapped in the envelope
    And the rows are capped at 8,000 estimated model tokens with a truncation marker stating that kernel variables persist
    And the envelope is tainted "untrusted" by the web event read during the execution
    And a later "kernel_exec" with code "SELECT * FROM r LIMIT 1", which reads no event, is tainted "untrusted" by that web event too
    And nothing other than the last row-returning statement's rows is returned

  @CMP-07 @P1 @I5 @I8 @pending
  Scenario: kernel reads honour quarantine and recall scope
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a"
    And an agent run with a Claude Code transcript "run-b"
    And the agent has bound A1·7 to the kernel variable "held" with "CREATE TEMP TABLE held AS SELECT * FROM event_get('A1:7')"
    And the person has quarantined A1·7 with "parinn quarantine add --range A1:7-7"
    When the agent calls the MCP tool "kernel_exec" with code "SELECT * FROM event_get('A1:7'); SELECT * FROM event_search('x', 'rooms')"
    Then the kernel returns exactly what the MCP tools "event_get" and "event_search" return for the same arguments
    And A1·7 and rooms the agent has no seat in are absent
    And the kernel states that the run's namespace was lost, and no kernel variable "held" remains
    And a purge of an event a kernel variable holds resets that namespace likewise

  @CMP-08 @P2 @I4 @pending
  Scenario: the opt-in Python kernel is confined, network-less and read-only
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a"
    And the external Python kernel is enabled in the person's configuration
    When the agent calls the MCP tool "kernel_exec" with code "import socket; socket.create_connection(('example.com', 80))"
    Then what the tool returns reports that the network is unavailable
    And a write to the store from the Python worker is rejected
    And the record is unchanged

  @CMP-09 @P2 @I4 @pending
  Scenario: parinn never calls model APIs itself
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a"
    When the shipped binary's imports and outbound network calls are inspected
    Then no model API client or model endpoint is referenced
    And parallel sub-calls over record data are only offered through harness subagents
