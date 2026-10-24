Feature: Boundary to durable memory (MEM)

  The scenarios for SRS §5.9, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @MEM-01 @P0 @I2
  Scenario: record content is never promoted into long-term memory
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "memory-canary"
    When every hook runs over the whole transcript and the person runs "parinn rebuild"
    Then no file outside the ParleyInn home's store contains the canary text
    And no memory, summary or cross-room store under the ParleyInn home or HOME contains the canary text

  @MEM-02 @P1 @I2 @pending
  Scenario: downstream memory consumes only the trusted export
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "mixed-provenance"
    When the person runs "parinn export --trusted-only"
    Then the command exits 0
    And the JSONL output contains only trusted events, each with full provenance
    And the documented integration path for downstream memory systems is that export alone
