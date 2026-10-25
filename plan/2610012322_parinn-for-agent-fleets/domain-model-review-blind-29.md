# Domain model: twenty-ninth round of blind reviews, merged

Three blind domain-model agents reviewed the model after the
twenty-eighth-round decisions were applied (commit c87c999), in the
review workflow: two skeptics, on text accuracy and on whether the
defect is real, checked every needs-fix finding. Of 16, 15 survived; one
(the seat ingest starts beside a personal-room seat) the model already
settles through Member. Each question takes its recommended option, but
the two in section 4.

## 1. Questions and decisions

| #   | Question                                                                         | Found | Decision                                                                                                                         |
| --- | -------------------------------------------------------------------------------- | ----- | -------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Does a trust grant over the owner make the intent restore to the grantor?        | 1/3   | Yes, but after a handover, as LANE-11 says (Intent, LANE-20).                                                                    |
| Q2  | What does "its own pins" mean for a device seat's pin capability?                | 1/3   | Every pin its principal wrote from a device seat within that seat's device scope (Contributor, LANE-16).                         |
| Q3  | May `parinn uninstall` delete the store outside the purge it offers?             | 1/3   | No: only the offered purge removes stored content, with tombstones and a purge receipt (ADM-02, ADM-07, SEC-31).                 |
| Q4  | Are room summaries versioned?                                                    | 1/3   | No: "and version" goes; the latest is derived in causal order.                                                                   |
| Q5  | Which rule stands for joins before routed events?                                | 1/3   | LANE-01's routing: a device seat also joins before a tombstone, a retention erasure request or a bridge's or launcher's event.   |
| Q6  | Does the owner's notice allowance contradict "only an agent's principal widens"? | 1/3   | No: the allowance only lets through opt-in notices an agent's principal opted it in to (Principal).                              |
| Q7  | Is disabling held requests disabling their record?                               | 1/3   | No: managed policy disables keeping held requests waiting; the held request is still recorded and mirrored (OWN-05, OWN-06).     |
| Q8  | Is bare "range" an address range?                                                | 1/3   | Yes, stated under Address; `--range` stays.                                                                                      |
| Q9  | How is the room-view component or the launcher turned off?                       | 1/3   | A configuration turning it off applies with no acceptance, and the CLI records it as the cut act; the acceptance stops standing. |
| Q10 | Are a read-only CLI verb's piped output and the kernel's built-ins recall?       | 1/3   | Yes: both are recall tools, each recording a recall event (OWN-12, CMP-03); I2 keeps its words.                                  |
| Q11 | May a list removal take a verdict?                                               | 1/3   | No: never the intent or a verdict.                                                                                               |

## 2. Fixes that need no decision

- **Model:** routed ingest appends are a witnessed run's; verdicts are
  recorded and unpinned by neutral principal acts, not room acts; the
  room merge and room state stand beside the seats with no add and the
  ingest seat; **writer-label assignment** named; Retire a writer marks
  a writer retired (PEER-05); the review step covers a publish; Trust
  grant names managed-policy listings; Origin, Provenance, Taint, Held
  request and Run reworded.
- **SRS and scenarios:** ADM-13 says git configuration and git hooks;
  SEC-22's row, §9.1 and NFR-01 keep held requests recorded; RCL-08
  "writer-label assignment"; lane.feature "no pin author"; LANE-13
  "for each room's owner"; "managed policy" in full; §6.3's component
  names; PRV-08's `harness_text`.

The adversarial check put the expire act back into LANE-23, which a
cross-reference to LANE-01 had dropped, and the model now cites SEC-01
beside ADM-04 for turning the room-view component or the launcher off,
since ADM-04 states only the acceptance that turns it on.

## 3. Optional, not chased

A naming rule for crates (ENG-02), which a reviewer marks hard to revert
before M1; "forwarder" and "typist" in two scenarios.

## 4. Invariants and reviews

No invariant changes. Waiting for the stakeholder:

- the I4 commit-trailer wording;
- the signing of browser acts, now widened by a reviewer to every event a
  non-core component records: room acts taken in the browser room view,
  and what the launcher and the bridge component append, under SEC-10
  and SEC-20;
- I10's "statuses".
