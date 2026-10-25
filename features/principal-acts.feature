Feature: Principal acts (OWN)

  The scenarios for SRS §5.13, one per requirement, tagged with its id,
  priority and traced invariants. A scenario still tagged @pending is
  declared but not yet written: its steps are bound when the plan that
  implements the requirement lands.

  @OWN-01 @P1 @I2
  Scenario: only the run's principal can instruct it
    Given an isolated ParleyInn home
    And an agent's run started on "alice"'s node
    Then the run's one principal is "alice", the principal of the node that records it

  @OWN-02 @P1 @I2 @I6 @pending
  Scenario Outline: a principal act is recorded only from an authenticated principal surface
    Given an isolated ParleyInn home
    And an agent's active run
    When a principal act arrives from <channel>
    Then the act is "<expected>"
    And a widening act whose required presence proof does not verify takes effect in no derived artifact, and "parinn verify" reports it

    Examples:
      | channel                                   | expected                                                                                                                                                                                                                                         |
      | the browser room view under SEC-20        | recorded as an operator event on the signing node's device seat in the room it acts on, or in the personal room naming the room where its principal has no member seat there, covered by its writer's seal and marked with its principal surface |
      | the CLI or TUI at a terminal under OWN-12 | recorded as an operator event on the signing node's device seat in the room it acts on, or in the personal room naming the room where its principal has no member seat there, covered by its writer's seal and marked with its principal surface |
      | a paired phone within its scope           | signed with the phone's device key and recorded on its device seat in the personal room, naming the room it acts on, which shows it by address, whose writer the phone seals with that seat's key and the node only holds                        |
      | the harness's own prompt                  | recorded as a user event or a harness_meta event recording only that input arrived, not a principal act                                                                                                                                          |
      | the terminal the launcher hosts           | recorded as a user event or a harness_meta event recording only that input arrived, not a principal act                                                                                                                                          |
      | anywhere else                             | refused and audited                                                                                                                                                                                                                              |

  @OWN-03 @P1 @I2 @pending
  Scenario: principal-typed text reaches an agent only through the harness's input interface
    Given an isolated ParleyInn home
    And an active run of an agent of "alice", whose harness offers an input interface
    And an untrusted event in the record
    When "alice" sends a steer naming that event's id
    Then the steer reaches the agent through the harness's own input interface
    And no output of a hook handler carries the steer
    And the text sent is principal-typed text or a fixed template its requirement names
    And no field of the untrusted event is embedded in it
    And purging content the steer's text carries erases or tombstones that copy, and the purge receipt names it

  @OWN-04 @P1 @I2 @pending
  Scenario: hook output carries only a computed permission decision with a templated reason
    Given an isolated ParleyInn home
    And rule levels and permission grants recorded as principal acts, covering an action
    When a hook reports a permission request for that action
    Then the hook output carries a decision of allow, ask, deny or defer, computed from the recorded rule levels and permission grants
    And its reason comes from the fixed template set, references ids and is built as TrustedText
    And no untrusted event, nor an event "parinn verify" reports, shapes a rule level or a permission grant
    And the hook output carries no principal-typed text and no field of the record
    And with hook permission decisions disabled by managed policy, the hook makes no permission decision

  @OWN-05 @P1 @I6 @I10 @pending
  Scenario: the first valid answer to a held request wins and conflicts resolve to deny
    Given an isolated ParleyInn home
    And a permission request recorded as a held request with a stable id bound to the harness's own id for it
    When two principal surfaces answer it, one after the other and then concurrently with conflicting answers
    Then the first valid answer in causal order wins
    And the later answer is recorded as superseded
    And the concurrent conflicting answers resolve to deny
    And the conflict is recorded and shown

  @OWN-06 @P1 @I9 @I6 @pending
  Scenario Outline: a permission request is held only while an answer stays possible and never allowed on failure
    Given an isolated ParleyInn home
    And a permission request from an agent
    And <situation>
    When the permission request is raised
    Then <expected>

    Examples:
      | situation                                                                | expected                                                               |
      | no away policy is on and its hold window ends                            | it is not denied when its hold window ends                             |
      | no principal surface is connected and the harness prompt is unanswerable | it is only mirrored, not kept waiting                                  |
      | the harness adapter cannot keep the harness's own prompt answerable      | it is only mirrored and the answer is left to the harness              |
      | ParleyInn fails while the permission request is held                         | the harness falls back to its own prompt and the action is not allowed |
      | managed policy disables keeping held requests waiting                    | it is recorded and only mirrored, and the agent is not kept waiting    |

  @OWN-07 @P1 @I2 @I9 @pending
  Scenario Outline: an away policy only answers the agent's own held request
    Given an isolated ParleyInn home
    And "alice" turned on the away policy "<policy>" for the room as a principal act
    And her agent's permission request is held past the hold window
    When the hold window ends
    Then the agent's held request gets the answer "<answer>"
    And no turn is started or resumed
    And a later allow reaches only the requesting run as the fixed template "Held request {id} was allowed by your principal" with a permission grant for the identical action
    And the held-request id resolves through "event_get" to the held request's recorded event
    And with away policies disabled by managed policy, turning one on is refused

    Examples:
      | policy                 | answer                                                                                  |
      | keep going             | a deny with the fixed text "Held for your principal as <id>; continue with other tasks" |
      | pause at the first ask | the agent pauses                                                                        |
      | stop at the first ask  | the agent stops                                                                         |

  @OWN-08 @P2 @I2 @I6 @pending
  Scenario: an endorsement sends exactly the confirmed text inside a fixed template
    Given an isolated ParleyInn home
    And a post by "bob"'s device seat in the room, longer than its preview, with invisible characters
    When "alice" expands it, edits it and endorses it to one of her own agents
    Then Endorse was enabled only after the post was expanded
    And a quarantined post is never endorsed: the endorsement is refused, audited and counted
    And the agent receives exactly the confirmed text with invisible characters stripped and their count recorded
    And the endorsement is a signed principal act naming the post's commitment, its author's seat key, the target agent, the original text and the sent text
    And "bob" is shown the edit as a diff
    And the text reaches the agent through the harness's input interface in a fixed template naming the seat key's short fingerprint and the post's address, with no petname
    When "bob"'s device seat is given the moderator role and the room's settings are changed to the most open values
    Then "bob"'s later posts still reach "alice"'s agents only as untrusted recall or by endorsement
    And only a trust grant "alice" records for "bob"'s key under OWN-29 makes "bob"'s posts trusted for her agents

  @OWN-09 @P1 @I2 @pending
  Scenario Outline: nothing but a current principal act starts, resumes or sends text to an agent
    Given an isolated ParleyInn home
    And an agent's idle run
    When <arrival> arrives for the run
    Then no turn is started or resumed
    And no text is sent to the agent

    Examples:
      | arrival                        |
      | a post no trust grant covers   |
      | an opt-in notice               |
      | an agent's post or model reply |
      | a watchdog observation         |

  @OWN-10 @P1 @I2 @I7 @pending
  Scenario: each action class has one rule level that only tightens
    Given an isolated ParleyInn home
    And "alice" set the action class "force pushes", which SEC-13 configures as sensitive, to "ask first"
    And a repository configuration sets "force pushes" to "act without asking"
    When a recall-tainted subagent of a parent run at that level tries a force push
    Then the repository configuration does not loosen the level
    And "force pushes" is tightened one level for the tainted run
    And the subagent, a delegate of the agent that started it, has no looser level than that agent
    And every rule level change "alice" made is on record as a principal act
    And with managed policy capping "force pushes" at "ask first", "alice" setting it to "act without asking" leaves it at the cap

  @OWN-11 @P1 @I2 @I8 @pending
  Scenario Outline: a principal act is accepted according to its class
    Given an isolated ParleyInn home
    And a principal surface "<principal surface>"
    And OWN-22 "<risk state>" widening acts
    When "alice" writes a "<class>" principal act there
    Then the act is "<expected>"

    Examples:
      | principal surface | risk state      | class                   | expected                                                                                                |
      | authenticated     | permits         | widening                | accepted                                                                                                |
      | authenticated     | does not permit | widening                | refused                                                                                                 |
      | authenticated     | permits         | named by no requirement | treated as widening and accepted                                                                        |
      | authenticated     | permits         | cut                     | recorded with a mark naming its principal surface, free text untrusted whatever its event's trust level |
      | authenticated     | permits         | neutral                 | recorded with a mark naming its principal surface, free text untrusted whatever its event's trust level |

  @OWN-12 @P1 @I2 @pending
  Scenario: a CLI verb writing a principal act refuses without a terminal
    Given an isolated ParleyInn home
    And a run on the node leaves a residual risk open with no recorded risk acceptance
    When the person runs "parinn pin add" for a constraint pin that restores, written from the person's device seat a device key certified (before PRV-10 ships, this node's own device seat), a widening principal act, at a terminal
    Then the verb refuses
    And the refusal names the open residual risks and the runs that leave them open
    And the same verb with standard input or output not a terminal refuses before any other check
    When the person runs "parinn counter ack" at a terminal
    Then the acknowledgement is recorded as a neutral principal act with nothing more asked
    When the person runs "parinn pin unpin" at a terminal on a pin her own agent's run seat wrote
    Then the unpin is recorded as a neutral principal act with nothing more asked
    When the person runs "parinn room post" at a terminal
    Then the post, a room act, is signed with the seat key of the person's device seat in that room, carries no OWN-11 class and names the room it acted in
    When the agent's tool call runs "parinn room post" with its standard input and output not a terminal
    Then the verb refuses, so the agent never takes a room act as the person's device seat
    When the agent runs "parinn event search" for an untrusted event with its output not a terminal
    Then the verb is a recall tool: it prints the record content inside the envelope, records a recall event and taints the calling run
    When the person revises the intent of a room she owns at a terminal where an authenticator is required
    Then the revision applies only with the presence proof a widening act needs, and one whose presence proof fails changes nothing

  @OWN-13 @P1 @I2 @pending
  Scenario: a steer arriving after its turn ended is not applied without confirmation
    Given an isolated ParleyInn home
    And "alice" writes a steer naming her agent's current turn
    When the steer arrives after that turn ended
    Then the steer is not applied to another turn
    And "alice" is asked to confirm before it is applied
    And her confirmation is recorded as a widening principal act

  @OWN-14 @P1 @I6 @pending
  Scenario: a stop lists effects first and shows stopped only on acknowledgement
    Given an isolated ParleyInn home
    And an agent's active run with completed, in-flight and waiting effects
    When its principal "alice" asks to stop the run
    Then the room view lists the completed effects, what is in flight and what is waiting, with whether and how each can be undone
    And no surface shows the run as stopped before the harness's acknowledgement is recorded

  @OWN-15 @P1 @I6 @I7 @pending
  Scenario: a control the harness adapter cannot honour is shown unavailable and never simulated
    Given an isolated ParleyInn home
    And Claude Code installed through plain hooks
    When the person runs "parinn install"
    Then it offers, as a shown diff, an opt-in routing every harness launch through the launcher with "parinn launch -- <harness>"
    And it states what the installed path lacks, including the run controls mid-turn steer, interrupt and stop, and terminal takeover
    And each such control is shown unavailable with its reason and the launch path that offers it
    And no such control is simulated

  @OWN-16 @P1 @I2 @I8 @pending
  Scenario: a paired phone is enrolled by a widening act and its device key is limited to its scope
    Given an isolated ParleyInn home
    And a paired phone whose device key "alice" certified with its scope by enrolling it, a widening principal act
    When the paired phone, reaching its node over B2, tries to answer a held permission request with the harness's "allow-session"
    Then the answer is refused on the server
    And the phone can only read, allow once and deny held permission requests
    And no untrusted event, nor an event "parinn verify" reports, shapes an enrollment
    And each answer the phone gives is signed with its own device key and recorded on its device seat in her personal room, naming the room, whose writer the phone seals with that seat's key and the node it pairs with only holds
    And where an authenticator is required each allow carries the phone's own presence proof bound to that answer
    And "parinn uninstall" lists the phone's enrollment, with an offer to remove it
    And without the peer component a browser on the principal's phone reaching the room view through the principal's tunnel is a principal surface, not a paired phone, through a phone-scoped room-view secret that "parinn ui --phone" issued at a terminal as a widening principal act, whose scope the room-view component enforces on the server, and it sees the room view's own origin, port included

  @OWN-17 @P2 @I2 @I8 @pending
  Scenario: a principal act from another of the principal's devices takes effect only within its scope
    Given an isolated ParleyInn home
    And a paired phone of "alice"
    When the phone writes a principal act loosening a rule level
    Then the act does not take effect here
    And the phone is limited to reading and to allowing or denying held permission requests
    And the phone seals its device seat's writer with that seat's key, and this node only holds the writer
    And the phone joins no room: its acts are recorded on its device seat in the personal room, naming the room, which shows them by address
    And any other event with no run that the phone records goes to its device seat in the personal room of the node it pairs with
    And principal acts from "alice"'s other nodes take effect only under PRV-10, for the kinds of principal act, post and pin within their device scope and up to their maximum rule level

  @OWN-18 @P1 @I2 @pending
  Scenario: a command from an untrusted event runs only after the person confirms its exact text
    Given an isolated ParleyInn home
    And an untrusted event carrying a command with invisible characters
    When the person runs "parinn witness-check start" on a result whose check runs that command, and confirms it
    Then the person was shown its exact text with invisible characters made visible before confirming
    And the act is recorded as their widening principal act
    And until that confirmation the command reached no terminal and the launcher did not run it
    And the confirmation sends the command to no agent, since what reaches an agent stays under I2 and OWN-03
    And the witness check runs only through the launcher, started by "parinn witness-check start", outside any agent context, on a fresh checkout of the exact commit, by a node whose git identity authored no commit on the branch since it left its base, with network, local endpoints, the OS user's home directory (`HOME`), the principal's home (its platform key-store entries included) and every write outside that checkout (the repository's git directory and its other worktrees included) denied
    And a check whose code writes to the repository's git directory, a git hook included, or to another of its worktrees leaves both byte-identical
    And its fresh checkout lies outside the run's worktree and adds no ref to the run's repository
    And the launcher records its command by commitment, its exit status and the tree hash as a structural event
    And unless the platform can deny the check network, local endpoints, `HOME`, the principal's home with its key-store entries and every write outside its checkout, ParleyInn refuses the witness check before confirmation
    And the refusal says which of them the platform cannot deny

  @OWN-19 @P1 @I1 @I4 @pending
  Scenario: terminal takeover stays local and no-echo input is not stored
    Given an isolated ParleyInn home
    And a harness hosted by the launcher on this machine
    When "alice" takes over its terminal and types at a no-echo prompt
    Then the takeover runs only in the launcher on the harness's machine
    And the no-echo input is not stored
    And no takeover from another machine is offered
    And what "alice" types in the takeover is the harness's own channel, never recorded as a principal act
    And those keystrokes are the harness's own input from the terminal the launcher hosts, which the launcher never carries or changes, and it writes nothing of its own
    And takeover input sent over the launcher's listener, from the room view or from a paired phone is refused and audited

  @OWN-20 @P1 @I2 @pending
  Scenario: hand-off raises an untrusted reason and hand-back sends only the note and worktree checkpoint address
    Given an isolated ParleyInn home
    And an agent of "alice" hands off with a stated reason
    When "alice" hands back with a note
    Then a Needs you item shows the agent's reason marked untrusted
    And a worktree checkpoint is recorded
    And the agent receives only "alice"'s note and the worktree checkpoint's address through the harness's input interface

  @OWN-21 @P1 @I6 @I10 @pending
  Scenario: Ready for review follows only a principal act and a later edit clears it
    Given an isolated ParleyInn home
    And a room owned by "alice" whose branch is at head "h1", where "bob"'s device seat is a moderator and "carol"'s a contributor
    When "alice" runs "parinn room ready" and the agent then edits the worktree
    Then the room showed Ready for review only after the act was recorded with the branch head "h1"
    And after the edit the room returns to Running or Quiet until marked again
    And a ready mark "bob" records makes the room show Ready for review too, while one "carol" records does not
    And only a principal act of "alice" marking it abandoned makes the room show Abandoned

  @OWN-22 @P1 @I2 @I6 @I9 @pending
  Scenario: an open residual risk refuses widening acts unless the principal accepted it
    Given an isolated ParleyInn home
    And a run with no recorded sandbox state
    When "alice" writes a widening principal act
    Then the run counts as unsandboxed and is still recorded
    And a run whose recorded sandbox state names R1 as blocked, but which no measuring component signed with a key no agent can read, leaves R1 open
    And the act is refused unless a recorded risk acceptance names each open risk, including that the agent can forge it
    And a risk acceptance is shown on every surface beside each run relying on it
    And the risk acceptance is asked again when the set of open risks grows

  @OWN-23 @P1 @I2 @I6 @pending
  Scenario Outline: delegation beyond a subagent rests on a delegation grant in force
    Given an isolated ParleyInn home
    And an agent of "alice" in an active run
    And <precondition>
    When the agent delegates a task to <target>
    Then the delegation is <expected>
    And no untrusted event, nor an event "parinn verify" reports, shapes a delegation grant

    Examples:
      | precondition                                                               | target                                  | expected                                                                                                                                |
      | no delegation grant                                                        | its own subagent                        | recorded under OWN-24, with no delegation grant                                                                                         |
      | no delegation grant                                                        | a new run in another worktree           | refused, audited and shown                                                                                                              |
      | a delegation grant naming that worktree, a delegation budget and an expiry | a new run in that worktree              | started by the launcher, through its harness adapter, and recorded                                                                      |
      | a delegation grant past its expiry                                         | an existing agent of the same principal | refused by the delegating node at use time, audited and shown, while "alice"'s node, once PRV-10 ships, ends the grant by an expire act |

  @OWN-24 @P1 @I2 @pending
  Scenario: a delegate inherits its maximum rule level from the delegating agent and its recall taint from the delegating run
    Given an isolated ParleyInn home
    And an agent whose run is recall-tainted, delegating under a delegation grant whose maximum rule level is "ask first"
    When the delegate's run starts
    Then the delegation is recorded on both sides with the delegating agent, the delegation grant and the delegated task's address
    And the delegate has no rule level looser than the delegation grant's or the delegating agent's
    And the delegate's run is recall-tainted
    And the delegated task reached the delegate through the harness's input inside the fixed template, marked as written by the delegating agent
    And a quarantined delegated task never reaches the delegate in that template: the write is refused, audited and counted
    And a further delegation beyond the delegation grant's depth is refused
    When the delegate returns its delegate report
    Then the delegate report is recorded on the delegating side, within the delegating run's "run" recall scope

  @OWN-25 @P1 @I2 @I6 @pending
  Scenario: a delegate report is pulled, never pushed
    Given an isolated ParleyInn home
    And a delegation to an agent other than a subagent, whose delegate has returned its report
    When the delegating agent calls "delegation_get" for that delegation
    Then the delegate report arrives inside the envelope
    And the call needs no scope and logs no extended recall scope, since the delegate report is recorded on the delegating side
    And no text of the report entered the delegating agent's context before that call
    And ending the delegation grant, a cut principal act, stops every delegate it covers
    When a subagent whose run carries recall taint returns its report to its parent's run as "subagent_result"
    Then the parent's run carries recall taint

  @OWN-26 @P2 @I2 @I8 @pending
  Scenario: another principal's agent takes a delegated task only under its principal's acceptance grant
    Given an isolated ParleyInn home
    And a room where "alice" and "bob" each have a seat
    And "bob" has recorded an acceptance grant naming "alice", a target agent, a maximum rule level, a delegation budget and an expiry
    And "alice" has recorded a delegation grant naming that target
    When an agent of "alice" delegates a task to that target
    Then the delegated task reaches the target in the fixed template, marked as from "alice"'s agent
    And the target keeps "bob" as its one principal
    And the target's run carries recall taint from its first event while "bob"'s node holds none of the delegating run's writers
    And the same delegation without the acceptance grant is refused and audited
    And once the acceptance grant's expiry passes, the delegating node refuses the same delegation at use time, and, once PRV-10 ships, "bob"'s node ends the acceptance grant by an expire act

  @OWN-27 @P1 @I2 @I10 @pending
  Scenario: only a person records a verdict, and it goes stale when what it was bound to changes
    Given an isolated ParleyInn home
    And a room whose agent stated "C1 is done" and recorded a passing check's result linked to C1
    When "alice", a person whose device seat in the room has the pin capability, records "met" on C1 and the agent then edits a file
    Then the verdict is recorded as a "verdict" pin by "alice"'s own neutral principal act, on the signing device's seat in the room, bound to, and recorded with, the intent version, the heads of every branch the room names on her node and the results and evidence shown
    And ParleyInn pre-filled no verdict, and the agent's statement stays a claim
    And the agent's next restore block includes no "verdict" pin, since only "constraint", "preference" and "intent" pins restore
    And after the edit the verdict reads stale
    And a branch the room names that the verdict does not list, linked later or named on another principal's node only, stales it on no node
    And a verdict on the room as a whole, rather than on a criterion, is refused
    And a verdict is refused from a service account, whose principal key a person, another service account or managed policy certified
    And the verdict approves nothing for landing, which stays with git and the forge
    When "alice" records "not met" on C1
    Then the new verdict supersedes the earlier one, which stays on record unedited
    And "alice" unpinning a verdict is recorded as her neutral principal act
    And a list removal of the verdict, by the room's owner or a moderator, is refused
    When "alice" records "needs changes" on C1
    Then that verdict too is recorded as her neutral principal act

  @OWN-28 @P1 @I1 @I2 @pending
  Scenario: a person course-corrects from the verdict
    Given an isolated ParleyInn home
    And a room owned by "alice", where she recorded "needs changes" on C2
    When "alice" sends the correction "keep the header row in every file" through "parinn correction send" and retries from an earlier worktree checkpoint with "--worktree-checkpoint"
    Then the correction reaches the agent through the harness's input in the fixed template naming the verdict, C2 and the results it concerns
    And the retry starts a new run through the launcher in a new worktree at that worktree checkpoint, joined to the room since the retry is its principal's ask for that join, given the correction and the intent pin in force through its restore block
    And the retry receives no content of the earlier run except what it recalls
    And the earlier run and its branch stay on record, shown beside the retry
    And C2 reads "no verdict" until the next verdict
    When "alice" revises the intent from the same verdict
    Then a new version of the room's intent pin is recorded
    And it reaches "alice"'s agents in the room in the same fixed template and, as a pin, in their next restore block

  @OWN-29 @P1 @I2 @I6 @I8 @pending
  Scenario: a principal's trust grant makes another principal's posts trusted for its own agents only
    Given an isolated ParleyInn home
    And a room where agents of "alice" and "bob" have seats, and "carol" posts from her device seat
    When "alice" records a trust grant naming "carol"'s principal key for this room
    Then the trust grant is a widening principal act, shown in the room with its grantor, "carol"'s key and its scope
    And no untrusted event, nor an event "parinn verify" reports, shapes a trust grant
    And "carol"'s next post reaches "alice"'s agent through the harness's input in a fixed template naming her key fingerprint and the post's address
    And the post starts or resumes no turn, reaches no agent of "alice" whose run has no seat in the room, and reaches "bob"'s agent only as untrusted recall
    And a quarantined post of "carol"'s reaches no agent under the trust grant: the write is refused, audited and counted
    And a post from "alice"'s own device seat reaches her agent only by recall or her endorsement, never in the trust grant's template
    When "carol" pins the constraint "keep the public API stable" from her device seat
    Then the pin restores word for word to "alice"'s agent and reaches "bob"'s agent only through a tool call, enveloped
    And a trust grant naming a run seat's key, or a service account its certificate or managed-policy listing marks as relaying text others wrote, or one a managed-policy listing names without that mark, is refused and audited, while one naming any other service account shows a warning
    And a trust grant recorded before a certificate or managed-policy listing marking its key as relaying, or a listing of that key without the mark, reaches the node covers nothing once it arrives, as if it had arrived first
    And no revocation of that certificate or removal of that listing makes the trust grant cover anything again
    And a revocation of any certificate of a key, or a removal of any listing of it, makes every trust grant naming that key that does not causally follow it cover nothing
    And the trust grant does not cover a service account whose principal key "carol" certified
    And it does not cover a post or pin "carol" writes from a token-key-only node
    And it does not cover a post or pin "carol" writes from a device seat outside the device scope of the device key that certified it
    And a trust grant naming the room's facilitator is recorded only after "alice" is shown that the facilitator reads untrusted room text, and covers its posts but never its room summaries
    And no role, membership or room setting makes any other principal trusted
    And a principal act "bob" writes directed to "alice"'s agent's run does not instruct it, since its one principal is "alice"
    And "bob"'s text reaches that run only as untrusted recall, a post "alice" endorses, a pin version "alice" stamped, a delegated task under "alice"'s acceptance grant (OWN-26), or a post or pin "bob" wrote from a device seat his device key certified that a trust grant of "alice" covers
    And "carol"'s posts in a foreign room stay untrusted for "alice"'s agent whatever trust grant covers "carol"'s key
    And in a foreign room a post from "alice"'s own device seat, and a pin a trust grant of hers covers, are untrusted for her agents too, but for a pin PIN-10 keeps restoring there after a leave, kick or bar, while a pin from her device seat and a pin version she stamped are trusted there as elsewhere
    And a cross-room post "carol" sends from another room, shown in this room, reaches "alice"'s agent only as untrusted recall
    When "alice" revokes the trust grant as a cut principal act
    Then "carol"'s later posts reach "alice"'s agent only as untrusted recall, and her pin no longer restores to it
