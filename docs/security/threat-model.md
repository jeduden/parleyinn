---
summary: >-
  ParleyInn's threat model by boundary register row: the threat sources
  acting across each of the register's rows, the controls that hold, a
  control-by-control comparison with Zed Delta, and the review of each
  minor release (SEC-17).
---
# Threat model

This is the threat model SEC-17 asks for. It reads the
[boundary register](../srs/06b-boundary-register.md) row by row, names the
threat sources of [§6.1](../srs/06-security.md) that act across each row, and
points at the controls that hold. The assets, the threats T1 to T29 and the
residual risks stay in §6.1. A check reads this page against the register and
§6.1 (`the_threat_model_covers_the_register_and_compares_with_zed_delta`), so
a register row or a threat source added there fails it until it is covered
here.

## Threat sources

| Id  | Threat source                                      | In scope                                                    |
| --- | -------------------------------------------------- | ----------------------------------------------------------- |
| S1  | Third party behind external content                | Yes, the primary threat                                     |
| S2  | Malicious or compromised MCP server                | Yes                                                         |
| S3  | Another principal on a shared host                 | Yes                                                         |
| S4  | Repository's commit authors                        | Yes                                                         |
| S5  | Supply-chain attacker                              | Yes                                                         |
| S6  | Process running as the same OS user                | For principal acts and a device seat's room acts only (T21) |
| S7  | Another principal with a seat, or an enrolled peer | Yes                                                         |
| S8  | Network attacker between peers                     | Yes                                                         |
| S9  | Web page in the person's own browser               | Yes                                                         |
| S10 | Root on the host, compromised Claude Code binary   | No                                                          |

## Across each boundary

- **B0, the core.** Opens no socket and starts no program but its kernel
  worker (tests:
  `the_core_executable_links_only_crates_its_reach_allows` and
  `the_core_opens_no_socket_and_starts_no_program_but_its_kernel_worker`).
  The sources that act across it carry bytes in through the harness's
  stdin and stdout, the transcripts and the files the core reads: S1 and S2
  (content), S3 (the home), S4 (repository files), S5 (the build) and S6.
  Controls: I2's envelope and closed paths, SEC-01 and the reach evidence,
  SEC-05, SEC-08, SEC-16, SEC-18, I8's strict home.
- **B1, machine.** Loopback or a same-user local endpoint only. S9 and S6 act
  across it; S3 reaches it only through the host. Controls: SEC-20's launch
  secret and Host and Origin checks, the acceptance that turns a component on,
  T21's residual risk.
- **B2, peer.** Encrypted, mutually authenticated connections to enrolled
  keys. S7 and S8 act across it. Controls: PEER-10, segments that carry their
  own signatures, I8's kept-as-theirs rule, SEC-22's switches.
- **B3, public.** Read-only publishing and outbound exchange with named hosts.
  S8 and S1 act across it. Controls: signed bundles, named hosts only, digests
  in receipts, nothing accepted from a notification host.
- **Outside.** What the harness, the principal's own tunnel or `git push`
  carry. ParleyInn adds nothing there (I4) and relies on S10 being out of
  scope.

## Register rows

| Row | Component or protocol                     | Boundary | Sources            | Controls that hold                                         |
| --- | ----------------------------------------- | -------- | ------------------ | ---------------------------------------------------------- |
| 1   | Hook handlers and the CLI                 | B0       | S1, S2, S3, S4, S6 | T1, T3, T9: SEC-05, SEC-16, SEC-18; I9 fail-open           |
| 2   | MCP server                                | B0       | S1, S2, S6         | T1, T16, T29: SEC-06 envelope, RCL pull-only recall        |
| 3   | Kernel worker                             | B0       | S1                 | T8: bounds and deadlines (CMP-05, ENG-03)                  |
| 4   | CLI and TUI                               | B0       | S1, S3, S6         | T6, T16, T21: strict home, safe terminal text              |
| 5   | Harness strip                             | B0       | S1                 | T16: text printed, never added to the model's context      |
| 6   | Terminal notifications                    | B0       | S1                 | T16: terminal only, room petname, class and count          |
| 7   | Bundle export and import                  | B0       | S1, S4, S7         | T14, T17, T18: local files only, kept as theirs (I8)       |
| 8   | Room-view component                       | B1       | S6, S9             | T15, T21: SEC-20 secret, loopback only, off until accepted |
| 9   | Desktop notifications and presence proofs | B1       | S6, S9             | T15, T21: raised by the loopback room view only            |
| 10  | Launcher                                  | B1       | S1, S6             | T20, T24: off until accepted, connects nowhere else        |
| 11  | Phone, before B2                          | B1       | S6, S8, S9         | T15, T22: the principal's own tunnel, phone-scoped secret  |
| 12  | Peer transport                            | B2       | S7, S8             | T17, T23, T26: mutual authentication, PEER-10, off         |
| 13  | Local discovery                           | B2       | S8                 | T17: random per-boot id and port only, never enrolls       |
| 14  | Access token for an ephemeral node        | B2       | S6, S8             | T22: secret from the environment, outbound only to named   |

| Row | Component or protocol                                   | Boundary | Sources    | Controls that hold                                                                          |
| --- | ------------------------------------------------------- | -------- | ---------- | ------------------------------------------------------------------------------------------- |
| 15  | Presence and typing hints                               | B2       | S7, S8     | T14: signed, ephemeral, never stored                                                        |
| 16  | Erasure, purge and quarantine requests                  | B2       | S7, S8     | T18: signed events, applied by this node's principal                                        |
| 17  | Phone, over B2                                          | B2       | S7, S8     | T22: signed allow and deny events in its device seat                                        |
| 18  | Publish component's listener                            | B3       | S1, S8     | T16: inbound, read-only, signed bundles                                                     |
| 19  | Forge bridge                                            | B3       | S1, S7, S8 | T28: exact grants, replies only, no approval write-back                                     |
| 20  | CI bridge                                               | B3       | S1, S8     | T19: attestations for an exact commit only                                                  |
| 21  | Notification bridge                                     | B3       | S8         | T16: named hosts, petname, class and count, accepts nothing                                 |
| 22  | Merging several seats' edits (rejected)                 | none     | S7         | Rejected for v1                                                                             |
| 23  | The harness's own model traffic                         | outside  | S10        | I4: carries only what I2's closed paths and recall write                                    |
| 24  | Harness started by the launcher                         | outside  | S6, S10    | The launcher adds no reach                                                                  |
| 25  | Witness-check command                                   | B1       | S1, S4, S6 | T24, T19: fresh checkout, denied network and home                                           |
| 26  | Blind peer storage                                      | B2       | S7, S8     | T17: stores ranges encrypted to the room's device keys                                      |
| 27  | Commit hook                                             | B0       | S4, S6     | I7: installed only by explicit operation; no socket                                         |
| 28  | Room trailers and trailer links                         | outside  | S1         | Ids and a named host only, in the principal's own push                                      |
| X1  | `parinn import https://…` (rejected)                    | none     | S1, S8     | A fetch from the core would break B0                                                        |
| X2  | Phone view served over B2 (rejected)                    | none     | S7, S8     | B2 carries only segments and hints                                                          |
| X3  | Terminal takeover across machines (rejected)            | none     | S6, S8     | Takeover stays on the harness's machine                                                     |
| X4  | Web Push and vendor relays (rejected)                   | none     | S8         | A vendor's central service (CON-06)                                                         |
| X5  | Update checks and telemetry (rejected)                  | none     | S5         | SEC-15: none is linked or written (test: `no_telemetry_crash_report_or_update_check_ships`) |
| X6  | Forge approval and landing write-back (rejected)        | none     | S1, S7     | Replies only under LANE-08, LANE-34 and SEC-28                                              |
| X7  | A ParleyInn rendezvous or relay host (rejected)         | none     | S5, S8     | No central service                                                                          |
| X8  | A core process starting a B1 to B3 component (rejected) | none     | S1, S6     | The core never starts another component (SEC-01)                                            |

## Comparison with Zed Delta

Zed Delta is the one shipping system that keeps an agent's conversation and
its edits as one replicated record. The research notes in
`research/notes/git-alternatives-for-agent-sessions` hold what it publishes.
That is blog text. Where it says nothing about a control, the cell says so.

| Control                    | ParleyInn                                                                                                                                                                              | Zed Delta                                                                                       |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| Record signing             | Each writer's log is a hash chain; the device key signs seat certificates and key events (PRV-11, PRV-12). Signed segment seals come with REC-18                                       | Not documented publicly; conversations are replicated by Zed's service                          |
| Trust in other principals  | Another principal's content is kept as theirs and untrusted unless this principal's own stamp or trust grant covers it (I2, I8)                                                        | Not documented publicly; teammates' conversations are replayed to agents as first-class content |
| Central-service dependence | None: no central or third-party service, no telemetry (I4, SEC-15, CON-06; test: `no_telemetry_crash_report_or_update_check_ships`); every component runs on the principal's own nodes | Conversations live in DeltaDB, Zed's service with local replicas                                |
| Key custody                | Keys stay on the node in its OS key store or owner-only files, never in the store, a log or an export (SEC-10)                                                                         | Not documented publicly                                                                         |

## Review record

Every minor release is reviewed against the register and §6.1 before it ships,
and the release checklist names this page. A row is added in the change that
reviews the release.

| Release | Reviewed by                                                                     | Outcome                                                            |
| ------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| 0.0     | Phase 15 author (an agent); the security reviewer's review is still to be given | Written against the register of M1; awaiting the security reviewer |
