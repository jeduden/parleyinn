Feature: Record (REC)

  The scenarios for SRS §5.1, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @REC-01 @P0 @I1
  Scenario Outline: harness transcripts are ingested from the configured transcript roots
    Given an isolated ParleyInn home
    And the person's configuration <roots>
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" lies under "<root>"
    When the person runs "parinn ingest --all"
    Then the command exits 0
    And every line of "main-run" is stored as an event of its run

    Examples:
      | roots                                             | root                 |
      | leaves transcript.roots unset                     | ~/.claude/projects   |
      | sets transcript.roots to ["~/runner-transcripts"] | ~/runner-transcripts |

  @REC-02 @P0 @I1 @pending
  Scenario: a subagent is ingested as its own run, tied to its parent's run by a parent link, a harness resume continues the run only when it keeps its harness session, and a harness fork starts a new one
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "parent-run"
    And a subagent transcript "explore-agent" of "parent-run" naming the agent "Explore"
    And "parent-run" continues after a harness resume that kept its harness session
    And a harness resume of "parent-run" that started a new harness session, with the transcript "new-harness-session-run"
    And a harness fork of "parent-run", with the transcript "forked-run"
    And a compaction of "parent-run" whose summary the harness wrote with an agent of its own
    When the person runs "parinn ingest --all"
    Then the run ingested from "explore-agent" is tied to the run of "parent-run" by a parent link, not a delegation link
    And the run ingested from "explore-agent" records the harness's agent id "Explore"
    And the events "parent-run" gained after the harness resume that kept its harness session belong to the run of "parent-run"
    And the events of "new-harness-session-run" belong to a new run, not to the run of "parent-run"
    And the events "forked-run" adds after the lines it repeats belong to a new run, not to the run of "parent-run"
    And the run count of the recorded Claude Code 2.1.296 harness session "subagent-compaction" is its main run plus its one subagent, none for the harness's own call that wrote each compaction summary
    And after a harness clear of "parent-run", its later events belong to a new run
    And that new run belongs to the agent of "parent-run" only when the harness gives an agent id stable across harness sessions, else it is its own agent
    And a later run of an agent reaches the agent's rooms through the "rooms" recall scope and its restore blocks, but holds no seat, add or role there until it joins under LANE-23

  @REC-03 @P0 @I1 @I10
  Scenario: re-ingesting a transcript source creates no duplicate events
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" has 40 lines and has been ingested, then grows by 10 lines
    When the person runs "parinn ingest --all"
    And the person runs "parinn ingest --all"
    Then the record contains exactly 50 events, one per line of "main-run"
    And the ingest position of "main-run" stores its byte length and the SHA-256 of its consumed prefix

  @REC-04 @P0 @I1 @I10 @pending
  Scenario: events are attributed to the run of their transcript, not the hook input, and a line a harness fork repeats is recorded once, in its owning run's writer, whichever transcript is ingested first
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "parent-run"
    And a subagent transcript "explore-agent" of "parent-run" naming the agent "Explore"
    When the hook "SessionStart" runs with the session_id of "parent-run" and the transcript_path of "explore-agent"
    And the person runs "parinn ingest --all"
    Then every event read from "explore-agent" is attributed to the subagent run of "explore-agent"
    And no event read from "explore-agent" is attributed to "parent-run"
    Given two fresh isolated ParleyInn homes "A" and "B"
    And a harness fork of "parent-run" with the transcript "forked-run", whose harness session id sorts before that of "parent-run", which repeats 40 lines of "parent-run" with their uuids, and whose "SessionStart" hook observation with source "fork" both homes recorded
    When home "A" ingests "parent-run", then "forked-run"
    And home "B" ingests "forked-run", then "parent-run"
    Then in both homes each of the 40 repeated lines is one event in the writer of the run of "parent-run", and the run of "forked-run" records a repeat reference to its address, never a copy
    And in home "B" the repeated lines of "forked-run" were held at an ingest marker until "parent-run" was ingested
    And the run membership of the two homes is identical, and each derived artifact is identical in content apart from each home's own writer ids
    Given two fresh isolated ParleyInn homes "C" and "D" with no "SessionStart" hook observation for either transcript
    When home "C" ingests "parent-run", then "forked-run"
    And home "D" ingests "forked-run", then "parent-run"
    Then in each home each of the 40 repeated lines is one event in the writer of the run that first recorded it there, "parent-run"'s in "C" and "forked-run"'s in "D", the other run recording a repeat reference, and a structural event records and audits that choice
    Given a fresh isolated ParleyInn home "E" where "parinn ingest" recorded "parent-run"
    When the MCP server of "forked-run" ingests it live
    Then the 40 repeated lines are events of their own in the writer of the run of "forked-run", never repeat references to the ingested events, each audited
    Given a fresh isolated ParleyInn home "F" on an injected clock, where no MCP server holds a session binding for "parent-run"
    When the MCP server of "forked-run" ingests its repeated lines live
    Then they go at once into the writer of the run of "forked-run", audited, with no hold
    Given a fresh isolated ParleyInn home "G" on an injected clock, where an MCP server holds the session binding for "parent-run" but has not ingested the repeated lines
    When the MCP server of "forked-run" ingests them live
    Then the hold is counted and shown by "parinn status"
    And once the clock passes 30 s the lines are released into the writer of the run of "forked-run" and audited
    Given a fresh isolated ParleyInn home "H" where the lines of "forked-run" are held for "parent-run"
    When the transcript of "parent-run" is deleted
    Then the held lines are released into the writer of the run of "forked-run" and audited (REC-08)
    Given a fresh isolated ParleyInn home "I" where two witnessed transcripts that repeat each other's lines were recorded by different recorders
    When both are ingested
    Then the repeated lines are events of their own in each run's writer, never repeat references, each audited
    Given a transcript "planted-run" in the transcript directory of "parent-run" that repeats the uuid of a line of "parent-run" with other text
    When the person runs "parinn ingest --all"
    Then that line of "planted-run" is recorded as its own event, the counter "repeat_conflicts" increases by 1 and an audit entry records the conflict
    Given the person purged the events of the 40 repeated lines from the run of "parent-run"
    When a harness fork of "parent-run" with the transcript "late-fork", repeating those 40 lines, is ingested
    Then none of those lines is recorded again, each refusal is counted and audited, and the record keeps only their uuids beside the tombstones

  @REC-05 @P0 @I1 @I6
  Scenario: unreadable transcript lines are kept as redacted unparsed events
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" contains a malformed JSON line and a line of unknown type "future_kind", each containing an API key
    When the person runs "parinn ingest --all"
    Then both lines are stored as events with provenance "unparsed" carrying the raw line with the API key redacted
    And no line of "main-run" is missing from the record
    And the counter "events_unparsed" increases by 2

  @REC-06 @P0 @I1 @I10 @pending
  Scenario: every event's address is its writer and a gap-free seq that writer alone assigns
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "main-run" whose writer has appended seq 1 to 20 and purged seq 5-9
    And an append transaction of that writer that rolled back
    When the person runs "parinn ingest --all" while the run's MCP server ingests "main-run" concurrently
    Then every event carries the address (writer, seq) of the writer that appended it
    And that writer's new events continue from seq 21 with no gap and no repeat, and no seq is reused
    And each seq was assigned by its writer inside the append transaction that wrote its event
    And a local index position is never shown or accepted as an address

  @REC-07 @P0 @I1 @I6
  Scenario Outline: a shrunk or rewritten transcript source starts a new transcript generation without losing events
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" has 40 lines and has been ingested
    When "main-run" is <change>
    And the person runs "parinn ingest --all"
    Then "main-run" starts transcript generation 2 and all 40 events of transcript generation 1 remain in the record
    And an audit entry records "new transcript generation of main-run"

    Examples:
      | change                                |
      | truncated to 10 lines                 |
      | rewritten with a different first line |

  @REC-08 @P0 @I1 @I6 @pending
  Scenario: the record outlives its transcript sources and verify reports transcript sources lost before full ingestion
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" has 40 lines and has been ingested
    And "cut-short" has 30 lines of which 20 were ingested before both transcript files were deleted
    When the person runs "parinn verify --json"
    Then the output names "cut-short", and only it, as a transcript source that disappeared before it was fully ingested
    And "parinn event expand" still returns all 40 events of "main-run"

  @REC-09 @P0 @I1
  Scenario: content above the payload threshold goes to a payload store under a name that reveals nothing off the node
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" contains a Bash tool result of 20 KiB and one of 2 KiB, against the default 8 KiB threshold
    When the person runs "parinn ingest --all"
    Then the 20 KiB content is stored in the payload store under a name from which no one off this node can confirm a guess at the content, and its event carries a preview of at most 512 bytes, the payload reference and the commitment
    And the 2 KiB content is stored inline in its event
    And nothing in an export lets a reader off this node confirm a guess at the content from a payload name
    And each payload was written to a temporary file, fsynced, then renamed into place

  @REC-10 @P0 @I10
  Scenario: each writer's events form their own hash chain over a header that carries no content
    Given an isolated ParleyInn home
    And a record in which two writers have each appended 20 events, each having seen the other's log, the second writer's clock running an hour behind
    When the person runs "parinn verify"
    Then the command exits 0
    And each event's hash is a collision-resistant hash of prev_hash, its writer's previous event, followed by the canonical encoding of its header
    And the header contains only the address, kind, provenance class, prev_hash, the heads of the other writers' logs the writer had seen, and the commitment
    And events of the two writers are ordered by the heads each cites, never by wall-clock time

  @REC-11 @P0
  Scenario: event and payload text is full-text indexed up to the indexing cap, with ranked search
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "main-run"
    And "main-run" contains a 2 MiB tool result with "quokka-early" in its first KiB and "quokka-late" beyond its first 1 MiB
    And "main-run" contains one "user" event naming "wombat" once and another naming it five times
    When the person runs "parinn ingest --all"
    And the agent calls the MCP tool "event_search" with query "quokka-early"
    Then the tool result event is a hit
    And a search for "quokka-late" returns no hit, because indexing stops at the default 1 MiB cap
    When the agent calls the MCP tool "event_search" with query "wombat"
    Then the hits are ranked by score, the "user" event naming "wombat" five times above the one naming it once

  @REC-12 @P0 @I10 @pending
  Scenario: events are appended in transcript order and seq alone defines order
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "main-run"
    And the timestamp of line 11 of "main-run" is earlier than that of line 10
    And line 12 of "main-run" carries no timestamp
    When the person runs "parinn ingest --all"
    Then the events of "main-run" have seq in the order of their transcript lines
    And each event whose line carries a timestamp keeps it as metadata
    And the event of line 12 carries no transcript timestamp
    And a second fresh home that ingests "main-run" under the same writer id derives the same chain head hash
    And "parinn event expand" returns line 10 before line 11

  @REC-13 @P0 @I1 @I9
  Scenario Outline: the harness session's MCP server ingests its transcripts as they grow, once bound to its harness session, and marks what it leaves when it stops
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "main-run" and a subagent transcript "explore-agent"
    And the "SessionStart" hook handler of the run's harness process recorded the session binding at line 12 of "main-run"
    And a neighbouring transcript "other-run" in the same transcript directory
    And a session binding another harness process's "SessionStart" hook handler recorded in the home for "other-run"
    When the harness session's MCP server starts before any session binding exists, and the harness appends 5 lines to "main-run"
    Then none of those lines is ingested as witnessed
    When the MCP server takes the session binding of its own harness process, ignoring the one for "other-run"
    And the harness appends 200 lines to "main-run" and 50 to "explore-agent"
    Then the events of the first 120 lines of "main-run" are in the record before the MCP server stops
    And nothing of "other-run" is ingested as witnessed
    When <stop>
    Then the events ingested are a prefix of each transcript, in its order
    And the event the "SessionStart" hook input made sits at line 12 of "main-run"
    And an ingest marker records the remainder of each transcript ingest did not reach
    And <resume> resumes from the ingest marker, and the record then holds every line once
    And <auditor> audits the stop under a named counter that is not a failure counter, shown by "parinn status"
    When after its session binding is checked, the subagent transcript "explore-agent" is replaced by a symlink to "~/.ssh/id_ed25519"
    Then the MCP server refuses to read it, audited, and opened no socket throughout

    Examples:
      | stop                                                             | resume                             | auditor                                   |
      | the MCP server stops after ingesting 120 lines of "main-run"     | the next MCP server for "main-run" | the MCP server                            |
      | the MCP server is killed after ingesting 120 lines of "main-run" | "parinn ingest --all"               | "parinn ingest --all", finding the marker, |

  @REC-14 @P1 @I1 @pending
  Scenario: Claude Agent SDK transcripts are ingested from configurable roots
    Given an isolated ParleyInn home
    And the person's configuration adds a transcript root containing a Claude Agent SDK transcript "sdk-run"
    When the person runs "parinn ingest --all"
    Then the command exits 0
    And every line of "sdk-run" is stored as an event of its run

  @REC-15 @P1 @I1 @I5 @pending
  Scenario: retention policies purge content per room and provenance class, leaving tombstones
    Given an isolated ParleyInn home
    And a home whose record contains "web" payloads and "user" events that are 40 days old
    And the person's configuration sets "retention_policy.*.web" to 30 days
    When the retention policy is applied
    Then the node purges the content of the "web" events, records each purge naming the policy "retention_policy.*.web", and leaves a tombstone, a structural event with provenance "structural", with reason "retention" in each purged range
    And the "user" events are kept and "parinn verify" exits 0
    And with no retention policy configured, applying the default policy removes nothing

  @REC-16 @P2 @I1 @pending
  Scenario: a Claude Managed Agents event history can be ingested
    Given an isolated ParleyInn home
    And an exported Claude Managed Agents event history "managed-run"
    When the person runs "parinn ingest --path managed-run"
    Then the command exits 0
    And every event of "managed-run" is stored with its run and a provenance class

  @REC-17 @P0 @I1 @I5 @I10 @pending
  Scenario: event content is reached only through a keyed commitment whose key is erased with it
    Given an isolated ParleyInn home
    And a home whose record contains 40 events, of which seq 10-14 have since been purged
    When the person runs "parinn verify"
    Then the command exits 0
    And every event carries a commitment to its canonical content under a random per-event key of at least 256 bits, stored with the content and absent from its chained header
    And the hash chain, every tombstone and every exported structure refer to event content, structural fields included, only through the commitment, but for the structural fields, provenance and recorder a tombstone keeps of a purged act, tombstone or event the key set holds
    And the commitment keys of seq 10-14 were erased with their content
    And no retained or exported value confirms a guess at the purged content

  @REC-18 @P1 @I6 @I10 @pending
  Scenario: writer seals are verified and events past the newest seal show as unsigned
    Given an isolated ParleyInn home
    And a writer whose log contains 30 events and is sealed at seq 20 with its seat key
    And a second writer whose seal at seq 10 has one altered byte
    When the person runs "parinn verify"
    Then the command exits 3 and names the second writer's seal at seq 10 as a mismatch
    And the first writer's seal, over its writer id, seq 20 and the chain head at seq 20, verifies against the seat key that made it
    And events 21 to 30 of the first writer are shown and recalled as "unsigned"
    And each seal refers to event content only through the commitments its chain head covers
    And merging the first writer's closed segments outside every hook budget, or rewriting them for a purge, keeps each of its seals verifiable
    And "parinn backup create" copies an open segment only up to a fresh seal

  @REC-19 @P1 @I1 @I6 @pending
  Scenario Outline: a witnessed run's MCP server seals each of its run seats' writers within 2 s of any unsealed append and when the run stops, and the open segment closes at every stop and every 30 s
    Given an isolated ParleyInn home
    And an active run with seats in its personal room and in room "L1", whose MCP server has ingested events into both run seats' writers for 31 s while it served calls, and whose "SessionStart" hook handler appended a hook observation to each run seat's writer
    And another writer whose chain ended without a closed segment
    And a paired phone whose device seat's writer this node holds
    And an ingested run whose run seat's writer this node holds
    When <moment>
    Then the run's MCP server sealed each of its run seats' writers with that seat's key within 2 s of any unsealed append, covering what the hook handlers had appended
    And it seals those writers again when the run stops, and events after the newest seal are shown as "unsigned"
    And what "parinn ingest" later appends for the run, whether or not its MCP server still runs, goes to one run seat and writer it starts beside each run seat it extends, in that seat's room, which a second "parinn ingest" extends without starting another, whose key the core keeps like a device seat's key, never in the MCP server, and whose writer the core seals, naming that run seat, taking over its role and sharing its add where it has one, so a kick or leave of either seat ends that one add for both, and the one beside the run's personal-room seat is a member as long as that seat is
    And once the device key that certified a run seat, or the key of the seat "parinn ingest" started beside it, is missing with the node identity unchanged, "parinn ingest" appends what remains of that run to such a seat in the personal room, certified by the device key minted anew by a seat certificate naming it an ingest seat, and stops counting the run
    And only a seat that the key which certified the run seat it names, or the key a "rotated" key event names in its place, certified by a seat certificate naming it an ingest seat beside that run seat shares that run seat's add and role; a seat another node's key certified naming that run seat, or a new run seat naming it after its run-seat key was lost, shares no add or role
    And a segment was closed once 30 s had passed though no turn had ended, and the open segment is closed at <moment>
    And a second run, whose "SessionStart" hook handler ran before any MCP server held its run-seat key, appended nothing, and its MCP server appended the hook observation from the ingest marker once it held that key
    And a third harness session, for whose "SessionStart" hook observation no MCP server took the session binding within 30 s, is counted and audited as a failure once, by whichever of its next hook handler, "parinn status" and "parinn ingest" finds it first
    And that failure's audit entry and "parinn status" each say whether any MCP server on this node has taken a session binding
    And "parinn verify" and "parinn status" each report the other writer's chain as ended without a closed segment
    And the paired phone sealed its device seat's writer with that seat's key after each append to it
    And the core sealed every writer but the witnessed run's run seats' and the paired phone's, the ingested run's run seat's writer and this node's device seat's writer included, after each append to it

    Examples:
      | moment                                                             |
      | a turn end the run's transcript records                            |
      | a subagent end the parent's transcript records                     |
      | the end of the harness session, when its MCP server's input closes |

  @REC-20 @P1 @I1 @pending
  Scenario Outline: worktree checkpoints record the commit, branch and redacted diff at every turn end, harness session end, hand-back and branch-head rewrite
    Given an isolated ParleyInn home
    And a run on a git worktree with a previous worktree checkpoint, a tracked change containing an API key, an untracked file and an ignored file
    When <moment>
    Then the record gains an event with provenance "file" carrying the checked-out commit id, the branch and a payload diff against the previous worktree checkpoint
    And the diff covers the tracked change and the untracked file but not the ignored file, with the API key redacted
    And a worktree checkpoint taken for a rewrite of the branch head keeps the replaced head
    And purging content a worktree checkpoint contains erases or tombstones that copy, and the purge receipt names it
    And ParleyInn obtained the worktree state and git data inside the core, with no socket opened and no program started, leaving any rest to an ingest marker
    And it read the worktree's files, "README.md" among them, only inside the run's worktree and its repository's git directory, and wrote nothing in the repository

    Examples:
      | moment                                        |
      | the run's transcript records a turn end       |
      | the run's harness session ends                |
      | the principal hands control back to the agent |
      | the branch head is rewritten by a rebase      |

  @REC-21 @P1 @I6 @I10 @pending
  Scenario: a segment of an unsupported format version is refused, audited, recorded once and left untouched
    Given an isolated ParleyInn home
    And a writer log containing a segment of a format version this node supports and a segment of format version 99
    When the person runs "parinn verify" twice
    Then every segment this node wrote carries a format version
    And the supported segment is read
    And the version 99 segment is refused each time, and an audit entry records each refusal with its unsupported format version
    And exactly one structural event in this node's device seat in its personal room records the refusal, appended when the segment was first refused, carrying only the writer id, the refused seq range, the segment's digest and the typed reason "unsupported format version", never the segment's content or free text
    And the second refusal of that writer for that reason is counted and audited, and appends no event
    When the person runs "parinn rebuild"
    Then the rebuild only reads the recorded refusal and appends no event
    And the version 99 segment file is byte-identical to before

  @REC-22 @P1 @I1 @I2 @pending
  Scenario: events from transcripts the hook handlers did not watch, or read past an ingest marker, are marked ingested and untrusted
    Given an isolated ParleyInn home
    And deployment mode "interactive"
    And a transcript "pre-install" under the configured transcript roots, written before ParleyInn's hook handlers were installed, containing a user turn and lifecycle metadata
    And a transcript "elsewhere" from outside the configured transcript roots
    And a watched transcript "cut-short" whose hook handler left an ingest marker
    When the person runs "parinn ingest --path" on "pre-install", on "elsewhere" and on "cut-short"
    Then every event of "pre-install" carries the "ingested" origin, freshness mark and trust mark with its transcript source and ingest position, shown on every surface, and none is shown as witnessed
    And the "user" event and the "harness_meta" event from "pre-install" have trust "untrusted"
    And this node records "elsewhere" as an ingested run in the principal's personal room, shown as "ingested", and every event of it has trust "untrusted"
    And every event "parinn ingest" read past the ingest marker of "cut-short" carries the "ingested" origin and has trust "untrusted"

  @REC-23 @P2 @I2 @I4 @I6 @pending
  Scenario Outline: room bundles are imported only from local files and fetched git refs, verified, redacted and audited
    Given an isolated ParleyInn home
    And this node holds the chain of a writer of a foreign room under the principal key "principal-b"
    And a room bundle in a local file that <bundle>, carrying a withheld event and an event with an API key the redaction rules of this node's principal match
    When the person imports the bundle
    Then the import is <expected> and an audit entry records it
    And an accepted bundle had its signature by the exporter's device key, chaining to the bundle's principal key, and its seals and chains verified as received, across the withheld event from its retained header, and the API key redacted with its event's commitment key erased, both verifications recorded
    And accepted events form a foreign room, which this node's principal neither owns nor has a seat in
    And a bundle named by a URL is refused without network access, while one at a git ref already fetched into a local clone is read

    Examples:
      | bundle                                                             | expected |
      | continues that writer chain under "principal-b"                    | accepted |
      | forks that writer chain under "principal-b"                        | refused  |
      | claims a room of this node's principal under "principal-b"         | refused  |
      | claims a writer of this node                                       | refused  |
      | is signed by a device key that does not chain to its principal key | refused  |

  @REC-24 @P1 @I1 @I8 @I10 @pending
  Scenario Outline: a home whose node identity changed mints a new device key and new seat keys, keeps the personal room's id and appends nothing under an old key
    Given an isolated ParleyInn home
    And a device key and a seat key bound to the node identity read, from outside the home, on the node the home was created on
    And a quarantine in force in the home
    And a configuration setting "node.deployment_mode" to "interactive", accepted by "parinn configuration accept"
    And <change>
    When the hook "SessionStart" runs and appends its first event after the start
    Then ParleyInn mints a new device key and a new seat key before that append, the new seat key starting a new seat and writer that names the old seat
    And the new seat inherits no add or role of the old seat, and, outside the personal room, joins a room only as any seat does, while a personal-room seat, a device seat's included, is a member from its first event
    And the personal room keeps its room id
    And every event recorded before the change counts on the node as another node's, received, each deriving origin "peer" or "bundle", never "witnessed", and is trusted only as PRV-02 trusts received events
    And that quarantine stays in force on the node until a release there
    And that configuration acceptance takes no effect on the node: the deployment mode stays "automation" until the person accepts the configuration there
    And an audit entry records the node identity change
    And a structural node identity change event in the new device seat in the personal room carries only the fingerprints of the replaced and new device key or token key, followed by the new key's inception, which starts a new key event log
    And the new seat's writer reads origin "peer" until a confirm signed by a managed-policy confirmation key names that inception, then "witnessed"
    And no event is appended under the old device key or the old seat key
    And an unavailable node identity value read again within the hook budgets and found unchanged counts as no change
    And the home keeps only a digest of the node identity value, never the value
    And an unavailable node identity value counts as one changed identity, so a node that never reads a value changes once and not at every later start

    Examples:
      | change                                                       |
      | the home is part of a cloned runner image on another machine |
      | the home is a copied volume mounted on another machine       |
      | the home is recovered from a snapshot on another machine     |
      | the node identity value is unavailable on every read        |
      | the home is part of a cloned runner image on macOS, whose hardware UUID differs |
      | the home is a copied volume on Windows, whose MachineGuid differs |

  @REC-25 @P1 @I1 @I10 @pending
  Scenario: closed segments are merged later, outside every hook budget, so a writer has at most 48 segments a day
    Given an isolated ParleyInn home
    And a writer that closed 200 segments during one day of activity
    When the deferred merge runs outside every hook budget
    Then the writer has at most 48 stored segments for that day
    And every address, commitment and chain head is unchanged
    And "parinn verify" exits 0

  @REC-26 @P0 @I1 @I2 @I6 @pending
  Scenario Outline: a transcript line or hook input whose shape differs from its harness version's is kept, counted and audited as format drift
    Given an isolated ParleyInn home
    And deployment mode "interactive"
    And the shapes recorded for the supported Claude Code versions
    And an agent run with a Claude Code transcript "main-run", served by its harness session's MCP server
    When <drift>, carrying an AWS access key
    Then <kept>
    And the stored text contains no AWS access key
    And the counter "<counter>" increases by 1 and the audit log names the harness version and what differed
    And "parinn status" shows the format drift
    And ingest goes on with the next line of "main-run"
    And regenerating the shapes from "testdata/claude-code" reproduces the recorded ones byte for byte, and a drift case fails when they differ

    Examples:
      | drift                                                                                                            | kept                                                                                                                     | counter      |
      | the harness appends a line of a type no supported version records                                                | the line is stored as an "unparsed" event, counted as format drift too, and the counter "events_unparsed" increases by 1 | format_drift |
      | the harness appends an "assistant" line without the "message" key                                                | the line is stored whole as an untrusted "harness_text" event                                                            | format_drift |
      | the harness appends a "user" line typed at the input, matching a prompt, whose "version" is no supported version | the line is stored whole as an untrusted "harness_text" event, never "user"                                              | format_drift |
      | the harness appends a "user" line typed at the input, matching a prompt, with a key no recorded shape has        | the line is stored whole as an untrusted "harness_text" event, never "user"                                              | format_drift |
      | the hook "SessionStart" runs with a hook input that lacks "source"                                               | the hook handler gives the empty answer and exits 0                                                                      | format_drift |
