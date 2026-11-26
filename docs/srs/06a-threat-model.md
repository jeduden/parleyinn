---
title: "6. Security: threat model"
summary: >-
  The threat model: assets, threat sources, threats T1–T12 and T14–T29
  with their controls, and the residual risks R1–R8 only a sandbox
  removes. Part of §6.
---
# 6. Security: threat model

## 6.1 Threat model

### Assets

| Asset                             | Why it matters                                                                             |
| --------------------------------- | ------------------------------------------------------------------------------------------ |
| Record content                    | Contains code, messages, posts, tool output, and possibly secrets that escaped redaction   |
| Pins                              | Directly shape agent behaviour after every compaction                                      |
| Closed paths                      | What ParleyInn writes to an agent without its recall (I2), placed into the model's context |
| Recall                            | Text the agent pulls into context through a recall tool call                               |
| Harness configuration             | Hooks and MCP registrations control what runs in every agent run                           |
| ParleyInn binary and dependencies | Runs inside every agent run                                                                |

### Threat sources

| Threat source                                      | Can do                                                                                         | In scope                                                                                  |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Third party behind external content                | Controls text the agent reads: web pages, repositories, issues, dependency READMEs, file names | Yes — primary threat                                                                      |
| Malicious or compromised MCP server                | Controls its tool results and tool names                                                       | Yes                                                                                       |
| Another principal on a shared host                 | Separate OS user on the same machine                                                           | Yes                                                                                       |
| Repository's commit authors                        | Controls repository files, including `.parinn.toml` and `CLAUDE.md`                            | Yes                                                                                       |
| Supply-chain attacker                              | Targets ParleyInn's dependencies, build, or release                                            | Yes                                                                                       |
| Process running as the same OS user                | Whatever that OS user can do (T21)                                                             | **For principal acts and a device seat's room acts only**, under the residual risks below |
| Another principal with a seat, or an enrolled peer | Writes posts and segments into a shared room                                                   | Yes                                                                                       |
| Network attacker between peers                     | Observes or alters B2 and B3 traffic                                                           | Yes                                                                                       |
| Web page in the person's own browser               | Reaches loopback ports from the browser                                                        | Yes                                                                                       |
| Root on the host, compromised Claude Code binary   | Full control                                                                                   | **No**                                                                                    |

### Threats and controls

- **T1**: Persistent prompt injection
  - Vector: Untrusted content stored, then placed into a later run's context
  - Controls: Pull-only recall and `TrustedText` (INJ-03, INJ-04), envelopes
    (RCL-04), trust policy (PRV-02..05), no promotion (MEM-01)
- **T2**: Compaction-driven poisoning
  - Vector: Text crafted to survive summarization into compaction summaries or
    derived artifacts
  - Controls: `harness_text` untrusted and never restored (PRV-03, INJ-09);
    structural landmarks only (LMK-02..04)
- **T3**: Injection through structural fields
  - Vector: File named, or MCP tool named, like an instruction
  - Controls: Sanitization to allow-list (LMK-03)
- **T4**: Pin forgery
  - Vector: Agent or external content creates a pin
  - Controls: Only a principal makes a pin restore (PIN-01, PIN-02, PIN-05)
- **T5**: Security downgrade via repository
  - Vector: Malicious `.parinn.toml` enables per-prompt injection or trusts more
    provenance
  - Controls: Repository configuration may only tighten (SEC-11)
- **T6**: Cross-principal access
  - Vector: Another OS user reads or writes a principal's home
  - Controls: File permission and file-owner enforcement (SEC-02), home binding
    (SEC-03), no sockets (SEC-01)
- **T7**: Secret disclosure
  - Vector: Secrets in transcripts stored, recalled, or backed up
  - Controls: Redaction before storage (SEC-08), encryption at rest (SEC-09),
    purge (ADM-07, SEC-14)
- **T8**: Denial of service
  - Vector: Pathological queries, oversized inputs, runaway kernel code
  - Controls: Bounded queries (SEC-04), input limits (SEC-05), kernel limits
    (CMP-05), fail-open (NFR-06)
- **T9**: Path traversal
  - Vector: Hook input or transcript fields pointing outside allowed roots
  - Controls: Path validation (SEC-18)
- **T10**: Configuration tampering
  - Vector: Tool silently rewrites harness configuration
  - Controls: Plugin distribution, explicit install, read-only doctor
    (ADM-01..03, ADM-10)
- **T11**: Supply-chain compromise
  - Vector: Malicious dependency or release artifact
  - Controls: Dependency policy, signed builds (ENG-16, ENG-18, ENG-20, ENG-33)
- **T12**: Unnoticed compromise
  - Vector: Poisoned content acted on by unattended agents
  - Controls: Audit log (OPS-01, OPS-02), flags (PRV-07), recall taint (SEC-13),
    quarantine (SEC-12)
- **T14**: Post or segment injection
  - Vector: A post or segment written to instruct another principal's agents
  - Controls: PRV-02, PRV-09, INJ-10, OWN-08, OWN-09
- **T15**: Local web attack on the UI
  - Vector: DNS rebinding, CSRF, cross-origin reads, a room-view secret shared
    across ports
  - Controls: SEC-20
- **T16**: Rendered-content attack
  - Vector: Record content that runs script, loads remote resources or drives
    the terminal it is shown on
  - Controls: SEC-06, SEC-21, VIEW-07
- **T17**: Forged or rewritten segments
  - Vector: A forged writer, rewritten chain or equivocation from a peer
  - Controls: REC-18, PRV-09, SEC-25, PEER-10
- **T18**: Erasure gap
  - Vector: Purged content confirmable from hashes, or kept on peers and bundles
  - Controls: REC-17, ADM-07, SEC-31, PEER-11, SEC-30
- **T19**: Forged verification
  - Vector: Tool output or agent text claiming a check passed
  - Controls: LANE-05, LANE-08, VIEW-06
- **T20**: Principal-voice laundering
  - Vector: Untrusted text delivered as a principal's instruction or a launcher
    command
  - Controls: OWN-03, OWN-04, OWN-07, OWN-18
- **T21**: Same-user self-approval
  - Vector: An unsandboxed agent calls loopback or the CLI to answer its own
    held request, opens its own pty, starts its own room-view component or
    launcher, reads a terminal's screen, injects keystrokes or GUI input, reads
    the browser profile, enrolls a software authenticator, writes the store,
    keys or configuration directly, makes a backup restore, replaces ParleyInn's
    executables or hooks, deletes the home, or signals ParleyInn's processes to
    lower a confirmation
  - Controls: SEC-20, OWN-02, OWN-11, OWN-12, OWN-22
- **T22**: Device key compromise
  - Vector: A stolen paired phone or node signs principal acts
  - Controls: PRV-10, OWN-16, OWN-17, SEC-27, REC-24
- **T23**: Resource exhaustion by peers
  - Vector: A peer or ephemeral node fills the disk or floods posts
  - Controls: ADM-15, INJ-10, LANE-10
- **T24**: Launcher command execution
  - Vector: A recorded command started through the launcher
  - Controls: SEC-29, OWN-18
- **T25**: Delegation laundering
  - Vector: Injected text in an agent's context handed to a delegate as its task
  - Controls: OWN-23, OWN-24, OWN-25, SEC-13
- **T26**: Membership state reset
  - Vector: A late, reordered or withheld sync undoes a kick or a bar, or a
    fresh key evades a bar
  - Controls: LANE-25, PEER-13
- **T27**: Swayed facilitator
  - Vector: Injected room text steers a facilitator or an appointed run seat
    into kicking, barring or muting a room, or into posts or pins that sway the
    agents of a principal who trusts the facilitator
  - Controls: SEC-32, LANE-16, OWN-29
- **T28**: Trust laundering
  - Vector: An agent that read untrusted content is trusted through a trust
    grant, or a bridge or a trusted service account's posts relay third parties'
    text as trusted
  - Controls: OWN-29, LANE-27, LANE-29, SEC-32
- **T29**: Seat impersonation
  - Vector: A harness or agent posts under another seat's id, a model leaks its
    seat key, or a process of the same OS user reads that key from the MCP
    server's memory
  - Controls: LANE-23, LANE-24, SEC-10, and a sandbox (SEC-19)

### Residual risks

These risks remain while an agent runs unsandboxed as its principal's own
OS user; only a sandbox removes them.

- **R1**: The agent types into the harness's own prompt to approve itself
- **R2**: The agent edits the harness's settings or hooks, or replaces
  ParleyInn's executables
- **R3**: The agent writes to, deletes or rolls back `PARINN_HOME`, or signs
  events with the node's device key or a seat key; only a head receipt kept off
  the machine shows it
- **R4**: The agent shows a harmless screen while a widening act waits for
  confirmation
- **R5**: The agent reads every room from the store or its own room-view
  component, unenveloped
- **R6**: The agent uses cut acts (stop, deny, a quarantine removing nothing
  from a restore block) against other agents; audited, not prevented
- **R7**: The agent confirms widening acts, or takes its principal's device
  seat's room acts, at a terminal, by design (OWN-12); on a facilitator's node,
  it takes the facilitator's acts through that node's CLI as the program does
  (SEC-32)
- **R8**: The agent, or any process of the same OS user, appends lines to a
  harness transcript or runs a hook handler with an input it made, which no
  record can tell apart from the harness's own; so while any run on the node
  leaves R3 or R8 open, ParleyInn trusts no `user` event, no hook observation
  and no `harness_meta` but the launcher's, and no transcript line ends INJ-05's
  ambiguity (PRV-08).
  Such a process can still misattribute lines, or plant subagent transcripts
  that keep every compaction ambiguous and cost restored pins (I3), but never
  make content trusted
