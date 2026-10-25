# Persona re-review of the reconciled proposal

The persona-review skill ran all nine persona agents again, in
parallel, on one target: [proposal.md](../../proposal.md) at commit
ff6fa8e. Each was given its own round-1 report and asked which findings
the proposal resolves, which it resolves partly, and what is new. Each
persona's full report is beside this file.

## Verdicts

| Persona                 | Verdict                                  | Still blocking                                |
| ----------------------- | ---------------------------------------- | --------------------------------------------- |
| fleet developer         | serves, if spike S9 holds                | remote answering rests on ASM-18              |
| multi-machine developer | most blockers fixed in text              | shallow-clone identity                        |
| reviewer                | v1 serves the story and the landed trace | none; approvals need presence and one human   |
| OSS maintainer          | the trust model serves                   | cannot tell a real lane from a fabricated one |
| live collaborator       | serves in contract terms                 | none; five states and rules missing           |
| returning owner         | serves                                   | a lane started with capture broken is unseen  |
| platform operator       | partly served                            | managed policy source; backup and restore     |
| security officer        | serves in structure                      | CLI owner acts; copies survive erasure        |
| Claude, the agent       | serves                                   | none; two text paths and lane_status fields   |

All eleven round-1 blockers have an answer in the proposal. What is
left is narrower: holes in the new rules, not missing rules.

## Merged findings

Numbered for citation; the numbers stay. R1–R8 must be fixed before
the SRS edit. R9–R18 go into the same revision, each as a MUST
sentence drafted from the reports.

| #   | Finding                                                                                                                                                                                                 | Raised by                                     | Severity  | Action                                                                                                                                          |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | --------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| R1  | Owner acts outside OWN-11 and OWN-12: endorse, pin, purge, quarantine release, approve, request changes, peer enroll and revoke; an agent can run them from Bash                                        | security, reviewer, fleet                     | blocking  | Every owner act on every surface passes OWN-12; the sensitive ones also pass OWN-11; say whether a session allow is sensitive                   |
| R2  | Erasure leaves copies: other writers' logs, sent endorsements, recall results, re-ingestable transcripts, git-carrier refs, PIN-03's text hash, segment files, the payload reference                    | security                                      | blocking  | Purge erases or tombstones every stored copy, blocks re-ingestion of the range, and reports copies it cannot erase                              |
| R3  | A shallow clone has no root commit, so a sandbox mints a second project identity                                                                                                                        | multi-machine                                 | blocking  | Take the identity from the enrollment token or a peer's bound identity; never mint a different one                                              |
| R4  | A lane that starts while capture is broken never exists; search footers omit capture gaps; the board can say Quiet over a dead lane                                                                     | returning                                     | blocking  | Catch up lists every uningested transcript and risen failure counter; footers name gaps; a lane takes its worst freshness mark                  |
| R5  | Managed policy has no source and no ceiling on rule levels, away policies, hook decisions or holds; holds stall headless runners                                                                        | operator                                      | blocking  | Managed policy from a path the tenant cannot write, overriding tenant and project config; caps per action class; holds off                      |
| R6  | Backup and restore still assume SQLite; writer keys are never backed up; cloned images share a writer key                                                                                               | operator                                      | blocking  | Back up segments, payloads and derived state; a restore or a copied home mints and audits a new writer; never reuse a seq                       |
| R7  | A bundle cannot be told real from fabricated: writer keys are not bound to anything the maintainer knows, withheld events break the chain, and a bundle can claim a local lane id                       | maintainer                                    | blocking  | Keep chained headers of withheld events; refuse or separate colliding lane ids; bind writer keys to a key the receiver can check                |
| R8  | Remote answering rests on unverified ASM-18 (spike S9); a parked approval is lost on the plain-hook path                                                                                                | fleet, agent                                  | blocking  | Install says when held requests cannot be answered elsewhere; a parked approval reaches the agent as fixed TrustedText or is listed absent      |
| R9  | Text paths into the agent: an away policy or resume can start a turn; endorsed text has no source; an edited endorsement records one text; I2 contradicts the automatic writes it allows                | agent, collaborator, security                 | important | Away policy only answers a pending request; endorsements in a fixed sourced template with both texts; I2 lists automatic writes as a closed set |
| R10 | `lane_status` fields are undefined and returned outside an envelope; titles, branches and lane ids are writer-chosen; a stranger's key shows the name it offers                                         | agent, security, maintainer                   | important | A closed list of structural fields; opaque lane ids; petnames default to a fingerprint                                                          |
| R11 | Key chain gaps: sandbox writers cannot chain while the owner key is offline; a stolen key can backdate past its revocation; a same-user agent can read the `0600` writer key; receipts stay on the node | multi-machine, security, returning            | important | Delegated device certificates; refuse unheld events after a revocation; say what protects the writer key; write receipts outside PARINN_HOME    |
| R12 | Peer sync: PEER-04's 5 s bound against 30 s seals; a partition mints two lanes for one branch; whole lanes reach co-authors unreviewed; git-carrier segments are plaintext                              | multi-machine, security                       | important | Seal at PEER-04's pace while connected; merge concurrent lanes deterministically; review before sharing; encrypt carrier segments               |
| R13 | Boundary register gaps: `parinn-run`'s children and listener, `parinn-peer` in two boundaries, bridges outside ENG-16, witness runs as the user, B3 credentials, wildcard binds, discovery payload      | security, operator, maintainer                | important | One row per process `parinn-run` starts; witness runs network- and home-denied; a B3 credential rule; no wildcard bind by default               |
| R14 | Review gaps: "bound human" undefined; a result is not bound to its tree; request-changes follow-up has no rule; forge double review and agent verdicts left open; witness default unclear               | reviewer                                      | important | One human per owner key; results carry their tree or show unbound; show the delta since a verdict; settle OQ-22 and the forge hand-off          |
| R15 | Lane states without a source: a branch change mid-session, Ready for review, seen, overlap dismissal, handover lost, the role request; only the owner can endorse                                       | fleet, collaborator                           | important | Derive each from a recorded event or drop it; any principal endorses to their own agents                                                        |
| R16 | Delivery: no maintainer journey works in v1; a D7 veto would remove catch-up and search; M8 has no collaborator exit; plan 2610022338's phase 1 has no security-review gate                             | maintainer, returning, collaborator, security | important | Keep the CLI forms of VIEW-08..11 at their priority; add exit criteria and the ENG-29 gate to the phase files                                   |
| R17 | `parinn uninstall` no longer covers every artifact; about 2,880 segments a day per writer with no compaction rule                                                                                       | operator                                      | important | Uninstall lists and offers to remove every artifact and audits what it left; a segment compaction requirement                                   |
| R18 | The pitch still sells the gate, reviewer re-runs and CI as current                                                                                                                                      | reviewer                                      | important | Mark them Next in pitch.md                                                                                                                      |

Minor findings stay in the persona reports: chain-state vocabulary,
PIN-03's bare seq, restore blocks naming their lane, Bash edits in
LANE-13, `parinn open` focus, the interrupt key, unsigned-tail wording,
the first catch-up screen, presence display, LAN discovery and import
from a read-only view.

## Next

Revise proposal.md against R1–R18, then run this skill a third time on
the revision. The SRS edit follows only once no persona reports a
blocker.
