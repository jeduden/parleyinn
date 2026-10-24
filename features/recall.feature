Feature: Recall (RCL)

  The scenarios for SRS §5.4, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @RCL-01 @P0 @I1 @pending
  Scenario: the MCP server's tool list includes the six P0 recall tools
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "short-run"
    When the agent lists the tools of the MCP server started by "parinn mcp"
    Then the tool list includes "event_search", "event_expand", "event_get", "landmark_list", "pin_list" and "stat_list", among the other tools SRS section 9.2 specifies
    And each tool declares the parameters specified in SRS section 9.2
    When the agent calls each of those six tools
    Then each returns its content in the envelope, "stat_list" its counts in an envelope marked structural
    And each call is recorded as a recall event
    When the agent calls "mcp_server_get", with a session binding held and with none
    Then it returns closed words and counts and no recalled content, whether or not a run is found for the call
    And the call is recorded as no recall event

  @RCL-02 @P0 @pending
  Scenario Outline: event search honours its filters and caps hits at k
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "mixed-provenance"
    When the agent calls the MCP tool "event_search" with <args>
    Then the recall result is wrapped in the envelope
    And the envelope contains at most <hits> items, each matching the filter, ranked by BM25 score
    And a relevant hit from a short run still ranks above the repeated hits of a long run

    Examples:
      | args                                 | hits |
      | query "deploy"                       | 10   |
      | query "deploy", k 50                 | 50   |
      | query "deploy", k 500                | 50   |
      | query "deploy", scope "rooms"        | 10   |
      | query "deploy", provenance ["web"]   | 10   |
      | query "deploy", kind ["tool_result"] | 10   |
      | query "deploy", trust "trusted"      | 10   |
      | query "deploy", since "2026-01-01"   | 10   |
      | query "deploy", until "2026-06-30"   | 10   |

  @RCL-03 @P0 @I1 @pending
  Scenario: event expand returns exact post-redaction content under the model-token cap
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "large-payloads"
    When the agent calls the MCP tool "event_expand" with range "A1:1-400"
    Then the recall result is wrapped in the envelope
    And the items carry the exact post-redaction content with payload references resolved
    And the envelope is at most 8,000 estimated model tokens with "truncated" true and a "next_cursor"
    And calling "event_expand" with that cursor returns the following events without gap or overlap

  @RCL-04 @P0 @I2 @pending
  Scenario Outline: every recall tool wraps its content in the envelope
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "mixed-provenance"
    When the agent calls the MCP tool "<tool>" with <args>
    Then the recall result is wrapped in the envelope
    And the envelope carries "parinn_envelope" 1 and the fixed envelope warning in its field "warning"

    Examples:
      | tool         | args                         |
      | event_search | query "deploy"               |
      | event_expand | range from "A1:1" to "A1:20" |
      | event_get    | address "A1:7"               |

  @RCL-05 @P0 @I8 @pending
  Scenario: recall defaults to the agent's current run and extending it to its rooms is explicit and logged
    Given an isolated ParleyInn home
    And a run with seats in its personal room and in room "L1", whose writers this node holds beside those of other runs in "L1" on two worktrees and of room "L2" where the run has no seat
    When the agent calls the MCP tool "event_search" with query "deploy" and no scope
    Then every hit belongs to the calling run, across the writers of both its seats
    And with scope "room" and room "L1" the hits come from every writer of "L1", and with scope "rooms" from every room the run has a seat in, and an audit entry logs each call whose recall scope extends past the run
    And no scope returns any hit from "L2"
    When the agent calls the MCP tool "event_get" with address "A2:5", an event of another run in "L1", and no scope
    Then the event is not returned, and the error says the address lies outside the current scope
    And with scope "room" and room "L1" the event is returned, and an audit entry logs the extended recall scope

  @RCL-06 @P0 @I5 @pending
  Scenario: quarantined events are never recalled and purged ranges return a tombstone
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "poisoned-web"
    And the person runs "parinn quarantine add --range A1:12-12"
    And the person runs "parinn purge --range A1:30-40"
    When the agent calls the MCP tool "event_expand" with range "A1:1-50"
    Then the recall result is wrapped in the envelope
    And no item has address A1·12 and no item lies in A1·30–40, which appears as a tombstone with reason "purged"

  @RCL-07 @P0 @I6 @pending
  Scenario: every recall call is appended to the record
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "short-run"
    When the agent calls the MCP tool "event_search" with query "deploy"
    Then the record gains one recall event with provenance "assistant" carrying the query
    And that event lists the addresses of the returned events
    And the counter "recall_calls" increases by 1

  @RCL-08 @P1 @I1 @pending
  Scenario Outline: every address ParleyInn shows, and its ASCII input form, resolves wherever an address is taken
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "short-run"
    And an event address shown to a person or an agent in <form> form
    When the agent calls the MCP tool "<tool>" with that address
    Then the recall result contains exactly the events the address names
    And an address of a purged or quarantined event resolves to its tombstone or quarantine marker
    And "parinn event expand" given the same address in the same form returns the same events
    And no two writers this node holds share a writer label, and each writer-label assignment is a structural event in the personal-room writer of the device seat that assigned it

    Examples:
      | form                                | tool         |
      | short                               | event_get    |
      | range                               | event_expand |
      | full                                | event_get    |
      | ASCII input, such as "A1:7"         | event_get    |
      | ASCII input range, such as "A1:3-9" | event_expand |

  @RCL-09 @P1 @I2 @I6 @pending
  Scenario Outline: every recalled item carries its writer, author, trust, origin and integrity status
    Given an isolated ParleyInn home
    And a node whose record contains <item>
    When the agent recalls that item with the MCP tool "event_get"
    Then the item carries its writer, its author and its trust level, or "untrusted" in its place for an item from a foreign room
    And the item carries origin "<origin>" and integrity status "<status>"

    Examples:
      | item                                                                   | origin    | status      |
      | a sealed event this node witnessed                                     | witnessed | verified    |
      | a sealed principal act this node's CLI recorded                        | witnessed | verified    |
      | an event past its writer's newest seal                                 | witnessed | unsigned    |
      | a sealed event "parinn ingest" read from a transcript                   | ingested  | verified    |
      | a sealed event "parinn ingest" read past an ingest marker               | ingested  | verified    |
      | a sealed event imported from a room bundle                             | bundle    | verified    |
      | a sealed event of a paired phone's device seat                         | peer      | verified    |
      | a peer's event after a break in its writer's chain                     | peer      | unverified  |
      | a peer's event whose chain check fails                                 | peer      | broken      |
      | a peer's event whose writer's earlier events are missing here          | peer      | incomplete  |
      | a peer's event at a seq its writer sealed twice with different content | peer      | equivocated |

  @RCL-10 @P2 @I2 @I8 @pending
  Scenario: a foreign room is recalled only by naming it in the call, enveloped, every item untrusted and tainting
    Given an isolated ParleyInn home
    And a node holding a foreign room "vendor-room" imported from a room bundle
    And the node's principal has recorded a trust grant covering the bundle's principal key
    And a foreign room "left-room" that every seat of the node's principal has left, holding a "constraint" pin the principal wrote there from its device seat
    When the agent calls the MCP tool "event_search" with query "deploy" and room "vendor-room"
    Then the hits come from "vendor-room", wrapped in the envelope, each marked "untrusted" though the trust grant covers the foreign room's keys
    And an audit entry logs the call and the calling run is tainted under SEC-13
    And a following call without the room parameter, under any scope, returns no hit from "vendor-room"
    And neither does the kernel built-in "parinn.event_search" in a "kernel_exec" call without the room parameter, under any scope
    And a "pin_list" call naming room "left-room" returns the principal's "constraint" pin marked "untrusted", though that pin still restores to the runs that had a seat in "left-room" and to their agents' later runs (PIN-10, REC-02)

  @RCL-11 @P2 @I6 @pending
  Scenario: recalling another seat's post is recorded and shown in the room view
    Given an isolated ParleyInn home
    And a room owned by "owner-a" whose conversation has a post written by the seat "bob"
    When an agent on this node calls the MCP tool "event_get" with the post's address
    Then the writer of the calling run's seat gains the recall event, listing the post's address among the returned events
    And the room view shows the recall, with the recall event's address, to "owner-a" and to the principal of the seat "bob"
