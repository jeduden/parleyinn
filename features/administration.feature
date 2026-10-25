Feature: Administration and lifecycle (ADM)

  The scenarios for SRS §5.8, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @ADM-01 @P0 @I7 @pending
  Scenario: the plugin registers the hooks and the MCP server with the core executable
    Given an isolated ParleyInn home
    When the Claude Code plugin bundle is built
    Then the plugin manifest registers every hook of the hook contract as "parinn hook <hook>"
    And the plugin manifest registers the MCP server "parinn mcp"
    And the bundle carries the parinn core executable for each supported platform

  @ADM-02 @P0 @I7 @I6 @pending
  Scenario: install asks before every change, and uninstall needs a terminal, changes nothing when turning capture off is refused, records turning capture off before it removes the hook registrations, and lists, offers and audits every artifact
    Given an isolated ParleyInn home
    And a Claude Code settings file with unrelated user entries
    And "parinn install --scope user" showed a diff of every harness configuration change, was declined and left the settings file unchanged
    And "parinn install --scope user --yes" has run and every ParleyInn component has created its artifacts
    And at ParleyInn's first start after that install, the principal's device seat recorded the create room act of the node's personal room as its first act, that seat's add there
    And "parinn uninstall" run without a terminal refused and changed nothing
    And "parinn uninstall" run at a terminal while a run on the node left a residual risk open that no risk acceptance covered changed nothing, named the open residual risks and the runs that left them open, recorded an audit entry of the refusal and pointed to the harness's own plugin removal
    And "parinn uninstall" run at a terminal while managed policy required an authenticator for widening acts changed nothing, named that policy as the reason, recorded an audit entry of the refusal and pointed to the harness's own plugin removal, and that policy no longer applies
    And every run on the node now blocks the residual risks
    When the person runs "parinn uninstall" at a terminal and keeps only the device key
    Then before it removed the hook registrations, it recorded the widening principal act turning capture off
    And the output lists the hook, plugin and MCP registrations, seat and device keys and every other file ParleyInn wrote outside the store, each with an offer to remove it, and offers purge as the only removal of the store's content
    And the settings file is byte-identical to the one before install
    And an audit entry names the device key as left in place

  @ADM-03 @P0 @I7 @pending
  Scenario: the harness's managed settings are detected and never written, and the managed install path creates the personal room
    Given an isolated ParleyInn home
    And a file of the harness's managed settings that registers ParleyInn's hook handlers
    When an agent runs, so that ParleyInn first starts through those hook handlers
    Then the node's personal room exists, its create room act the first act of the principal's device seat, recorded at ParleyInn's first start
    When the person runs "parinn install --scope user --yes"
    Then the command exits 1
    And that file of the harness's managed settings is byte-identical to before
    And the output states that the harness's managed settings are in force and names the documented managed install path

  @ADM-04 @P0 @I6 @I7
  Scenario Outline: every settings layer is validated strictly, managed policy overrides the others but never turns a component on, repository configuration only tightens, and a settings change that needs a widening principal act waits for one
    Given an isolated ParleyInn home
    And <settings>
    When an agent runs and the person starts "<component>"
    Then "<component>" <expected>
    And the core <recording>

    Examples:
      | settings                                                                            | component    | expected                                                                                                                                                                    | recording                                                                                                   |
      | the person's config.toml containing "recal.max_k = 10"                              | parinn status | exits 2, and the error names the key "recal.max_k" and the problem "unknown key"                                                                                            | refuses the run's ingest, audits and counts it in "ingest_refused_settings_invalid", and records none of it |
      | the person's config.toml containing "recall.max_k = 'ten'"                          | parinn status | exits 2, and the error names the key "recall.max_k" and the problem "type error"                                                                                            | refuses the run's ingest, audits and counts it in "ingest_refused_settings_invalid", and records none of it |
      | the person's config.toml containing "recall.max_k = 500"                            | parinn status | exits 2, and the error names the key "recall.max_k" and the problem "out of range"                                                                                          | refuses the run's ingest, audits and counts it in "ingest_refused_settings_invalid", and records none of it |
      | the person's config.toml containing "payload.threshold_bytes = -1"                  | parinn status | exits 2, and the error names the key "payload.threshold_bytes" and the problem "out of range"                                                                               | refuses the run's ingest, audits and counts it in "ingest_refused_settings_invalid", and records none of it |
      | a repository ".parinn.toml" containing "recall.max_k = 500"                          | parinn status | exits 2, and the error names the key "recall.max_k" and the problem "out of range"                                                                                          | refuses the run's ingest, audits and counts it in "ingest_refused_settings_invalid", and records none of it |
      | the person's config.toml containing "flag.enabled = false"                          | parinn status | exits 0, shows deployment mode "automation" and "flag.enabled" awaiting acceptance: it stands as it stood until a widening principal act records that file's digest         | records every event of the run                                                                              |
      | the person's config.toml containing "node.deployment_mode = 'interactive'"          | parinn status | exits 0, shows deployment mode "automation" and "node.deployment_mode" awaiting acceptance: it stands as it stood until a widening principal act records that file's digest | records every event of the run                                                                              |
      | a repository ".parinn.toml" containing "node.deployment_mode = 'interactive'"        | parinn status | exits 0, shows deployment mode "automation" and counts "repository_configuration_key_ignored": repository configuration only tightens                                       | records every event of the run                                                                              |
      | a managed policy file the person's OS user can write, containing "recall.max_k = 5" | parinn status | exits 0, shows managed policy refused as writable by the person's OS user and counts "managed_policy_invalid"                                                               | records every event of the run                                                                              |

  @ADM-05 @P0 @I1 @pending
  Scenario: segment and schema migrations run forward after a verified backup and newer versions are refused
    Given an isolated ParleyInn home
    And a home whose segments and derived artifacts are at the previous version
    And a segment whose format version is newer than this node supports
    When the person runs "parinn migrate"
    Then a verified backup with its audit chain exists from before the migration
    And the segments and derived artifacts report the current version
    And the newer segment is refused, named in the output and left unchanged
    Given derived artifacts whose schema version is newer than this node supports
    When the person runs "parinn status"
    Then the command exits 3
    And the derived artifacts are unchanged

  @ADM-06 @P0 @I1 @I6 @pending
  Scenario: a backup restore keeps every later removal and never reuses a writer's log
    Given an isolated ParleyInn home
    And a home with closed segments, an open segment, payloads, derived artifacts and an audit log
    And a quarantine in force in the home
    And a configuration setting "node.deployment_mode" to "interactive", accepted by "parinn configuration accept"
    And a backup taken by "parinn backup create", followed by a purge of run "run-a" and the unpin of a pin
    When the person runs "parinn backup restore" as a widening principal act
    Then the backup contained every segment, the open segment, the payload store, derived artifacts and the audit log with its chain, no seat, device, token, recovery or at-rest key, and an audit entry recorded it
    And "parinn verify" passed on the copy and its audit chain before anything was reinstated
    And before it was confirmed, the backup restore named each pin of the copy that will not restore once its events count as received
    And every event the copy holds counts on the node as received, each deriving origin "peer" or "bundle", never "witnessed", and is trusted only as PRV-02 trusts received events
    And that quarantine stays in force on the node until a release there
    And the copy's configuration acceptance takes no effect on the node: the deployment mode stays "automation" until the person accepts the configuration there
    And a backup another node of the principal made, restored the same way, gives no run of this node a seat from its first event in that node's personal room
    And every event and payload outside run "run-a" recalled before the backup is recalled identically
    And run "run-a" stays purged and the unpinned pin stays unpinned
    And each local seat the copy contains is followed by a new seat and writer under a newly minted seat key, which names the old seat and inherits no add or role of it, audited, and no reinstated writer's log gains an event or reuses a seq
    And no derived artifact the copy contained was reinstated: each was rebuilt from the writer logs the node held and its key set
    And "parinn verify" exits 0 on the home after the backup restore
    And the backup restore recorded a "minted anew" its principal act names, whose writers read origin "peer" until it counts as PRV-12 says, as when a confirm signed by a managed-policy confirmation key names it, and a later "parinn rebuild" derives the same current key event log (PRV-12)

  @ADM-07 @P0 @I1 @I5 @pending
  Scenario Outline: purge removes a scope everywhere, erases its commitment keys and leaves a tombstone
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a" in room "room-a"
    And a range received from a peer, with its arrival event, a node identity change, and the key events it caused, all in this node's personal room
    When the person runs "parinn purge <scope>"
    Then the command exits 0
    And the purged events are gone from the segments, events, the search index, derived artifacts, the payload store and every copy of their content
    And the commitment key and payload reference of every purged event are erased
    And a tombstone event per purged range carries only addresses, counts, reason and commitments, and the structural fields, provenance and recorder of each purged act, tombstone and event the key set holds, never their text
    And "parinn rebuild" then derives room state, the key set and the quarantine set as before the purge, but for the purged content
    And the store is vacuumed and "parinn verify" exits 0
    And once the purge reports the range erased, with encryption at rest on and off, no file in the home or the storage engine's temporary directory, read decrypted, contains a canary token only the purged events carried, their commitment keys or a search index entry naming a purged address
    And while another process holds a read transaction on the store, the purge does not report the range erased and audits and counts each residue; once that transaction closes, the purge reports the range erased and the residue check passes
    And the audit log records the purge with its scope, ranges, reason and the principal act behind it, never the purged content
    And no arrival event, node identity change event, key event or certificate record is purged, whatever the scope, and the origin of every remaining event derives as before

    Examples:
      | scope                  |
      | --room room-a          |
      | --run run-a            |
      | --writer writer-a      |
      | --author s-1           |
      | --range writer-a:10-20 |
      | --before 2026-01-01    |
      | --provenance web       |
      | --provenance operator  |
      | --provenance structural |
      | --room personal-room   |

  @ADM-08 @P0 @I10 @pending
  Scenario: rebuild regenerates derived artifacts identical in content whatever order the logs arrived in
    Given an isolated ParleyInn home
    And a second isolated ParleyInn home with the same key set, the two nodes holding the same writer logs with pins, quarantine and landmarks, received in opposite orders
    And a configuration acceptance, a withdrawn risk acceptance and a released quarantine, each recorded after the events they affect were indexed
    When the person runs "parinn rebuild" in each home
    Then the command exits 0 in each home
    And "parinn verify" confirms every derived artifact is identical in content to its state before rebuild, by a canonical digest of that content
    And the derived artifacts of the two homes are identical in content, apart from what each home's own arrival events derive
    And the restore block, recall results, the landmark list and an export are byte-identical across the two homes
    And no canonical digest appears in the audit log, a log, a receipt, a backup or an export, and none is kept once "parinn verify" completes
    When a purge covers events a kept canonical digest covers, between "parinn rebuild" and "parinn verify"
    Then "parinn verify" names that purge and does not report the comparison passed
    When SQLite's "secure_delete" is turned off in the store
    Then "parinn verify" fails

  @ADM-09 @P0 @I6 @I10 @pending
  Scenario Outline: verify exits non-zero on any integrity failure
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a"
    And <damage>
    When the person runs "parinn verify"
    Then the command exits 3
    And the output names the failed check "<check>"

    Examples:
      | damage                                                          | check                          |
      | the transcript has lines not in the record                      | transcript source completeness |
      | an event row's hash has been altered                            | hash chain                     |
      | a payload file's bytes have been altered                        | payload integrity              |
      | a landmark row has been altered                                 | derived artifact consistency   |
      | a trust level has been altered in place                         | derived artifact consistency   |
      | a schema object ParleyInn does not ship has been added to the store | derived artifact consistency   |
      | the search index's ranking configuration has been altered       | derived artifact consistency   |
      | a search index entry names an event whose text lacks the term   | derived artifact consistency   |

  @ADM-10 @P0 @I7 @pending
  Scenario: doctor is read-only and doctor --fix shows each change first
    Given an isolated ParleyInn home
    And a ParleyInn home whose store file has mode 0644
    When the person runs "parinn doctor"
    Then the store file still has mode 0644
    When the person runs "parinn doctor --fix"
    Then the change "chmod 0600" is shown before it is applied
    And the store file has mode 0600

  @ADM-11 @P0 @I6 @pending
  Scenario: status shows locations, sizes, schema, deployment mode and failure counters
    Given an isolated ParleyInn home
    And deployment mode "automation"
    And an agent run with a Claude Code transcript "run-a"
    And the counter "hook_budget_exceeded" is 2
    When the person runs "parinn status --json"
    Then the command exits 0
    And the output shows the store location, its size, the schema version and the deployment mode "automation"
    And the output shows the counter "hook_budget_exceeded" with value 2
    And the output says, for this OS, whether node identity is checked
    And the output says that a second event at one (writer, seq) is refused for the segments the node holds, that the "equivocated" mark waits for seals and that the check between peers arrives with the peer network

  @ADM-12 @P1 @I2 @pending
  Scenario: export writes only trusted events with provenance as JSONL
    Given an isolated ParleyInn home
    And an agent run with a Claude Code transcript "run-a" with user turns and web tool results
    And one of its trusted events is quarantined
    And the person took a cut principal act carrying a stated reason at a terminal
    When the person runs "parinn export --trusted-only"
    Then the command exits 0
    And every exported JSONL line is a trusted event carrying its address (writer, seq), provenance and trust
    And no untrusted event appears in the export
    And the quarantined event does not appear in it, and the export's review step shows it as withheld
    And the cut act's line carries no stated reason, which the review step shows as withheld

  @ADM-13 @P0 @I7 @pending
  Scenario: ParleyInn writes to no git repository beyond the confirmed settings file, the commit hook of LANE-28 and the launcher's fresh checkouts
    Given an isolated ParleyInn home
    And a git repository with a worktree, refs, notes, git configuration and git hooks
    When an agent runs, the person confirms "parinn install --scope project" and every ParleyInn component runs
    Then the only changed file in the worktree is the harness settings file that install wrote
    And no ref is new or changed, and no new object was written
    And no git hook is new or changed but the commit hook for room trailers that a confirmed install sets up
    And the repository's notes, git configuration and other git hooks are byte-identical to before

  @ADM-14 @P1 @I1 @I5 @pending
  Scenario Outline: purge by seat or principal removes one principal's data with an audit trail on every node
    Given an isolated ParleyInn home
    And a node that holds the events that the seat "s-alice" of the principal "alice" wrote
    When the person runs "parinn purge <scope>" on the node
    Then the node holds no event of the selected scope
    And its audit log records the purge with its scope and ranges

    Examples:
      | scope             |
      | --seat s-alice    |
      | --principal alice |

  @ADM-15 @P1 @I6 @I9 @I1 @pending
  Scenario Outline: quotas refuse and audit what is received over them and never drop an accepted event
    Given an isolated ParleyInn home
    And managed policy sets every quota
    And the <quota> quota is reached
    When <arrival>
    Then <expected>
    And every event accepted before is still stored and recallable
    And each refusal over a quota increases the counter "quota_rejected" by 1
    And where this node's own recording crossed the quota, that first crossing at the quota's limit is a structural event in its device seat's personal-room writer, from which the Needs you item derives, and a rebuild appends none

    Examples:
      | quota               | arrival                                                                             | expected                                                                                  |
      | received writer     | a peer offers another event of that writer                                          | the event is refused and an audit entry records it                                        |
      | peer                | the peer offers another segment                                                     | the segment is refused, and a structural event and an audit entry record the refusal      |
      | worktree checkpoint | the run's transcript records a turn end and another worktree checkpoint is recorded | the worktree checkpoint is recorded, a failure counter rises and a Needs you item appears |
      | node                | the run appends another event                                                       | the event is recorded, a failure counter rises and a Needs you item appears               |

  @ADM-16 @P1 @I6 @pending
  Scenario Outline: status and doctor report the launcher and every other component outside the core, peer lag, open chains and boundaries
    Given an isolated ParleyInn home
    And managed policy that permits the room-view, peer and publish components, disables the launcher and one boundary
    And the room-view component is running, the bridge component has failed twice, a peer lags behind one writer and a writer chain ended without a closed segment
    When the person runs "parinn <command>"
    Then for the room-view component, the launcher, the peer component, the publish component and the bridge component the output shows whether managed policy permits it, whether it runs and its failure counters
    And the output shows the peer's sync lag for each writer
    And the output names the writer chain that ended without a closed segment
    And the output shows each boundary's state and whether managed policy disables it

    Examples:
      | command |
      | status  |
      | doctor  |

  @ADM-17 @P1 @I2 @pending
  Scenario Outline: the hook commands ParleyInn installs carry no secret, home path or record content
    Given an isolated ParleyInn home whose path lies under the OS user's home directory
    And a record holding events and pins
    When ParleyInn is installed by <path>
    Then each hook command and the MCP server command it registers, with their arguments and environment, names the core executable through the plugin's root variable or on "PATH"
    And none of them spells a path under the OS user's home directory, the ParleyInn home's included
    And none of them carries a secret, a key, a token or any text from the record

    Examples:
      | path                          |
      | running "parinn install --yes" |
      | the Claude Code plugin        |
      | the managed install path      |
